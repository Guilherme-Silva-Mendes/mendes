# Simple example without struct methods (for now)
# Struct methods are parsed but need more testing

server:
    host "0.0.0.0"
    port 8080

struct Point:
    x: int
    y: int

fn distance(p: Point) -> int:
    return p.x + p.y

api GET /point:
    return int
    let p: Point = Point { x: 3, y: 4 }
    return distance(p)
