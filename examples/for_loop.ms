# For loop example

server:
    host "0.0.0.0"
    port 8080

# Function with for loop over range
fn sum_range(n: int) -> int:
    let mut total: int = 0
    for i in 0..n:
        total = total + i
    return total

# Function with inclusive range
fn sum_inclusive(n: int) -> int:
    let mut total: int = 0
    for i in 0..=n:
        total = total + i
    return total

api GET /sum/{n:int}:
    return int
    let result = sum_range(n)
    return result
