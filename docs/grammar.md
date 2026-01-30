# Gramática Formal da Linguagem Mendes

> Versão 0.1.0

Este documento define a gramática formal da linguagem Mendes em notação EBNF (Extended Backus-Naur Form).

---

## Convenções

- `UPPERCASE` = tokens terminais (keywords, operadores)
- `lowercase` = não-terminais (regras)
- `*` = zero ou mais
- `+` = um ou mais
- `?` = opcional
- `|` = alternativa
- `( )` = agrupamento
- `INDENT` / `DEDENT` = tokens de indentação

---

## Estrutura de Nível Superior

```ebnf
program         = statement* EOF ;

statement       = module_decl
                | import_stmt
                | server_decl
                | db_decl
                | struct_decl
                | enum_decl
                | fn_decl
                | api_decl
                | middleware_decl
                | let_stmt
                | if_stmt
                | for_stmt
                | while_stmt
                | return_stmt
                | expr_stmt
                ;
```

---

## Módulos e Imports

```ebnf
module_decl     = "module" IDENT NEWLINE ;

import_stmt     = "import" import_path NEWLINE ;
import_path     = IDENT ( "::" IDENT )* ;
```

---

## Servidor e Banco de Dados

```ebnf
server_decl     = "server" ":" NEWLINE INDENT server_body DEDENT ;
server_body     = server_field+ ;
server_field    = "host" STRING_LIT NEWLINE
                | "port" INT_LIT NEWLINE
                ;

db_decl         = "db" db_type IDENT ":" NEWLINE INDENT db_body DEDENT ;
db_type         = "postgres" | "mysql" | "sqlite" ;
db_body         = db_field+ ;
db_field        = "url" STRING_LIT NEWLINE
                | "pool" INT_LIT NEWLINE
                ;
```

---

## Structs e Enums

```ebnf
struct_decl     = "struct" IDENT copy_modifier? ":" NEWLINE INDENT struct_body DEDENT ;
copy_modifier   = "copy" ;
struct_body     = field_decl+ ;
field_decl      = IDENT ":" type NEWLINE ;

enum_decl       = "enum" IDENT ":" NEWLINE INDENT enum_body DEDENT ;
enum_body       = variant_decl+ ;
variant_decl    = IDENT ( "(" type_list ")" )? NEWLINE ;
```

---

## Funções

```ebnf
fn_decl         = pub_modifier? "fn" IDENT "(" param_list? ")" return_type? async_modifier? ":" NEWLINE INDENT block DEDENT ;
pub_modifier    = "pub" ;
async_modifier  = "async" ;
return_type     = "->" type ;
param_list      = param ( "," param )* ;
param           = IDENT ":" type ;
```

---

## API HTTP

```ebnf
api_decl        = "api" http_method path_pattern async_modifier? ":" NEWLINE INDENT api_body DEDENT ;
http_method     = "GET" | "POST" | "PUT" | "DELETE" | "PATCH" ;
path_pattern    = PATH_STRING ;
api_body        = api_directive* block ;
api_directive   = middleware_use
                | body_decl
                | query_decl
                | return_decl
                ;
middleware_use  = "use" IDENT NEWLINE ;
body_decl       = "body" type NEWLINE ;
query_decl      = "query" type NEWLINE ;
return_decl     = "return" type NEWLINE ;
```

### Path Pattern

```ebnf
PATH_STRING     = "/" path_segment* ;
path_segment    = IDENT
                | "{" IDENT ":" type "}"    (* parâmetro tipado *)
                ;
```

---

## Middleware

```ebnf
middleware_decl = "middleware" IDENT ":" NEWLINE INDENT block DEDENT ;
```

---

## Declarações e Statements

```ebnf
let_stmt        = "let" "mut"? IDENT ( ":" type )? "=" expr NEWLINE ;

if_stmt         = "if" expr ":" NEWLINE INDENT block DEDENT else_clause? ;
else_clause     = "else" ":" NEWLINE INDENT block DEDENT
                | "else" if_stmt
                ;

for_stmt        = "for" IDENT "in" expr ":" NEWLINE INDENT block DEDENT ;

while_stmt      = "while" expr ":" NEWLINE INDENT block DEDENT ;

return_stmt     = "return" expr? NEWLINE ;

expr_stmt       = expr NEWLINE ;

block           = statement+ ;
```

---

## Tipos

```ebnf
type            = primitive_type
                | IDENT                                 (* tipo definido pelo usuário *)
                | generic_type
                | ref_type
                | array_type
                ;

primitive_type  = "int" | "float" | "bool" | "string" ;

generic_type    = IDENT "<" type_list ">" ;
type_list       = type ( "," type )* ;

ref_type        = "&" "mut"? type ;

array_type      = "[" type "]" ;
```

---

## Expressões

```ebnf
expr            = assignment ;

assignment      = IDENT ( "=" | "+=" | "-=" | "*=" | "/=" ) assignment
                | ternary
                ;

ternary         = or_expr ( "if" or_expr "else" ternary )? ;

or_expr         = and_expr ( "or" and_expr )* ;

and_expr        = not_expr ( "and" not_expr )* ;

not_expr        = "not" not_expr
                | comparison
                ;

comparison      = additive ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" | "is" ) additive )* ;

additive        = multiplicative ( ( "+" | "-" ) multiplicative )* ;

multiplicative  = unary ( ( "*" | "/" | "%" ) unary )* ;

unary           = ( "-" | "&" | "&mut" ) unary
                | await_expr
                ;

await_expr      = "await" await_expr
                | postfix
                ;

postfix         = primary postfix_op* ;
postfix_op      = "." IDENT                              (* field access *)
                | "." IDENT "(" arg_list? ")"            (* method call *)
                | "(" arg_list? ")"                      (* function call *)
                | "[" expr "]"                           (* index *)
                ;

primary         = INT_LIT
                | FLOAT_LIT
                | STRING_LIT
                | "true"
                | "false"
                | "None"
                | IDENT
                | "(" expr ")"
                | struct_literal
                | array_literal
                | "Ok" "(" expr ")"
                | "Err" "(" expr ")"
                | "Some" "(" expr ")"
                ;

arg_list        = expr ( "," expr )* ;

struct_literal  = IDENT "{" field_init_list? "}" ;
field_init_list = field_init ( "," field_init )* ;
field_init      = IDENT ":" expr ;

array_literal   = "[" ( expr ( "," expr )* )? "]" ;
```

---

## Response HTTP

```ebnf
response_literal = "Response" ":" NEWLINE INDENT response_body DEDENT ;
response_body    = response_field+ ;
response_field   = "status" INT_LIT NEWLINE
                 | "body" expr NEWLINE
                 | "header" STRING_LIT expr NEWLINE
                 ;
```

---

## Transação de Banco

```ebnf
transaction_stmt = "await" "db" "." IDENT "." "transaction" ":" NEWLINE INDENT block DEDENT ;
```

---

## Tokens Terminais

```ebnf
IDENT           = ( LETTER | "_" ) ( LETTER | DIGIT | "_" )* ;
INT_LIT         = DIGIT+ | "0x" HEX_DIGIT+ | "0b" BIN_DIGIT+ | "0o" OCT_DIGIT+ ;
FLOAT_LIT       = DIGIT+ "." DIGIT+ ( "e" ( "+" | "-" )? DIGIT+ )? ;
STRING_LIT      = '"' ( ESC_CHAR | [^"\\] )* '"' ;

LETTER          = [a-zA-Z] ;
DIGIT           = [0-9] ;
HEX_DIGIT       = [0-9a-fA-F] ;
BIN_DIGIT       = [01] ;
OCT_DIGIT       = [0-7] ;
ESC_CHAR        = "\\" ( "n" | "t" | "r" | "\\" | '"' | "'" | "0" ) ;

NEWLINE         = "\n" ;
INDENT          = (* aumento de indentação *) ;
DEDENT          = (* diminuição de indentação *) ;
```

---

## Comentários

```ebnf
COMMENT         = "#" [^\n]* NEWLINE ;
```

Comentários são ignorados pelo lexer.

---

## Precedência de Operadores

| Precedência | Operadores | Associatividade |
|-------------|------------|-----------------|
| 1 (menor)   | `=` `+=` `-=` `*=` `/=` | Direita |
| 2           | `or` | Esquerda |
| 3           | `and` | Esquerda |
| 4           | `not` | Prefixo |
| 5           | `==` `!=` `<` `<=` `>` `>=` `is` | Esquerda |
| 6           | `+` `-` | Esquerda |
| 7           | `*` `/` `%` | Esquerda |
| 8           | `-` (unário) `&` `&mut` | Prefixo |
| 9           | `await` | Prefixo |
| 10 (maior)  | `.` `()` `[]` | Esquerda |

---

## Exemplos de Código Válido

### Hello World

```mendes
server:
    host "0.0.0.0"
    port 8080

api GET /health:
    return string

    return "ok"
```

### CRUD com Banco

```mendes
db postgres main:
    url "postgres://localhost/app"
    pool 20

struct User:
    id: int
    name: string
    email: string

api GET /users/{id:int} async:
    return Result<User, HttpError>

    let user = await db.main.query_one(
        "SELECT * FROM users WHERE id = $1",
        id
    )
    return Ok(user)

api POST /users async:
    body User
    return User

    let user = await db.main.query_one(
        "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *",
        body.name,
        body.email
    )
    return user
```

### Middleware

```mendes
middleware auth:
    let token = request.header("Authorization")
    if token is None:
        return HttpError(401, "unauthorized")

api GET /private:
    use auth
    return string

    return "secret data"
```
