# Changelog

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
