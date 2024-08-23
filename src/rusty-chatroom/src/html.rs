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
    document.getElementById("username").value = username;
  }
  else {
    username = "";
  }

  if (chatroomId !== undefined && chatroomId !== null) {
    document.getElementById("chatroom_id").value = chatroomId;
  }
  else {
    chatroomId = "";
  }

  if (username.length > 0 && chatroomId.length > 0){
    connectWebsockets();
  }

  document.getElementById('message').addEventListener('keydown', function(event) {
      if (event.key === 'Enter') {
        sendmessage();
      }
  });
});

function connectToChat() {
  if (isConnected){
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
      document.getElementById("message").value = '';
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
    localStorage.setItem('last_chatroom_id', '');
    const connectionsOnline = document.getElementById("connectionsOnline");
    connectionsOnline.innerText = '';
    const activeUserTest = document.getElementById("activeUsers");
    activeUserTest.innerText = '';
  }
}

function connectWebsockets(){
  ws = new WebSocket(`${ws_root}/connect/${chatroomId}?user_id=${username}`);
  ws.onopen = () => {
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

    if (jsonMessageData.message_type === "NewMessage"){
      messages.push(jsonMessageData.message);

      const messagesDiv = document.getElementById("messages");
      messagesDiv.innerHTML = "";

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
    } else if (jsonMessageData.message_type === "ConnectionUpdate") {
      const connectionsOnline = document.getElementById("connectionsOnline");
      connectionsOnline.innerText = `There are ${jsonMessageData.message.connection_count} users online`;

      let onlineUsers = '';

      console.log(jsonMessageData.message.online_users);

      jsonMessageData.message.online_users.forEach((user) => {
        if (onlineUsers === '') {
          onlineUsers = user;
        } else {
          onlineUsers = `${onlineUsers},${user}`
         }
        
      });

      const activeUserTest = document.getElementById("activeUsers");
      activeUserTest.innerText = `Active users: ${onlineUsers}`;
    }
  };

  ws.onclose = () => {
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
        <p id="connectionsOnline"></p>
        <p id="activeUsers"></p>
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
