# Break and Continue example

server:
    host "0.0.0.0"
    port 8080

# Find first number divisible by both 3 and 5
fn find_divisible() -> int:
    for i in 1..100:
        if i % 3 == 0 and i % 5 == 0:
            return i
    return -1

# Sum only odd numbers, skip evens
fn sum_odd(limit: int) -> int:
    let mut total = 0
    for i in 0..limit:
        if i % 2 == 0:
            continue
        total = total + i
    return total

# Find first number greater than threshold
fn find_greater(threshold: int) -> int:
    let mut i = 0
    while true:
        i = i + 1
        if i > threshold:
            break
        if i > 1000:
            break
    return i

api GET /divisible:
    return int
    return find_divisible()

api GET /sum-odd/{limit:int}:
    return int
    return sum_odd(limit)

api GET /find/{threshold:int}:
    return int
    return find_greater(threshold)
