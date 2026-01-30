# Tutorial Completo da Linguagem Mendes

> **Versao**: 0.1.0
> **Nivel**: Iniciante a Avancado
> **Tempo estimado**: 2-4 horas

Este tutorial vai guia-lo desde os conceitos basicos ate recursos avancados da linguagem Mendes. Ao final, voce sera capaz de construir APIs REST completas, trabalhar com bancos de dados e implementar WebSockets.

---

## Indice

1. [Introducao](#1-introducao)
2. [Configuracao do Ambiente](#2-configuracao-do-ambiente)
3. [Conceitos Basicos](#3-conceitos-basicos)
4. [Tipos de Dados](#4-tipos-de-dados)
5. [Controle de Fluxo](#5-controle-de-fluxo)
6. [Funcoes](#6-funcoes)
7. [Structs e Metodos](#7-structs-e-metodos)
8. [Enums e Pattern Matching](#8-enums-e-pattern-matching)
9. [Sistema de Ownership](#9-sistema-de-ownership)
10. [Generics e Traits](#10-generics-e-traits)
11. [HTTP e APIs](#11-http-e-apis)
12. [Banco de Dados](#12-banco-de-dados)
13. [Async/Await](#13-asyncawait)
14. [WebSockets](#14-websockets)
15. [Tratamento de Erros](#15-tratamento-de-erros)
16. [Modulos e Imports](#16-modulos-e-imports)
17. [Projeto Final](#17-projeto-final)

---

## 1. Introducao

### 1.1 O que e Mendes?

Mendes e uma linguagem de programacao compilada, projetada especificamente para desenvolvimento backend. Ela combina:

- **Seguranca de tipos** do Rust
- **Sintaxe limpa** do Python
- **HTTP nativo** como parte da linguagem
- **Async/await** para operacoes nao-bloqueantes

### 1.2 Filosofia da Linguagem

```
"Simples para o comum, possivel para o complexo"
```

Mendes foi projetada com os seguintes principios:

1. **HTTP e Cidadao de Primeira Classe**: Nao e uma biblioteca, e parte da linguagem
2. **Seguranca em Tempo de Compilacao**: Erros sao capturados antes de rodar
3. **Performance sem Sacrificio**: Compila para binarios nativos
4. **Expressividade**: Codigo legivel e conciso

### 1.3 Primeiro Programa

Todo tutorial comeca com "Hello World". Em Mendes:

```mendes
# hello.ms - Meu primeiro programa Mendes

server:
    host "0.0.0.0"
    port 8080

api GET /hello:
    return string
    return "Hello, World!"
```

Compile e execute:

```bash
mendes build hello.ms
./hello
```

Teste:

```bash
curl http://localhost:8080/hello
# Hello, World!
```

**O que aconteceu?**

1. `server:` define a configuracao do servidor HTTP
2. `api GET /hello:` declara um endpoint GET na rota /hello
3. `return string` declara o tipo de retorno
4. `return "Hello, World!"` retorna a resposta

---

## 2. Configuracao do Ambiente

### 2.1 Instalacao

#### Pre-requisitos

| Software | Versao Minima | Verificar |
|----------|---------------|-----------|
| Rust | 1.70+ | `rustc --version` |
| Cargo | 1.70+ | `cargo --version` |
| Git | 2.0+ | `git --version` |

#### Instalando Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows
# Baixe e execute: https://win.rustup.rs
```

#### Compilando Mendes

```bash
git clone https://github.com/guilhermemendes/linguagem-mendes.git
cd linguagem-mendes
cargo build --release
```

#### Adicionando ao PATH

```bash
# Linux/macOS (adicione ao ~/.bashrc ou ~/.zshrc)
export PATH="$PATH:/caminho/para/linguagem-mendes/target/release"

# Windows PowerShell
$env:Path += ";C:\caminho\para\linguagem-mendes\target\release"
```

### 2.2 Verificando a Instalacao

```bash
mendes --version
# mendes 0.1.0

mendes --help
# Mendes language compiler
#
# Usage: mendes <COMMAND>
# ...
```

### 2.3 Estrutura de um Projeto

```
meu-projeto/
├── src/
│   ├── main.ms        # Ponto de entrada
│   ├── models.ms      # Definicoes de dados
│   └── handlers.ms    # Logica de negocios
├── tests/
│   └── api_test.ms    # Testes
└── README.md
```

### 2.4 Editor e Extensoes

Atualmente, Mendes nao tem extensoes de IDE, mas funciona bem com:

- **VS Code**: Use highlighting de Python como aproximacao
- **Vim/Neovim**: Syntax highlighting basico
- **Qualquer editor**: Mendes usa indentacao significativa

---

## 3. Conceitos Basicos

### 3.1 Sintaxe Fundamental

Mendes usa **indentacao significativa** (como Python). Blocos sao definidos por `:` seguido de indentacao:

```mendes
# Correto
if x > 10:
    print("maior")

# Incorreto (erro de compilacao)
if x > 10:
print("maior")  # Falta indentacao!
```

### 3.2 Comentarios

```mendes
# Comentario de linha unica

# Mendes nao tem comentarios de bloco,
# use multiplos comentarios de linha
```

### 3.3 Variaveis

#### Declaracao Imutavel (padrao)

```mendes
let nome = "Maria"
let idade: int = 30
let pi: float = 3.14159

# Erro! Variaveis sao imutaveis por padrao
nome = "Ana"  # Erro de compilacao
```

#### Declaracao Mutavel

```mendes
let mut contador = 0
contador = 1       # Ok!
contador += 1      # Ok! contador agora e 2
```

#### Inferencia de Tipos

```mendes
let x = 42          # int (inferido)
let y = 3.14        # float (inferido)
let z = "texto"     # string (inferido)
let b = true        # bool (inferido)
```

#### Tipo Explicito

```mendes
let x: int = 42
let y: float = 3.14
let z: string = "texto"
let b: bool = true
```

### 3.4 Constantes

Todas as variaveis `let` sem `mut` sao constantes:

```mendes
let MAX_USERS = 1000
let API_VERSION = "v1"
```

### 3.5 Impressao (Debug)

```mendes
let nome = "Mendes"
print(nome)                    # Mendes
print(f"Ola, {nome}!")         # Ola, Mendes!
```

---

## 4. Tipos de Dados

### 4.1 Tipos Primitivos

| Tipo | Descricao | Exemplo |
|------|-----------|---------|
| `int` | Inteiro 64-bit | `42`, `-17`, `0xFF` |
| `float` | Ponto flutuante 64-bit | `3.14`, `-0.5`, `1e10` |
| `bool` | Booleano | `true`, `false` |
| `string` | Texto UTF-8 | `"hello"`, `"ola"` |

#### Literais Numericos

```mendes
# Inteiros
let decimal = 42
let hexadecimal = 0xFF      # 255
let binario = 0b1010        # 10
let octal = 0o777           # 511

# Floats
let pi = 3.14159
let avogadro = 6.022e23
let pequeno = 1.5e-10
```

#### Strings

```mendes
# String simples
let s1 = "Hello, World!"

# Escape sequences
let s2 = "Linha1\nLinha2"    # Nova linha
let s3 = "Tab:\taqui"        # Tab
let s4 = "Aspas: \"ola\""    # Aspas

# String interpolada (f-string)
let nome = "Maria"
let idade = 30
let msg = f"{nome} tem {idade} anos"
# "Maria tem 30 anos"

# Expressoes em f-strings
let x = 10
let y = 20
let calc = f"Soma: {x + y}, Produto: {x * y}"
# "Soma: 30, Produto: 200"
```

### 4.2 Arrays

```mendes
# Declaracao
let numeros: [int] = [1, 2, 3, 4, 5]
let nomes = ["Ana", "Bruno", "Carlos"]  # Tipo inferido

# Acesso por indice (base 0)
let primeiro = numeros[0]    # 1
let ultimo = numeros[4]      # 5

# Tamanho
let tamanho = numeros.len()  # 5

# Iteracao
for num in numeros:
    print(num)

# Metodos
numeros.push(6)              # Adiciona ao final
let removido = numeros.pop() # Remove do final
let contem = numeros.contains(3)  # true
```

### 4.3 Tuplas

```mendes
# Tupla com tipos diferentes
let pessoa: (string, int, bool) = ("Maria", 30, true)

# Acesso por indice
let nome = pessoa.0          # "Maria"
let idade = pessoa.1         # 30
let ativo = pessoa.2         # true

# Desestruturacao
let (n, i, a) = pessoa
print(n)  # "Maria"

# Funcao retornando tupla
fn divide(a: int, b: int) -> (int, int):
    let quociente = a / b
    let resto = a % b
    return (quociente, resto)

let (q, r) = divide(17, 5)   # q=3, r=2
```

### 4.4 Option e Result

#### Option<T>

Representa um valor que pode ou nao existir:

```mendes
# Some(valor) ou None
let talvez_numero: Option<int> = Some(42)
let nada: Option<int> = None

# Verificacao
if talvez_numero is Some:
    print("Tem valor!")

# Match
match talvez_numero:
    Some(n):
        print(f"Valor: {n}")
    None:
        print("Sem valor")

# Metodos uteis
let valor = talvez_numero.unwrap()           # 42 (panic se None)
let seguro = talvez_numero.unwrap_or(0)      # 42 (ou 0 se None)
let mapeado = talvez_numero.map(|x| x * 2)   # Some(84)
```

#### Result<T, E>

Representa sucesso ou erro:

```mendes
fn divide(a: int, b: int) -> Result<int, string>:
    if b == 0:
        return Err("Divisao por zero")
    return Ok(a / b)

let resultado = divide(10, 2)

match resultado:
    Ok(valor):
        print(f"Resultado: {valor}")
    Err(erro):
        print(f"Erro: {erro}")

# Operador ? (propagacao de erro)
fn calcula() -> Result<int, string>:
    let x = divide(10, 2)?   # Propaga erro se Err
    let y = divide(x, 2)?
    return Ok(y)
```

### 4.5 Structs

```mendes
# Definicao
struct User:
    id: int
    name: string
    email: string
    active: bool

# Instanciacao
let user = User {
    id: 1,
    name: "Maria",
    email: "maria@email.com",
    active: true
}

# Acesso a campos
print(user.name)       # "Maria"
print(user.active)     # true

# Struct copy (copiado em vez de movido)
struct Point copy:
    x: float
    y: float

let p1 = Point { x: 1.0, y: 2.0 }
let p2 = p1            # Copia, nao move!
print(p1.x)            # Ainda funciona
```

### 4.6 Enums

```mendes
# Enum simples
enum Status:
    Active
    Inactive
    Pending

let s = Status::Active

# Enum com dados
enum Message:
    Text(string)
    Number(int)
    Move { x: int, y: int }

let m1 = Message::Text("Ola")
let m2 = Message::Number(42)
let m3 = Message::Move { x: 10, y: 20 }

# Match com enum
match m1:
    Message::Text(t):
        print(f"Texto: {t}")
    Message::Number(n):
        print(f"Numero: {n}")
    Message::Move { x, y }:
        print(f"Mover para ({x}, {y})")
```

---

## 5. Controle de Fluxo

### 5.1 If/Else

```mendes
let idade = 18

if idade < 13:
    print("Crianca")
else if idade < 20:
    print("Adolescente")
else if idade < 60:
    print("Adulto")
else:
    print("Idoso")
```

#### If como Expressao

```mendes
let categoria = if idade < 18:
    "Menor"
else:
    "Maior"
```

### 5.2 Match

O `match` e a forma mais poderosa de controle de fluxo:

```mendes
let numero = 5

match numero:
    1:
        print("Um")
    2:
        print("Dois")
    3 | 4 | 5:
        print("Tres, quatro ou cinco")
    n if n > 10:
        print(f"Grande: {n}")
    _:
        print("Outro")
```

#### Match com Tipos

```mendes
let opt: Option<int> = Some(42)

match opt:
    Some(0):
        print("Zero")
    Some(n) if n < 0:
        print("Negativo")
    Some(n):
        print(f"Positivo: {n}")
    None:
        print("Nenhum")
```

### 5.3 For Loop

```mendes
# Iterando sobre array
let frutas = ["maca", "banana", "laranja"]
for fruta in frutas:
    print(fruta)

# Range exclusivo (0 ate 4)
for i in 0..5:
    print(i)   # 0, 1, 2, 3, 4

# Range inclusivo (0 ate 5)
for i in 0..=5:
    print(i)   # 0, 1, 2, 3, 4, 5

# Com indice
for (i, fruta) in frutas.enumerate():
    print(f"{i}: {fruta}")
```

### 5.4 While Loop

```mendes
let mut contador = 0

while contador < 5:
    print(contador)
    contador += 1

# While com condicao complexa
let mut tentativas = 0
let mut sucesso = false

while tentativas < 3 and not sucesso:
    sucesso = tentar_conexao()
    tentativas += 1
```

### 5.5 Break e Continue

```mendes
# Break - sai do loop
for i in 0..100:
    if i == 10:
        break
    print(i)   # 0 ate 9

# Continue - pula para proxima iteracao
for i in 0..10:
    if i % 2 == 0:
        continue
    print(i)   # 1, 3, 5, 7, 9
```

---

## 6. Funcoes

### 6.1 Declaracao Basica

```mendes
fn saudacao():
    print("Ola!")

fn soma(a: int, b: int) -> int:
    return a + b

fn multiplica(a: int, b: int) -> int:
    a * b   # Return implicito (ultima expressao)
```

### 6.2 Parametros e Retorno

```mendes
# Multiplos parametros
fn calcula(a: int, b: int, c: int) -> int:
    return a + b * c

# Retorno de tupla
fn divide_resto(a: int, b: int) -> (int, int):
    return (a / b, a % b)

let (q, r) = divide_resto(17, 5)

# Retorno Option
fn busca(items: [int], alvo: int) -> Option<int>:
    for (i, item) in items.enumerate():
        if item == alvo:
            return Some(i)
    return None
```

### 6.3 Funcoes Async

```mendes
fn busca_usuario(id: int) async -> Result<User, Error>:
    let response = await http.get(f"/users/{id}")
    if response.status != 200:
        return Err(Error::NotFound)
    return Ok(response.json())
```

### 6.4 Closures (Funcoes Anonimas)

```mendes
# Closure simples
let dobra = |x: int| x * 2

print(dobra(5))   # 10

# Closure com bloco
let processa = |x: int| -> int:
    let temp = x * 2
    return temp + 1

# Closures como parametro
fn aplica(arr: [int], f: fn(int) -> int) -> [int]:
    let mut resultado: [int] = []
    for item in arr:
        resultado.push(f(item))
    return resultado

let nums = [1, 2, 3, 4, 5]
let dobrados = aplica(nums, |x| x * 2)   # [2, 4, 6, 8, 10]

# Closures de alta ordem
let nums = [1, 2, 3, 4, 5]
let pares = nums.filter(|x| x % 2 == 0)     # [2, 4]
let dobrados = nums.map(|x| x * 2)          # [2, 4, 6, 8, 10]
let soma = nums.reduce(0, |acc, x| acc + x) # 15
```

### 6.5 Funcoes Publicas

```mendes
# Privada (padrao)
fn interna():
    # Visivel apenas no modulo atual
    pass

# Publica
pub fn externa():
    # Visivel para outros modulos
    pass
```

---

## 7. Structs e Metodos

### 7.1 Definindo Structs

```mendes
struct User:
    id: int
    name: string
    email: string
    created_at: string
```

### 7.2 Metodos

```mendes
struct Rectangle:
    width: float
    height: float

    # Metodo que empresta self (leitura)
    fn area(&self) -> float:
        return self.width * self.height

    # Metodo que empresta self mutavelmente
    fn scale(&mut self, factor: float):
        self.width *= factor
        self.height *= factor

    # Metodo que consome self
    fn into_square(self) -> Rectangle:
        let side = (self.width + self.height) / 2.0
        return Rectangle { width: side, height: side }

    # Funcao associada (sem self)
    fn new(width: float, height: float) -> Rectangle:
        return Rectangle { width: width, height: height }
```

### 7.3 Usando Metodos

```mendes
# Criando instancia
let mut rect = Rectangle::new(10.0, 5.0)

# Chamando metodos
let area = rect.area()      # 50.0
print(f"Area: {area}")

rect.scale(2.0)             # Agora 20x10
print(f"Nova area: {rect.area()}")  # 200.0

# Consumindo a instancia
let square = rect.into_square()
# rect nao pode mais ser usado aqui!
```

### 7.4 Structs Copy vs Move

```mendes
# Por padrao, structs sao "movidas"
struct User:
    name: string

let u1 = User { name: "Maria" }
let u2 = u1            # u1 foi MOVIDO para u2
# print(u1.name)       # ERRO! u1 nao e mais valido

# Structs "copy" sao copiadas
struct Point copy:
    x: float
    y: float

let p1 = Point { x: 1.0, y: 2.0 }
let p2 = p1            # p1 foi COPIADO
print(p1.x)            # OK! p1 ainda e valido
```

### 7.5 Struct com Generics

```mendes
struct Container<T>:
    value: T
    count: int

    fn new(value: T) -> Container<T>:
        return Container { value: value, count: 1 }

    fn get(&self) -> &T:
        return &self.value

let c1 = Container::new(42)
let c2 = Container::new("hello")
```

---

## 8. Enums e Pattern Matching

### 8.1 Enums Simples

```mendes
enum Direction:
    North
    South
    East
    West

let dir = Direction::North

match dir:
    Direction::North:
        print("Indo para o norte")
    Direction::South:
        print("Indo para o sul")
    _:
        print("Indo para leste ou oeste")
```

### 8.2 Enums com Dados

```mendes
enum Shape:
    Circle(float)                    # Raio
    Rectangle(float, float)          # Largura, altura
    Triangle { a: float, b: float, c: float }

fn area(shape: Shape) -> float:
    match shape:
        Shape::Circle(r):
            return 3.14159 * r * r
        Shape::Rectangle(w, h):
            return w * h
        Shape::Triangle { a, b, c }:
            # Formula de Heron
            let s = (a + b + c) / 2.0
            return (s * (s-a) * (s-b) * (s-c)).sqrt()
```

### 8.3 Option<T> em Detalhes

```mendes
enum Option<T>:
    Some(T)
    None

# Uso pratico
fn find_user(id: int) -> Option<User>:
    let users = get_all_users()
    for user in users:
        if user.id == id:
            return Some(user)
    return None

# Tratando Option
let maybe_user = find_user(42)

# Forma 1: Match
match maybe_user:
    Some(user):
        print(f"Encontrado: {user.name}")
    None:
        print("Usuario nao encontrado")

# Forma 2: If let
if maybe_user is Some(user):
    print(f"Encontrado: {user.name}")

# Forma 3: Metodos
let name = maybe_user.map(|u| u.name).unwrap_or("Anonimo")
```

### 8.4 Result<T, E> em Detalhes

```mendes
enum Result<T, E>:
    Ok(T)
    Err(E)

# Uso pratico
fn parse_number(s: string) -> Result<int, string>:
    # Tenta converter
    match s.parse_int():
        Some(n):
            return Ok(n)
        None:
            return Err(f"'{s}' nao e um numero valido")

# Tratando Result
let result = parse_number("42")

match result:
    Ok(n):
        print(f"Numero: {n}")
    Err(e):
        print(f"Erro: {e}")

# Operador ? para propagacao
fn soma_strings(a: string, b: string) -> Result<int, string>:
    let x = parse_number(a)?   # Retorna Err se falhar
    let y = parse_number(b)?
    return Ok(x + y)
```

### 8.5 Pattern Matching Avancado

```mendes
# Guardas
match numero:
    n if n < 0:
        print("Negativo")
    n if n == 0:
        print("Zero")
    n if n > 0 and n < 100:
        print("Pequeno positivo")
    _:
        print("Grande")

# Or patterns
match char:
    'a' | 'e' | 'i' | 'o' | 'u':
        print("Vogal")
    _:
        print("Consoante")

# Desestruturacao aninhada
struct Point { x: int, y: int }
enum Shape:
    Circle { center: Point, radius: float }

let shape = Shape::Circle {
    center: Point { x: 0, y: 0 },
    radius: 5.0
}

match shape:
    Shape::Circle { center: Point { x: 0, y: 0 }, radius }:
        print(f"Circulo na origem com raio {radius}")
    Shape::Circle { center, radius }:
        print(f"Circulo em ({center.x}, {center.y})")
```

---

## 9. Sistema de Ownership

### 9.1 Conceitos Fundamentais

Mendes usa um sistema de ownership inspirado em Rust:

1. **Cada valor tem um unico dono**
2. **Quando o dono sai de escopo, o valor e destruido**
3. **Valores podem ser emprestados temporariamente**

### 9.2 Move Semantics

```mendes
struct Data:
    value: string

let d1 = Data { value: "importante" }
let d2 = d1   # d1 foi MOVIDO para d2

# print(d1.value)  # ERRO! d1 nao e mais valido
print(d2.value)    # OK
```

### 9.3 Referencias e Emprestimos

```mendes
struct User:
    name: string
    age: int

fn print_user(user: &User):   # Emprestimo imutavel
    print(f"{user.name}, {user.age} anos")
    # Nao pode modificar user

fn birthday(user: &mut User):  # Emprestimo mutavel
    user.age += 1

let mut u = User { name: "Maria", age: 30 }

print_user(&u)      # Empresta imutavelmente
birthday(&mut u)    # Empresta mutavelmente
print(u.age)        # 31
```

### 9.4 Regras de Emprestimo

```mendes
let mut data = "hello"

# Regra 1: Multiplas referencias imutaveis sao OK
let r1 = &data
let r2 = &data
print(r1)   # OK
print(r2)   # OK

# Regra 2: Apenas UMA referencia mutavel por vez
let r3 = &mut data
# let r4 = &mut data  # ERRO! Ja existe referencia mutavel

# Regra 3: Referencias imutaveis e mutaveis nao podem coexistir
let r5 = &data
# let r6 = &mut data  # ERRO! Ja existe referencia imutavel
```

### 9.5 Lifetime Implicito

Mendes gerencia lifetimes automaticamente na maioria dos casos:

```mendes
fn first_word(s: &string) -> &string:
    # Retorna referencia com mesmo lifetime do parametro
    for (i, char) in s.chars().enumerate():
        if char == ' ':
            return &s[0..i]
    return s

let sentence = "hello world"
let word = first_word(&sentence)
print(word)   # "hello"
```

---

## 10. Generics e Traits

### 10.1 Funcoes Genericas

```mendes
fn identity<T>(x: T) -> T:
    return x

let a = identity(42)        # T = int
let b = identity("hello")   # T = string

fn swap<T>(a: T, b: T) -> (T, T):
    return (b, a)

let (x, y) = swap(1, 2)     # (2, 1)
```

### 10.2 Structs Genericas

```mendes
struct Pair<T, U>:
    first: T
    second: U

let p1 = Pair { first: 1, second: "one" }
let p2 = Pair { first: true, second: 3.14 }

struct Stack<T>:
    items: [T]

    fn new() -> Stack<T>:
        return Stack { items: [] }

    fn push(&mut self, item: T):
        self.items.push(item)

    fn pop(&mut self) -> Option<T>:
        return self.items.pop()

let mut stack = Stack::new()
stack.push(1)
stack.push(2)
let top = stack.pop()   # Some(2)
```

### 10.3 Traits

Traits definem comportamento compartilhado:

```mendes
trait Display:
    fn display(&self) -> string

trait Clone:
    fn clone(&self) -> Self

struct User:
    name: string
    age: int

impl Display for User:
    fn display(&self) -> string:
        return f"User({self.name}, {self.age})"

impl Clone for User:
    fn clone(&self) -> User:
        return User { name: self.name, age: self.age }

let u = User { name: "Maria", age: 30 }
print(u.display())   # "User(Maria, 30)"
let u2 = u.clone()
```

### 10.4 Trait Bounds

```mendes
fn print_all<T: Display>(items: [T]):
    for item in items:
        print(item.display())

fn clone_and_print<T: Clone + Display>(item: T):
    let copy = item.clone()
    print(copy.display())
```

---

## 11. HTTP e APIs

### 11.1 Configuracao do Servidor

```mendes
server:
    host "0.0.0.0"    # Escuta em todas as interfaces
    port 8080         # Porta 8080
```

### 11.2 Metodos HTTP

```mendes
# GET - Buscar recursos
api GET /users:
    return [User]
    return get_all_users()

# POST - Criar recursos
api POST /users:
    body User
    return User
    return create_user(body)

# PUT - Atualizar recursos
api PUT /users/{id:int}:
    body User
    return User
    return update_user(id, body)

# DELETE - Remover recursos
api DELETE /users/{id:int}:
    return bool
    return delete_user(id)

# PATCH - Atualizar parcialmente
api PATCH /users/{id:int}:
    body PartialUser
    return User
    return patch_user(id, body)
```

### 11.3 Parametros de Rota

```mendes
# Parametro simples
api GET /users/{id:int}:
    return User
    return find_user(id)

# Multiplos parametros
api GET /posts/{year:int}/{month:int}:
    return [Post]
    return find_posts_by_date(year, month)

# Parametro string
api GET /users/{username:string}:
    return User
    return find_by_username(username)
```

### 11.4 Request Body

```mendes
struct CreateUser:
    name: string
    email: string
    password: string

api POST /users:
    body CreateUser
    return User

    # O body e automaticamente deserializado
    let user = create_user(body.name, body.email, body.password)
    return user
```

### 11.5 Query Parameters

```mendes
struct UserQuery:
    page: int
    limit: int
    sort: string

api GET /users:
    query UserQuery
    return [User]

    # /users?page=1&limit=10&sort=name
    return find_users(query.page, query.limit, query.sort)
```

### 11.6 Headers

```mendes
api GET /protected:
    return string

    let token = request.header("Authorization")
    if token is None:
        return Response { status: 401, body: "Unauthorized" }

    if not validate_token(token):
        return Response { status: 403, body: "Forbidden" }

    return "Secret data"
```

### 11.7 Responses Customizadas

```mendes
api POST /users:
    body CreateUser
    return User

    let user = create_user(body)

    return Response:
        status 201
        header "Location" f"/users/{user.id}"
        body user
```

### 11.8 Middleware

```mendes
# Definindo middleware
middleware auth:
    let token = request.header("Authorization")
    if token is None:
        return Response { status: 401, body: "Token required" }

    match validate_token(token):
        Ok(user_id):
            request.set("user_id", user_id)
        Err(_):
            return Response { status: 401, body: "Invalid token" }

middleware log:
    let start = time.now()
    # Continua para o handler
    let response = next()
    let duration = time.since(start)
    print(f"{request.method} {request.path} - {duration}ms")
    return response

# Usando middleware
api GET /admin/users:
    use auth
    use log
    return [User]

    let user_id = request.get("user_id")
    return get_users_for_admin(user_id)
```

---

## 12. Banco de Dados

### 12.1 Configuracao

```mendes
# PostgreSQL
db postgres main:
    url "postgres://user:pass@localhost/mydb"
    pool 20

# MySQL
db mysql secondary:
    url "mysql://user:pass@localhost/mydb"
    pool 10

# SQLite
db sqlite local:
    url "sqlite://./data.db"
    pool 5
```

### 12.2 Queries Basicas

```mendes
# SELECT multiplas linhas
api GET /users async:
    return [User]
    let users = await db.main.query("SELECT * FROM users")
    return users

# SELECT uma linha
api GET /users/{id:int} async:
    return Option<User>
    let user = await db.main.query_one(
        "SELECT * FROM users WHERE id = $1",
        id
    )
    return user

# INSERT
api POST /users async:
    body CreateUser
    return User
    let user = await db.main.query_one(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        body.name, body.email
    )
    return user

# UPDATE
api PUT /users/{id:int} async:
    body User
    return User
    let user = await db.main.query_one(
        "UPDATE users SET name = $1, email = $2 WHERE id = $3 RETURNING *",
        body.name, body.email, id
    )
    return user

# DELETE
api DELETE /users/{id:int} async:
    return bool
    await db.main.execute(
        "DELETE FROM users WHERE id = $1",
        id
    )
    return true
```

### 12.3 Transacoes

```mendes
api POST /transfer async:
    body TransferRequest
    return Result<Transfer, string>

    await db.main.transaction:
        # Debita da conta origem
        await db.main.execute(
            "UPDATE accounts SET balance = balance - $1 WHERE id = $2",
            body.amount, body.from_account
        )

        # Credita na conta destino
        await db.main.execute(
            "UPDATE accounts SET balance = balance + $1 WHERE id = $2",
            body.amount, body.to_account
        )

        # Registra a transferencia
        let transfer = await db.main.query_one(
            "INSERT INTO transfers (from_id, to_id, amount) VALUES ($1, $2, $3) RETURNING *",
            body.from_account, body.to_account, body.amount
        )

        return Ok(transfer)
```

### 12.4 Prepared Statements

```mendes
# Parametros sao automaticamente escapados (SQL injection safe)
let name = "Robert'); DROP TABLE users;--"
let users = await db.main.query(
    "SELECT * FROM users WHERE name = $1",
    name
)
# Query segura - parametro e tratado como dado, nao SQL
```

---

## 13. Async/Await

### 13.1 Funcoes Async

```mendes
fn fetch_data() async -> Result<Data, Error>:
    let response = await http.get("https://api.example.com/data")
    return Ok(response.json())

fn process_all() async -> [Result]:
    let data = await fetch_data()
    let processed = await process(data)
    return processed
```

### 13.2 Await

```mendes
api GET /composite async:
    return CompositeData

    # Operacoes sequenciais
    let users = await db.main.query("SELECT * FROM users")
    let orders = await db.main.query("SELECT * FROM orders")

    return CompositeData { users: users, orders: orders }
```

### 13.3 Concorrencia

```mendes
api GET /dashboard async:
    return Dashboard

    # Executar em paralelo
    let (users, orders, stats) = await all(
        db.main.query("SELECT * FROM users"),
        db.main.query("SELECT * FROM orders"),
        compute_stats()
    )

    return Dashboard {
        users: users,
        orders: orders,
        stats: stats
    }
```

### 13.4 Timeout

```mendes
fn fetch_with_timeout() async -> Result<Data, Error>:
    match await timeout(5000, http.get(url)):
        Ok(response):
            return Ok(response.json())
        Err(_):
            return Err(Error::Timeout)
```

---

## 14. WebSockets

### 14.1 Endpoint Basico

```mendes
server:
    host "0.0.0.0"
    port 8080

ws /chat:
    on_connect:
        print(f"Cliente {conn.id} conectou")

    on_message:
        print(f"Mensagem de {conn.id}: {message}")
        conn.send(f"Echo: {message}")

    on_disconnect:
        print(f"Cliente {conn.id} desconectou")
```

### 14.2 Broadcast

```mendes
ws /chat:
    on_connect:
        broadcast(f"Usuario {conn.id} entrou no chat")

    on_message:
        broadcast(f"[{conn.id}]: {message}")

    on_disconnect:
        broadcast(f"Usuario {conn.id} saiu do chat")
```

### 14.3 Rooms

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

### 14.4 Estado por Conexao

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
        broadcast(f"{user.nickname} entrou")

    on_message:
        let user = conn.get_state<ChatUser>()
        broadcast(f"[{user.nickname}]: {message}")
```

---

## 15. Tratamento de Erros

### 15.1 Result e Operador ?

```mendes
fn divide(a: int, b: int) -> Result<int, string>:
    if b == 0:
        return Err("Divisao por zero")
    return Ok(a / b)

fn calculate() -> Result<int, string>:
    let x = divide(10, 2)?   # Propaga erro se Err
    let y = divide(x, 2)?
    return Ok(y)

# Tratando o resultado
match calculate():
    Ok(result):
        print(f"Resultado: {result}")
    Err(error):
        print(f"Erro: {error}")
```

### 15.2 Try-Catch (via Match)

```mendes
fn risky_operation() -> Result<Data, Error>:
    # ...

let result = risky_operation()

match result:
    Ok(data):
        process(data)
    Err(Error::NotFound):
        print("Recurso nao encontrado")
    Err(Error::Timeout):
        print("Operacao expirou")
    Err(e):
        print(f"Erro desconhecido: {e}")
```

### 15.3 Erros HTTP

```mendes
enum HttpError:
    NotFound(string)
    BadRequest(string)
    Unauthorized
    InternalError(string)

fn to_response(error: HttpError) -> Response:
    match error:
        HttpError::NotFound(msg):
            return Response { status: 404, body: msg }
        HttpError::BadRequest(msg):
            return Response { status: 400, body: msg }
        HttpError::Unauthorized:
            return Response { status: 401, body: "Unauthorized" }
        HttpError::InternalError(msg):
            return Response { status: 500, body: msg }

api GET /users/{id:int} async:
    return Result<User, HttpError>

    match await find_user(id):
        Some(user):
            return Ok(user)
        None:
            return Err(HttpError::NotFound(f"User {id} not found"))
```

### 15.4 Unwrap e Expect

```mendes
# unwrap - panic se None/Err
let user = find_user(id).unwrap()

# expect - panic com mensagem customizada
let user = find_user(id).expect("Usuario deve existir")

# unwrap_or - valor padrao
let user = find_user(id).unwrap_or(default_user)

# unwrap_or_else - valor computado
let user = find_user(id).unwrap_or_else(|| create_default())
```

---

## 16. Modulos e Imports

### 16.1 Estrutura de Modulos

```
projeto/
├── main.ms           # Ponto de entrada
├── models/
│   ├── mod.ms        # Declaracao do modulo
│   ├── user.ms       # Definicao de User
│   └── post.ms       # Definicao de Post
├── handlers/
│   ├── mod.ms
│   ├── users.ms
│   └── posts.ms
└── utils/
    ├── mod.ms
    └── validation.ms
```

### 16.2 Declarando Modulos

```mendes
# models/mod.ms
module models

pub use user::User
pub use post::Post
```

```mendes
# models/user.ms
module models::user

pub struct User:
    id: int
    name: string
    email: string

pub fn create(name: string, email: string) -> User:
    return User { id: 0, name: name, email: email }
```

### 16.3 Importando

```mendes
# Import simples
import models::user

let u = user::User { ... }

# Import com alias
import models::user as u

let user = u::User { ... }

# From import
from models::user import User, create

let user = User { ... }

# Import all
from models import *

let user = User { ... }
let post = Post { ... }
```

### 16.4 Visibilidade

```mendes
# Privado (padrao) - visivel apenas no modulo
fn internal_function():
    pass

struct InternalData:
    value: int

# Publico - visivel para outros modulos
pub fn external_function():
    pass

pub struct PublicData:
    value: int
```

---

## 17. Projeto Final

Vamos construir uma API REST completa de gerenciamento de tarefas (Todo List).

### 17.1 Estrutura do Projeto

```
todo-api/
├── main.ms
├── models.ms
├── handlers.ms
└── middleware.ms
```

### 17.2 models.ms

```mendes
# models.ms - Definicao dos modelos de dados

module models

pub struct Todo:
    id: int
    title: string
    description: string
    completed: bool
    created_at: string
    user_id: int

pub struct CreateTodo:
    title: string
    description: string

pub struct UpdateTodo:
    title: Option<string>
    description: Option<string>
    completed: Option<bool>

pub struct User:
    id: int
    username: string
    email: string
    password_hash: string

pub struct CreateUser:
    username: string
    email: string
    password: string

pub struct LoginRequest:
    email: string
    password: string

pub struct AuthToken:
    token: string
    expires_at: string
```

### 17.3 middleware.ms

```mendes
# middleware.ms - Middlewares da aplicacao

module middleware

from models import User

pub middleware auth:
    let auth_header = request.header("Authorization")

    if auth_header is None:
        return Response { status: 401, body: "Token required" }

    let token = auth_header.replace("Bearer ", "")

    match validate_jwt(token):
        Ok(user_id):
            request.set("user_id", user_id)
        Err(e):
            return Response { status: 401, body: f"Invalid token: {e}" }

pub middleware log:
    let start = time.now()
    let method = request.method
    let path = request.path

    let response = next()

    let duration = time.since(start)
    let status = response.status

    print(f"{method} {path} -> {status} ({duration}ms)")

    return response

fn validate_jwt(token: string) -> Result<int, string>:
    # Implementacao JWT aqui
    pass
```

### 17.4 handlers.ms

```mendes
# handlers.ms - Handlers HTTP

module handlers

from models import *
from middleware import auth, log

# ============================================
# Autenticacao
# ============================================

api POST /auth/register async:
    use log
    body CreateUser
    return Result<User, string>

    # Verificar se email ja existe
    let existing = await db.main.query_one(
        "SELECT id FROM users WHERE email = $1",
        body.email
    )

    if existing is Some:
        return Err("Email already registered")

    # Hash da senha
    let password_hash = hash_password(body.password)

    # Criar usuario
    let user = await db.main.query_one(
        "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
        body.username, body.email, password_hash
    )

    return Ok(user)

api POST /auth/login async:
    use log
    body LoginRequest
    return Result<AuthToken, string>

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

            let token = generate_jwt(u.id)
            return Ok(AuthToken {
                token: token,
                expires_at: time.add_hours(24)
            })

# ============================================
# Todos
# ============================================

api GET /todos async:
    use auth
    use log
    return [Todo]

    let user_id = request.get("user_id")

    let todos = await db.main.query(
        "SELECT * FROM todos WHERE user_id = $1 ORDER BY created_at DESC",
        user_id
    )

    return todos

api GET /todos/{id:int} async:
    use auth
    use log
    return Result<Todo, string>

    let user_id = request.get("user_id")

    let todo = await db.main.query_one(
        "SELECT * FROM todos WHERE id = $1 AND user_id = $2",
        id, user_id
    )

    match todo:
        Some(t):
            return Ok(t)
        None:
            return Err("Todo not found")

api POST /todos async:
    use auth
    use log
    body CreateTodo
    return Todo

    let user_id = request.get("user_id")

    let todo = await db.main.query_one(
        "INSERT INTO todos (title, description, completed, user_id, created_at) VALUES ($1, $2, false, $3, NOW()) RETURNING *",
        body.title, body.description, user_id
    )

    return todo

api PUT /todos/{id:int} async:
    use auth
    use log
    body UpdateTodo
    return Result<Todo, string>

    let user_id = request.get("user_id")

    # Verificar se existe e pertence ao usuario
    let existing = await db.main.query_one(
        "SELECT id FROM todos WHERE id = $1 AND user_id = $2",
        id, user_id
    )

    if existing is None:
        return Err("Todo not found")

    # Construir query dinamicamente
    let mut updates: [string] = []
    let mut params: [string] = []

    if body.title is Some(title):
        updates.push("title = $" + (params.len() + 1).to_string())
        params.push(title)

    if body.description is Some(desc):
        updates.push("description = $" + (params.len() + 1).to_string())
        params.push(desc)

    if body.completed is Some(comp):
        updates.push("completed = $" + (params.len() + 1).to_string())
        params.push(comp.to_string())

    if updates.is_empty():
        return Err("No fields to update")

    params.push(id.to_string())

    let query = f"UPDATE todos SET {updates.join(', ')} WHERE id = ${params.len()} RETURNING *"

    let todo = await db.main.query_one(query, ...params)

    return Ok(todo)

api DELETE /todos/{id:int} async:
    use auth
    use log
    return Result<bool, string>

    let user_id = request.get("user_id")

    let result = await db.main.execute(
        "DELETE FROM todos WHERE id = $1 AND user_id = $2",
        id, user_id
    )

    if result.rows_affected == 0:
        return Err("Todo not found")

    return Ok(true)

# ============================================
# Health Check
# ============================================

api GET /health:
    use log
    return string
    return "ok"
```

### 17.5 main.ms

```mendes
# main.ms - Ponto de entrada da aplicacao

module main

import handlers
import middleware

# Configuracao do banco de dados
db postgres main:
    url "postgres://user:pass@localhost/todo_app"
    pool 20

# Configuracao do servidor
server:
    host "0.0.0.0"
    port 8080

# Inicializacao
fn init():
    print("Todo API iniciada em http://0.0.0.0:8080")
    print("Endpoints disponiveis:")
    print("  POST   /auth/register")
    print("  POST   /auth/login")
    print("  GET    /todos")
    print("  GET    /todos/{id}")
    print("  POST   /todos")
    print("  PUT    /todos/{id}")
    print("  DELETE /todos/{id}")
    print("  GET    /health")
```

### 17.6 Compilando e Executando

```bash
# Compilar
mendes build main.ms -o todo-api

# Executar
./todo-api

# Testar
curl http://localhost:8080/health
# ok

# Registrar usuario
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"maria","email":"maria@email.com","password":"123456"}'

# Login
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"maria@email.com","password":"123456"}'
# {"token":"eyJ...","expires_at":"..."}

# Criar todo (com token)
curl -X POST http://localhost:8080/todos \
  -H "Authorization: Bearer eyJ..." \
  -H "Content-Type: application/json" \
  -d '{"title":"Aprender Mendes","description":"Tutorial completo"}'

# Listar todos
curl http://localhost:8080/todos \
  -H "Authorization: Bearer eyJ..."
```

---

## Conclusao

Parabens! Voce completou o tutorial da linguagem Mendes. Agora voce sabe:

- **Sintaxe basica**: Variaveis, tipos, controle de fluxo
- **Funcoes e closures**: Declaracao, parametros, retorno
- **Structs e enums**: Dados estruturados e algebraicos
- **Ownership**: Sistema de gerenciamento de memoria
- **Generics e traits**: Codigo reutilizavel e polimorfismo
- **HTTP nativo**: APIs REST completas
- **Banco de dados**: Queries, transacoes, pools
- **Async/await**: Programacao assincrona
- **WebSockets**: Comunicacao em tempo real
- **Tratamento de erros**: Result, Option, operador ?
- **Modulos**: Organizacao de codigo

### Proximos Passos

1. Leia a [Referencia da Linguagem](language-reference.md) para detalhes completos
2. Explore os [Exemplos](examples.md) para mais padroes
3. Consulte a [API do Runtime](runtime-api.md) para funcoes disponiveis
4. Contribua com o projeto no [GitHub](https://github.com/guilhermemendes/linguagem-mendes)

---

<p align="center">
  <strong>Bom desenvolvimento com Mendes!</strong>
</p>
