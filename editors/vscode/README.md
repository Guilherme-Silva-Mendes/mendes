# Mendes Language for VS Code

Syntax highlighting, snippets, and language support for the Mendes programming language.

## Features

- **Syntax Highlighting** - Full support for Mendes syntax
- **Snippets** - Common code patterns
- **Bracket Matching** - Auto-close brackets and strings
- **Indentation** - Automatic indentation after `:`
- **Folding** - Code folding based on indentation

## Installation

### Option 1: Install from VSIX (Recommended)

1. Package the extension:
   ```bash
   cd editors/vscode
   npx vsce package
   ```

2. Install in VS Code:
   - Press `Ctrl+Shift+P`
   - Type "Install from VSIX"
   - Select the generated `.vsix` file

### Option 2: Development Mode

1. Copy the `vscode` folder to your VS Code extensions directory:
   - **Windows**: `%USERPROFILE%\.vscode\extensions\mendes-language`
   - **macOS**: `~/.vscode/extensions/mendes-language`
   - **Linux**: `~/.vscode/extensions/mendes-language`

2. Restart VS Code

### Option 3: Symlink (for development)

```bash
# Windows (PowerShell as Admin)
New-Item -ItemType SymbolicLink -Path "$env:USERPROFILE\.vscode\extensions\mendes-language" -Target "C:\path\to\editors\vscode"

# macOS/Linux
ln -s /path/to/editors/vscode ~/.vscode/extensions/mendes-language
```

## Snippets

| Prefix | Description |
|--------|-------------|
| `fn` | Function |
| `afn` | Async function |
| `struct` | Struct definition |
| `impl` | Impl block |
| `enum` | Enum definition |
| `trait` | Trait definition |
| `apiget` | GET endpoint |
| `apipost` | POST endpoint |
| `apiput` | PUT endpoint |
| `apidel` | DELETE endpoint |
| `server` | Server configuration |
| `db` | Database configuration |
| `if` | If statement |
| `ife` | If-else statement |
| `for` | For loop |
| `while` | While loop |
| `match` | Match expression |
| `matchopt` | Match Option |
| `matchres` | Match Result |
| `let` | Variable declaration |
| `letm` | Mutable variable |
| `middleware` | Middleware definition |
| `dbquery` | Database query |
| `json` | JSON response |
| `error` | Error response |

## Syntax Highlighting

The extension provides highlighting for:

- **Keywords**: `fn`, `let`, `mut`, `if`, `else`, `for`, `while`, `match`, etc.
- **Types**: `int`, `float`, `bool`, `string`, `Option`, `Result`, etc.
- **HTTP Methods**: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`
- **Literals**: Numbers, strings, booleans
- **Comments**: `//` and `/* */`
- **Operators**: `+`, `-`, `*`, `/`, `==`, `!=`, `->`, `=>`, etc.
- **String Interpolation**: `{variable}` inside strings

## File Association

Files with `.ms` extension are automatically recognized as Mendes files.

## Adding Custom Icons

Place icon files in the `icons/` directory:
- `mendes-light.png` - For light themes
- `mendes-dark.png` - For dark themes

Recommended size: 16x16 or 32x32 pixels.

## Contributing

1. Fork the repository
2. Make changes to the grammar or snippets
3. Test in VS Code
4. Submit a pull request

## License

MIT License - See the main project LICENSE file.
