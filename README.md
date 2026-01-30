# Mendes Programming Language

<p align="center">
  <strong>Uma linguagem de programação compilada para backend de alta performance</strong>
</p>

<p align="center">
  <a href="#instalacao">Instalacao</a> •
  <a href="#inicio-rapido">Inicio Rapido</a> •
  <a href="#documentacao">Documentacao</a> •
  <a href="#exemplos">Exemplos</a> •
  <a href="#contribuindo">Contribuindo</a>
</p>

---

## O que e Mendes?

**Mendes** e uma linguagem de programacao moderna, compilada e focada em desenvolvimento backend. Combina a seguranca de tipos e o sistema de ownership do Rust com uma sintaxe limpa inspirada em Python, oferecendo HTTP e banco de dados como cidadaos de primeira classe.

### Caracteristicas Principais

| Caracteristica | Descricao |
|---------------|-----------|
| **HTTP Nativo** | Defina APIs REST diretamente na linguagem com `api GET /users` |
| **Async Real** | Suporte nativo a programacao assincrona com `async`/`await` |
| **Sistema de Tipos** | Tipagem estatica forte com inferencia de tipos |
| **Ownership** | Sistema de ownership inspirado em Rust para seguranca de memoria |
| **Compilado** | Compila para binarios nativos de alta performance |
| **Banco de Dados** | Pool de conexoes integrado para PostgreSQL, MySQL e SQLite |
| **WebSocket** | Suporte nativo a WebSocket para comunicacao em tempo real |

### Por que Mendes?

```mendes
# Isso e tudo que voce precisa para uma API funcional
server:
    host "0.0.0.0"
    port 8080

api GET /hello:
    return string
    return "Hello, World!"
```

Compare com outras linguagens:
- **Go**: Precisa de imports, struct handlers, mux setup
- **Node.js**: Express, callbacks, configuracao manual
- **Python/Flask**: Decorators, WSGI, setup complexo

**Mendes**: HTTP e parte da linguagem, nao uma biblioteca.

---

## Instalacao

### Pre-requisitos

- **Rust** (1.70+): [rustup.rs](https://rustup.rs)
- **Git**: Para clonar o repositorio

### Compilando do Fonte

```bash
# Clone o repositorio
git clone https://github.com/guilhermemendes/linguagem-mendes.git
cd linguagem-mendes

# Compile o compilador
cargo build --release

# Adicione ao PATH (opcional)
# Linux/macOS:
export PATH="$PATH:$(pwd)/target/release"
# Windows PowerShell:
$env:Path += ";$(pwd)\target\release"
```

### Verificando a Instalacao

```bash
mendes --version
# Mendes 0.1.0
```

---

## Inicio Rapido

### 1. Crie seu primeiro programa

Crie um arquivo `hello.ms`:

```mendes
# Meu primeiro programa Mendes

server:
    host "0.0.0.0"
    port 8080

api GET /health:
    return string
    return "ok"

api GET /hello/{name:string}:
    return string
    return f"Hello, {name}!"
```

### 2. Compile

```bash
mendes build hello.ms
```

### 3. Execute

```bash
# Windows
.\hello.exe

# Linux/macOS
./hello
```

### 4. Teste

```bash
curl http://localhost:8080/health
# ok

curl http://localhost:8080/hello/Mendes
# Hello, Mendes!
```

---

## Documentacao

### Guias

| Documento | Descricao |
|-----------|-----------|
| [Tutorial](docs/tutorial.md) | Guia passo a passo para iniciantes |
| [Referencia da Linguagem](docs/language-reference.md) | Especificacao completa da linguagem |
| [Referencia do CLI](docs/cli-reference.md) | Todos os comandos do compilador |
| [API do Runtime](docs/runtime-api.md) | HTTP, Banco de Dados, WebSocket |
| [Arquitetura](docs/architecture.md) | Como o compilador funciona |
| [Exemplos](docs/examples.md) | Exemplos detalhados e comentados |
| [Gramatica](docs/grammar.md) | Gramatica formal em EBNF |

### Recursos da Linguagem

#### Tipos Primitivos

```mendes
let idade: int = 30
let preco: float = 19.99
let ativo: bool = true
let nome: string = "Maria"
```

#### Structs e Metodos

```mendes
struct User:
    id: int
    name: string
    email: string

    fn display(&self) -> string:
        return f"User({self.name})"
```

#### Enums com Dados

```mendes
enum Result<T, E>:
    Ok(T)
    Err(E)

enum Option<T>:
    Some(T)
    None
```

#### APIs HTTP

```mendes
api GET /users:
    return [User]
    let users = await db.main.query("SELECT * FROM users")
    return users

api POST /users async:
    body User
    return User
    let user = await db.main.query_one(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        body.name, body.email
    )
    return user
```

#### Async/Await

```mendes
fn fetch_data() async -> Result<Data, Error>:
    let response = await http.get("https://api.example.com/data")
    return Ok(response)
```

#### Ownership e Referencias

```mendes
fn process(data: &User):           # Emprestimo imutavel
    print(data.name)

fn update(data: &mut User):        # Emprestimo mutavel
    data.name = "Novo Nome"

fn consume(data: User):            # Move (transfere ownership)
    # data e movido para esta funcao
```

---

## Exemplos

### API REST Completa

```mendes
db postgres main:
    url "postgres://localhost/myapp"
    pool 20

struct User:
    id: int
    name: string
    email: string

struct CreateUser:
    name: string
    email: string

server:
    host "0.0.0.0"
    port 8080

# Middleware de autenticacao
middleware auth:
    let token = request.header("Authorization")
    if token is None:
        return Response { status: 401, body: "Unauthorized" }

# Listar usuarios
api GET /users async:
    return [User]
    let users = await db.main.query("SELECT * FROM users")
    return users

# Buscar usuario por ID
api GET /users/{id:int} async:
    return Result<User, HttpError>
    let user = await db.main.query_one(
        "SELECT * FROM users WHERE id = $1",
        id
    )
    match user:
        Some(u):
            return Ok(u)
        None:
            return Err(HttpError(404, "User not found"))

# Criar usuario (protegido)
api POST /users async:
    use auth
    body CreateUser
    return User
    let user = await db.main.query_one(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        body.name, body.email
    )
    return user

# Atualizar usuario (protegido)
api PUT /users/{id:int} async:
    use auth
    body User
    return User
    let user = await db.main.query_one(
        "UPDATE users SET name = $1, email = $2 WHERE id = $3 RETURNING *",
        body.name, body.email, id
    )
    return user

# Deletar usuario (protegido)
api DELETE /users/{id:int} async:
    use auth
    return bool
    await db.main.execute(
        "DELETE FROM users WHERE id = $1",
        id
    )
    return true
```

### WebSocket Chat

```mendes
server:
    host "0.0.0.0"
    port 8080

ws /chat:
    on_connect:
        broadcast(f"User {conn.id} joined")

    on_message:
        broadcast(f"[{conn.id}]: {message}")

    on_disconnect:
        broadcast(f"User {conn.id} left")
```

Mais exemplos em [docs/examples.md](docs/examples.md).

---

## Estrutura do Projeto

```
linguagem-mendes/
├── Cargo.toml              # Workspace Rust
├── README.md               # Este arquivo
├── crates/
│   ├── mendes-lexer/       # Tokenizacao
│   ├── mendes-parser/      # Parser e AST
│   ├── mendes-semantic/    # Analise semantica
│   ├── mendes-ir/          # Representacao intermediaria
│   ├── mendes-codegen/     # Geracao de codigo
│   ├── mendes-cli/         # Interface de linha de comando
│   ├── mendes-runtime/     # Runtime (HTTP, DB, WebSocket)
│   ├── mendes-error/       # Sistema de diagnosticos
│   └── mendes-tests/       # Testes de integracao
├── docs/                   # Documentacao
└── examples/               # Programas de exemplo
```

---

## Comandos do CLI

| Comando | Descricao |
|---------|-----------|
| `mendes build <file>` | Compila para executavel |
| `mendes check <file>` | Verifica erros sem compilar |
| `mendes lex <file>` | Mostra tokens (debug) |
| `mendes parse <file>` | Mostra AST (debug) |
| `mendes ir <file>` | Mostra IR (debug) |
| `mendes emit <file>` | Gera codigo C |
| `mendes emit-rust <file>` | Gera codigo Rust |

### Opcoes de Build

```bash
# Build padrao (debug)
mendes build app.ms

# Build otimizado
mendes build app.ms --release

# Especificar nome do executavel
mendes build app.ms -o meu_app

# Usar backend C
mendes build app.ms --backend c
```

---

## Performance

Mendes compila para binarios nativos atraves do Rust, resultando em:

- **Startup instantaneo**: Sem JIT warmup ou interpretador
- **Memoria eficiente**: Sem garbage collector
- **Alta throughput**: Async runtime baseado em Tokio
- **Baixa latencia**: Zero-cost abstractions

### Benchmarks (preliminares)

| Metrica | Mendes | Go | Node.js |
|---------|--------|-----|---------|
| Requests/sec | ~120k | ~100k | ~40k |
| Latencia p99 | 2ms | 3ms | 15ms |
| Memoria (idle) | 8MB | 12MB | 50MB |

*Benchmarks em servidor simples GET /health. Resultados podem variar.*

---

## Roteiro

### v0.1 (Atual)
- [x] Lexer completo
- [x] Parser com AST
- [x] Analise semantica basica
- [x] Geracao de codigo Rust
- [x] CLI funcional
- [x] Runtime HTTP
- [x] Testes de integracao

### v0.2 (Proximo)
- [ ] LSP (Language Server Protocol)
- [ ] LLVM Backend nativo
- [ ] Hot reload em desenvolvimento
- [ ] Debugger integrado

### v0.3 (Futuro)
- [ ] Package manager
- [ ] Biblioteca padrao expandida
- [ ] Templates de projeto
- [ ] Plugins de IDE

---

## Contribuindo

Contribuicoes sao bem-vindas! Veja como ajudar:

### Reportando Bugs

1. Verifique se o bug ja foi reportado
2. Crie uma issue com:
   - Versao do Mendes
   - Sistema operacional
   - Codigo minimo que reproduz o problema
   - Mensagem de erro completa

### Contribuindo Codigo

1. Fork o repositorio
2. Crie uma branch: `git checkout -b feature/minha-feature`
3. Faca suas mudancas
4. Execute os testes: `cargo test --all`
5. Commit: `git commit -m "feat: descricao"`
6. Push: `git push origin feature/minha-feature`
7. Abra um Pull Request

### Convencoes

- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)
- **Codigo Rust**: Siga `rustfmt` e `clippy`
- **Documentacao**: Documente funcoes publicas

---

## Licenca

Este projeto esta licenciado sob a [MIT License](LICENSE).

---

## Autor

**Guilherme Mendes**

- GitHub: [@guilhermemendes](https://github.com/guilhermemendes)
- Email: contato@exemplo.com

---

## Agradecimentos

- [Rust](https://rust-lang.org) - A linguagem que torna Mendes possivel
- [Tokio](https://tokio.rs) - Runtime async de alta performance
- [Hyper](https://hyper.rs) - Biblioteca HTTP
- Comunidade open source

---

<p align="center">
  Feito com mass amor e muito cafe por Guilherme Mendes
</p>
