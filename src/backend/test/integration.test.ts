import { readdir, readFileSync } from "fs";
import { Miniflare } from "miniflare";
import { v4 as uuidv4 } from "uuid";

let mf: Miniflare | undefined = undefined;

interface LoginResponse {
  token: string;
}
interface Chat {
  id: string;
  name: string;
}

interface NewMessageResponseWrapper {
  message: NewMessageResponse;
  message_type: string
}

interface NewMessageResponse {
  contents: string;
  user: string;
}

describe("backend integration tests", () => {
  beforeAll(async () => {
    mf = new Miniflare({
      scriptPath: "./src/backend/build/worker/shim.mjs",
      modules: true,
      modulesRules: [
        { type: "CompiledWasm", include: ["**/*.wasm"], fallthrough: true },
      ],
      d1Databases: ["CHAT_METADATA"],
      durableObjects: {
        CHATROOM: "Chatroom",
      },
      bindings: {
        JWT_SECRET: "hello",
      },
      durableObjectsPersist: true, // Defaults to ./.mf/do
    });

    const DB = await mf.getD1Database("CHAT_METADATA");

    readdir("./src/backend/migrations", function (err, files) {
      if (err) {
        console.error("Could not list the directory.", err);
        process.exit(1);
      }

      files.forEach(async function (file, index) {
        const content = readFileSync(
          `./src/backend/migrations/${file}`,
          "utf8"
        );

        const preparedStatement = content
          .replace(/(\r\n|\n|\r)/gm, "")
          .split(";");

        preparedStatement.forEach(async function (statement) {
          if (statement === "") {
            return;
          }
          await DB.exec(statement);
        });
      });
    });
  });

  afterAll(async () => {
    mf?.dispose();
  });

  it("get-chats-should-fail-without-authentication", async () => {
    const res = await mf!.dispatchFetch("http://localhost/api/chats", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer 12345",
      },
    });
    expect(res.status).toBe(401);
  });

  it("create-chat-should-fail-without-authentication", async () => {
    const res = await mf!.dispatchFetch("http://localhost/api/chats", {
      method: "POST",
      body: JSON.stringify({ name: "testchat", password: "Password!23" }),
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer 12345",
      },
    });
    expect(res.status).toBe(401);
  });

  it("user-can-register-login-and-make-authenticated-calls", async () => {
    const username = uuidv4();
    const userPassword = uuidv4();
    const testChatName = uuidv4();

    const res = await mf!.dispatchFetch("http://localhost/api/register", {
      method: "POST",
      body: JSON.stringify({ username: username, password: userPassword }),
      headers: {
        "Content-Type": "application/json",
      },
    });
    expect(res.status).toBe(200);

    const loginRes = await mf!.dispatchFetch("http://localhost/api/login", {
      method: "POST",
      body: JSON.stringify({ username: username, password: userPassword }),
      headers: {
        "Content-Type": "application/json",
      },
    });

    expect(loginRes.status).toBe(200);

    const resultBody = (await loginRes.json()) as LoginResponse;

    expect(resultBody.token).toBeDefined();

    const createChatRes = await mf!.dispatchFetch(
      "http://localhost/api/chats",
      {
        method: "POST",
        body: JSON.stringify({ name: testChatName }),
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${resultBody.token}`,
        },
      }
    );

    expect(createChatRes.status).toBe(200);

    const listRes = await mf!.dispatchFetch("http://localhost/api/chats", {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${resultBody.token}`,
      },
    });
    expect(listRes.status).toBe(200);

    const listResBody = (await listRes.json()) as Chat[];
    expect(listResBody.length).toBeGreaterThan(0);
    expect(listResBody[0].name).toBe(testChatName);
  });

  it("user-can-register-login-and-connect-to-chat", async () => {
    const username = uuidv4();
    const userPassword = uuidv4();
    const testChatName = uuidv4();

    const res = await mf!.dispatchFetch("http://localhost/api/register", {
      method: "POST",
      body: JSON.stringify({ username: username, password: userPassword }),
      headers: {
        "Content-Type": "application/json",
      },
    });
    expect(res.status).toBe(200);

    const loginRes = await mf!.dispatchFetch("http://localhost/api/login", {
      method: "POST",
      body: JSON.stringify({ username: username, password: userPassword }),
      headers: {
        "Content-Type": "application/json",
      },
    });

    expect(loginRes.status).toBe(200);

    const resultBody = (await loginRes.json()) as LoginResponse;

    expect(resultBody.token).toBeDefined();

    const createChatRes = await mf!.dispatchFetch(
      "http://localhost/api/chats",
      {
        method: "POST",
        body: JSON.stringify({ name: testChatName }),
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${resultBody.token}`,
        },
      }
    );

    expect(createChatRes.status).toBe(200);

    const createChatBody = (await createChatRes.json()) as Chat;

    const webSocketConnect = await mf!.dispatchFetch(
      `http://localhost/api/connect/${createChatBody.id}?key=${resultBody.token}`,
      {
        headers: {
          Upgrade: "websocket",
        },
      }
    );

    const websocket = webSocketConnect.webSocket!;
    websocket?.accept();

    const webSocketData = {
      message: {
        user: username,
        contents: "Hello there",
      },
      message_type: "NewMessage",
    };

    let responseMessage: NewMessageResponseWrapper | undefined = undefined;

    let receivedMessages = 0;

    websocket?.addEventListener("message", (evt) => {
      console.log(evt.data);
      responseMessage = JSON.parse(evt.data as string);
      receivedMessages++;
    });

    // 2) Send a client message to the server
    websocket?.send(JSON.stringify(webSocketData));

    await new Promise((r) => setTimeout(r, 2000));

    // 3) Perform assertions on the response message that the client receives
    expect(responseMessage!.message.user).toBe(username);
    expect(responseMessage!.message.contents).toBe("Hello there");

    const secondUser = uuidv4();
    const secondUserPassword = uuidv4();

    await mf!.dispatchFetch("http://localhost/api/register", {
      method: "POST",
      body: JSON.stringify({
        username: secondUser,
        password: secondUserPassword,
      }),
      headers: {
        "Content-Type": "application/json",
      },
    });
    expect(res.status).toBe(200);

    const secondUserLogin = await mf!.dispatchFetch(
      "http://localhost/api/login",
      {
        method: "POST",
        body: JSON.stringify({
          username: secondUser,
          password: secondUserPassword,
        }),
        headers: {
          "Content-Type": "application/json",
        },
      }
    );

    expect(secondUserLogin.status).toBe(200);

    const secondUserResultBody =
      (await secondUserLogin.json()) as LoginResponse;

    await mf!.dispatchFetch(
      `http://localhost/api/connect/${createChatBody.id}?key=${secondUserResultBody.token}`,
      {
        headers: {
          Upgrade: "websocket",
        },
      }
    );

    const secondConnect = webSocketConnect.webSocket!;
    secondConnect?.accept();

    // 4) Close the client when everything is done
    websocket.close();

    await new Promise((r) => setTimeout(r, 2000));

    // First user connects
    // First user sends message
    // Second user connects
    // Second user disconnects
    // Total 4 messages expected on the open connection
    expect(receivedMessages).toBe(4);
  }, 10000);
});
