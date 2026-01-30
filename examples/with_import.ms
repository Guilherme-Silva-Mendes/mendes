# Test file with imports
import "utils.ms"

server:
    host "0.0.0.0"
    port 8080

api GET /hello/{name:string}:
    return string
    return greet(name)
