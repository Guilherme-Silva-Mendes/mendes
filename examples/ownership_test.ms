# Ownership test - should generate error

server:
    host "0.0.0.0"
    port 8080

fn use_string(s: string) -> string:
    return s

api GET /ownership:
    return string
    let data: string = "hello"
    let result: string = use_string(data)
    return data
