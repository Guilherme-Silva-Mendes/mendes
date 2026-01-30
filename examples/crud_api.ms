# CRUD API completa em Mendes

module users

server:
    host "0.0.0.0"
    port 8080

db postgres main:
    url "postgres://user:pass@localhost:5432/app"
    pool 20

struct User:
    id: int
    name: string
    email: string
    active: bool

struct CreateUserInput:
    name: string
    email: string

middleware auth:
    let token = request.header("Authorization")
    if token is None:
        return HttpError(401, "unauthorized")

api GET /health:
    return string
    return "ok"

api GET /users async:
    use auth
    return Result<[User], HttpError>
    let users = await db.main.query("SELECT * FROM users")
    return Ok(users)

api GET /users/{id:int} async:
    use auth
    return Result<User, HttpError>
    if id <= 0:
        return Err(HttpError(400, "invalid id"))
    let user = await db.main.query_one("SELECT * FROM users WHERE id = $1", id)
    return Ok(user)

api POST /users async:
    use auth
    body CreateUserInput
    return Result<User, HttpError>
    let user = await db.main.query_one("INSERT INTO users VALUES ($1, $2)", body.name, body.email)
    return Ok(user)

api DELETE /users/{id:int} async:
    use auth
    return Result<bool, HttpError>
    if id <= 0:
        return Err(HttpError(400, "invalid id"))
    await db.main.execute("DELETE FROM users WHERE id = $1", id)
    return Ok(true)

fn validate_email(email: &string) -> bool:
    return email.contains("@")

fn activate_user(user: &mut User):
    user.active = true
