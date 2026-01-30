# Pattern matching example

server:
    host "0.0.0.0"
    port 8080

# Simple enum
enum Color:
    Red
    Green
    Blue

# Enum with data
enum Message:
    Quit
    Text(string)

fn get_color_name(c: int) -> string:
    let result = match c:
        1:
            return "red"
        2:
            return "green"
        3:
            return "blue"
        _:
            return "unknown"
    return result

api GET /color/{id:int}:
    return string
    let name = get_color_name(id)
    return name
