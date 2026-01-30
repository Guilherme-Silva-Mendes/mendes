# Ranges example

server:
    host "0.0.0.0"
    port 8080

# Sum numbers in a range
fn sum_range(start: int, end: int) -> int:
    let mut total = 0
    for i in start..end:
        total = total + i
    return total

# Inclusive range
fn sum_inclusive(start: int, end: int) -> int:
    let mut total = 0
    for i in start..=end:
        total = total + i
    return total

# Find first even number in range
fn first_even(start: int, end: int) -> Option<int>:
    for i in start..end:
        if i % 2 == 0:
            return Some(i)
    return None

api GET /sum/{start:int}/{end:int}:
    return int
    return sum_range(start, end)

api GET /sum-inclusive/{start:int}/{end:int}:
    return int
    return sum_inclusive(start, end)

api GET /first-even/{start:int}/{end:int}:
    return string
    match first_even(start, end):
        Some(n):
            return f"First even: {n}"
        None:
            return "No even number found"
