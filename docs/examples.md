# Exemplos Completos - Linguagem Mendes

> **Versao**: 0.1.0
> **Nivel**: Todos os niveis

Este documento contem exemplos completos e comentados de programas em Mendes, organizados por complexidade e categoria.

---

## Indice

1. [Basico](#1-basico)
   - [Hello World](#11-hello-world)
   - [Variaveis e Tipos](#12-variaveis-e-tipos)
   - [Funcoes](#13-funcoes)
   - [Controle de Fluxo](#14-controle-de-fluxo)
2. [Intermediario](#2-intermediario)
   - [Structs e Metodos](#21-structs-e-metodos)
   - [Enums e Match](#22-enums-e-match)
   - [Closures](#23-closures)
   - [Generics](#24-generics)
3. [APIs HTTP](#3-apis-http)
   - [API Minima](#31-api-minima)
   - [CRUD Completo](#32-crud-completo)
   - [Autenticacao JWT](#33-autenticacao-jwt)
   - [Upload de Arquivos](#34-upload-de-arquivos)
4. [Banco de Dados](#4-banco-de-dados)
   - [Queries Basicas](#41-queries-basicas)
   - [Transacoes](#42-transacoes)
   - [Migrations](#43-migrations)
5. [WebSocket](#5-websocket)
   - [Echo Server](#51-echo-server)
   - [Chat Room](#52-chat-room)
   - [Notificacoes Real-Time](#53-notificacoes-real-time)
6. [Projetos Completos](#6-projetos-completos)
   - [Todo List API](#61-todo-list-api)
   - [Blog API](#62-blog-api)
   - [E-commerce API](#63-e-commerce-api)

---

## 1. Basico

### 1.1 Hello World

O programa mais simples em Mendes:

```mendes
# hello.ms
# O classico Hello World como API HTTP

# Configuracao do servidor
server:
    host "0.0.0.0"   # Escuta em todas as interfaces
    port 8080        # Porta 8080

# Endpoint GET /hello
api GET /hello:
    return string    # Tipo de retorno
    return "Hello, World!"
```

**Compilando e executando:**

```bash
mendes build hello.ms
./hello
# Server starting on 0.0.0.0:8080

curl http://localhost:8080/hello
# Hello, World!
```

---

### 1.2 Variaveis e Tipos

```mendes
# variaveis.ms
# Demonstra tipos e variaveis

fn demonstra_tipos():
    # Tipos primitivos
    let inteiro: int = 42
    let decimal: float = 3.14159
    let texto: string = "Mendes"
    let verdadeiro: bool = true

    # Inferencia de tipos
    let inferido_int = 100          # int
    let inferido_float = 2.5        # float
    let inferido_string = "auto"    # string
    let inferido_bool = false       # bool

    # Variaveis mutaveis
    let mut contador = 0
    contador = 1
    contador += 10
    print(f"Contador: {contador}")   # Contador: 11

    # Arrays
    let numeros: [int] = [1, 2, 3, 4, 5]
    let primeiro = numeros[0]        # 1
    let tamanho = numeros.len()      # 5

    # Tuplas
    let pessoa: (string, int) = ("Ana", 25)
    let nome = pessoa.0              # "Ana"
    let idade = pessoa.1             # 25

    # Desestruturacao
    let (n, i) = pessoa
    print(f"{n} tem {i} anos")       # Ana tem 25 anos

fn demonstra_literais():
    # Inteiros em diferentes bases
    let decimal = 255
    let hex = 0xFF          # 255
    let binario = 0b11111111  # 255
    let octal = 0o377       # 255

    # Floats com notacao cientifica
    let grande = 1.5e10     # 15000000000
    let pequeno = 1.5e-10   # 0.00000000015

    # Strings com escape
    let com_quebra = "Linha 1\nLinha 2"
    let com_tab = "Col1\tCol2"
    let com_aspas = "Ele disse: \"Ola!\""

    # String interpolada
    let nome = "Mundo"
    let saudacao = f"Ola, {nome}!"
    let calculo = f"2 + 2 = {2 + 2}"

server:
    host "0.0.0.0"
    port 8080

api GET /tipos:
    return string
    demonstra_tipos()
    demonstra_literais()
    return "Tipos demonstrados no console"
```

---

### 1.3 Funcoes

```mendes
# funcoes.ms
# Demonstra definicao e uso de funcoes

# Funcao simples sem retorno
fn saudacao():
    print("Ola!")

# Funcao com parametros e retorno
fn soma(a: int, b: int) -> int:
    return a + b

# Funcao com retorno implicito
fn multiplica(a: int, b: int) -> int:
    a * b   # Ultima expressao e o retorno

# Funcao com multiplos retornos (tupla)
fn divide_e_resto(dividendo: int, divisor: int) -> (int, int):
    let quociente = dividendo / divisor
    let resto = dividendo % divisor
    return (quociente, resto)

# Funcao recursiva
fn fatorial(n: int) -> int:
    if n <= 1:
        return 1
    return n * fatorial(n - 1)

# Funcao com Option
fn buscar_por_indice(arr: [int], indice: int) -> Option<int>:
    if indice < 0 or indice >= arr.len():
        return None
    return Some(arr[indice])

# Funcao com Result
fn dividir_seguro(a: int, b: int) -> Result<int, string>:
    if b == 0:
        return Err("Divisao por zero")
    return Ok(a / b)

# Funcao async
fn buscar_dados() async -> Result<string, string>:
    let response = await http.get("https://api.example.com/data")
    if response.status != 200:
        return Err("Falha na requisicao")
    return Ok(response.body)

# Funcao publica (visivel para outros modulos)
pub fn api_publica(valor: int) -> int:
    return valor * 2

server:
    host "0.0.0.0"
    port 8080

api GET /demo:
    return string

    saudacao()

    let s = soma(10, 20)
    print(f"Soma: {s}")

    let m = multiplica(5, 6)
    print(f"Multiplicacao: {m}")

    let (q, r) = divide_e_resto(17, 5)
    print(f"17 / 5 = {q} resto {r}")

    let f = fatorial(5)
    print(f"5! = {f}")

    let nums = [10, 20, 30]
    match buscar_por_indice(nums, 1):
        Some(v):
            print(f"Encontrado: {v}")
        None:
            print("Nao encontrado")

    match dividir_seguro(10, 2):
        Ok(v):
            print(f"Resultado: {v}")
        Err(e):
            print(f"Erro: {e}")

    return "Funcoes demonstradas no console"
```

---

### 1.4 Controle de Fluxo

```mendes
# controle.ms
# Demonstra estruturas de controle

fn demonstra_if():
    let x = 15

    # If simples
    if x > 10:
        print("x e maior que 10")

    # If-else
    if x % 2 == 0:
        print("x e par")
    else:
        print("x e impar")

    # If-else if-else
    if x < 0:
        print("Negativo")
    else if x == 0:
        print("Zero")
    else if x < 10:
        print("Pequeno")
    else if x < 100:
        print("Medio")
    else:
        print("Grande")

    # If como expressao
    let categoria = if x < 18: "menor" else: "maior"
    print(f"Categoria: {categoria}")

fn demonstra_for():
    # For com range exclusivo
    print("Range 0..5:")
    for i in 0..5:
        print(f"  {i}")    # 0, 1, 2, 3, 4

    # For com range inclusivo
    print("Range 0..=5:")
    for i in 0..=5:
        print(f"  {i}")    # 0, 1, 2, 3, 4, 5

    # For com array
    let frutas = ["maca", "banana", "laranja"]
    print("Frutas:")
    for fruta in frutas:
        print(f"  {fruta}")

    # For com enumerate
    print("Frutas com indice:")
    for (i, fruta) in frutas.enumerate():
        print(f"  {i}: {fruta}")

fn demonstra_while():
    # While basico
    let mut contador = 0
    while contador < 5:
        print(f"Contador: {contador}")
        contador += 1

    # While com break
    let mut i = 0
    while true:
        if i >= 10:
            break
        i += 1
    print(f"i final: {i}")

    # While com continue
    let mut j = 0
    while j < 10:
        j += 1
        if j % 2 == 0:
            continue
        print(f"Impar: {j}")

fn demonstra_match():
    let numero = 3

    # Match basico
    match numero:
        1:
            print("Um")
        2:
            print("Dois")
        3:
            print("Tres")
        _:
            print("Outro")

    # Match com or pattern
    let char = 'a'
    match char:
        'a' | 'e' | 'i' | 'o' | 'u':
            print("Vogal")
        _:
            print("Consoante")

    # Match com guard
    let valor = 50
    match valor:
        n if n < 0:
            print("Negativo")
        n if n == 0:
            print("Zero")
        n if n <= 10:
            print("Pequeno")
        n if n <= 100:
            print("Medio")
        _:
            print("Grande")

    # Match com Option
    let maybe: Option<int> = Some(42)
    match maybe:
        Some(v):
            print(f"Valor: {v}")
        None:
            print("Nenhum valor")

    # Match com Result
    let result: Result<int, string> = Ok(100)
    match result:
        Ok(v):
            print(f"Sucesso: {v}")
        Err(e):
            print(f"Erro: {e}")

server:
    host "0.0.0.0"
    port 8080

api GET /controle:
    return string
    demonstra_if()
    demonstra_for()
    demonstra_while()
    demonstra_match()
    return "Controle demonstrado no console"
```

---

## 2. Intermediario

### 2.1 Structs e Metodos

```mendes
# structs.ms
# Demonstra structs e metodos

# Struct simples
struct Point copy:
    x: float
    y: float

    # Funcao associada (construtor)
    fn new(x: float, y: float) -> Point:
        return Point { x: x, y: y }

    # Metodo que le self
    fn distance_to_origin(&self) -> float:
        return (self.x * self.x + self.y * self.y).sqrt()

    # Metodo que modifica self
    fn translate(&mut self, dx: float, dy: float):
        self.x += dx
        self.y += dy

    # Metodo que consome self
    fn into_tuple(self) -> (float, float):
        return (self.x, self.y)

# Struct com campos opcionais
struct User:
    id: int
    name: string
    email: string
    bio: Option<string>
    age: Option<int>

    fn new(id: int, name: string, email: string) -> User:
        return User {
            id: id,
            name: name,
            email: email,
            bio: None,
            age: None
        }

    fn with_bio(self, bio: string) -> User:
        return User {
            id: self.id,
            name: self.name,
            email: self.email,
            bio: Some(bio),
            age: self.age
        }

    fn display(&self) -> string:
        let bio_str = match self.bio:
            Some(b): b
            None: "Sem bio"
        return f"User({self.name}, {bio_str})"

# Struct com array
struct Team:
    name: string
    members: [User]

    fn new(name: string) -> Team:
        return Team { name: name, members: [] }

    fn add_member(&mut self, user: User):
        self.members.push(user)

    fn member_count(&self) -> int:
        return self.members.len()

# Struct generica
struct Container<T>:
    value: T

    fn new(value: T) -> Container<T>:
        return Container { value: value }

    fn get(&self) -> &T:
        return &self.value

    fn set(&mut self, value: T):
        self.value = value

server:
    host "0.0.0.0"
    port 8080

api GET /structs:
    return string

    # Point
    let mut p = Point::new(3.0, 4.0)
    print(f"Distancia: {p.distance_to_origin()}")  # 5.0

    p.translate(1.0, 1.0)
    print(f"Novo ponto: ({p.x}, {p.y})")  # (4.0, 5.0)

    # User
    let user = User::new(1, "Maria", "maria@email.com")
        .with_bio("Desenvolvedora")
    print(user.display())

    # Team
    let mut team = Team::new("Backend")
    team.add_member(User::new(1, "Ana", "ana@email.com"))
    team.add_member(User::new(2, "Bruno", "bruno@email.com"))
    print(f"Membros: {team.member_count()}")

    # Container
    let mut c = Container::new(42)
    print(f"Valor: {c.get()}")
    c.set(100)
    print(f"Novo valor: {c.get()}")

    return "Structs demonstradas"
```

---

### 2.2 Enums e Match

```mendes
# enums.ms
# Demonstra enums e pattern matching

# Enum simples
enum Color:
    Red
    Green
    Blue
    Custom(int, int, int)

# Enum com dados variados
enum Message:
    Quit
    Move { x: int, y: int }
    Write(string)
    ChangeColor(Color)

# Enum Result customizado
enum ApiResult<T>:
    Success(T)
    Error { code: int, message: string }
    Loading

fn color_to_rgb(color: Color) -> (int, int, int):
    match color:
        Color::Red:
            return (255, 0, 0)
        Color::Green:
            return (0, 255, 0)
        Color::Blue:
            return (0, 0, 255)
        Color::Custom(r, g, b):
            return (r, g, b)

fn process_message(msg: Message):
    match msg:
        Message::Quit:
            print("Saindo...")
        Message::Move { x, y }:
            print(f"Movendo para ({x}, {y})")
        Message::Write(text):
            print(f"Escrevendo: {text}")
        Message::ChangeColor(color):
            let (r, g, b) = color_to_rgb(color)
            print(f"Cor: RGB({r}, {g}, {b})")

fn handle_api_result<T>(result: ApiResult<T>):
    match result:
        ApiResult::Success(data):
            print("Sucesso!")
        ApiResult::Error { code, message }:
            print(f"Erro {code}: {message}")
        ApiResult::Loading:
            print("Carregando...")

# Pattern matching avancado
fn advanced_matching():
    let numbers = [1, 2, 3, 4, 5]

    # Match com array patterns
    match numbers:
        []:
            print("Vazio")
        [x]:
            print(f"Um elemento: {x}")
        [first, .., last]:
            print(f"Primeiro: {first}, Ultimo: {last}")

    # Match com guardas
    let value = 42
    match value:
        n if n < 0:
            print("Negativo")
        0:
            print("Zero")
        n if n % 2 == 0:
            print(f"Par: {n}")
        n:
            print(f"Impar: {n}")

    # Match com Option aninhado
    let nested: Option<Option<int>> = Some(Some(42))
    match nested:
        Some(Some(n)):
            print(f"Valor aninhado: {n}")
        Some(None):
            print("Inner None")
        None:
            print("Outer None")

server:
    host "0.0.0.0"
    port 8080

api GET /enums:
    return string

    process_message(Message::Quit)
    process_message(Message::Move { x: 10, y: 20 })
    process_message(Message::Write("Ola Mendes!"))
    process_message(Message::ChangeColor(Color::Custom(128, 64, 255)))

    advanced_matching()

    return "Enums demonstrados"
```

---

### 2.3 Closures

```mendes
# closures.ms
# Demonstra closures e funcoes de alta ordem

fn demonstra_closures_basicas():
    # Closure simples
    let dobra = |x: int| x * 2
    print(f"Dobro de 5: {dobra(5)}")   # 10

    # Closure com bloco
    let processa = |x: int| -> int:
        let temp = x * 2
        let result = temp + 10
        return result

    print(f"Processado: {processa(5)}")  # 20

    # Closure capturando variavel
    let multiplicador = 3
    let multiplica = |x: int| x * multiplicador
    print(f"5 * 3 = {multiplica(5)}")    # 15

fn demonstra_alta_ordem():
    let numeros = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    # map - transforma cada elemento
    let dobrados = numeros.map(|x| x * 2)
    print(f"Dobrados: {dobrados}")   # [2, 4, 6, 8, 10, 12, 14, 16, 18, 20]

    # filter - filtra elementos
    let pares = numeros.filter(|x| x % 2 == 0)
    print(f"Pares: {pares}")         # [2, 4, 6, 8, 10]

    # reduce - acumula resultado
    let soma = numeros.reduce(0, |acc, x| acc + x)
    print(f"Soma: {soma}")           # 55

    # find - encontra primeiro
    let maior_que_5 = numeros.find(|x| x > 5)
    match maior_que_5:
        Some(n):
            print(f"Primeiro > 5: {n}")  # 6
        None:
            print("Nenhum > 5")

    # any/all - predicados
    let tem_par = numeros.any(|x| x % 2 == 0)
    let todos_positivos = numeros.all(|x| x > 0)
    print(f"Tem par: {tem_par}, Todos positivos: {todos_positivos}")

    # Encadeamento
    let resultado = numeros
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .reduce(0, |acc, x| acc + x)
    print(f"Soma dos quadrados dos pares: {resultado}")  # 220

fn funcao_que_retorna_closure() -> fn(int) -> int:
    let fator = 10
    return |x| x * fator

fn funcao_que_recebe_closure(f: fn(int) -> int, valor: int) -> int:
    return f(valor)

server:
    host "0.0.0.0"
    port 8080

api GET /closures:
    return string

    demonstra_closures_basicas()
    demonstra_alta_ordem()

    let minha_closure = funcao_que_retorna_closure()
    let resultado = funcao_que_recebe_closure(minha_closure, 5)
    print(f"Resultado: {resultado}")  # 50

    return "Closures demonstradas"
```

---

### 2.4 Generics

```mendes
# generics.ms
# Demonstra generics e traits

# Funcao generica
fn identity<T>(value: T) -> T:
    return value

fn swap<A, B>(pair: (A, B)) -> (B, A):
    return (pair.1, pair.0)

# Struct generica
struct Pair<T, U>:
    first: T
    second: U

    fn new(first: T, second: U) -> Pair<T, U>:
        return Pair { first: first, second: second }

# Stack generico
struct Stack<T>:
    items: [T]

    fn new() -> Stack<T>:
        return Stack { items: [] }

    fn push(&mut self, item: T):
        self.items.push(item)

    fn pop(&mut self) -> Option<T>:
        return self.items.pop()

    fn peek(&self) -> Option<&T>:
        if self.items.is_empty():
            return None
        return Some(&self.items[self.items.len() - 1])

    fn is_empty(&self) -> bool:
        return self.items.is_empty()

# Trait
trait Display:
    fn display(&self) -> string

trait Clone:
    fn clone(&self) -> Self

# Implementando traits
struct Point:
    x: int
    y: int

impl Display for Point:
    fn display(&self) -> string:
        return f"Point({self.x}, {self.y})"

impl Clone for Point:
    fn clone(&self) -> Point:
        return Point { x: self.x, y: self.y }

# Funcao com trait bound
fn print_all<T: Display>(items: [T]):
    for item in items:
        print(item.display())

fn clone_and_modify<T: Clone>(item: T) -> T:
    let copy = item.clone()
    return copy

server:
    host "0.0.0.0"
    port 8080

api GET /generics:
    return string

    # Identity
    let x = identity(42)
    let s = identity("hello")
    print(f"x = {x}, s = {s}")

    # Swap
    let (b, a) = swap((1, "um"))
    print(f"a = {a}, b = {b}")

    # Pair
    let pair = Pair::new("chave", 123)
    print(f"Pair: ({pair.first}, {pair.second})")

    # Stack
    let mut stack = Stack::new()
    stack.push(1)
    stack.push(2)
    stack.push(3)
    print(f"Topo: {stack.peek()}")  # Some(3)
    print(f"Pop: {stack.pop()}")    # Some(3)

    # Traits
    let p1 = Point { x: 10, y: 20 }
    print(p1.display())             # Point(10, 20)

    let p2 = p1.clone()
    print(f"Clone: {p2.display()}")

    return "Generics demonstrados"
```

---

## 3. APIs HTTP

### 3.1 API Minima

```mendes
# api_minima.ms
# API HTTP mais simples possivel

server:
    host "0.0.0.0"
    port 8080

api GET /health:
    return string
    return "ok"

api GET /hello/{name:string}:
    return string
    return f"Hello, {name}!"

api GET /add/{a:int}/{b:int}:
    return int
    return a + b
```

---

### 3.2 CRUD Completo

```mendes
# crud.ms
# API CRUD completa com banco de dados

db postgres main:
    url "postgres://user:pass@localhost/mydb"
    pool 20

struct User:
    id: int
    name: string
    email: string
    created_at: string

struct CreateUser:
    name: string
    email: string

struct UpdateUser:
    name: Option<string>
    email: Option<string>

server:
    host "0.0.0.0"
    port 8080

# CREATE - Criar usuario
api POST /users async:
    body CreateUser
    return Result<User, string>

    # Verificar se email ja existe
    let existing = await db.main.query_one(
        "SELECT id FROM users WHERE email = $1",
        body.email
    )
    if existing is Some:
        return Err("Email already exists")

    let user = await db.main.query_one(
        "INSERT INTO users (name, email, created_at) VALUES ($1, $2, NOW()) RETURNING *",
        body.name, body.email
    )
    return Ok(user)

# READ - Listar usuarios
api GET /users async:
    return [User]
    let users = await db.main.query("SELECT * FROM users ORDER BY id")
    return users

# READ - Buscar usuario por ID
api GET /users/{id:int} async:
    return Result<User, string>

    let user = await db.main.query_one(
        "SELECT * FROM users WHERE id = $1",
        id
    )
    match user:
        Some(u):
            return Ok(u)
        None:
            return Err(f"User {id} not found")

# UPDATE - Atualizar usuario
api PUT /users/{id:int} async:
    body UpdateUser
    return Result<User, string>

    # Verificar se existe
    let existing = await db.main.query_one(
        "SELECT * FROM users WHERE id = $1",
        id
    )
    if existing is None:
        return Err(f"User {id} not found")

    let user = await db.main.query_one(
        "UPDATE users SET name = COALESCE($1, name), email = COALESCE($2, email) WHERE id = $3 RETURNING *",
        body.name, body.email, id
    )
    return Ok(user)

# DELETE - Remover usuario
api DELETE /users/{id:int} async:
    return Result<bool, string>

    let result = await db.main.execute(
        "DELETE FROM users WHERE id = $1",
        id
    )
    if result.rows_affected == 0:
        return Err(f"User {id} not found")
    return Ok(true)
```

---

### 3.3 Autenticacao JWT

```mendes
# auth.ms
# Sistema de autenticacao com JWT

db postgres main:
    url "postgres://user:pass@localhost/auth_db"
    pool 10

struct User:
    id: int
    email: string
    password_hash: string
    created_at: string

struct RegisterRequest:
    email: string
    password: string

struct LoginRequest:
    email: string
    password: string

struct AuthResponse:
    token: string
    expires_at: string
    user_id: int

struct Claims:
    user_id: int
    email: string
    exp: int

# Funcoes auxiliares
fn hash_password(password: string) -> string:
    # Em producao, use bcrypt/argon2
    return crypto.sha256(password + "salt")

fn verify_password(password: string, hash: string) -> bool:
    return hash_password(password) == hash

fn generate_jwt(user_id: int, email: string) -> string:
    let claims = Claims {
        user_id: user_id,
        email: email,
        exp: time.now() + 86400  # 24 horas
    }
    return jwt.sign(claims, env.get("JWT_SECRET"))

fn validate_jwt(token: string) -> Result<Claims, string>:
    return jwt.verify(token, env.get("JWT_SECRET"))

# Middleware de autenticacao
middleware auth:
    let auth_header = request.header("Authorization")
    if auth_header is None:
        return Response { status: 401, body: "Authorization header required" }

    let token = auth_header.replace("Bearer ", "")
    match validate_jwt(token):
        Ok(claims):
            request.set("user_id", claims.user_id)
            request.set("email", claims.email)
        Err(e):
            return Response { status: 401, body: f"Invalid token: {e}" }

server:
    host "0.0.0.0"
    port 8080

# Registro
api POST /auth/register async:
    body RegisterRequest
    return Result<AuthResponse, string>

    # Verificar email existente
    let existing = await db.main.query_one(
        "SELECT id FROM users WHERE email = $1",
        body.email
    )
    if existing is Some:
        return Err("Email already registered")

    # Criar usuario
    let hash = hash_password(body.password)
    let user = await db.main.query_one(
        "INSERT INTO users (email, password_hash, created_at) VALUES ($1, $2, NOW()) RETURNING *",
        body.email, hash
    )

    # Gerar token
    let token = generate_jwt(user.id, user.email)
    return Ok(AuthResponse {
        token: token,
        expires_at: time.add_hours(24),
        user_id: user.id
    })

# Login
api POST /auth/login async:
    body LoginRequest
    return Result<AuthResponse, string>

    let user = await db.main.query_one(
        "SELECT * FROM users WHERE email = $1",
        body.email
    )

    match user:
        None:
            return Err("Invalid credentials")
        Some(u):
            if not verify_password(body.password, u.password_hash):
                return Err("Invalid credentials")

            let token = generate_jwt(u.id, u.email)
            return Ok(AuthResponse {
                token: token,
                expires_at: time.add_hours(24),
                user_id: u.id
            })

# Rota protegida
api GET /auth/me async:
    use auth
    return User

    let user_id = request.get<int>("user_id")
    let user = await db.main.query_one(
        "SELECT id, email, created_at FROM users WHERE id = $1",
        user_id
    )
    return user

# Trocar senha
api POST /auth/change-password async:
    use auth
    body ChangePasswordRequest
    return Result<string, string>

    let user_id = request.get<int>("user_id")
    let user = await db.main.query_one(
        "SELECT * FROM users WHERE id = $1",
        user_id
    )

    if not verify_password(body.current_password, user.password_hash):
        return Err("Current password is incorrect")

    let new_hash = hash_password(body.new_password)
    await db.main.execute(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        new_hash, user_id
    )

    return Ok("Password changed successfully")
```

---

### 3.4 Upload de Arquivos

```mendes
# upload.ms
# Upload e download de arquivos

struct FileInfo:
    id: string
    filename: string
    size: int
    content_type: string
    uploaded_at: string

struct UploadResponse:
    id: string
    url: string

server:
    host "0.0.0.0"
    port 8080

# Upload de arquivo
api POST /files async:
    return Result<UploadResponse, string>

    let file = request.file("file")
    if file is None:
        return Err("No file provided")

    # Gerar ID unico
    let id = uuid.v4()

    # Salvar arquivo
    let path = f"./uploads/{id}_{file.filename}"
    file.save(path)?

    # Salvar metadados
    await db.main.execute(
        "INSERT INTO files (id, filename, size, content_type, path, uploaded_at) VALUES ($1, $2, $3, $4, $5, NOW())",
        id, file.filename, file.size, file.content_type, path
    )

    return Ok(UploadResponse {
        id: id,
        url: f"/files/{id}"
    })

# Download de arquivo
api GET /files/{id:string} async:
    return Result<Response, string>

    let file_info = await db.main.query_one(
        "SELECT * FROM files WHERE id = $1",
        id
    )

    match file_info:
        None:
            return Err("File not found")
        Some(info):
            let content = fs.read(info.path)?
            return Ok(Response:
                status 200
                header "Content-Type" info.content_type
                header "Content-Disposition" f"attachment; filename=\"{info.filename}\""
                body content
            )

# Listar arquivos
api GET /files async:
    return [FileInfo]
    let files = await db.main.query("SELECT * FROM files ORDER BY uploaded_at DESC")
    return files

# Deletar arquivo
api DELETE /files/{id:string} async:
    return Result<bool, string>

    let file_info = await db.main.query_one(
        "SELECT path FROM files WHERE id = $1",
        id
    )

    match file_info:
        None:
            return Err("File not found")
        Some(info):
            fs.delete(info.path)?
            await db.main.execute("DELETE FROM files WHERE id = $1", id)
            return Ok(true)
```

---

## 4. Banco de Dados

### 4.1 Queries Basicas

```mendes
# queries.ms
# Exemplos de queries de banco de dados

db postgres main:
    url "postgres://localhost/mydb"
    pool 20

struct Product:
    id: int
    name: string
    price: float
    stock: int
    category_id: int

# Busca simples
fn get_all_products() async -> [Product]:
    return await db.main.query("SELECT * FROM products")

# Busca com filtro
fn get_products_by_category(category_id: int) async -> [Product]:
    return await db.main.query(
        "SELECT * FROM products WHERE category_id = $1",
        category_id
    )

# Busca com paginacao
fn get_products_paginated(page: int, limit: int) async -> [Product]:
    let offset = (page - 1) * limit
    return await db.main.query(
        "SELECT * FROM products ORDER BY id LIMIT $1 OFFSET $2",
        limit, offset
    )

# Busca com ordenacao
fn get_products_sorted(sort_by: string, order: string) async -> [Product]:
    let query = f"SELECT * FROM products ORDER BY {sort_by} {order}"
    return await db.main.query(query)

# Busca um registro
fn get_product(id: int) async -> Option<Product>:
    return await db.main.query_one(
        "SELECT * FROM products WHERE id = $1",
        id
    )

# Inserir
fn create_product(name: string, price: float, stock: int, category_id: int) async -> Product:
    return await db.main.query_one(
        "INSERT INTO products (name, price, stock, category_id) VALUES ($1, $2, $3, $4) RETURNING *",
        name, price, stock, category_id
    )

# Atualizar
fn update_product_stock(id: int, new_stock: int) async -> Option<Product>:
    return await db.main.query_one(
        "UPDATE products SET stock = $1 WHERE id = $2 RETURNING *",
        new_stock, id
    )

# Deletar
fn delete_product(id: int) async -> bool:
    let result = await db.main.execute(
        "DELETE FROM products WHERE id = $1",
        id
    )
    return result.rows_affected > 0

# Agregacoes
fn get_product_stats() async -> ProductStats:
    return await db.main.query_one(
        "SELECT COUNT(*) as total, AVG(price) as avg_price, SUM(stock) as total_stock FROM products"
    )
```

---

### 4.2 Transacoes

```mendes
# transacoes.ms
# Exemplos de transacoes de banco de dados

db postgres main:
    url "postgres://localhost/bank_db"
    pool 20

struct Account:
    id: int
    user_id: int
    balance: float

struct Transfer:
    id: int
    from_account: int
    to_account: int
    amount: float
    created_at: string

# Transferencia atomica
fn transfer_money(from_id: int, to_id: int, amount: float) async -> Result<Transfer, string>:
    # Verificar saldo
    let from_account = await db.main.query_one(
        "SELECT * FROM accounts WHERE id = $1",
        from_id
    )

    if from_account is None:
        return Err("Source account not found")

    if from_account.balance < amount:
        return Err("Insufficient funds")

    # Executar transacao
    await db.main.transaction:
        # Debitar
        await db.main.execute(
            "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
            amount, from_id
        )

        # Creditar
        await db.main.execute(
            "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
            amount, to_id
        )

        # Registrar transferencia
        let transfer = await db.main.query_one(
            "INSERT INTO transfers (from_account, to_account, amount, created_at) VALUES ($1, $2, $3, NOW()) RETURNING *",
            from_id, to_id, amount
        )

        return Ok(transfer)

# Ordem de compra atomica
fn create_order(user_id: int, items: [OrderItem]) async -> Result<Order, string>:
    await db.main.transaction:
        # Criar pedido
        let order = await db.main.query_one(
            "INSERT INTO orders (user_id, status, created_at) VALUES ($1, 'pending', NOW()) RETURNING *",
            user_id
        )

        let mut total: float = 0.0

        for item in items:
            # Verificar estoque
            let product = await db.main.query_one(
                "SELECT * FROM products WHERE id = $1 FOR UPDATE",
                item.product_id
            )

            if product.stock < item.quantity:
                return Err(f"Insufficient stock for product {item.product_id}")

            # Reduzir estoque
            await db.main.execute(
                "UPDATE products SET stock = stock - $1 WHERE id = $2",
                item.quantity, item.product_id
            )

            # Adicionar item ao pedido
            await db.main.execute(
                "INSERT INTO order_items (order_id, product_id, quantity, price) VALUES ($1, $2, $3, $4)",
                order.id, item.product_id, item.quantity, product.price
            )

            total += product.price * item.quantity

        # Atualizar total
        await db.main.execute(
            "UPDATE orders SET total = $1 WHERE id = $2",
            total, order.id
        )

        return Ok(order)
```

---

## 5. WebSocket

### 5.1 Echo Server

```mendes
# echo.ms
# Servidor WebSocket simples (echo)

server:
    host "0.0.0.0"
    port 8080

ws /echo:
    on_connect:
        print(f"Cliente {conn.id} conectou")
        conn.send("Bem-vindo ao Echo Server!")

    on_message:
        print(f"Recebido de {conn.id}: {message}")
        conn.send(f"Echo: {message}")

    on_disconnect:
        print(f"Cliente {conn.id} desconectou")
```

---

### 5.2 Chat Room

```mendes
# chat.ms
# Chat em tempo real com salas

struct ChatUser:
    nickname: string
    joined_at: string

server:
    host "0.0.0.0"
    port 8080

# Chat global
ws /chat:
    on_connect:
        let user = ChatUser {
            nickname: f"User{conn.id}",
            joined_at: time.now()
        }
        conn.set_state(user)
        broadcast(f"[Sistema] {user.nickname} entrou no chat")

    on_message:
        let user = conn.get_state<ChatUser>()

        # Comandos especiais
        if message.starts_with("/nick "):
            let new_nick = message.replace("/nick ", "")
            let old_nick = user.nickname
            user.nickname = new_nick
            conn.set_state(user)
            broadcast(f"[Sistema] {old_nick} agora e {new_nick}")
        else if message == "/users":
            let count = get_connection_count()
            conn.send(f"[Sistema] {count} usuarios online")
        else:
            broadcast(f"[{user.nickname}]: {message}")

    on_disconnect:
        let user = conn.get_state<ChatUser>()
        broadcast(f"[Sistema] {user.nickname} saiu do chat")

# Chat com salas
ws /chat/{room:string}:
    on_connect:
        join_room(room)
        let user = ChatUser {
            nickname: f"User{conn.id}",
            joined_at: time.now()
        }
        conn.set_state(user)
        broadcast_room(room, f"[{room}] {user.nickname} entrou")

    on_message:
        let user = conn.get_state<ChatUser>()
        broadcast_room(room, f"[{room}][{user.nickname}]: {message}")

    on_disconnect:
        let user = conn.get_state<ChatUser>()
        broadcast_room(room, f"[{room}] {user.nickname} saiu")
        leave_room(room)
```

---

### 5.3 Notificacoes Real-Time

```mendes
# notifications.ms
# Sistema de notificacoes em tempo real

db postgres main:
    url "postgres://localhost/notifications_db"
    pool 10

struct Notification:
    id: int
    user_id: int
    type: string
    message: string
    read: bool
    created_at: string

server:
    host "0.0.0.0"
    port 8080

# Middleware de autenticacao para WebSocket
middleware ws_auth:
    let token = request.query("token")
    if token is None:
        return Response { status: 401, body: "Token required" }

    match validate_jwt(token):
        Ok(claims):
            request.set("user_id", claims.user_id)
        Err(e):
            return Response { status: 401, body: "Invalid token" }

# WebSocket de notificacoes
ws /notifications:
    use ws_auth

    on_connect:
        let user_id = request.get<int>("user_id")
        join_room(f"user:{user_id}")

        # Enviar notificacoes nao lidas
        let unread = await db.main.query(
            "SELECT * FROM notifications WHERE user_id = $1 AND read = false ORDER BY created_at DESC",
            user_id
        )
        for notif in unread:
            conn.send(notif.to_json())

    on_message:
        # Cliente pode marcar como lida
        let data = json.parse(message)
        if data.action == "mark_read":
            await db.main.execute(
                "UPDATE notifications SET read = true WHERE id = $1",
                data.id
            )
            conn.send(json.stringify({ "action": "marked_read", "id": data.id }))

    on_disconnect:
        let user_id = request.get<int>("user_id")
        leave_room(f"user:{user_id}")

# API para enviar notificacao
api POST /notifications async:
    use auth
    body CreateNotification
    return Notification

    let notif = await db.main.query_one(
        "INSERT INTO notifications (user_id, type, message, read, created_at) VALUES ($1, $2, $3, false, NOW()) RETURNING *",
        body.user_id, body.type, body.message
    )

    # Enviar via WebSocket se usuario estiver conectado
    broadcast_room(f"user:{body.user_id}", notif.to_json())

    return notif
```

---

## 6. Projetos Completos

### 6.1 Todo List API

Veja o projeto completo no [Tutorial](tutorial.md#17-projeto-final).

### 6.2 Blog API

```mendes
# blog.ms
# API completa de blog

db postgres main:
    url "postgres://localhost/blog_db"
    pool 20

# Modelos
struct User:
    id: int
    username: string
    email: string
    bio: Option<string>
    created_at: string

struct Post:
    id: int
    title: string
    slug: string
    content: string
    author_id: int
    published: bool
    created_at: string
    updated_at: string

struct Comment:
    id: int
    post_id: int
    author_id: int
    content: string
    created_at: string

struct CreatePost:
    title: string
    content: string

struct PostWithAuthor:
    post: Post
    author: User
    comment_count: int

# Middlewares
middleware auth:
    # ... (implementacao de autenticacao)
    pass

middleware rate_limit:
    let ip = request.header("X-Forwarded-For").unwrap_or(request.ip())
    let key = f"rate:{ip}"
    let count = cache.increment(key, 1, 60)
    if count > 100:
        return Response { status: 429, body: "Rate limit exceeded" }

server:
    host "0.0.0.0"
    port 8080

# Posts
api GET /posts async:
    use rate_limit
    query PaginationQuery
    return [PostWithAuthor]

    let page = query.page.unwrap_or(1)
    let limit = query.limit.unwrap_or(10)
    let offset = (page - 1) * limit

    let posts = await db.main.query(
        """
        SELECT p.*, u.id as author_id, u.username, u.email,
               (SELECT COUNT(*) FROM comments WHERE post_id = p.id) as comment_count
        FROM posts p
        JOIN users u ON p.author_id = u.id
        WHERE p.published = true
        ORDER BY p.created_at DESC
        LIMIT $1 OFFSET $2
        """,
        limit, offset
    )
    return posts

api GET /posts/{slug:string} async:
    use rate_limit
    return Result<PostWithAuthor, string>

    let post = await db.main.query_one(
        """
        SELECT p.*, u.id as author_id, u.username, u.email,
               (SELECT COUNT(*) FROM comments WHERE post_id = p.id) as comment_count
        FROM posts p
        JOIN users u ON p.author_id = u.id
        WHERE p.slug = $1 AND p.published = true
        """,
        slug
    )
    match post:
        Some(p):
            return Ok(p)
        None:
            return Err("Post not found")

api POST /posts async:
    use auth
    use rate_limit
    body CreatePost
    return Post

    let user_id = request.get<int>("user_id")
    let slug = slugify(body.title)

    let post = await db.main.query_one(
        """
        INSERT INTO posts (title, slug, content, author_id, published, created_at, updated_at)
        VALUES ($1, $2, $3, $4, false, NOW(), NOW())
        RETURNING *
        """,
        body.title, slug, body.content, user_id
    )
    return post

api PUT /posts/{id:int}/publish async:
    use auth
    return Result<Post, string>

    let user_id = request.get<int>("user_id")
    let post = await db.main.query_one(
        "UPDATE posts SET published = true, updated_at = NOW() WHERE id = $1 AND author_id = $2 RETURNING *",
        id, user_id
    )
    match post:
        Some(p):
            return Ok(p)
        None:
            return Err("Post not found or unauthorized")

# Comentarios
api GET /posts/{post_id:int}/comments async:
    return [Comment]
    return await db.main.query(
        "SELECT * FROM comments WHERE post_id = $1 ORDER BY created_at",
        post_id
    )

api POST /posts/{post_id:int}/comments async:
    use auth
    body CreateComment
    return Comment

    let user_id = request.get<int>("user_id")
    return await db.main.query_one(
        "INSERT INTO comments (post_id, author_id, content, created_at) VALUES ($1, $2, $3, NOW()) RETURNING *",
        post_id, user_id, body.content
    )
```

---

### 6.3 E-commerce API

```mendes
# ecommerce.ms
# API de e-commerce simplificada

db postgres main:
    url "postgres://localhost/ecommerce_db"
    pool 30

# Modelos
struct Product:
    id: int
    name: string
    description: string
    price: float
    stock: int
    category_id: int
    images: [string]
    created_at: string

struct Cart:
    id: int
    user_id: int
    items: [CartItem]

struct CartItem:
    product_id: int
    quantity: int
    price: float

struct Order:
    id: int
    user_id: int
    status: string
    total: float
    items: [OrderItem]
    shipping_address: Address
    created_at: string

struct OrderItem:
    product_id: int
    quantity: int
    price: float

struct Address:
    street: string
    city: string
    state: string
    zip: string
    country: string

# Middlewares
middleware auth:
    # ... autenticacao
    pass

server:
    host "0.0.0.0"
    port 8080

# Produtos
api GET /products async:
    query ProductQuery
    return [Product]

    let mut sql = "SELECT * FROM products WHERE 1=1"
    let mut params: [any] = []

    if query.category_id is Some(cat):
        sql += f" AND category_id = ${params.len() + 1}"
        params.push(cat)

    if query.min_price is Some(min):
        sql += f" AND price >= ${params.len() + 1}"
        params.push(min)

    if query.max_price is Some(max):
        sql += f" AND price <= ${params.len() + 1}"
        params.push(max)

    sql += " ORDER BY created_at DESC"

    return await db.main.query(sql, ...params)

api GET /products/{id:int} async:
    return Result<Product, string>
    let product = await db.main.query_one("SELECT * FROM products WHERE id = $1", id)
    match product:
        Some(p): return Ok(p)
        None: return Err("Product not found")

# Carrinho
api GET /cart async:
    use auth
    return Cart

    let user_id = request.get<int>("user_id")
    let cart = await get_or_create_cart(user_id)
    return cart

api POST /cart/items async:
    use auth
    body AddToCartRequest
    return Cart

    let user_id = request.get<int>("user_id")
    let cart = await get_or_create_cart(user_id)

    # Verificar produto
    let product = await db.main.query_one("SELECT * FROM products WHERE id = $1", body.product_id)
    if product is None:
        return Response { status: 404, body: "Product not found" }

    if product.stock < body.quantity:
        return Response { status: 400, body: "Insufficient stock" }

    # Adicionar ou atualizar item
    await db.main.execute(
        """
        INSERT INTO cart_items (cart_id, product_id, quantity, price)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (cart_id, product_id)
        DO UPDATE SET quantity = cart_items.quantity + $3
        """,
        cart.id, body.product_id, body.quantity, product.price
    )

    return await get_cart_with_items(cart.id)

api DELETE /cart/items/{product_id:int} async:
    use auth
    return Cart

    let user_id = request.get<int>("user_id")
    let cart = await get_or_create_cart(user_id)

    await db.main.execute(
        "DELETE FROM cart_items WHERE cart_id = $1 AND product_id = $2",
        cart.id, product_id
    )

    return await get_cart_with_items(cart.id)

# Pedidos
api POST /orders async:
    use auth
    body CreateOrderRequest
    return Result<Order, string>

    let user_id = request.get<int>("user_id")
    let cart = await get_cart_with_items(user_id)

    if cart.items.is_empty():
        return Err("Cart is empty")

    await db.main.transaction:
        # Calcular total e verificar estoque
        let mut total: float = 0.0
        for item in cart.items:
            let product = await db.main.query_one(
                "SELECT * FROM products WHERE id = $1 FOR UPDATE",
                item.product_id
            )

            if product.stock < item.quantity:
                return Err(f"Insufficient stock for {product.name}")

            total += item.price * item.quantity

            # Reduzir estoque
            await db.main.execute(
                "UPDATE products SET stock = stock - $1 WHERE id = $2",
                item.quantity, item.product_id
            )

        # Criar pedido
        let order = await db.main.query_one(
            """
            INSERT INTO orders (user_id, status, total, shipping_address, created_at)
            VALUES ($1, 'pending', $2, $3, NOW())
            RETURNING *
            """,
            user_id, total, body.shipping_address.to_json()
        )

        # Adicionar itens
        for item in cart.items:
            await db.main.execute(
                "INSERT INTO order_items (order_id, product_id, quantity, price) VALUES ($1, $2, $3, $4)",
                order.id, item.product_id, item.quantity, item.price
            )

        # Limpar carrinho
        await db.main.execute("DELETE FROM cart_items WHERE cart_id = $1", cart.id)

        return Ok(order)

api GET /orders async:
    use auth
    return [Order]

    let user_id = request.get<int>("user_id")
    return await db.main.query(
        "SELECT * FROM orders WHERE user_id = $1 ORDER BY created_at DESC",
        user_id
    )

api GET /orders/{id:int} async:
    use auth
    return Result<Order, string>

    let user_id = request.get<int>("user_id")
    let order = await db.main.query_one(
        "SELECT * FROM orders WHERE id = $1 AND user_id = $2",
        id, user_id
    )
    match order:
        Some(o): return Ok(o)
        None: return Err("Order not found")
```

---

## Apendice: Executando os Exemplos

### Compilar

```bash
mendes build exemplo.ms
```

### Executar

```bash
./exemplo
```

### Testar

```bash
# Health check
curl http://localhost:8080/health

# GET com parametros
curl http://localhost:8080/users/1

# POST com body
curl -X POST http://localhost:8080/users \
  -H "Content-Type: application/json" \
  -d '{"name":"Maria","email":"maria@email.com"}'

# Com autenticacao
curl http://localhost:8080/protected \
  -H "Authorization: Bearer <token>"
```

---

<p align="center">
  <strong>Mendes Examples v0.1.0</strong>
</p>
