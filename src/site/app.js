const usernameLocalStorageKey = "username";
const chatRoomIdLocalStorageKey = "chatroom_id";
let isConnected = false;
let username = localStorage.getItem(usernameLocalStorageKey);
let chatroomId = localStorage.getItem(chatRoomIdLocalStorageKey);
let api_root = "";
let ws_root = "";
let messages = [];
let ws = undefined;

$(document).ready(function () {
  isConnected = false;

  if (username !== undefined && username !== null) {
    document.getElementById("username").value = username;
  } else {
    username = "";
  }

  checkForChatIdQueryParam();

  if (
    chatroomId === undefined ||
    chatroomId === null ||
    chatroomId.length <= 0
  ) {
    window.location = "/chats";
  }

  document
    .getElementById("message")
    .addEventListener("keydown", function (event) {
      if (event.key === "Enter") {
        sendmessage();
      }
    });
});

function checkForChatIdQueryParam() {
  const chatUrlParameters = new URLSearchParams(window.location.search);
  const chatIdParam = chatUrlParameters.get("chat_id");

  if (
    chatIdParam !== undefined &&
    chatIdParam !== null &&
    chatIdParam.length > 0
  ) {
    console.log('setting item');
    localStorage.setItem(chatRoomIdLocalStorageKey, chatIdParam);
    window.location = "/";
  }
}

function connectBtnClick() {
  if (isConnected) {
    disconnectWebsockets();
  } else {
    connectWebsockets();
  }
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

  const data = {
    message: {
      user: username,
      contents: messageContents,
    },
    message_type: "NewMessage",
  };

  ws.send(JSON.stringify(data));
  document.getElementById("message").value = "";
}

function updateConnectionStatus() {
  const connectionStatusText = document.getElementById("connectionStatus");
  const connectButton = document.getElementById("connectBtn");
  connectionStatusText.innerText = "";
  connectButton.innerText = "";

  if (isConnected) {
    connectionStatusText.innerText = "Connected!";
    connectButton.innerText = "Disconnect";
  } else {
    window.location = "/chats";
  }
}

function connectWebsockets() {
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

  //TODO: Update password to use 'tickets' instead.
  ws = new WebSocket(
    `${ws_root}/api/connect/${chatroomId}?user_id=${username}&password=${passwordInputValue}`
  );

  ws.onopen = () => {
    isConnected = true;
    updateConnectionStatus();
    localStorage.setItem(usernameLocalStorageKey, username);
    localStorage.setItem(chatRoomIdLocalStorageKey, chatroomId);
  };

  ws.onmessage = (message) => {
    const jsonMessageData = JSON.parse(message.data);
    switch (jsonMessageData.message_type) {
      case "NewMessage":
        handleNewMessage(jsonMessageData);
        break;
      case "ConnectionUpdate":
        handleConnectionUpdateMessage(jsonMessageData);
        break;
      case "ChatroomEnded":
        handleChatroomEndedMessage();
        break;
      case "MessageHistory":
        handleMessageHistoryMessage();
        break;
    }
  };

  ws.onclose = (e) => {
    isConnected = false;
    updateConnectionStatus();
  };

  ws.onerror = (e) => {
    console.log(e);
  };
}

function disconnectWebsockets() {
  const connectButton = document.getElementById("connectBtn");
  connectButton.innerText = "Disconnecting...";
  const connectionStatusText = document.getElementById("connectionStatus");
  connectionStatusText.innerText = "Disconnecting";

  ws.close(1000);
  return;
}

function handleNewMessage(jsonMessageData) {
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
}

function handleConnectionUpdateMessage(jsonMessageData) {
  const connectionsOnline = document.getElementById("connectionsOnline");
  connectionsOnline.innerText = `There are ${jsonMessageData.message.connection_count} users online`;

  let onlineUsers = "";

  console.log(jsonMessageData.message.online_users);

  jsonMessageData.message.online_users.forEach((user) => {
    if (user === username){
      user = "You";
    }
    
    if (onlineUsers === "") {
      onlineUsers = user;
    } else {
      onlineUsers = `${onlineUsers},${user}`;
    }
  });

  const activeUserTest = document.getElementById("activeUsers");
  activeUserTest.innerText = `Active users: ${onlineUsers}`;
}

function handleChatroomEndedMessage() {
  alert("Unfortunately, this chatroom has ended. Thankyou for chatting");
  window.location = "/chats";
}

function handleMessageHistoryMessage(jsonMessageData) {
  const messagesDiv = document.getElementById("messages");
  messagesDiv.innerHTML = "";

  if (jsonMessageData === undefined){
    return;
  }

  messages = jsonMessageData.message.history;

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
}
