# WebSocket example - Real-time chat

server:
    host "0.0.0.0"
    port 8080

# Simple WebSocket endpoint
ws /chat:
    on_connect:
        conn.send("Welcome to the chat!")

    on_message:
        # Echo message back to sender
        conn.send(f"You said: {message}")

    on_disconnect:
        conn.send("Goodbye!")

# WebSocket with path parameters
ws /room/{room_id}:
    on_connect:
        conn.send(f"Welcome to room {room_id}!")

    on_message:
        # Broadcast to room
        conn.send(f"[{room_id}] {message}")

    on_disconnect:
        conn.send(f"User left room {room_id}")

# HTTP endpoint to check server status
api GET /status:
    return string
    return "WebSocket server running"
