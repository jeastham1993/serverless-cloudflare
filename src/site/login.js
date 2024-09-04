function login() {
    const username = $('#username').val();
    const password = $('#password').val();

    $.ajax({
        url: '/api/login',
        method: 'POST',
        contentType: 'application/json',
        data: JSON.stringify({ username, password }),
        success: function(response) {
            localStorage.setItem('username', username);
            localStorage.setItem('jwt', response);
            window.location.href = '/chats';
        },
        error: function(xhr, status, error) {
            alert('Login failed: ' + error);
        }
    });
}