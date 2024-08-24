let api_root = "";

$(document).ready(function () {
  refreshData();
});

function createChat() {
  const name = document.getElementById("chat_name").value;
  const chatPassword = document.getElementById("chat_password").value;

  if (name.length <= 0){
    alert('Name must not be empty');
    return;
  }

  if (chatPassword.length <= 0) {
    alert('Password must not be empty');
    return;
  }

  var xhr = new XMLHttpRequest();
  xhr.open("POST", `${api_root}/api/chats`, true);
  xhr.setRequestHeader("Content-Type", "application/json");
  xhr.send(
    JSON.stringify({
      name: name,
      password: chatPassword,
    })
  );
  xhr.onload = () => {
    if (xhr.readyState == 4 && xhr.status == 200) {
      refreshData();

      const data = JSON.parse(xhr.response);

      localStorage.setItem("chatroom_id", data.id);

      window.location = '/';
    } else {
      console.log(`Error: ${xhr.status}`);
    }
  };
}

function joinChat(chat_id) {
  localStorage.setItem("chatroom_id", chat_id);

  window.location = '/';
}

function refreshData() {
  // Load all chats
  var xhr = new XMLHttpRequest();
  xhr.open("GET", `${api_root}/api/chats`, true);
  xhr.setRequestHeader("Content-Type", "application/json");
  xhr.send();
  xhr.onload = () => {
    if (xhr.readyState == 4 && xhr.status == 200) {
      let tableBodyElement = document.getElementById("tableBody");
      tableBodyElement.innerHTML = "";

      const data = JSON.parse(xhr.response);
      console.log(data);
      data.forEach((chat) => {
        const chatId = chat.id;
        const chatName = chat.name;

        var rowElement = document.createElement("tr");
        var tableCellElement = document.createElement("td");
        tableCellElement.innerText = chatName;

        var button = document.createElement("button");
        button.innerText = "Join Chat";
        button.onclick = function () {
          joinChat(chatId);
        };

        var hrefElement = document.createElement("td");
        hrefElement.appendChild(button);

        rowElement.appendChild(tableCellElement);
        rowElement.appendChild(hrefElement);

        tableBodyElement.appendChild(rowElement);
      });
    } else {
      console.log(`Error: ${xhr.status}`);
    }
  };
}
