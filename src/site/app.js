let isConnected = false;
let username = localStorage.getItem("username");
let chatroomId = localStorage.getItem("last_chatroom_id");
let api_root = "";
let ws_root = "";
let messages = [];
let ws = undefined;

$(document).ready(function () {
  isConnected = false;

  if (username !== undefined && username !== null) {
    document.getElementById("username").value = username;
  }
  else {
    username = "";
  }

  if (chatroomId === undefined || chatroomId === null || chatroomId.length <= 0) {
    window.location = '/chats';
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

    ws.close(1000);
    return;
  }

  const userNameInputValue = document.getElementById("username").value;
  const passwordInputValue = document.getElementById("password").value;

  if (userNameInputValue.length <= 0) {
    alert("User name must not be empty.");
    return;
  }

  if (passwordInputValue.length <= 0) {
    alert("Password must not be empty");
    return;
  }

  username = userNameInputValue;

  connectWebsockets(passwordInputValue);
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
  xhr.open("POST", `${api_root}/api/message/${chatroomId}`, true);
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
    window.location ='/chats';
  }
}

function connectWebsockets(password){
  ws = new WebSocket(`${ws_root}/api/connect/${chatroomId}?user_id=${username}&password=${password}`);
  ws.onopen = () => {
    isConnected = true;
    updateConnectionStatus();
    localStorage.setItem('username', username);
    localStorage.setItem('last_chatroom_id', chatroomId);

    var xhr = new XMLHttpRequest();
    xhr.open("GET", `${api_root}/api/message/${chatroomId}`, true);
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
            document.createTextNode(`${user}: ${message.contents}`)
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
          document.createTextNode(`${user}: ${message.contents}`)
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

  ws.onclose = (e) => {
    console.log(e);

    isConnected = false;
    updateConnectionStatus();
  };

  ws.onerror = (e) => {
    console.log(e);
  }
}