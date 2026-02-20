# Changelog

## v0.2.0 — 2026-02-20

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
