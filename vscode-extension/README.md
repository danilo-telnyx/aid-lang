# AID Language — VS Code Extension

Syntax highlighting and language support for the [AID programming language](https://github.com/danilo-telnyx/aid-lang).

## Features

### Syntax Highlighting

Full TextMate grammar covering all AID language constructs:

- **Keywords** — `fn`, `let`, `mut`, `if`, `else`, `match`, `for`, `while`, `return`, `use`, `entity`, `async`, `try`, `catch`, `await`, `break`, `continue`
- **AI constructs** — `reason` (with `goal`, `examples`, `constraints`, `fallback`), `evolve` (with `track`, `window`, `threshold`), `contract` (with rules), `intent`
- **Types** — `string`, `int`, `float`, `bool`, `byte`, `void`, `Option`, `Result`, `array`, `map`, `stream`
- **Imports** — `use std.db`, `std.env`, `std.auth`, `std.html`, `std.http` (highlighted as stdlib modules)
- **Comments** — `//` line comments, `///` doc comments, `/* */` block comments
- **Strings** — double-quoted with escape sequences (`\n`, `\t`, `\\`, `\"`, `\r`, `\0`)
- **Numbers** — integers, floats, hex literals (`0xFF`)
- **Operators** — `:=` (walrus), `=>` (arrow), `->` (return type), comparisons, logical, arithmetic, range (`..`)
- **HTTP methods** — `GET`, `POST`, `PUT`, `DELETE`, `PATCH`
- **Constructors** — `Some`, `None`, `Ok`, `Err`

### Code Snippets

| Prefix | Description |
|--------|-------------|
| `fn` | Function declaration |
| `afn` | Async function declaration |
| `reason` | Reason block with goal, constraints, examples, fallback |
| `evolve` | Evolve block for self-improving code |
| `contract` | Contract with validation rules |
| `entity` | Entity (struct) declaration |
| `get` | HTTP GET route |
| `post` | HTTP POST route |
| `server` | Full HTTP server boilerplate |
| `module` | Module declaration |
| `match` | Match expression |
| `ife` | If-else block |
| `intent` | Intent routing block |

### Language Configuration

- Auto-closing brackets, parentheses, quotes
- Comment toggling (`Cmd+/` for line, `Shift+Alt+A` for block)
- Code folding on braces
- Auto-indentation

## Installation

### Option 1: Copy to extensions directory

```bash
cp -r vscode-extension ~/.vscode/extensions/aid-lang
```

Then reload VS Code (`Cmd+Shift+P` → "Reload Window").

### Option 2: Symlink (for development)

```bash
ln -s "$(pwd)/vscode-extension" ~/.vscode/extensions/aid-lang
```

## Usage

Open any `.aid` file and syntax highlighting will activate automatically.

## Requirements

- VS Code 1.75.0 or later

## License

BSL-1.1 — See [LICENSE](../LICENSE.md)
