# Codegen test for generics and traits

server:
    host "0.0.0.0"
    port 8080

# Type alias
type UserId = int

# Generic struct
struct Pair<T, U>:
    first: T
    second: U

# Simple trait
trait Printable:
    fn to_string(&self) -> string

# Simple struct
struct User:
    id: int
    name: string

# Implement trait for User
impl Printable for User:
    fn to_string(&self) -> string:
        return "User"
