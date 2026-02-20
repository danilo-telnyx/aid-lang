# AID — Auto-Intelligent Development Language

[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](https://github.com/danilo-telnyx/aid-lang/releases/tag/v0.2.0)
[![License](https://img.shields.io/badge/license-BSL%201.1-green.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/compiler-Rust%201.93-orange.svg)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-v0.2.0%20The%203%20Pillars-brightgreen.svg)](https://github.com/danilo-telnyx/aid-lang/releases/tag/v0.2.0)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20WASM-lightgrey.svg)](#)
[![Tests](https://img.shields.io/badge/tests-58%20passing-brightgreen.svg)](#)

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

## What's New in v0.2.0 — The 3 Pillars

### 🏛️ Pillar 1: Standard Library
- **`std.db`** — SQLite database (connect, query, execute, migrate)
- **`std.env`** — Environment variables & `.env` files
- **`std.auth`** — JWT tokens, bcrypt hashing, API keys, auth middleware
- **`std.html`** — HTML templates, static file serving, redirects

### 🧠 Pillar 2: Cortex V1 — Local AI
- Local LLM via **llama.cpp** sidecar — no cloud, no network
- `aid cortex pull` / `aid cortex serve` / `aid cortex status`
- Reason blocks try LLM first, fall back to keyword matching
- `cortex.toml` configuration

### 🛠️ Pillar 3: Developer Tools
- **`aid new`** — Project scaffolding (`--template api|minimal`)
- **VS Code extension** — Syntax highlighting & autocomplete
- **Package spec** — `aid.toml`, semver, registry design (`docs/PACKAGE-SPEC.md`)

### 🚀 Showcase: Webhook Classifier
Complete production-ready app using **every AID feature** — std.db, std.env, std.auth, std.html, reason blocks, evolve blocks, contracts, intent routing, Cortex V1.

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
        Response.json({ status: "ok", language: "AID", version: "0.2.0" })
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

## Quick Start

```bash
# Install (requires Rust)
git clone https://github.com/danilo-telnyx/aid-lang.git
cd aid-lang/compiler && cargo build --release

# Create a new project
./target/release/aid new myapp
cd myapp

# Build & run
../target/release/aid build main.aid
./build/aid-main
```

### `aid new` Templates

```bash
# Full REST API (default) — templates, static files, migrations
$ aid new myapp

# Minimal — just main.aid + config
$ aid new myapp --template minimal
```

| Template | Files | Description |
|----------|-------|-------------|
| `api` | 8 files | REST API with HTML templates, static assets, migrations |
| `minimal` | 5 files | Just the essentials — main.aid + config files |

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

## 🧠 Cortex V1 — Local AI for Reason Blocks

Cortex is AID's local AI engine. It runs a **llama.cpp sidecar** on your machine — no cloud, no network, complete privacy.

```bash
# Download a model (~670MB, TinyLlama-1.1B-Chat)
$ aid cortex pull

# Start the sidecar
$ aid cortex serve

# Check status
$ aid cortex status
  ✓ Model: TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf
  ✓ Sidecar: http://localhost:8090
  ✓ Fallback: keyword matching (V1)
```

**How it works:** When you build with Cortex running, reason blocks generate code that:
1. Sends the prompt (goal + constraints + examples) to the local LLM
2. Falls back to V1 keyword matching if the sidecar is unavailable
3. Your code stays exactly the same — Cortex is transparent

Configure via `cortex.toml`:
```toml
[model]
path = ".cortex/models/tinyllama.gguf"
temperature = 0.3
max_tokens = 100

[sidecar]
port = 8090
timeout_ms = 5000

[fallback]
enabled = true
```

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

Every call is logged. Next build, the compiler reads the telemetry and generates better logic.

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

The compiler reads the English rules and generates type-safe validators.

---

## Standard Library

### 🗄️ `std.db` — SQLite Database

```aid
use std.db

db.connect("sqlite://data.db")
db.migrate("migrations/")
db.execute("INSERT INTO items (name) VALUES ('Widget')")
items := db.query("SELECT * FROM items")
```

### 🌍 `std.env` — Environment Variables

```aid
use std.env

env.load_dotenv()
port := env.get("PORT")         // Option<String>
secret := env.require("SECRET") // panics if missing
all := env.all()                // HashMap
```

### 🔐 `std.auth` — Authentication

```aid
use std.auth

token := auth.jwt_sign(claims, "secret")
claims := auth.jwt_verify(token, "secret")
hash := auth.hash_password("password")
ok := auth.verify_password("password", hash)
key := auth.api_key("X-API-Key")
```

### 🌐 `std.html` — Templates & Static Files

```aid
use std.html

content := html.template("templates/page.html", data)
html.render(content)
html.serve_static("public/")
html.redirect("/dashboard")
```

Template syntax: `{{variable}}`, `{{#each items}}...{{/each}}`, `{{#if condition}}...{{/if}}`

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
| 🗄️ **std.db** | SQLite — connect, query, execute, migrate | ✅ Working |
| 🌍 **std.env** | Environment variables, .env files | ✅ Working |
| 🔐 **std.auth** | JWT, bcrypt, API keys, middleware | ✅ Working |
| 🌐 **std.html** | HTML templates, static files, redirects | ✅ Working |
| 🧠 **Cortex V1** | Local LLM via llama.cpp sidecar + keyword fallback | ✅ Working |
| 🆕 **`aid new`** | Project scaffolding (api, minimal templates) | ✅ Working |
| 🚀 **Webhook Classifier** | Full showcase app — every feature combined | ✅ Complete |
| 📦 **Package Spec** | aid.toml, semver, registry design | ✅ Designed |
| 🎨 **VS Code Extension** | Syntax highlighting & autocomplete | ✅ Available |

---

## Architecture

```
┌─────────────┐     ┌────────┐     ┌─────────┐     ┌────────────┐     ┌──────┐     ┌────────┐
│  .aid source │────▶│ Parser │────▶│ Cortex  │────▶│ Transpiler │────▶│ Rust │────▶│ Binary │
└─────────────┘     │ (pest) │     │ Engine  │     │ (codegen)  │     │(cargo│     │+ Docs  │
                    └────────┘     └─────────┘     └────────────┘     └──────┘     └────────┘
                                        │                │
                                   Analyzes:        Generates:
                                   • reason blocks   • std.db → rusqlite
                                   • evolve data     • std.auth → jsonwebtoken + bcrypt
                                   • contracts       • std.html → templates + tower-http
                                   • intent routes   • std.env → dotenvy
                                   • cortex config   • cortex → ureq + LLM client
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

## Examples

| Example | Features Used | File |
|---------|-------------|------|
| Hello World | HTTP server, text + JSON responses | [`examples/hello.aid`](examples/hello.aid) |
| Ticket Classifier | Reason blocks, evolve telemetry | [`examples/tickets.aid`](examples/tickets.aid) |
| Full Demo | Entities, contracts, async, pattern matching | [`examples/full-demo.aid`](examples/full-demo.aid) |
| Contract Validation | Natural language validation rules | [`examples/contracts.aid`](examples/contracts.aid) |
| Intent Routing | Auto-discovered routes, /api/routes | [`examples/intent.aid`](examples/intent.aid) |
| WASM Module | WASM compilation target | [`examples/wasm-module.aid`](examples/wasm-module.aid) |
| Env Demo | std.env, .env loading, config-driven server | [`examples/env-demo.aid`](examples/env-demo.aid) |
| Database | std.db, SQLite, CRUD + HTTP API | [`examples/database.aid`](examples/database.aid) |
| HTML Demo | std.html, templates, static files | [`examples/html-demo.aid`](examples/html-demo.aid) |
| Auth Demo | std.auth, JWT, bcrypt, API keys | [`examples/auth-demo.aid`](examples/auth-demo.aid) |
| Cortex Demo | Cortex V1, LLM-powered reason blocks | [`examples/cortex-demo.aid`](examples/cortex-demo.aid) |
| **Webhook Classifier** | **All features combined** — the showcase | [`examples/webhook-classifier/`](examples/webhook-classifier/) |

---

## Comparison

| Feature | AID | Go | Rust | Python |
|---------|-----|-----|------|--------|
| AI reasoning built-in | ✅ | ❌ | ❌ | ❌ |
| Self-improving code | ✅ | ❌ | ❌ | ❌ |
| Natural language validation | ✅ | ❌ | ❌ | ❌ |
| Local AI (no cloud) | ✅ | ❌ | ❌ | ❌ |
| Auto-documentation | ✅ | ❌ | ✅ | ❌ |
| Type safety | ✅ | ✅ | ✅ | ❌ |
| HTTP built-in | ✅ | ✅ | ❌ | ❌ |
| WASM target | ✅ | 🟡 | ✅ | ❌ |
| Built-in auth (JWT/bcrypt) | ✅ | ❌ | ❌ | ❌ |
| Built-in database | ✅ | ❌ | ❌ | ❌ |

---

## Package Manager

AID includes a built-in package manager for dependency management and distribution:

```bash
aid install community/redis   # Add a dependency
aid install                    # Install all deps
aid search redis               # Search packages
aid publish                    # Publish your package
```

Packages use `aid.toml` for configuration, support semver versioning, and work with both a central registry (`registry.aidlang.dev`) and Git-based sources.

See the full [Package Specification](docs/PACKAGE-SPEC.md) for details.

---

## License

**Business Source License 1.1** — Free for personal, educational, open source, and internal use.

Commercial use requires a [Commercial License](COMMERCIAL.md) with revenue-based pricing (first $100K free).

Converts to Apache 2.0 on February 19, 2030.

See [LICENSE.md](LICENSE.md) for full terms.

---

## Project

- **Owner:** [@danilo-telnyx](https://github.com/danilo-telnyx)
- **Language:** Compiler written in Rust (58 tests)
- **Created:** February 2026
- **Status:** v0.2.0 — The 3 Pillars

---

*Built with conviction that programming languages should think, not just execute.*
