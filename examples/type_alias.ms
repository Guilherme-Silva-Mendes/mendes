# Type aliases example

server:
    host "0.0.0.0"
    port 8080

# Simple type aliases
type UserId = int
type Username = string
type Email = string

# Generic type alias
type UserResult = Result<User, string>

struct User:
    id: UserId
    name: Username
    email: Email

fn get_user(id: UserId) -> UserResult:
    if id > 0:
        let user = User { id: id, name: "Alice", email: "alice@example.com" }
        return Ok(user)
    return Err("Invalid user ID")

api GET /user/{id:int}:
    return string
    let result = get_user(id)
    match result:
        Ok(user):
            return f"Found user: {user.name}"
        Err(msg):
            return f"Error: {msg}"
