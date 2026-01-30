# Try operator (?) example

server:
    host "0.0.0.0"
    port 8080

fn divide(a: int, b: int) -> Result<int, string>:
    if b == 0:
        return Err("division by zero")
    return Ok(a / b)

fn calculate(x: int) -> Result<int, string>:
    let result = divide(x, 2)?
    return Ok(result + 10)

api GET /calc/{num:int}:
    return Result<int, string>
    let result = calculate(num)?
    return Ok(result)
