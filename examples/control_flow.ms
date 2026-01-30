# Control flow test

server:
    host "0.0.0.0"
    port 8080

api GET /check/{n:int}:
    return string
    if n > 0:
        return "positive"
    else:
        return "not positive"
