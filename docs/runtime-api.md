# API do Runtime Mendes

> **Versao**: 0.1.0
> **Crate**: `mendes-runtime`

Este documento descreve a API do runtime Mendes, incluindo HTTP, banco de dados e WebSockets.

---

## Indice

1. [Visao Geral](#1-visao-geral)
2. [HTTP Server](#2-http-server)
3. [Request](#3-request)
4. [Response](#4-response)
5. [Router](#5-router)
6. [Middleware](#6-middleware)
7. [Database](#7-database)
8. [WebSocket](#8-websocket)
9. [Tipos Utilitarios](#9-tipos-utilitarios)
10. [Tratamento de Erros](#10-tratamento-de-erros)

---

## 1. Visao Geral

O runtime Mendes fornece:

| Modulo | Descricao |
|--------|-----------|
| `http` | Servidor HTTP de alta performance |
| `router` | Roteamento de requisicoes |
| `database` | Pool de conexoes para bancos SQL |
| `websocket` | Suporte a WebSocket |
| `types` | Tipos utilitarios (MendesString, MendesArray, etc.) |
| `error` | Tipos de erro padrao |

### Arquitetura

```
┌─────────────────────────────────────────────┐
│                  mendes-runtime             │
├─────────────────────────────────────────────┤
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │  HTTP   │  │ Router  │  │  Database   │  │
│  │ Server  │◄─┤         ├──┤    Pool     │  │
│  └─────────┘  └─────────┘  └─────────────┘  │
│       │            │              │         │
│       ▼            ▼              ▼         │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐  │
│  │ Request │  │Middleware│  │  WebSocket  │  │
│  │Response │  │  Chain  │  │  Handler    │  │
│  └─────────┘  └─────────┘  └─────────────┘  │
├─────────────────────────────────────────────┤
│                   Tokio                     │
└─────────────────────────────────────────────┘
```

---

## 2. HTTP Server

### Server

O servidor HTTP principal.

#### Criacao

```mendes
# Na linguagem Mendes
server:
    host "0.0.0.0"
    port 8080
```

```rust
// No Rust gerado
use mendes_runtime::Server;

let server = Server::new("0.0.0.0:8080");
```

#### Metodos

| Metodo | Descricao | Retorno |
|--------|-----------|---------|
| `new(addr)` | Cria servidor no endereco | `Server` |
| `router(router)` | Define o router | `Self` |
| `run()` | Inicia o servidor (async) | `Result<()>` |

#### Exemplo Completo

```rust
use mendes_runtime::{Server, Router, Request, Response};

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    router.get("/health", |_req| async {
        Response::ok("healthy")
    });

    Server::new("0.0.0.0:8080")
        .router(router)
        .run()
        .await
        .unwrap();
}
```

---

## 3. Request

Representa uma requisicao HTTP.

### Propriedades

| Propriedade | Tipo | Descricao |
|-------------|------|-----------|
| `method` | `string` | Metodo HTTP (GET, POST, etc.) |
| `path` | `string` | Caminho da requisicao |
| `query_string` | `string` | Query string completa |
| `headers` | `Headers` | Cabecalhos HTTP |
| `body` | `bytes` | Corpo da requisicao |

### Metodos

#### header(name) -> Option<string>

Obtem um cabecalho pelo nome.

```mendes
let auth = request.header("Authorization")
match auth:
    Some(token):
        validate(token)
    None:
        return Response { status: 401 }
```

#### param(name) -> Option<string>

Obtem um parametro de rota.

```mendes
# Rota: /users/{id}
api GET /users/{id:int}:
    let id = request.param("id")   # Automatico via path
```

#### query(name) -> Option<string>

Obtem um parametro de query string.

```mendes
# URL: /users?page=2&limit=10
let page = request.query("page").unwrap_or("1")
let limit = request.query("limit").unwrap_or("20")
```

#### body() -> bytes

Obtem o corpo da requisicao.

```mendes
let raw_body = request.body()
```

#### json<T>() -> Result<T, Error>

Deserializa o corpo como JSON.

```mendes
struct CreateUser:
    name: string
    email: string

api POST /users:
    let user: CreateUser = request.json()?
```

#### form<T>() -> Result<T, Error>

Deserializa o corpo como form data.

```mendes
let data = request.form<FormData>()?
```

#### set(key, value)

Define um valor no contexto da requisicao (para middleware).

```mendes
middleware auth:
    # ...
    request.set("user_id", user_id)
```

#### get<T>(key) -> Option<T>

Obtem um valor do contexto da requisicao.

```mendes
api GET /profile:
    use auth
    let user_id = request.get<int>("user_id")
```

---

## 4. Response

Representa uma resposta HTTP.

### Construtores

| Metodo | Descricao | Status |
|--------|-----------|--------|
| `ok(body)` | Sucesso | 200 |
| `created(body)` | Recurso criado | 201 |
| `no_content()` | Sem conteudo | 204 |
| `bad_request(msg)` | Erro do cliente | 400 |
| `unauthorized(msg)` | Nao autorizado | 401 |
| `forbidden(msg)` | Proibido | 403 |
| `not_found(msg)` | Nao encontrado | 404 |
| `internal_error(msg)` | Erro interno | 500 |

### Exemplos

```mendes
# Sucesso simples
return Response::ok("Hello, World!")

# JSON
return Response::ok(user)   # Serializa automaticamente

# Criado com location
return Response::created(user)
    .header("Location", f"/users/{user.id}")

# Erro
return Response::not_found("User not found")
```

### Struct Response

```mendes
# Construcao manual
return Response {
    status: 201,
    body: user
}

# Com sintaxe de bloco
return Response:
    status 201
    header "Location" f"/users/{id}"
    header "X-Request-Id" request_id
    body user
```

### Metodos

#### status(code) -> Self

Define o codigo de status.

```mendes
Response::ok(data).status(201)
```

#### header(name, value) -> Self

Adiciona um cabecalho.

```mendes
Response::ok(data)
    .header("Content-Type", "application/json")
    .header("Cache-Control", "no-cache")
```

#### json(data) -> Self

Define o corpo como JSON.

```mendes
Response::ok(()).json(user)
```

---

## 5. Router

Gerencia o roteamento de requisicoes.

### Criacao

```rust
let mut router = Router::new();
```

### Metodos de Rota

| Metodo | Descricao |
|--------|-----------|
| `get(path, handler)` | Registra rota GET |
| `post(path, handler)` | Registra rota POST |
| `put(path, handler)` | Registra rota PUT |
| `delete(path, handler)` | Registra rota DELETE |
| `patch(path, handler)` | Registra rota PATCH |

### Sintaxe de Path

| Pattern | Descricao | Exemplo |
|---------|-----------|---------|
| `/static` | Caminho literal | `/users` |
| `/:param` | Parametro de rota | `/users/:id` |
| `/*rest` | Wildcard | `/files/*path` |

### Exemplo

```rust
let mut router = Router::new();

// Rota simples
router.get("/health", |_| async {
    Response::ok("ok")
});

// Com parametro
router.get("/users/:id", |req| async move {
    let id = req.param("id").unwrap();
    Response::ok(format!("User {}", id))
});

// Wildcard
router.get("/files/*path", |req| async move {
    let path = req.param("path").unwrap();
    serve_file(path).await
});
```

### Grupos de Rotas

```rust
// API v1
router.group("/api/v1", |r| {
    r.get("/users", list_users);
    r.post("/users", create_user);
    r.get("/users/:id", get_user);
});

// API v2
router.group("/api/v2", |r| {
    r.get("/users", list_users_v2);
});
```

---

## 6. Middleware

Funcoes que interceptam requisicoes/respostas.

### Definicao em Mendes

```mendes
middleware auth:
    let token = request.header("Authorization")
    if token is None:
        return Response::unauthorized("Token required")

    match validate_token(token):
        Ok(user_id):
            request.set("user_id", user_id)
        Err(e):
            return Response::unauthorized(f"Invalid token: {e}")

middleware log:
    let start = time.now()
    let response = next()
    let duration = time.since(start)
    print(f"{request.method} {request.path} -> {response.status} ({duration}ms)")
    return response
```

### Usando Middleware

```mendes
api GET /admin/users:
    use auth
    use log
    return [User]
    # ...
```

### Ordem de Execucao

```
Request -> log -> auth -> handler -> auth -> log -> Response
           |       |        |         |       |
           v       v        v         v       v
         antes   antes    exec    depois  depois
```

### Middleware Comum

#### Rate Limiting

```mendes
middleware rate_limit:
    let ip = request.header("X-Forwarded-For").unwrap_or(request.ip())
    let count = cache.increment(f"rate:{ip}", 1, 60)

    if count > 100:
        return Response:
            status 429
            header "Retry-After" "60"
            body "Too many requests"
```

#### CORS

```mendes
middleware cors:
    let response = next()
    return response
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE")
        .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
```

#### Compressao

```mendes
middleware compress:
    let response = next()
    if response.body.len() > 1024:
        let compressed = gzip(response.body)
        return response
            .body(compressed)
            .header("Content-Encoding", "gzip")
    return response
```

---

## 7. Database

Pool de conexoes para bancos de dados SQL.

### Configuracao

```mendes
db postgres main:
    url "postgres://user:pass@localhost:5432/mydb"
    pool 20

db mysql secondary:
    url "mysql://user:pass@localhost:3306/mydb"
    pool 10

db sqlite local:
    url "sqlite://./data.db"
    pool 5
```

### Pool Methods

#### query(sql, params...) -> [Row]

Executa query e retorna multiplas linhas.

```mendes
let users = await db.main.query(
    "SELECT * FROM users WHERE active = $1",
    true
)
```

#### query_one(sql, params...) -> Option<Row>

Executa query e retorna uma linha.

```mendes
let user = await db.main.query_one(
    "SELECT * FROM users WHERE id = $1",
    id
)
```

#### execute(sql, params...) -> ExecuteResult

Executa SQL sem retorno de dados.

```mendes
let result = await db.main.execute(
    "DELETE FROM users WHERE id = $1",
    id
)
print(f"Deleted {result.rows_affected} rows")
```

### Transacoes

```mendes
await db.main.transaction:
    await db.main.execute(
        "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
        amount, from_id
    )
    await db.main.execute(
        "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
        amount, to_id
    )
```

### Parametros

| Banco | Sintaxe de Parametro |
|-------|---------------------|
| PostgreSQL | `$1`, `$2`, `$3`... |
| MySQL | `?` |
| SQLite | `?` |

### Tipos Suportados

| Mendes | PostgreSQL | MySQL | SQLite |
|--------|------------|-------|--------|
| `int` | INTEGER, BIGINT | INT, BIGINT | INTEGER |
| `float` | REAL, DOUBLE | DOUBLE | REAL |
| `string` | VARCHAR, TEXT | VARCHAR, TEXT | TEXT |
| `bool` | BOOLEAN | TINYINT(1) | INTEGER |
| `bytes` | BYTEA | BLOB | BLOB |

### Exemplo Completo

```mendes
struct User:
    id: int
    name: string
    email: string
    created_at: string

api GET /users async:
    return [User]
    let users = await db.main.query("SELECT * FROM users ORDER BY id")
    return users

api GET /users/{id:int} async:
    return Result<User, string>
    let user = await db.main.query_one("SELECT * FROM users WHERE id = $1", id)
    match user:
        Some(u):
            return Ok(u)
        None:
            return Err("User not found")

api POST /users async:
    body CreateUser
    return User
    let user = await db.main.query_one(
        "INSERT INTO users (name, email, created_at) VALUES ($1, $2, NOW()) RETURNING *",
        body.name, body.email
    )
    return user
```

---

## 8. WebSocket

Suporte a comunicacao bidirecional em tempo real.

### Definicao

```mendes
ws /chat:
    on_connect:
        # Executado quando cliente conecta
        print(f"Cliente {conn.id} conectou")

    on_message:
        # Executado quando recebe mensagem
        print(f"Mensagem: {message}")
        conn.send(f"Echo: {message}")

    on_disconnect:
        # Executado quando cliente desconecta
        print(f"Cliente {conn.id} desconectou")
```

### Connection Object

#### Propriedades

| Propriedade | Tipo | Descricao |
|-------------|------|-----------|
| `id` | `string` | ID unico da conexao |

#### Metodos

| Metodo | Descricao |
|--------|-----------|
| `send(msg)` | Envia mensagem para esta conexao |
| `close()` | Fecha a conexao |
| `set_state(data)` | Define estado customizado |
| `get_state<T>()` | Obtem estado customizado |

### Broadcast

```mendes
ws /chat:
    on_message:
        # Envia para todas as conexoes
        broadcast(message)

        # Envia para conexoes em uma sala
        broadcast_room("general", message)
```

### Rooms

```mendes
ws /chat/{room:string}:
    on_connect:
        join_room(room)
        broadcast_room(room, f"{conn.id} entrou")

    on_message:
        broadcast_room(room, f"[{conn.id}]: {message}")

    on_disconnect:
        broadcast_room(room, f"{conn.id} saiu")
        leave_room(room)
```

### Estado por Conexao

```mendes
struct ChatUser:
    nickname: string
    joined_at: string

ws /chat:
    on_connect:
        let user = ChatUser {
            nickname: f"User{conn.id}",
            joined_at: time.now()
        }
        conn.set_state(user)
        broadcast(f"{user.nickname} entrou no chat")

    on_message:
        let user = conn.get_state<ChatUser>()
        broadcast(f"[{user.nickname}]: {message}")

    on_disconnect:
        let user = conn.get_state<ChatUser>()
        broadcast(f"{user.nickname} saiu do chat")
```

### Exemplo: Chat Completo

```mendes
server:
    host "0.0.0.0"
    port 8080

struct ChatMessage:
    sender: string
    content: string
    timestamp: string

ws /chat:
    on_connect:
        let history = get_recent_messages(50)
        for msg in history:
            conn.send(msg.to_json())
        broadcast(f"[Sistema] Novo usuario conectado: {conn.id}")

    on_message:
        let msg = ChatMessage {
            sender: conn.id,
            content: message,
            timestamp: time.now()
        }
        save_message(msg)
        broadcast(msg.to_json())

    on_disconnect:
        broadcast(f"[Sistema] Usuario {conn.id} saiu")

# Endpoint para enviar mensagem via HTTP
api POST /chat/send:
    body ChatMessage
    broadcast_to_ws("/chat", body.to_json())
    return Response::ok("sent")
```

---

## 9. Tipos Utilitarios

### MendesString

String UTF-8 com metodos uteis.

```mendes
let s = "Hello, World!"

# Metodos
s.len()                    # 13
s.is_empty()               # false
s.contains("World")        # true
s.starts_with("Hello")     # true
s.ends_with("!")           # true
s.to_uppercase()           # "HELLO, WORLD!"
s.to_lowercase()           # "hello, world!"
s.trim()                   # Remove espacos
s.split(",")               # ["Hello", " World!"]
s.replace("World", "Mendes")  # "Hello, Mendes!"
```

### MendesArray<T>

Array dinamico.

```mendes
let arr = [1, 2, 3, 4, 5]

# Metodos
arr.len()                  # 5
arr.is_empty()             # false
arr.push(6)                # Adiciona ao final
arr.pop()                  # Remove do final
arr.first()                # Some(1)
arr.last()                 # Some(5)
arr.get(2)                 # Some(3)
arr.contains(3)            # true
arr.reverse()              # [5, 4, 3, 2, 1]
arr.sort()                 # [1, 2, 3, 4, 5]

# Funcionais
arr.map(|x| x * 2)         # [2, 4, 6, 8, 10]
arr.filter(|x| x > 2)      # [3, 4, 5]
arr.reduce(0, |a, b| a + b)  # 15
arr.find(|x| x > 3)        # Some(4)
arr.any(|x| x > 4)         # true
arr.all(|x| x > 0)         # true
```

### MendesOption<T>

Valor opcional.

```mendes
let opt: Option<int> = Some(42)

# Metodos
opt.is_some()              # true
opt.is_none()              # false
opt.unwrap()               # 42 (panic se None)
opt.unwrap_or(0)           # 42 (ou 0 se None)
opt.map(|x| x * 2)         # Some(84)
opt.and_then(|x| if x > 0: Some(x) else: None)
```

### MendesResult<T, E>

Resultado com erro.

```mendes
let res: Result<int, string> = Ok(42)

# Metodos
res.is_ok()                # true
res.is_err()               # false
res.unwrap()               # 42 (panic se Err)
res.unwrap_or(0)           # 42 (ou 0 se Err)
res.map(|x| x * 2)         # Ok(84)
res.map_err(|e| f"Error: {e}")
```

---

## 10. Tratamento de Erros

### MendesError

Enum de erros padrao.

```rust
pub enum MendesError {
    // HTTP
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    InternalError(String),

    // Database
    DatabaseError(String),
    ConnectionError(String),
    QueryError(String),

    // Validation
    ValidationError(String),

    // Generic
    Custom(String),
}
```

### Convertendo Erros

```mendes
fn process() -> Result<Data, MendesError>:
    let data = fetch_data().map_err(|e| MendesError::Custom(e.to_string()))?
    return Ok(data)
```

### Tratamento em APIs

```mendes
api GET /users/{id:int} async:
    return Result<User, MendesError>

    match await find_user(id):
        Ok(user):
            return Ok(user)
        Err(e):
            return Err(MendesError::NotFound(f"User {id} not found"))
```

### Respostas de Erro Automaticas

Quando uma API retorna `Result<T, MendesError>`, erros sao convertidos automaticamente:

| Erro | Status HTTP | Body |
|------|-------------|------|
| `BadRequest` | 400 | Mensagem |
| `Unauthorized` | 401 | Mensagem |
| `Forbidden` | 403 | Mensagem |
| `NotFound` | 404 | Mensagem |
| `InternalError` | 500 | Mensagem |
| `DatabaseError` | 500 | Mensagem |
| `ValidationError` | 422 | Mensagem |

---

## Apendice: Imports

### No Codigo Mendes

```mendes
# Imports sao automaticos baseados no uso
```

### No Codigo Rust Gerado

```rust
use mendes_runtime::{
    // HTTP
    Server, Request, Response, StatusCode,

    // Router
    Router,

    // Database
    PostgresPool, MysqlPool, SqlitePool,

    // WebSocket
    WsConnection,

    // Types
    MendesString, MendesArray, MendesResult, MendesOption,

    // Errors
    MendesError, Result,
};

// Re-export do Tokio
use mendes_runtime::tokio;
```

---

<p align="center">
  <strong>Mendes Runtime API Reference v0.1.0</strong>
</p>
