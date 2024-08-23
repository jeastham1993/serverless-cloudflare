pub fn load_css() -> String {
    let css_data = r#"
        .message-window{
    position: sticky;
    bottom: 0;
    left: 0;
    right: 0;
}

.messages{
    height: 60vh;
    overflow-x: scroll;
}"#;

    css_data.to_string()
}

pub fn load_js() -> String {
    let js_data = r#"
        let isConnected = false;
let username = localStorage.getItem("username");
let chatroomId = localStorage.getItem("last_chatroom_id");
let api_root = "";
let ws_root = "";
let messages = [];
let ws = undefined;

$(document).ready(function () {
  isConnected = false;
  updateConnectionStatus();

  if (username !== undefined && username !== null) {
    console.log('Username found in storage');
    document.getElementById("username").value = username;
  }
  else {
    console.log('Username not found in storage');
    username = "";
  }

  if (chatroomId !== undefined && chatroomId !== null) {
    console.log('ChatroomID found in storage');
    document.getElementById("chatroom_id").value = chatroomId;
  }
  else {
    console.log('ChatroomID not found in storage');
    chatroomId = "";
  }

  if (username.length > 0 && chatroomId.length > 0){
    console.log('Attempting auto-connect');
    connectWebsockets();
  }
});

function connectToChat() {
  if (isConnected){
    console.log('Closing connection');
    const connectButton = document.getElementById('connectBtn');
    connectButton.innerText = "Disconnecting..."
    const connectionStatusText = document.getElementById("connectionStatus");
    connectionStatusText.innerText = "Disconnecting";

    ws.close();
    document.getElementById("chatroom_id").value = "";
    return;
  }

  const userNameInputValue = document.getElementById("username").value;
  const chatroomIdInputValue = document.getElementById("chatroom_id").value;

  if (userNameInputValue.length <= 0) {
    alert("User name must not be empty.");
    return;
  }

  if (chatroomIdInputValue.length <= 0) {
    alert("Chatroom ID must not be empty.");
    return;
  }

  username = userNameInputValue;
  chatroomId = chatroomIdInputValue;

  connectWebsockets();
}

function sendmessage() {
  if (!isConnected) {
    alert("Please connect first");

    return;
  }

  let messageContents = document.getElementById("message").value;

  if (messageContents.length <= 0) {
    alert("Message must not be empty");
    return;
  }

  var xhr = new XMLHttpRequest();
  xhr.open("POST", `${api_root}/message/${chatroomId}`, true);
  xhr.setRequestHeader("Content-Type", "application/json");
  xhr.send(
    JSON.stringify({
      user: username,
      contents: messageContents,
    })
  );
  xhr.onload = () => {
    if (xhr.readyState == 4 && xhr.status == 200) {
      const data = xhr.response;
    } else {
      console.log(`Error: ${xhr.status}`);
    }
  };
}

function updateConnectionStatus() {
  const connectionStatusText = document.getElementById("connectionStatus");
  const connectButton = document.getElementById('connectBtn');
  connectionStatusText.innerText = "";
  connectButton.innerText = "";

  if (isConnected) {
    connectionStatusText.innerText = "Connected!";
    connectButton.innerText = "Disconnect";
  }
  else {
    connectionStatusText.innerText = "Disconnected...";
    connectButton.innerText = "Connect";
  }
}

function connectWebsockets(){
  ws = new WebSocket(`${ws_root}/connect/${chatroomId}`);
  ws.onopen = () => {
    console.log("ws opened on browser");
    isConnected = true;
    updateConnectionStatus();
    localStorage.setItem('username', username);
    localStorage.setItem('last_chatroom_id', chatroomId);

    var xhr = new XMLHttpRequest();
    xhr.open("GET", `${api_root}/message/${chatroomId}`, true);
    xhr.setRequestHeader("Content-Type", "application/json");
    xhr.send();

    xhr.onload = () => {
      if (xhr.readyState == 4 && xhr.status == 200) {
        const messagesDiv = document.getElementById("messages");
        messagesDiv.innerHTML = "";

        const data = xhr.response;
        messages = JSON.parse(data);

        console.log(messages);

        messages.forEach((message) => {
          let user = message.user;

          if (user === username) {
            user = "You";
          }

          var element = document.createElement("div");
          element.appendChild(
            document.createTextNode(`${user} said: ${message.contents}`)
          );
          messagesDiv.appendChild(element);
        });
      } else {
        console.log(`Error: ${xhr.status}`);
      }
    };
  };

  ws.onmessage = (message) => {
    const jsonMessageData = JSON.parse(message.data);
    messages.push(jsonMessageData);

    const messagesDiv = document.getElementById("messages");
    messagesDiv.innerHTML = "";

    console.log(messages);

    messages.forEach((message) => {
      let user = message.user;

      if (user === username) {
        user = "You";
      }

      var element = document.createElement("div");
      element.appendChild(
        document.createTextNode(`${user} said: ${message.contents}`)
      );
      messagesDiv.appendChild(element);
    });
  };

  ws.onclose = () => {
    console.log("Closing connection");
    isConnected = false;
    updateConnectionStatus();
  };
}
"#;

js_data.to_string()
}


pub fn load_html() -> String {
    let html_data = r#"<!DOCTYPE html>
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="color-scheme" content="light dark" />
    <title>Rusty Chat</title>
    <meta name="description" content="A chat app built with Rust and Cloudflare." />

    <!-- Pico.css -->
    <link
      rel='stylesheet'
      href="https://cdn.jsdelivr.net/npm/@picocss/pico@2.0.6/css/pico.min.css"
    />
    <link
        rel="stylesheet"
        href="style.css"
    />
  </head>
    <script src="https://code.jquery.com/jquery-3.7.1.min.js" integrity="sha256-/JqT3SQfawRcv/BIHPThkBvs0OEvtFFmqPF/lYI/Cxo=" crossorigin="anonymous"></script>
    <script type="text/javascript" src="app.js"></script>
  </head>
  <body>
    <header class="container">
      <hgroup>
        <h1>Rusty Chat</h1>
        <p id="connectionStatus">Disconnected...</p>
      </hgroup>
    </header>
    <main class="container">
        <div class="grid">
          <input id="chatroom_id" type="text"
            name="chatroom_id"
            placeholder="ChatroomID"
            aria-label="ChatroomID"
            required/>
            <input id="username" type="text"
            name="username"
            placeholder="User name"
            aria-label="User name"
            required/>
            <button id="connectBtn" onclick="connectToChat()">Connect</button>
        </div>
        <div class="messages" id="messages">

        </div>
        <div class="grid message-window">
            <input id="message" 
                type="text"
                name="message"
                placeholder="Message"
                aria-label="Message"
                required/>
            <button onclick="sendmessage()">Send</button>
        </div>
    </main>
  </body>
</html>
"#;

    html_data.to_string()
}