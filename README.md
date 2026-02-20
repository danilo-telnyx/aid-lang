# AID — Auto-Intelligent Development Language

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/danilo-telnyx/aid-lang/releases)
[![License](https://img.shields.io/badge/license-BSL%201.1-green.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/compiler-Rust%201.93-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-v0.1.0-brightgreen.svg)](https://github.com/danilo-telnyx/aid-lang/releases/tag/v0.1.0)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20WASM-lightgrey.svg)](#)

> *Code that thinks. Software that evolves.*

🌐 **Website:** [danilo-telnyx.github.io/aid-lang](https://danilo-telnyx.github.io/aid-lang/)
📖 **Documentation:** [Wiki](https://github.com/danilo-telnyx/aid-lang/wiki)
📋 **Roadmap:** [Issues](https://github.com/danilo-telnyx/aid-lang/issues)
📄 **Full Spec:** [Language Documentation](docs/AID-Language-Documentation.md)
📦 **Package Spec:** [Package & Registry Design](docs/PACKAGE-SPEC.md)

---

AID is a statically typed, compiled programming language with **embedded AI reasoning**. Write decision logic in natural language, and the compiler generates optimized code. Your software gets smarter with every deploy.

AID transpiles to Rust and compiles to native binaries or WASM. No cloud APIs. No ML infrastructure. Intelligence is a language feature.

---

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

```bash
$ aid build hello.aid
  ✓ Parse
  ✓ Transpile
  ✓ Compile
  ✓ Binary → build/aid-hello (1.5 MB)
  ✓ Docs
  Build complete in 0.4s

$ aid run
  🚀 AID server listening on http://0.0.0.0:8080
```

---

## The Killer Feature: `reason` Blocks

Declare AI-powered decisions directly in your code:

```aid
reason classify_ticket(text: string) -> string {
    goal: "Classify a customer support ticket into a category"
    constraints: [
        "Return one of: billing, technical, general, urgent",
        "Tickets mentioning outage or down are always urgent"
    ]
    examples: [
        ("My card was charged twice", "billing"),
        ("Server is down", "urgent"),
        ("How do I reset my password", "technical")
    ]
    fallback: "general"
}

// Use it like any function
category := classify_ticket(ticket.text)
```

The compiler analyzes your goal, constraints, and examples — then generates an optimized decision function. No ML pipeline. No API calls. It's just a language feature.

---

## Self-Improving Code: `evolve` Blocks

```aid
evolve classify_ticket {
    track: true
    retrain_every: 500
    min_accuracy: 0.95
    approve: true
}
```

Every call is logged. Next build, the compiler reads the telemetry and generates better logic. Your code improves just by being used.

```bash
$ curl http://localhost:8080/telemetry
{
  "classify_ticket": {
    "calls": 1247,
    "distribution": {"billing": "42%", "technical": "31%", "urgent": "15%", "general": "12%"}
  }
}
```

---

## Natural Language Validation: `contract`

```aid
contract UserAPI {
    "User ID must be a positive integer"
    "Email must contain exactly one @ symbol"
    "Age must be between 13 and 120"
    "Role must be one of: admin, editor, viewer"

    fn create(user: User) -> result<User, ValidationError>
}
```

The compiler reads the English rules and generates type-safe validators. No boilerplate. No regex. Just describe what valid looks like.

---

## Key Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🧠 **Reason Blocks** | AI-powered decision functions from natural language | ✅ Working |
| 🧬 **Evolve Blocks** | Self-improving code via runtime telemetry | ✅ Working |
| 📜 **Contracts** | English rules → type-safe validators | ✅ Working |
| 📄 **Auto-Documentation** | Docs generated at every build | ✅ Working |
| 🎯 **Intent Routing** | Compiler discovers handlers, builds route tables | ✅ Working |
| ⚡ **WASM Target** | Compile to WebAssembly, deploy anywhere | ✅ Working |
| 🌍 **std.env** | Environment variables, .env files, config-driven servers | ✅ Working |
| 🗄️ **std.db** | SQLite database — connect, query, execute, migrate | ✅ Working |
| 🌐 **std.html** | HTML templates, static files, render, redirect | ✅ Working |
| 🔐 **std.auth** | JWT tokens, bcrypt hashing, API keys, middleware | ✅ Working |
| 🔒 **Local Cortex** | AI runs locally — no cloud, no data leaves your machine | ✅ Architecture |

---

## Database: `std.db`

```aid
module database
use std.http
use std.db

fn main() {
    db.connect("sqlite://data.db")
    db.execute("CREATE TABLE IF NOT EXISTS items (id INTEGER PRIMARY KEY, name TEXT)")
    db.execute("INSERT OR IGNORE INTO items (id, name) VALUES (1, 'Widget')")

    items := db.query("SELECT * FROM items")

    server := http.new(port: 8080)
    server.get("/items") => fn(req) -> Response {
        Response.json({ items: items })
    }
}
```

Operations:
- `db.connect("sqlite://path.db")` — Open SQLite database
- `db.execute("SQL")` — Run DDL/DML statements
- `db.query("SQL")` — Query and return results as JSON
- `db.migrate("migrations/")` — Run `.sql` files in order

Data is queried at startup and served via HTTP. The compiler generates rusqlite code with full type-safe column mapping.

---

## HTML Templates: `std.html`

```aid
module html_demo
use std.http
use std.html

fn main() {
    server := http.new(port: 8080)

    server.get("/") => fn(req) -> Response {
        data := { "title": "My App", "heading": "Welcome", "show_footer": true }
        content := html.template("templates/index.html", data)
        html.render(content)
    }

    server.get("/static") => fn(req) -> Response {
        html.serve_static("public/")
    }

    server.start()
}
```

Operations:
- `html.template("path", data)` — Render HTML template with variable substitution
- `html.serve_static("dir/")` — Serve static files (CSS, JS, images)
- `html.render(content)` — Return HTML response
- `html.redirect(url)` — HTTP redirect

Template syntax: `{{variable}}`, `{{#each items}}...{{/each}}`, `{{#if condition}}...{{/if}}`

---

## Authentication: `std.auth`

```aid
module auth_demo
use std.http
use std.auth

fn main() {
    server := http.new(port: 8080)

    server.post("/login") => fn(req) -> Response {
        Response.json({
            token: auth.jwt_sign("{\"sub\":\"user1\",\"role\":\"admin\"}", "my_secret"),
            message: "Login successful"
        })
    }

    server.post("/register") => fn(req) -> Response {
        Response.json({
            hash: auth.hash_password("user_password")
        })
    }
}
```

Operations:
- `auth.jwt_sign(claims, secret)` — Generate JWT token with auto-expiry
- `auth.jwt_verify(token, secret)` — Verify and decode JWT
- `auth.hash_password(password)` — Bcrypt hash with default cost
- `auth.verify_password(password, hash)` — Bcrypt verify
- `auth.api_key(header_name)` — Extract API key from request header
- `auth.middleware(handler)` — Wrap routes with JWT auth middleware

The compiler generates code using the `jsonwebtoken` and `bcrypt` Rust crates. Dependencies are auto-detected and only included when used.

---

## Architecture

```
┌─────────────┐     ┌────────┐     ┌─────────┐     ┌────────────┐     ┌──────┐     ┌────────┐
│  .aid source │────▶│ Parser │────▶│ Cortex  │────▶│ Transpiler │────▶│ Rust │────▶│ Binary │
└─────────────┘     │ (pest) │     │ Engine  │     │ (codegen)  │     │(cargo│     │+ Docs  │
                    └────────┘     └─────────┘     └────────────┘     └──────┘     └────────┘
                                        │
                                   Analyzes:
                                   • reason blocks
                                   • evolve telemetry
                                   • contract rules
                                   • intent routing
```

---

## Language at a Glance

```aid
// Strong types with inference
name := "AID"
port: int = 8080

// Entities (structs)
entity User {
    id: int
    name: string
    email: string
    role: string = "viewer"
    fn is_admin() -> bool => role == "admin"
}

// Pattern matching
match status {
    200 => "OK"
    404 => "Not Found"
    500..599 => "Server Error"
    _ => "Unknown"
}

// Error handling
fn load(path: string) -> result<Config, Error> {
    content := try read_file(path)
    return Ok(parse(content))
}

// Async
async fn fetch(url: string) -> result<string, Error> {
    response := await http.get(url)
    return Ok(response.body)
}
```

---

## Quick Start

```bash
# Clone the repo
git clone https://github.com/danilo-telnyx/aid-lang.git
cd aid-lang

# Build the compiler (requires Rust)
cd compiler && cargo build --release

# Build an example
./target/release/aid build ../examples/hello.aid

# Run it
./build/aid-hello
```

---

## Examples

| Example | Features Used | File |
|---------|-------------|------|
| Hello World | HTTP server, text + JSON responses | [`examples/hello.aid`](examples/hello.aid) |
| Ticket Classifier | Reason blocks, evolve telemetry | [`examples/tickets.aid`](examples/tickets.aid) |
| Full Demo | Entities, contracts, async, pattern matching, loops | [`examples/full-demo.aid`](examples/full-demo.aid) |
| Contract Validation | Natural language validation rules | [`examples/contracts.aid`](examples/contracts.aid) |
| Intent Routing | Auto-discovered routes, /api/routes endpoint | [`examples/intent.aid`](examples/intent.aid) |
| WASM Module | WASM compilation target | [`examples/wasm-module.aid`](examples/wasm-module.aid) |
| Env Demo | std.env, .env loading, config-driven server | [`examples/env-demo.aid`](examples/env-demo.aid) |
| Database | std.db, SQLite, query + serve via HTTP, reason blocks | [`examples/database.aid`](examples/database.aid) |
| HTML Demo | std.html, templates, static files, redirects | [`examples/html-demo.aid`](examples/html-demo.aid) |
| Auth Demo | std.auth, JWT tokens, bcrypt hashing, API keys | [`examples/auth-demo.aid`](examples/auth-demo.aid) |

---

## Comparison

| Feature | AID | Go | Rust | Python |
|---------|-----|-----|------|--------|
| AI reasoning built-in | ✅ | ❌ | ❌ | ❌ |
| Self-improving code | ✅ | ❌ | ❌ | ❌ |
| Natural language validation | ✅ | ❌ | ❌ | ❌ |
| Auto-documentation | ✅ | ❌ | ✅ | ❌ |
| Type safety | ✅ | ✅ | ✅ | ❌ |
| HTTP built-in | ✅ | ✅ | ❌ | ❌ |
| WASM target | ✅ | 🟡 | ✅ | ❌ |

---

## Package Manager

AID includes a built-in package manager for dependency management and distribution:

```bash
# Add a dependency
aid install community/redis

# Install all dependencies
aid install

# Search packages
aid search redis

# Publish your package
aid publish
```

Packages use `aid.toml` for configuration (like Cargo.toml), support semver versioning, and work with both a central registry (`registry.aidlang.dev`) and Git-based sources.

See the full [Package Specification](docs/PACKAGE-SPEC.md) for details on the manifest format, registry API, dependency resolution, and security model.

---

## License

**Business Source License 1.1** — Free for personal, educational, open source, and internal use.

Commercial use requires a [Commercial License](COMMERCIAL.md) with revenue-based pricing (first $100K free).

Converts to Apache 2.0 on February 19, 2030.

See [LICENSE.md](LICENSE.md) for full terms.

---

## Project

- **Owner:** [@danilo-telnyx](https://github.com/danilo-telnyx)
- **Language:** Compiler written in Rust
- **Created:** February 2026
- **Status:** v0.1.0 (First Complete Release)

---

*Built with conviction that programming languages should think, not just execute.*
