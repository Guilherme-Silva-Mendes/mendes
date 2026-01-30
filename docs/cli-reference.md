# Referencia do CLI - Compilador Mendes

> **Versao**: 0.1.0
> **Executavel**: `mendes` (ou `mendes.exe` no Windows)

Este documento descreve todos os comandos e opcoes do compilador Mendes.

---

## Indice

1. [Instalacao](#1-instalacao)
2. [Uso Geral](#2-uso-geral)
3. [Comandos](#3-comandos)
   - [build](#31-build)
   - [check](#32-check)
   - [run](#33-run)
   - [lex](#34-lex)
   - [parse](#35-parse)
   - [ir](#36-ir)
   - [emit](#37-emit)
   - [emit-rust](#38-emit-rust)
4. [Opcoes Globais](#4-opcoes-globais)
5. [Codigos de Saida](#5-codigos-de-saida)
6. [Variaveis de Ambiente](#6-variaveis-de-ambiente)
7. [Exemplos Praticos](#7-exemplos-praticos)
8. [Troubleshooting](#8-troubleshooting)

---

## 1. Instalacao

### Compilando do Fonte

```bash
git clone https://github.com/guilhermemendes/linguagem-mendes.git
cd linguagem-mendes
cargo build --release
```

### Adicionando ao PATH

#### Linux/macOS

```bash
# Adicione ao ~/.bashrc ou ~/.zshrc
export PATH="$PATH:/caminho/para/linguagem-mendes/target/release"

# Recarregue
source ~/.bashrc
```

#### Windows (PowerShell)

```powershell
# Adicione temporariamente
$env:Path += ";C:\caminho\para\linguagem-mendes\target\release"

# Ou permanentemente (requer admin)
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";C:\caminho\para\linguagem-mendes\target\release", "Machine")
```

### Verificando Instalacao

```bash
mendes --version
# mendes 0.1.0

mendes --help
# Mostra ajuda
```

---

## 2. Uso Geral

### Sintaxe

```
mendes <COMANDO> [OPCOES] [ARGUMENTOS]
```

### Ajuda

```bash
# Ajuda geral
mendes --help
mendes -h

# Ajuda de comando especifico
mendes build --help
mendes check --help
```

### Versao

```bash
mendes --version
mendes -V
```

---

## 3. Comandos

### 3.1 build

Compila um arquivo `.ms` para um executavel nativo.

#### Sintaxe

```
mendes build <ARQUIVO> [OPCOES]
```

#### Opcoes

| Opcao | Curta | Descricao | Padrao |
|-------|-------|-----------|--------|
| `--output` | `-o` | Nome do executavel de saida | Nome do arquivo fonte |
| `--backend` | `-b` | Backend de geracao de codigo | `rust` |
| `--release` | - | Compila com otimizacoes | `false` |

#### Backends Disponiveis

| Backend | Descricao |
|---------|-----------|
| `rust` | Gera codigo Rust e compila com Cargo (padrao) |
| `c` | Gera codigo C e compila com GCC/Clang |

#### Exemplos

```bash
# Compilacao basica
mendes build app.ms

# Especificar nome de saida
mendes build app.ms -o minha_app

# Build otimizado
mendes build app.ms --release

# Usar backend C
mendes build app.ms --backend c

# Combinando opcoes
mendes build app.ms -o servidor --release --backend rust
```

#### Saida

```
Compiling: app.ms
  [ok] Lexer: 156 tokens
  [ok] Parser: 12 statements
  [ok] Semantic: types verified
  [ok] IR: 5 functions, 3 routes
  [ok] Codegen: Rust code generated
  [ok] Cargo project created at: /tmp/mendes_build_12345

Compiling with Cargo...

Compilation completed!
   Executable: /home/user/projeto/app

   To run:
   ./app
```

#### Erros Comuns

| Erro | Causa | Solucao |
|------|-------|---------|
| File not found | Arquivo nao existe | Verifique o caminho |
| Lexer errors | Caractere invalido | Corrija a sintaxe |
| Parser errors | Estrutura invalida | Verifique indentacao |
| Semantic errors | Erro de tipo | Corrija tipos |
| Cargo not found | Rust nao instalado | Instale Rust |

---

### 3.2 check

Verifica erros sem gerar executavel.

#### Sintaxe

```
mendes check <ARQUIVO>
```

#### Descricao

Executa todas as fases de analise (lexer, parser, semantic) mas nao gera codigo. Util para verificar erros rapidamente durante desenvolvimento.

#### Exemplos

```bash
mendes check app.ms
```

#### Saida (Sucesso)

```
Checking: app.ms

  [ok] Lexer: 156 tokens
  [ok] Parser: 12 statements
  [ok] Semantic: types verified

No errors found!
```

#### Saida (Com Erros)

```
Checking: app.ms

  [ok] Lexer: 156 tokens

Syntax errors:

error[E0104]: Unexpected token
 --> app.ms:15:5
   |
15 |     retrun "hello"
   |     ^^^^^^ expected expression, found identifier
   |
   = help: did you mean `return`?
```

---

### 3.3 run

Executa um arquivo `.ms` diretamente.

#### Sintaxe

```
mendes run <ARQUIVO>
```

#### Status

**Nota**: Este comando ainda nao esta totalmente implementado. Atualmente exibe:

```
Running: app.ms
   [TODO] Execution not yet implemented
```

#### Implementacao Futura

O comando `run` ira:
1. Compilar o arquivo em memoria
2. Executar via JIT ou interpretador
3. Suportar hot-reload para desenvolvimento

---

### 3.4 lex

Mostra os tokens gerados pelo lexer (debug).

#### Sintaxe

```
mendes lex <ARQUIVO>
```

#### Descricao

Util para depurar problemas de tokenizacao e entender como o lexer processa o codigo.

#### Exemplos

```bash
mendes lex hello.ms
```

#### Saida

```
Tokenizing: hello.ms

   1:1   Server               server
   1:7   Colon                :
   1:8   Newline              NEWLINE
   2:1   Indent               INDENT
   2:5   Ident("host")        host
   2:10  StringLit("0.0.0.0") "0.0.0.0"
   2:19  Newline              NEWLINE
   3:5   Ident("port")        port
   3:10  IntLit(8080)         8080
   3:14  Newline              NEWLINE
   4:1   Dedent               DEDENT
   ...

Total: 43 tokens
```

#### Legenda de Tokens

| Token | Descricao |
|-------|-----------|
| `Indent` | Aumento de indentacao |
| `Dedent` | Reducao de indentacao |
| `Newline` | Fim de linha |
| `Ident(x)` | Identificador |
| `IntLit(n)` | Literal inteiro |
| `StringLit(s)` | Literal string |
| `Server`, `Api`, etc. | Keywords |

---

### 3.5 parse

Mostra a AST (Abstract Syntax Tree) gerada (debug).

#### Sintaxe

```
mendes parse <ARQUIVO>
```

#### Descricao

Mostra a estrutura sintatica do programa apos o parsing.

#### Exemplos

```bash
mendes parse hello.ms
```

#### Saida

```
Parsing: hello.ms

AST (3 statements):

1. Server { host: "0.0.0.0", port: 8080 }
2. Api GET /health
  middlewares: []
  handler: 1 statements
3. Api GET /hello/{name:string}
  middlewares: []
  handler: 1 statements

Parse completed successfully!
```

---

### 3.6 ir

Mostra a Representacao Intermediaria (debug).

#### Sintaxe

```
mendes ir <ARQUIVO>
```

#### Descricao

Mostra o IR (Intermediate Representation) gerado apos lowering da AST.

#### Exemplos

```bash
mendes ir hello.ms
```

#### Saida

```
Generating IR: hello.ms

=== Mendes IR Module ===

--- Structs ---
(none)

--- Functions ---
[0] fn handle_health_get() -> String
    body: 1 blocks
    is_async: false

[1] fn handle_hello_name_get(name: String) -> String
    body: 1 blocks
    is_async: false

--- HTTP Routes ---
[0] GET /health -> handle_health_get
[1] GET /hello/{name} -> handle_hello_name_get

--- String Table ---
[0] "ok"
[1] "Hello, "
[2] "!"

Statistics:
   Functions: 2
   HTTP Routes: 2
   Structs: 0
   Strings: 3

IR generated successfully!
```

---

### 3.7 emit

Gera codigo C a partir do arquivo `.ms`.

#### Sintaxe

```
mendes emit <ARQUIVO> [OPCOES]
```

#### Opcoes

| Opcao | Curta | Descricao | Padrao |
|-------|-------|-----------|--------|
| `--output` | `-o` | Arquivo de saida | stdout |

#### Exemplos

```bash
# Imprime no terminal
mendes emit hello.ms

# Salva em arquivo
mendes emit hello.ms -o hello.c
```

#### Saida (Arquivo C)

```c
/* Generated by Mendes Compiler */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* String table */
static const char* __str_0 = "ok";
static const char* __str_1 = "Hello, ";

/* Function: handle_health_get */
char* handle_health_get() {
    return (char*)__str_0;
}

/* Function: handle_hello_name_get */
char* handle_hello_name_get(char* name) {
    /* String interpolation */
    size_t len = strlen(__str_1) + strlen(name) + 2;
    char* result = malloc(len);
    snprintf(result, len, "%s%s!", __str_1, name);
    return result;
}

int main() {
    /* Server startup */
    printf("Server starting on 0.0.0.0:8080\n");
    return 0;
}
```

#### Compilando o C Gerado

```bash
# Com GCC
gcc -o app hello.c -lpthread

# Com Clang
clang -o app hello.c -lpthread
```

---

### 3.8 emit-rust

Gera codigo Rust a partir do arquivo `.ms`.

#### Sintaxe

```
mendes emit-rust <ARQUIVO> [OPCOES]
```

#### Opcoes

| Opcao | Curta | Descricao | Padrao |
|-------|-------|-----------|--------|
| `--output` | `-o` | Arquivo de saida | stdout |

#### Exemplos

```bash
# Imprime no terminal
mendes emit-rust hello.ms

# Salva em arquivo
mendes emit-rust hello.ms -o main.rs
```

#### Saida (Arquivo Rust)

```rust
//! Generated by Mendes Compiler
#![allow(unused)]

use mendes_runtime::*;

/// Handler for GET /health
async fn handle_health_get(_req: Request) -> Response {
    Response::ok("ok")
}

/// Handler for GET /hello/{name}
async fn handle_hello_name_get(req: Request) -> Response {
    let name = req.param("name").unwrap();
    Response::ok(format!("Hello, {}!", name))
}

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    router.get("/health", |req| async move {
        handle_health_get(req).await
    });

    router.get("/hello/:name", |req| async move {
        handle_hello_name_get(req).await
    });

    println!("Server starting on 0.0.0.0:8080");

    Server::new("0.0.0.0:8080")
        .router(router)
        .run()
        .await
        .unwrap();
}
```

---

## 4. Opcoes Globais

Opcoes disponiveis para todos os comandos:

| Opcao | Curta | Descricao |
|-------|-------|-----------|
| `--help` | `-h` | Mostra ajuda |
| `--version` | `-V` | Mostra versao |

---

## 5. Codigos de Saida

| Codigo | Significado |
|--------|-------------|
| 0 | Sucesso |
| 1 | Erro geral (compilacao, arquivo nao encontrado) |

---

## 6. Variaveis de Ambiente

| Variavel | Descricao | Padrao |
|----------|-----------|--------|
| `MENDES_RUNTIME_PATH` | Caminho para mendes-runtime | Detectado automaticamente |
| `RUST_BACKTRACE` | Mostra backtrace em erros | `0` |
| `CARGO_HOME` | Diretorio do Cargo | `~/.cargo` |

---

## 7. Exemplos Praticos

### Fluxo de Desenvolvimento Tipico

```bash
# 1. Verificar erros durante desenvolvimento
mendes check app.ms

# 2. Se tudo OK, compilar
mendes build app.ms

# 3. Executar
./app
```

### Debug de Problemas

```bash
# Problema de tokenizacao
mendes lex problema.ms

# Problema de sintaxe
mendes parse problema.ms

# Problema de geracao de codigo
mendes ir problema.ms
mendes emit-rust problema.ms
```

### Build para Producao

```bash
# Compilar com otimizacoes
mendes build app.ms --release -o minha_app

# Verificar tamanho
ls -lh minha_app
```

### CI/CD Pipeline

```bash
#!/bin/bash
set -e

# Verificar todos os arquivos
for file in src/*.ms; do
    echo "Checking $file..."
    mendes check "$file"
done

# Build
mendes build src/main.ms --release -o app

# Teste basico
./app &
PID=$!
sleep 2
curl http://localhost:8080/health
kill $PID

echo "Build successful!"
```

---

## 8. Troubleshooting

### Problema: "command not found: mendes"

**Causa**: Executavel nao esta no PATH.

**Solucao**:
```bash
# Verifique onde esta o executavel
find . -name "mendes" -o -name "mendes.exe"

# Adicione ao PATH
export PATH="$PATH:/caminho/encontrado"
```

### Problema: "Cargo not found"

**Causa**: Rust nao esta instalado.

**Solucao**:
```bash
# Instale Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Recarregue o shell
source ~/.cargo/env
```

### Problema: "mendes-runtime not found"

**Causa**: Caminho do runtime incorreto.

**Solucao**:
```bash
# Compile a partir do diretorio do projeto
cd linguagem-mendes
mendes build examples/hello.ms

# Ou defina a variavel de ambiente
export MENDES_RUNTIME_PATH=/caminho/para/linguagem-mendes/crates/mendes-runtime
```

### Problema: Erros de compilacao do Cargo

**Causa**: Dependencias nao resolvidas.

**Solucao**:
```bash
# Atualize dependencias
cd linguagem-mendes
cargo update

# Rebuild
cargo build --release
```

### Problema: Indentation error

**Causa**: Mix de tabs e espacos, ou indentacao inconsistente.

**Solucao**:
- Use apenas espacos (recomendado: 4 espacos)
- Configure seu editor para usar espacos
- Use `mendes lex` para ver tokens de indentacao

---

## Apendice: Cheat Sheet

```bash
# Verificar sintaxe
mendes check arquivo.ms

# Compilar
mendes build arquivo.ms
mendes build arquivo.ms -o nome_executavel
mendes build arquivo.ms --release

# Debug
mendes lex arquivo.ms      # Ver tokens
mendes parse arquivo.ms    # Ver AST
mendes ir arquivo.ms       # Ver IR

# Gerar codigo
mendes emit arquivo.ms         # Gera C
mendes emit-rust arquivo.ms    # Gera Rust
mendes emit arquivo.ms -o saida.c

# Ajuda
mendes --help
mendes build --help
```

---

<p align="center">
  <strong>Mendes CLI Reference v0.1.0</strong>
</p>
