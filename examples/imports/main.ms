# Example using imports

from math import add, multiply

server:
    host "0.0.0.0"
    port 8080

fn calculate(x: int, y: int) -> int:
    let sum = add(x, y)
    let product = multiply(x, y)
    return sum + product

api GET /calc/{x:int}/{y:int}:
    return int
    let result = calculate(x, y)
    return result
