# Referencia Completa da Linguagem Mendes

> **Versao**: 0.1.0
> **Status**: Especificacao Oficial

Este documento e a referencia tecnica completa da linguagem Mendes. Ele descreve todos os aspectos da linguagem em detalhes, servindo como fonte definitiva para implementadores e usuarios avancados.

---

## Indice

1. [Visao Geral](#1-visao-geral)
2. [Estrutura Lexica](#2-estrutura-lexica)
3. [Tipos](#3-tipos)
4. [Expressoes](#4-expressoes)
5. [Statements](#5-statements)
6. [Funcoes](#6-funcoes)
7. [Structs](#7-structs)
8. [Enums](#8-enums)
9. [Traits](#9-traits)
10. [Generics](#10-generics)
11. [Pattern Matching](#11-pattern-matching)
12. [Ownership e Referencias](#12-ownership-e-referencias)
13. [Modulos e Visibilidade](#13-modulos-e-visibilidade)
14. [HTTP e APIs](#14-http-e-apis)
15. [Banco de Dados](#15-banco-de-dados)
16. [WebSockets](#16-websockets)
17. [Async/Await](#17-asyncawait)
18. [Tratamento de Erros](#18-tratamento-de-erros)
19. [Operadores](#19-operadores)
20. [Keywords Reservadas](#20-keywords-reservadas)

---

## 1. Visao Geral

### 1.1 Caracteristicas da Linguagem

| Caracteristica | Valor |
|----------------|-------|
| Paradigma | Multi-paradigma (imperativo, funcional) |
| Tipagem | Estatica, forte, inferida |
| Memoria | Ownership (sem GC) |
| Compilacao | AOT para binario nativo |
| Concorrencia | Async/await nativo |

### 1.2 Extensao de Arquivo

Arquivos Mendes usam a extensao `.ms`:

```
programa.ms
modulo.ms
```

### 1.3 Encoding

Arquivos fonte devem ser UTF-8. Identificadores podem conter caracteres Unicode da categoria XID_Start e XID_Continue.

---

## 2. Estrutura Lexica

### 2.1 Indentacao

Mendes usa **indentacao significativa**. Blocos sao delimitados por indentacao, nao por chaves.

```
INDENT  = Aumento de nivel de indentacao
DEDENT  = Reducao de nivel de indentacao
```

Regras:
- Indentacao pode ser espacos ou tabs, mas nao misturados
- Recomendado: 4 espacos por nivel
- Blocos comecam apos `:` e `NEWLINE`

```mendes
if x > 0:
    print("positivo")    # INDENT
    if x > 10:
        print("grande")  # INDENT
                         # DEDENT
                         # DEDENT
```

### 2.2 Comentarios

```ebnf
comment = "#" [^\n]* NEWLINE
```

Apenas comentarios de linha unica:

```mendes
# Isso e um comentario
let x = 42  # Comentario no final da linha
```

### 2.3 Identificadores

```ebnf
identifier = (LETTER | "_") (LETTER | DIGIT | "_")*
LETTER     = [a-zA-Z] | Unicode_XID_Start
DIGIT      = [0-9]
```

Exemplos validos:
```mendes
x
_private
userName
user_name
usuario123
_123
```

Exemplos invalidos:
```mendes
123abc    # Comeca com numero
my-var    # Hifen nao permitido
```

### 2.4 Literais

#### 2.4.1 Inteiros

```ebnf
int_literal = decimal | hexadecimal | binary | octal
decimal     = DIGIT+
hexadecimal = "0x" HEX_DIGIT+
binary      = "0b" BIN_DIGIT+
octal       = "0o" OCT_DIGIT+
```

| Formato | Exemplo | Valor Decimal |
|---------|---------|---------------|
| Decimal | `42` | 42 |
| Hexadecimal | `0xFF` | 255 |
| Binario | `0b1010` | 10 |
| Octal | `0o777` | 511 |

#### 2.4.2 Floats

```ebnf
float_literal = DIGIT+ "." DIGIT+ exponent?
exponent      = ("e" | "E") ("+" | "-")? DIGIT+
```

| Exemplo | Valor |
|---------|-------|
| `3.14` | 3.14 |
| `0.5` | 0.5 |
| `1e10` | 10000000000.0 |
| `2.5e-3` | 0.0025 |

#### 2.4.3 Strings

```ebnf
string_literal = '"' string_char* '"'
string_char    = escape_sequence | [^"\\]
escape_sequence = "\\" ( "n" | "t" | "r" | "\\" | '"' | "'" | "0" )
```

| Escape | Significado |
|--------|-------------|
| `\n` | Nova linha |
| `\t` | Tab |
| `\r` | Retorno de carro |
| `\\` | Barra invertida |
| `\"` | Aspas duplas |
| `\'` | Aspas simples |
| `\0` | Null |

#### 2.4.4 Strings Interpoladas

```ebnf
interpolated_string = 'f"' (string_char | "{" expression "}")* '"'
```

```mendes
let nome = "Maria"
let msg = f"Ola, {nome}!"           # "Ola, Maria!"
let calc = f"2 + 2 = {2 + 2}"       # "2 + 2 = 4"
let json = f"{{\"name\": \"{nome}\"}}"  # {"name": "Maria"}
```

#### 2.4.5 Booleanos

```mendes
true
false
```

#### 2.4.6 None

```mendes
None
```

### 2.5 Pontuacao e Operadores

| Token | Significado |
|-------|-------------|
| `+` | Adicao |
| `-` | Subtracao |
| `*` | Multiplicacao |
| `/` | Divisao |
| `%` | Modulo |
| `==` | Igualdade |
| `!=` | Diferenca |
| `<` | Menor que |
| `<=` | Menor ou igual |
| `>` | Maior que |
| `>=` | Maior ou igual |
| `=` | Atribuicao |
| `+=` | Adicao e atribuicao |
| `-=` | Subtracao e atribuicao |
| `*=` | Multiplicacao e atribuicao |
| `/=` | Divisao e atribuicao |
| `&` | Referencia imutavel |
| `&mut` | Referencia mutavel |
| `->` | Tipo de retorno |
| `:` | Delimitador de bloco/tipo |
| `::` | Separador de caminho |
| `,` | Separador |
| `.` | Acesso a membro |
| `..` | Range exclusivo |
| `..=` | Range inclusivo |
| `(` `)` | Agrupamento/chamada |
| `{` `}` | Struct literal |
| `[` `]` | Array/indice |
| `<` `>` | Generics |
| `\|` | Closure/or pattern |
| `?` | Operador try |

---

## 3. Tipos

### 3.1 Tipos Primitivos

| Tipo | Descricao | Tamanho | Range |
|------|-----------|---------|-------|
| `int` | Inteiro com sinal | 64 bits | -2^63 a 2^63-1 |
| `float` | Ponto flutuante | 64 bits | IEEE 754 double |
| `bool` | Booleano | 8 bits | true, false |
| `string` | Texto UTF-8 | Variavel | - |

### 3.2 Tipos Compostos

#### 3.2.1 Arrays

```mendes
[T]        # Array de T

# Exemplos
let nums: [int] = [1, 2, 3]
let strs: [string] = ["a", "b"]
let nested: [[int]] = [[1, 2], [3, 4]]
```

#### 3.2.2 Tuplas

```mendes
(T1, T2, ...)    # Tupla

# Exemplos
let pair: (int, string) = (42, "hello")
let triple: (int, int, int) = (1, 2, 3)
```

#### 3.2.3 Option

```mendes
Option<T>    # Some(T) ou None

# Exemplos
let maybe: Option<int> = Some(42)
let nothing: Option<int> = None
```

#### 3.2.4 Result

```mendes
Result<T, E>    # Ok(T) ou Err(E)

# Exemplos
let success: Result<int, string> = Ok(42)
let failure: Result<int, string> = Err("erro")
```

### 3.3 Tipos de Referencia

```mendes
&T        # Referencia imutavel
&mut T    # Referencia mutavel
```

### 3.4 Tipos Funcionais

```mendes
fn(T1, T2) -> R    # Tipo de funcao

# Exemplo
let f: fn(int, int) -> int = |a, b| a + b
```

### 3.5 Tipos Definidos pelo Usuario

```mendes
struct User { ... }    # Struct
enum Status { ... }    # Enum
type UserId = int      # Alias
```

### 3.6 Inferencia de Tipos

O compilador infere tipos quando possivel:

```mendes
let x = 42          # int inferido
let y = 3.14        # float inferido
let z = "hello"     # string inferido
let b = true        # bool inferido
let arr = [1, 2, 3] # [int] inferido
```

---

## 4. Expressoes

### 4.1 Expressoes Primarias

```ebnf
primary = literal
        | identifier
        | "(" expression ")"
        | struct_literal
        | array_literal
        | tuple_literal
        | closure
```

### 4.2 Expressoes de Acesso

```mendes
# Acesso a campo
user.name

# Acesso a indice
array[0]

# Chamada de metodo
user.display()

# Chamada de funcao
calculate(10, 20)
```

### 4.3 Expressoes Unarias

```mendes
-x          # Negacao
not x       # Negacao logica
&x          # Referencia
&mut x      # Referencia mutavel
await expr  # Await
```

### 4.4 Expressoes Binarias

Veja [Operadores](#19-operadores) para precedencia.

```mendes
a + b       # Aritmetica
a == b      # Comparacao
a and b     # Logica
a = b       # Atribuicao
```

### 4.5 Expressoes de Controle

```mendes
# If expression
let result = if x > 0: "pos" else: "neg"

# Match expression
let name = match status:
    Active: "ativo"
    Inactive: "inativo"
```

### 4.6 Closures

```mendes
# Sem tipo
|x| x * 2

# Com tipos
|x: int| -> int: x * 2

# Com bloco
|x: int| -> int:
    let temp = x * 2
    return temp + 1
```

### 4.7 Struct Literals

```mendes
User {
    name: "Maria",
    age: 30,
    active: true
}
```

### 4.8 Array Literals

```mendes
[1, 2, 3, 4, 5]
["a", "b", "c"]
[]  # Array vazio (tipo deve ser inferido ou anotado)
```

### 4.9 Tuple Literals

```mendes
(1, "hello", true)
(x, y)
```

### 4.10 Range Expressions

```mendes
0..10       # Exclusivo: 0 ate 9
0..=10      # Inclusivo: 0 ate 10
..10        # Ate 10 (inicio implicito)
0..         # A partir de 0 (fim implicito)
```

---

## 5. Statements

### 5.1 Let Statement

```ebnf
let_stmt = "let" "mut"? identifier (":" type)? "=" expression NEWLINE
```

```mendes
let x = 42                    # Imutavel, tipo inferido
let y: int = 42               # Imutavel, tipo explicito
let mut z = 42                # Mutavel
let mut w: int = 42           # Mutavel, tipo explicito
```

### 5.2 Assignment Statement

```ebnf
assign_stmt = expression "=" expression NEWLINE
            | expression "+=" expression NEWLINE
            | expression "-=" expression NEWLINE
            | expression "*=" expression NEWLINE
            | expression "/=" expression NEWLINE
```

```mendes
x = 10
x += 5
x -= 2
x *= 3
x /= 2
```

### 5.3 If Statement

```ebnf
if_stmt = "if" expression ":" NEWLINE INDENT block DEDENT else_clause?
else_clause = "else" ":" NEWLINE INDENT block DEDENT
            | "else" if_stmt
```

```mendes
if x > 0:
    print("positivo")
else if x < 0:
    print("negativo")
else:
    print("zero")
```

### 5.4 For Statement

```ebnf
for_stmt = "for" identifier "in" expression ":" NEWLINE INDENT block DEDENT
```

```mendes
for item in items:
    process(item)

for i in 0..10:
    print(i)

for (index, value) in items.enumerate():
    print(f"{index}: {value}")
```

### 5.5 While Statement

```ebnf
while_stmt = "while" expression ":" NEWLINE INDENT block DEDENT
```

```mendes
while condition:
    do_something()
```

### 5.6 Match Statement

```ebnf
match_stmt = "match" expression ":" NEWLINE INDENT match_arm+ DEDENT
match_arm = pattern ("if" expression)? ":" NEWLINE INDENT block DEDENT
```

```mendes
match value:
    1:
        print("um")
    2 | 3:
        print("dois ou tres")
    n if n > 10:
        print("grande")
    _:
        print("outro")
```

### 5.7 Return Statement

```ebnf
return_stmt = "return" expression? NEWLINE
```

```mendes
return 42
return Ok(result)
return  # Retorna unit ()
```

### 5.8 Break e Continue

```ebnf
break_stmt = "break" NEWLINE
continue_stmt = "continue" NEWLINE
```

```mendes
for i in 0..100:
    if i == 50:
        break
    if i % 2 == 0:
        continue
    print(i)
```

### 5.9 Expression Statement

```ebnf
expr_stmt = expression NEWLINE
```

```mendes
print("hello")
user.save()
calculate(10, 20)
```

---

## 6. Funcoes

### 6.1 Declaracao

```ebnf
fn_decl = "pub"? "fn" identifier generic_params? "(" param_list? ")" return_type? "async"? ":" NEWLINE INDENT block DEDENT
```

```mendes
fn simple():
    print("hello")

fn add(a: int, b: int) -> int:
    return a + b

fn async_fetch() async -> Result<Data, Error>:
    let data = await http.get(url)
    return Ok(data)

pub fn public_function():
    # Visivel para outros modulos
    pass
```

### 6.2 Parametros

```mendes
# Parametros por valor (move ou copy)
fn consume(data: Data):
    # data e movido para ca

# Parametros por referencia
fn read(data: &Data):
    # emprestimo imutavel

fn modify(data: &mut Data):
    # emprestimo mutavel
```

### 6.3 Retorno

```mendes
# Retorno explicito
fn explicit() -> int:
    return 42

# Retorno implicito (ultima expressao)
fn implicit() -> int:
    42

# Sem retorno
fn no_return():
    print("hello")
```

### 6.4 Funcoes Genericas

```mendes
fn identity<T>(x: T) -> T:
    return x

fn swap<T>(a: T, b: T) -> (T, T):
    return (b, a)

fn print_all<T: Display>(items: [T]):
    for item in items:
        print(item.display())
```

### 6.5 Funcoes Async

```mendes
fn fetch_user(id: int) async -> Result<User, Error>:
    let response = await http.get(f"/users/{id}")
    return response.json()
```

---

## 7. Structs

### 7.1 Declaracao

```ebnf
struct_decl = "struct" identifier generic_params? "copy"? ":" NEWLINE INDENT struct_body DEDENT
struct_body = field_decl* method_decl*
```

```mendes
struct User:
    id: int
    name: string
    email: string

struct Point copy:
    x: float
    y: float
```

### 7.2 Campos

```mendes
struct Example:
    # Campos primitivos
    count: int
    name: string

    # Campos compostos
    items: [int]
    metadata: Option<string>

    # Referencias (lifetime implicito)
    reference: &Data
```

### 7.3 Metodos

```mendes
struct Rectangle:
    width: float
    height: float

    # Metodo com &self (leitura)
    fn area(&self) -> float:
        return self.width * self.height

    # Metodo com &mut self (modificacao)
    fn scale(&mut self, factor: float):
        self.width *= factor
        self.height *= factor

    # Metodo com self (consome a instancia)
    fn into_square(self) -> Rectangle:
        let side = (self.width + self.height) / 2.0
        return Rectangle { width: side, height: side }

    # Funcao associada (sem self)
    fn new(w: float, h: float) -> Rectangle:
        return Rectangle { width: w, height: h }
```

### 7.4 Struct Copy

```mendes
# Por padrao, structs sao movidas
struct User:
    name: string

let u1 = User { name: "Maria" }
let u2 = u1   # u1 e MOVIDO

# Com 'copy', structs sao copiadas
struct Point copy:
    x: float
    y: float

let p1 = Point { x: 1.0, y: 2.0 }
let p2 = p1   # p1 e COPIADO
```

### 7.5 Structs Genericas

```mendes
struct Container<T>:
    value: T
    count: int

struct Pair<A, B>:
    first: A
    second: B

struct Node<T>:
    value: T
    next: Option<Box<Node<T>>>
```

---

## 8. Enums

### 8.1 Declaracao

```ebnf
enum_decl = "enum" identifier generic_params? ":" NEWLINE INDENT variant+ DEDENT
variant = identifier variant_data? NEWLINE
variant_data = "(" type_list ")" | "{" field_list "}"
```

### 8.2 Variantes Simples

```mendes
enum Direction:
    North
    South
    East
    West

let dir = Direction::North
```

### 8.3 Variantes com Dados

```mendes
# Dados de tupla
enum Shape:
    Circle(float)
    Rectangle(float, float)

let circle = Shape::Circle(5.0)
let rect = Shape::Rectangle(10.0, 20.0)

# Dados de struct
enum Message:
    Text { content: string }
    Move { x: int, y: int }

let msg = Message::Move { x: 10, y: 20 }
```

### 8.4 Enums Genericos

```mendes
enum Option<T>:
    Some(T)
    None

enum Result<T, E>:
    Ok(T)
    Err(E)
```

---

## 9. Traits

### 9.1 Declaracao

```ebnf
trait_decl = "trait" identifier generic_params? ":" NEWLINE INDENT trait_method+ DEDENT
trait_method = "fn" identifier "(" param_list? ")" return_type? "async"? NEWLINE
```

```mendes
trait Display:
    fn display(&self) -> string

trait Clone:
    fn clone(&self) -> Self

trait Serializable:
    fn to_json(&self) -> string
    fn from_json(json: string) -> Self
```

### 9.2 Implementacao

```ebnf
impl_decl = "impl" identifier "for" type ":" NEWLINE INDENT method+ DEDENT
```

```mendes
struct User:
    name: string
    age: int

impl Display for User:
    fn display(&self) -> string:
        return f"User({self.name}, {self.age})"

impl Clone for User:
    fn clone(&self) -> User:
        return User { name: self.name, age: self.age }
```

### 9.3 Trait Bounds

```mendes
fn print_all<T: Display>(items: [T]):
    for item in items:
        print(item.display())

fn clone_and_print<T: Clone + Display>(item: T):
    let copy = item.clone()
    print(copy.display())
```

---

## 10. Generics

### 10.1 Funcoes Genericas

```mendes
fn identity<T>(x: T) -> T:
    return x

fn pair<A, B>(a: A, b: B) -> (A, B):
    return (a, b)
```

### 10.2 Structs Genericas

```mendes
struct Box<T>:
    value: T

struct Map<K, V>:
    keys: [K]
    values: [V]
```

### 10.3 Enums Genericos

```mendes
enum Option<T>:
    Some(T)
    None

enum Result<T, E>:
    Ok(T)
    Err(E)
```

### 10.4 Bounds

```mendes
fn sorted<T: Ord>(items: [T]) -> [T]:
    # T deve implementar Ord
    pass

struct Cache<K: Hash + Eq, V>:
    data: Map<K, V>
```

---

## 11. Pattern Matching

### 11.1 Patterns Basicos

| Pattern | Exemplo | Descricao |
|---------|---------|-----------|
| Literal | `42`, `"hello"` | Compara com valor literal |
| Identifier | `x`, `name` | Vincula valor a variavel |
| Wildcard | `_` | Ignora valor |
| Tuple | `(a, b, c)` | Desestrutura tupla |
| Struct | `Point { x, y }` | Desestrutura struct |
| Enum | `Some(x)` | Desestrutura enum |

### 11.2 Match Expression

```mendes
match value:
    pattern1:
        body1
    pattern2:
        body2
    _:
        default_body
```

### 11.3 Guards

```mendes
match number:
    n if n < 0:
        print("negativo")
    n if n > 0:
        print("positivo")
    _:
        print("zero")
```

### 11.4 Or Patterns

```mendes
match char:
    'a' | 'e' | 'i' | 'o' | 'u':
        print("vogal")
    _:
        print("consoante")
```

### 11.5 Nested Patterns

```mendes
match data:
    Some(Point { x: 0, y: 0 }):
        print("origem")
    Some(Point { x, y }):
        print(f"ponto ({x}, {y})")
    None:
        print("sem ponto")
```

---

## 12. Ownership e Referencias

### 12.1 Regras de Ownership

1. Cada valor tem exatamente um dono
2. Quando o dono sai de escopo, o valor e destruido
3. Ownership pode ser transferido (move)

### 12.2 Move

```mendes
let s1 = "hello"
let s2 = s1         # s1 e movido para s2
# s1 nao pode mais ser usado
```

### 12.3 Copy

Tipos `copy` sao copiados, nao movidos:

```mendes
let x = 42
let y = x           # x e copiado
print(x)            # OK, x ainda valido
```

Tipos copy por padrao:
- `int`, `float`, `bool`
- Structs marcadas com `copy`

### 12.4 Referencias

```mendes
# Referencia imutavel
let r1 = &data
let r2 = &data      # Multiplas refs imutaveis OK

# Referencia mutavel
let r3 = &mut data  # Apenas uma ref mutavel por vez
```

### 12.5 Regras de Referencias

1. Em qualquer momento, voce pode ter:
   - UMA referencia mutavel, OU
   - Qualquer numero de referencias imutaveis
2. Referencias devem ser validas (nao dangling)

```mendes
let mut x = 5

# OK: multiplas refs imutaveis
let r1 = &x
let r2 = &x
print(r1, r2)

# ERRO: ref mutavel com imutaveis existentes
# let r3 = &mut x

# OK: refs imutaveis saem de escopo, ref mutavel permitida
let r4 = &mut x
```

---

## 13. Modulos e Visibilidade

### 13.1 Declaracao de Modulo

```mendes
module meu_modulo

# Todo o arquivo pertence a este modulo
```

### 13.2 Hierarquia

```
projeto/
├── main.ms              # module main
├── models/
│   ├── mod.ms           # module models
│   └── user.ms          # module models::user
└── utils/
    └── helpers.ms       # module utils::helpers
```

### 13.3 Imports

```mendes
# Import simples
import models::user

# Import com alias
import models::user as u

# From import
from models::user import User, create

# Import all
from models import *
```

### 13.4 Visibilidade

```mendes
# Privado (padrao)
fn internal():
    pass

struct InternalData:
    value: int

# Publico
pub fn external():
    pass

pub struct PublicData:
    value: int
```

---

## 14. HTTP e APIs

### 14.1 Server Declaration

```ebnf
server_decl = "server" ":" NEWLINE INDENT server_field+ DEDENT
server_field = "host" string_literal NEWLINE
             | "port" int_literal NEWLINE
```

```mendes
server:
    host "0.0.0.0"
    port 8080
```

### 14.2 API Declaration

```ebnf
api_decl = "api" http_method path "async"? ":" NEWLINE INDENT api_body DEDENT
http_method = "GET" | "POST" | "PUT" | "DELETE" | "PATCH"
api_body = directive* statement+
directive = "use" identifier NEWLINE
          | "body" type NEWLINE
          | "query" type NEWLINE
          | "return" type NEWLINE
```

```mendes
api GET /users:
    return [User]
    return get_all_users()

api POST /users async:
    body CreateUser
    return User
    let user = await create_user(body)
    return user

api GET /users/{id:int}:
    return Option<User>
    return find_user(id)
```

### 14.3 Path Parameters

```mendes
# Parametro inteiro
api GET /users/{id:int}:
    # id e int

# Parametro string
api GET /users/{username:string}:
    # username e string

# Multiplos parametros
api GET /posts/{year:int}/{month:int}/{slug:string}:
    # year, month, slug disponiveis
```

### 14.4 Request Object

```mendes
# Headers
let token = request.header("Authorization")

# Query params (se nao usar directive query)
let page = request.query("page")

# Path params (automaticos)
# Disponiveis como variaveis

# Body (se nao usar directive body)
let data = request.body()

# Method e path
let method = request.method
let path = request.path
```

### 14.5 Response Object

```mendes
# Response simples
return "ok"
return user

# Response customizada
return Response {
    status: 201,
    body: user
}

# Com headers
return Response:
    status 201
    header "Location" f"/users/{id}"
    body user
```

### 14.6 Middleware

```mendes
middleware auth:
    let token = request.header("Authorization")
    if token is None:
        return Response { status: 401, body: "Unauthorized" }
    # Continua para proximo handler

middleware log:
    let start = time.now()
    let response = next()
    print(f"{request.path} - {time.since(start)}ms")
    return response

api GET /protected:
    use auth
    use log
    return string
    return "secret"
```

---

## 15. Banco de Dados

### 15.1 Database Declaration

```ebnf
db_decl = "db" db_type identifier ":" NEWLINE INDENT db_field+ DEDENT
db_type = "postgres" | "mysql" | "sqlite"
db_field = "url" string_literal NEWLINE
         | "pool" int_literal NEWLINE
```

```mendes
db postgres main:
    url "postgres://user:pass@localhost/db"
    pool 20

db mysql secondary:
    url "mysql://user:pass@localhost/db"
    pool 10

db sqlite local:
    url "sqlite://./app.db"
    pool 5
```

### 15.2 Query Methods

```mendes
# Multiplas linhas
let users = await db.main.query(
    "SELECT * FROM users WHERE active = $1",
    true
)

# Uma linha
let user = await db.main.query_one(
    "SELECT * FROM users WHERE id = $1",
    id
)

# Executar sem retorno
await db.main.execute(
    "DELETE FROM users WHERE id = $1",
    id
)
```

### 15.3 Transactions

```mendes
await db.main.transaction:
    await db.main.execute("UPDATE accounts SET balance = balance - $1 WHERE id = $2", amount, from_id)
    await db.main.execute("UPDATE accounts SET balance = balance + $1 WHERE id = $2", amount, to_id)
```

### 15.4 Parameter Binding

| Banco | Sintaxe |
|-------|---------|
| PostgreSQL | `$1`, `$2`, `$3`... |
| MySQL | `?`, `?`, `?`... |
| SQLite | `?`, `?`, `?`... |

---

## 16. WebSockets

### 16.1 WebSocket Declaration

```ebnf
ws_decl = "ws" path ":" NEWLINE INDENT ws_handler+ DEDENT
ws_handler = "on_connect" ":" NEWLINE INDENT block DEDENT
           | "on_message" ":" NEWLINE INDENT block DEDENT
           | "on_disconnect" ":" NEWLINE INDENT block DEDENT
```

```mendes
ws /chat:
    on_connect:
        print(f"Cliente {conn.id} conectou")

    on_message:
        conn.send(f"Echo: {message}")

    on_disconnect:
        print(f"Cliente {conn.id} desconectou")
```

### 16.2 Connection Object

```mendes
# Propriedades
conn.id          # ID unico da conexao

# Metodos
conn.send(msg)   # Envia mensagem para esta conexao
conn.close()     # Fecha conexao

# Estado
conn.set_state(data)
conn.get_state<T>()
```

### 16.3 Broadcast

```mendes
ws /chat:
    on_message:
        broadcast(message)           # Envia para todos
        broadcast_room(room, msg)    # Envia para room
```

### 16.4 Rooms

```mendes
ws /chat/{room:string}:
    on_connect:
        join_room(room)

    on_message:
        broadcast_room(room, message)

    on_disconnect:
        leave_room(room)
```

---

## 17. Async/Await

### 17.1 Async Functions

```mendes
fn fetch_data() async -> Result<Data, Error>:
    let response = await http.get(url)
    return Ok(response)
```

### 17.2 Await Expression

```mendes
let result = await async_operation()
```

### 17.3 Error Propagation

```mendes
fn process() async -> Result<Output, Error>:
    let data = await fetch_data()?    # Propaga erro
    let processed = await transform(data)?
    return Ok(processed)
```

### 17.4 Concurrent Execution

```mendes
# Paralelo
let (a, b, c) = await all(
    fetch_a(),
    fetch_b(),
    fetch_c()
)

# Race (primeiro a completar)
let first = await race(
    fetch_primary(),
    fetch_backup()
)
```

---

## 18. Tratamento de Erros

### 18.1 Result Type

```mendes
enum Result<T, E>:
    Ok(T)
    Err(E)
```

### 18.2 Option Type

```mendes
enum Option<T>:
    Some(T)
    None
```

### 18.3 Operador ?

```mendes
fn may_fail() -> Result<int, string>:
    let x = operation1()?    # Propaga Err
    let y = operation2()?
    return Ok(x + y)
```

### 18.4 Metodos de Result

| Metodo | Descricao |
|--------|-----------|
| `unwrap()` | Retorna valor ou panic |
| `unwrap_or(default)` | Retorna valor ou default |
| `unwrap_or_else(f)` | Retorna valor ou resultado de f |
| `expect(msg)` | Retorna valor ou panic com msg |
| `map(f)` | Aplica f ao valor Ok |
| `map_err(f)` | Aplica f ao valor Err |
| `and_then(f)` | Chain de operacoes |
| `is_ok()` | Verifica se e Ok |
| `is_err()` | Verifica se e Err |

### 18.5 Metodos de Option

| Metodo | Descricao |
|--------|-----------|
| `unwrap()` | Retorna valor ou panic |
| `unwrap_or(default)` | Retorna valor ou default |
| `unwrap_or_else(f)` | Retorna valor ou resultado de f |
| `expect(msg)` | Retorna valor ou panic com msg |
| `map(f)` | Aplica f ao valor Some |
| `and_then(f)` | Chain de operacoes |
| `is_some()` | Verifica se e Some |
| `is_none()` | Verifica se e None |

---

## 19. Operadores

### 19.1 Precedencia (menor para maior)

| Precedencia | Operadores | Associatividade |
|-------------|------------|-----------------|
| 1 | `=` `+=` `-=` `*=` `/=` | Direita |
| 2 | `or` | Esquerda |
| 3 | `and` | Esquerda |
| 4 | `not` | Prefixo |
| 5 | `==` `!=` `<` `<=` `>` `>=` `is` | Esquerda |
| 6 | `+` `-` | Esquerda |
| 7 | `*` `/` `%` | Esquerda |
| 8 | `-` (unario) `&` `&mut` | Prefixo |
| 9 | `await` | Prefixo |
| 10 | `?` | Posfixo |
| 11 | `.` `()` `[]` | Esquerda |

### 19.2 Aritmeticos

| Operador | Descricao | Exemplo |
|----------|-----------|---------|
| `+` | Adicao | `a + b` |
| `-` | Subtracao | `a - b` |
| `*` | Multiplicacao | `a * b` |
| `/` | Divisao | `a / b` |
| `%` | Modulo | `a % b` |
| `-` | Negacao | `-a` |

### 19.3 Comparacao

| Operador | Descricao | Exemplo |
|----------|-----------|---------|
| `==` | Igual | `a == b` |
| `!=` | Diferente | `a != b` |
| `<` | Menor que | `a < b` |
| `<=` | Menor ou igual | `a <= b` |
| `>` | Maior que | `a > b` |
| `>=` | Maior ou igual | `a >= b` |

### 19.4 Logicos

| Operador | Descricao | Exemplo |
|----------|-----------|---------|
| `and` | E logico | `a and b` |
| `or` | Ou logico | `a or b` |
| `not` | Negacao | `not a` |

### 19.5 Atribuicao

| Operador | Descricao | Exemplo |
|----------|-----------|---------|
| `=` | Atribuicao | `a = b` |
| `+=` | Adicao | `a += b` |
| `-=` | Subtracao | `a -= b` |
| `*=` | Multiplicacao | `a *= b` |
| `/=` | Divisao | `a /= b` |

---

## 20. Keywords Reservadas

### Keywords de Controle
```
if else for in while break continue match return
```

### Keywords de Declaracao
```
let mut fn struct enum trait impl type pub
```

### Keywords de Tipo
```
int float bool string Result Option
```

### Keywords HTTP
```
api server middleware use body query header
GET POST PUT DELETE PATCH
```

### Keywords WebSocket
```
ws on_connect on_message on_disconnect
```

### Keywords Async
```
async await
```

### Keywords Database
```
db postgres mysql sqlite transaction
```

### Keywords de Modulo
```
module import from as
```

### Keywords de Valor
```
true false None Some Ok Err
```

### Keywords de Referencia
```
copy
```

### Keywords Logicas
```
and or not is
```

---

## Apendice A: Gramatica EBNF Completa

Veja [grammar.md](grammar.md) para a gramatica formal completa.

---

## Apendice B: Mensagens de Erro

### Erros do Lexer

| Codigo | Mensagem | Causa |
|--------|----------|-------|
| E0001 | Unexpected character | Caractere invalido |
| E0002 | Unterminated string | String nao fechada |
| E0003 | Invalid number | Formato numerico invalido |
| E0004 | Inconsistent indentation | Mix de tabs e espacos |

### Erros do Parser

| Codigo | Mensagem | Causa |
|--------|----------|-------|
| E0101 | Expected expression | Expressao esperada |
| E0102 | Expected type | Tipo esperado |
| E0103 | Expected identifier | Identificador esperado |
| E0104 | Unexpected token | Token inesperado |
| E0105 | Missing colon | Falta dois-pontos |
| E0106 | Invalid indentation | Indentacao invalida |

### Erros Semanticos

| Codigo | Mensagem | Causa |
|--------|----------|-------|
| E0201 | Undefined variable | Variavel nao declarada |
| E0202 | Undefined function | Funcao nao declarada |
| E0203 | Type mismatch | Tipos incompativeis |
| E0204 | Borrow error | Erro de emprestimo |
| E0205 | Move after use | Uso apos move |
| E0206 | Multiple mutable borrows | Multiplas refs mutaveis |

---

<p align="center">
  <strong>Linguagem Mendes - Referencia Oficial v0.1.0</strong>
</p>
