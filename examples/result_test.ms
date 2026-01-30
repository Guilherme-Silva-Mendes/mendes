# Result error handling test

server:
    host "0.0.0.0"
    port 8080

fn divide(a: int, b: int) -> Result<int, string>:
    if b == 0:
        return Err("division by zero")
    return Ok(a / b)

api GET /divide/{a:int}/{b:int}:
    return string
    let result: Result<int, string> = divide(a, b)
    return str(result)
