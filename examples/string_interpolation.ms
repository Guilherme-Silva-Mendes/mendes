# String interpolation example

server:
    host "0.0.0.0"
    port 8080

fn greet(name: string) -> string:
    return f"Hello, {name}!"

fn format_user(name: string, age: int) -> string:
    return f"User: {name}, Age: {age}"

api GET /greet/{name}:
    return string
    let message = f"Welcome, {name}!"
    return message

api GET /user/{name}/{age:int}:
    return string
    return format_user(name, age)
