# Tuples example

server:
    host "0.0.0.0"
    port 8080

# Function returning a tuple
fn get_coordinates() -> (int, int):
    return (10, 20)

# Function with tuple parameter
fn sum_tuple(coords: (int, int)) -> int:
    let (x, y) = coords
    return x + y

# Tuple with multiple types
fn get_user_info() -> (string, int, bool):
    return ("Alice", 30, true)

# Empty tuple (unit type)
fn do_nothing() -> ():
    return ()

api GET /coords:
    return int
    let coords = get_coordinates()
    return sum_tuple(coords)

api GET /user-info:
    return string
    let (name, age, active) = get_user_info()
    return f"{name} is {age} years old, active: {active}"
