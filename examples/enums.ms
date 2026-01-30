# Enum example

server:
    host "0.0.0.0"
    port 8080

# Simple enum
enum Color:
    Red
    Green
    Blue

# Enum with tuple data
enum Status:
    Success(int)
    Error(string)

# Enum with struct-style data
enum Message:
    Quit
    Move { x: int, y: int }
    Write(string)
    ChangeColor(int, int, int)

api GET /health:
    return string
    return "ok"
