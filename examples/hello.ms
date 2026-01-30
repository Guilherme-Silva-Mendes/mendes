# Primeiro programa em Mendes

server:
    host "0.0.0.0"
    port 8080

db postgres main:
    url "postgres://localhost/mendes"
    pool 10

struct User:
    id: int
    name: string
    email: string

api GET /health:
    return string
    return "ok"

api GET /users/{id:int} async:
    return User
    let user = await db.main.query_one("SELECT * FROM users WHERE id = $1", id)
    return user

fn create_greeting(name: string) -> string:
    return "Hello, " + name + "!"
