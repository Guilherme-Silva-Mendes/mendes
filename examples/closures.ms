# Closure examples

server:
    host "0.0.0.0"
    port 8080

# Simple closure
fn use_closure() -> int:
    let add = |x: int, y: int| x + y
    return add(5, 3)

# Closure with type annotation
fn typed_closure() -> int:
    let double = |n: int| -> int: n * 2
    return double(10)

# Higher-order function
fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn test_higher_order() -> int:
    let square = |n: int| n * n
    return apply(square, 5)

api GET /closure:
    return int
    let result = use_closure()
    return result

api GET /higher:
    return int
    let result = test_higher_order()
    return result
