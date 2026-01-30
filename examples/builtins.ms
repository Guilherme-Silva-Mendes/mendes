# Built-in functions test

server:
    host "0.0.0.0"
    port 8080

api GET /testprint:
    return string
    println("Hello from Mendes!")
    return "ok"

api GET /testlen/{text:string}:
    return int
    let size: int = len(text)
    return size

api GET /teststr/{n:int}:
    return string
    let result: string = str(n)
    return result
