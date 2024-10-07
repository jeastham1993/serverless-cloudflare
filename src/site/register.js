function register() {
    const username = $('#username').val();
    const password = $('#password').val();
    const email = $('#emailAddress').val();

    $.ajax({
        url: '/api/register',
        method: 'POST',
        contentType: 'application/json',
        data: JSON.stringify({ username, password, email }),
        success: function(response) {
            alert('Registration successful. Please login.');
            window.location.href = '/login';
        },
        error: function(xhr, status, error) {
            alert('Registration failed: ' + error);
        }
    });
}