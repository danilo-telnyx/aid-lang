# AID — Auto-Intelligent Development Language

> *Code that thinks. Software that evolves.*

AID is a programming language designed to blur the line between writing code and expressing intent. It combines familiar syntax with AI-native constructs — `reason` blocks that generate logic at compile time, `evolve` blocks that improve themselves across builds, and natural-language contracts that compile to type-safe validation. AID transpiles to Rust and targets native binaries and WebAssembly.

## Hello World

```aid
module main

use std.http

fn main() {
    server := http.new(port: 8080)

    server.get("/") => fn(req) -> Response {
        Response.text("Hello from AID")
    }

    server.get("/health") => fn(req) -> Response {
        Response.json({ status: "ok", language: "AID", version: "0.1.0" })
    }

    server.start()
}
```

## Key Features

- **Reason Blocks** — Describe *what* you want; the compiler generates *how*
- **Evolve Blocks** — Functions that improve themselves across build cycles using telemetry
- **Intent Routing** — Declare endpoints by intent; the compiler discovers and wires handlers
- **Contracts** — Natural-language rules compiled to type-safe validation
- **Auto-Docs** — Documentation generated directly from code and contracts
- **WASM Target** — Compile to WebAssembly for universal deployment

## Quick Start

```bash
# Build the project
aid build

# Run the compiled binary
aid run
```

## Architecture

```
┌──────────┐     ┌────────┐     ┌────────┐     ┌────────────┐     ┌──────┐     ┌──────┐
│ .aid file│────▶│ Parser │────▶│ Cortex │────▶│ Transpiler │────▶│ Rust │────▶│ WASM │
└──────────┘     └────────┘     └────────┘     └────────────┘     └──────┘     └──────┘
                  (pest)        (AI engine)     (code gen)        (cargo)     (target)
```

## Current Status

### ✅ Working Today
- Pest-based parser (hello-world subset)
- Rust transpiler with Axum HTTP mapping
- CLI: `aid build`, `aid run`, `aid clean`, `aid docs`
- Hello world example with text and JSON routes
- Auto-documentation generation

### 🚧 Planned
- Reason block transpilation
- Evolve block telemetry
- WASM compilation target
- Contract validation generation
- Intent routing
- Full grammar (entities, async, pattern matching, loops, error handling)

## Documentation

Full language specification and documentation: [`docs/AID-Language-Documentation.md`](docs/AID-Language-Documentation.md)

## License

[MIT](LICENSE)

## Project Owner

- **[@danilo-telnyx](https://github.com/danilo-telnyx)** — Owner & Approver
