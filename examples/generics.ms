# Generics example

server:
    host "0.0.0.0"
    port 8080

# Generic function that works with any type
fn identity<T>(value: T) -> T:
    return value

# Generic function with multiple type parameters
fn pair<T, U>(first: T, second: U) -> (T, U):
    return (first, second)

# Generic function with trait bounds
fn compare<T: Comparable>(a: T, b: T) -> bool:
    return a == b

api GET /test:
    return int
    let x = identity(42)
    return x
