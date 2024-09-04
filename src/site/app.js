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

  checkForChatIdQueryParam();

  if (
    chatroomId === undefined ||
    chatroomId === null ||
    chatroomId.length <= 0
  ) {
    window.location = "/chats";
  }

  connectWebsockets();

  document
    .getElementById("message")
    .addEventListener("keydown", function (event) {
      if (event.key === "Enter") {
        sendmessage();
      }
    });
});

// Update your AJAX calls to include the JWT and handle 401 errors
$.ajaxSetup({
  beforeSend: function(xhr) {
      const jwt = localStorage.getItem('jwt');
      if (jwt) {
          xhr.setRequestHeader('Authorization', 'Bearer ' + jwt);
      }
  },
  statusCode: {
      401: handleUnauthorized
  }
});

function logout() {
  localStorage.removeItem('jwt');
  window.location.href = '/login';
}

function handleUnauthorized() {
  localStorage.removeItem('jwt');
  window.location.href = '/login';
}

function checkForChatIdQueryParam() {
  const chatUrlParameters = new URLSearchParams(window.location.search);
  const chatIdParam = chatUrlParameters.get("chat_id");

  if (
    chatIdParam !== undefined &&
    chatIdParam !== null &&
    chatIdParam.length > 0
  ) {
    console.log("setting item");
    localStorage.setItem(chatRoomIdLocalStorageKey, chatIdParam);
    window.location = "/";
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
  ws = new WebSocket(
    `${ws_root}/api/connect/${chatroomId}?key=${localStorage.getItem('jwt')}`
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
        handleMessageHistoryMessage(jsonMessageData);
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

function leaveRoom() {
  window.location = "/chats";
}

function handleNewMessage(jsonMessageData) {
  messages.push(jsonMessageData.message);

  refreshMessages();
}

function handleConnectionUpdateMessage(jsonMessageData) {
  const connectionsOnline = document.getElementById("connectionsOnline");
  connectionsOnline.innerText = `There are ${jsonMessageData.message.connection_count} users online`;

  let onlineUsers = "";

  console.log(jsonMessageData.message.online_users);

  jsonMessageData.message.online_users.forEach((user) => {
    if (user === username) {
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
  console.log(jsonMessageData);
  if (jsonMessageData === undefined) {
    return;
  }

  messages = jsonMessageData.message.history;

  refreshMessages();
}

function refreshMessages() {
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
