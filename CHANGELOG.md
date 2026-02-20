# Changelog

## v0.2.0 — 2026-02-20

### 🔐 std.auth Module (JWT + Bcrypt + Middleware)

#### New Features
- **`use std.auth`** — import the authentication module
- **`auth.jwt_sign(claims, secret)`** — generate JWT tokens with auto-expiry (1h default)
- **`auth.jwt_verify(token, secret)`** — verify and decode JWT tokens
- **`auth.hash_password(password)`** — bcrypt hash with default cost
- **`auth.verify_password(password, hash)`** — bcrypt password verification
- **`auth.api_key(header_name)`** — extract API key from request headers
- **`auth.middleware(handler)`** — wrap routes with JWT auth middleware
- New codegen module: `compiler/src/codegen/auth.rs` with 14 unit tests
- New example: `examples/auth-demo.aid` — login, registration, protected endpoints
- Auth calls auto-extracted as local variables when used inside `Response.json()`

#### Architecture
- JWT operations use the `jsonwebtoken` Rust crate (v9)
- Password hashing uses the `bcrypt` Rust crate (v0.15)
- Auth middleware generates Axum `from_fn` middleware with Bearer token extraction
- API key middleware generates header-based auth checking
- Dependencies auto-detected: only `jsonwebtoken` and/or `bcrypt` added when used

#### Dependencies
- `jsonwebtoken = "9"` added when JWT operations are used
- `bcrypt = "0.15"` added when password hashing is used

---

### 🌐 std.html Module (Templates + Static Files)

#### New Features
- **`use std.html`** — import the HTML module
- **`html.template("path", data)`** — render HTML templates with variable substitution
- **`html.serve_static("dir/")`** — serve static files via tower-http ServeDir
- **`html.render(content)`** — return HTML response
- **`html.redirect(url)`** — HTTP redirect response
- **Template syntax:** `{{variable}}`, `{{#each items}}...{{/each}}`, `{{#if condition}}...{{/if}}`
- Built-in recursive template engine with regex-based processing
- New codegen module: `compiler/src/codegen/html.rs` with 11 unit tests
- New example: `examples/html-demo.aid` — templates, static files, dynamic pages, redirects
- Example templates in `examples/templates/` and static assets in `examples/public/`

#### Architecture
- Template engine supports nested `#each` loops and `#if` conditionals
- Static file serving via `tower_http::services::ServeDir`
- `regex` crate added for template variable substitution
- HTML responses via `axum::response::Html`

#### Dependencies
- `regex = "1"` added when `html.template()` is used
- `tower-http = { version = "0.6", features = ["fs"] }` added when `html.serve_static()` is used

---

### 🗄️ std.db Module (SQLite)

#### New Features
- **`use std.db`** — import the database module
- **`db.connect("sqlite://path.db")`** — open SQLite database via rusqlite
- **`db.execute("SQL")`** — run DDL/DML statements (CREATE, INSERT, UPDATE, DELETE)
- **`db.query("SQL")`** — query database, returns `Vec<serde_json::Value>` with column mapping
- **`db.migrate("dir/")`** — run `.sql` migration files in alphabetical order
- Database query results shared via Axum State for HTTP handlers
- New codegen module: `compiler/src/codegen/db.rs` with 5 unit tests
- New example: `examples/database.aid` — tickets table, CRUD, HTTP API, reason blocks

#### Architecture
- Query results materialized at startup, served via AppState
- Automatic column-to-JSON mapping (Integer, Real, Text, Blob, Null)
- `rusqlite` with `bundled` feature (no system SQLite dependency)

#### Dependencies
- `rusqlite = { version = "0.31", features = ["bundled"] }` added to generated Cargo.toml when `use std.db` is detected

---

### 🌍 std.env Module

#### New Features
- **`use std.env`** — import the environment module
- **`env.get("KEY")`** — read env var as `Option<String>` (returns `None` if unset)
- **`env.require("KEY")`** — read env var as `String` (panics if missing)
- **`env.load_dotenv()`** — load `.env` file using `dotenvy` crate
- **`env.all()`** — get all env vars as `HashMap<String, String>`
- Config-driven HTTP server: port from env var overrides default
- New codegen module: `compiler/src/codegen/env.rs` with 6 unit tests
- New example: `examples/env-demo.aid` — .env loading + config-driven server

#### Dependencies
- `dotenvy = "0.15"` added automatically when `env.load_dotenv()` is used

---

## v0.1.0 — 2026-02-19

### 🎉 Initial Release — All Roadmap Features Complete

#### Core Language
- Full parser: entities, functions, async, pattern matching, loops, error handling
- Strong static type system with inference
- Immutable by default, Go simplicity + Rust safety
- Modules and imports

#### AI Features
- **Reason Blocks**: AI-powered decision functions (V1 keyword matching)
- **Evolve Blocks**: Runtime telemetry, /telemetry endpoint, self-improving code
- **Contract Validation**: English rules → type-safe validators, /validate endpoint
- **Intent Routing**: Auto-discover handlers, build route tables, /api/routes endpoint

#### Compiler
- Rust compiler with pest parser (~5,000+ lines)
- AID → Rust/Axum transpilation
- Native binary output (macOS/Linux)
- WASM output (`--target wasm` → wasm32-wasip1)
- Auto-documentation at every build

#### CLI
- `aid build` (with `--target native|wasm`, `--release`)
- `aid run`, `aid clean`, `aid docs`
- `aid fmt`, `aid lint`, `aid test`
- `aid cortex status`, `aid rollback`, `aid evolve`

#### Examples
- hello.aid — Hello world HTTP server
- tickets.aid — Ticket classifier with reason + evolve
- contracts.aid — Natural language validation
- intent.aid — Auto-discovered routing (7 routes, zero config)
- full-demo.aid — All language features combined
- wasm-module.aid — WASM compilation target

#### Project
- BSL 1.1 license with commercial revenue share
- GitHub Wiki (19 documentation pages)
- Project website (GitHub Pages)
