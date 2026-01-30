# Traits example

server:
    host "0.0.0.0"
    port 8080

# Define a trait
trait Printable:
    fn print(&self) -> string

# Define a struct
struct User:
    name: string
    age: int

# Implement the trait for User
impl Printable for User:
    fn print(&self) -> string:
        return f"User: {self.name}, age {self.age}"

# Another trait example
trait Comparable:
    fn equals(&self, other: &self) -> bool

struct Point:
    x: int
    y: int

impl Comparable for Point:
    fn equals(&self, other: &Point) -> bool:
        return self.x == other.x and self.y == other.y

api GET /user:
    return string
    let user = User { name: "Alice", age: 30 }
    return user.print()
