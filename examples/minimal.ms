# Minimal Mendes example

server:
    host "0.0.0.0"
    port 8080

api GET /health:
    return string
    return "ok"

api GET /hello:
    return string
    return "Hello, World!"
