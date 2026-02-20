# AID Language — Project Status & Context

> **For DAInilo:** Read this file at the start of any AID-related session to restore full context.

## What Is AID?

**AID (Auto-Intelligent Development Language)** — a statically typed, compiled programming language with embedded AI reasoning. Code that thinks. Software that evolves.

- **Creator & Owner:** Danilo Smaldone (@danilo-telnyx)
- **Repo:** https://github.com/danilo-telnyx/aid-lang (private)
- **License:** BSL 1.1 + Commercial revenue share (see COMMERCIAL.md)
- **Started:** 2026-02-18

## Core Design Decisions (Locked In)

1. **`reason` blocks are explicit** — developer opts in, no magic
2. **Target audience:** Solo devs & small enterprise building APIs fast
3. **Cortex is local-only** — no cloud, no network. Privacy by architecture.
4. **Compile to WASM** (via Rust transpilation) — native binary + WASM output (`--target wasm`)
5. **Auto-docs at every build** — documentation is a compiler output
6. **Immutable by default** — `mut` to opt into mutability
7. **Go's simplicity + Rust's safety** — easy to read, hard to break
8. **Novel ideas welcome** — not just copying existing languages

## The Five Unique Features

| Priority | Feature | Status | Description |
|----------|---------|--------|-------------|
| ⭐ 1 | `reason` blocks | ✅ V1 done | Declarative AI functions with goal/constraints/examples/fallback |
| ⭐ 2 | `evolve` blocks | ✅ V1 done | Self-improving code — runtime telemetry feeds next build |
| ⭐ 3 | `intent` routing | ✅ V1 done | AI-native HTTP routing — Cortex builds route tables at compile time |
| ⭐ 4 | `contract` validation | ✅ V1 done | Natural language rules → type-safe validators, /validate endpoint |
| ⭐ 5 | Auto-documentation | ✅ Basic | Generated at every build, includes reason block explanations |

## Project Structure

```
~/Documents/projects/aid-lang/
├── docs/AID-Language-Documentation.md   # Full spec (25 sections, EBNF grammar)
├── spec/v0.1-syntax.md                  # Original syntax draft
├── examples/
│   ├── hello.aid                        # Hello world (text + JSON routes)
│   ├── tickets.aid                      # Reason blocks + evolve telemetry
│   ├── full-demo.aid                    # All language features combined
│   ├── contracts.aid                    # Contract validation demo
│   ├── intent.aid                       # Intent routing demo
│   ├── wasm-module.aid                  # WASM compilation target
│   ├── env-demo.aid                     # std.env demo (.env + config-driven server)
│   └── database.aid                     # std.db demo (SQLite + HTTP + reason blocks)
├── poc/transpile.mjs                    # PoC transpiler (Node.js, keep for reference)
├── aid                                  # PoC CLI wrapper (bash)
├── compiler/                            # ⭐ Real compiler (Rust)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── parser/aid.pest              # PEG grammar
│       ├── parser/mod.rs                # pest → AST bridge
│       ├── ast/mod.rs                   # AST type definitions
│       ├── transpiler/mod.rs            # AST → Rust code generation
│       ├── codegen/http.rs              # std.http → Axum mapping
│       ├── codegen/env.rs               # std.env → Rust env/dotenvy mapping
│       ├── codegen/db.rs               # std.db → rusqlite mapping
│       └── cli/mod.rs                   # clap CLI
├── LICENSE.md                           # BSL 1.1
├── COMMERCIAL.md                        # Revenue share tiers
├── README.md
├── CHANGELOG.md
└── CODEOWNERS                           # @danilo-telnyx
```

## What Works Today

- `aid build hello.aid` → parse → transpile → Rust → native binary
- `aid run` → starts HTTP server
- `reason` blocks → keyword-matching V1 (extracts keywords from examples, parses constraints for rules)
- `evolve` blocks → runtime telemetry logging to `.cortex/telemetry/*.jsonl`
- `GET /telemetry` → live stats (call counts + output distribution)
- `contract` blocks → English rules parsed into type-safe validators, /validate endpoint
- Auto-docs generated at every build
- Full CLI: build, run, clean, docs, fmt, lint, cortex, rollback, evolve
- Intent routing: compiler auto-discovers handlers by naming convention, builds route tables, /api/routes endpoint
- WASM compilation: `aid build --target wasm` → wasm32-wasip1 binaries
- `std.env` module: env.get, env.require, env.load_dotenv, env.all → Rust std::env + dotenvy
- `std.db` module: db.connect, db.execute, db.query, db.migrate → Rust rusqlite with AppState
- 8 working examples: hello.aid, tickets.aid, full-demo.aid, contracts.aid, intent.aid, wasm-module.aid, env-demo.aid, database.aid

## How to Build & Test

```bash
# Build the compiler
source "$HOME/.cargo/env"
cd ~/Documents/projects/aid-lang/compiler
cargo build --release

# Build hello world
./target/release/aid build ../examples/hello.aid
./build/aid-hello

# Build tickets (with reason + evolve)
./target/release/aid build ../examples/tickets.aid
./build/aid-tickets
# Test: curl http://localhost:8080/
# Test: curl -X POST http://localhost:8080/classify -d "Server is down"
# Test: curl http://localhost:8080/telemetry
```

## Roadmap (GitHub Issues)

| # | Issue | Status | Effort | Notes |
|---|-------|--------|--------|-------|
| 1 | Reason block transpilation | ✅ Done | — | V1 keyword matching from examples/constraints |
| 2 | Evolve block telemetry | ✅ Done | — | JSONL logging + /telemetry endpoint |
| 3 | WASM compilation target | ✅ Done | — | `--target wasm` → wasm32-wasip1 output |
| 4 | Contract validation generation | ✅ Done | — | English rules → type-safe validators, /validate endpoint |
| 5 | Intent routing | ✅ Done | — | Auto-discover handlers, build route tables, /api/routes endpoint |
| 6 | Project website | ✅ Done | — | GitHub Pages site live |
| 7 | Expand parser to full grammar | ✅ Done | — | Entities, methods, match, loops, try, async, contracts, implements |

**🎉 v0.1.0 COMPLETE — All 7 roadmap issues implemented!**

| 8 | std.env module | ✅ Done | — | env.get, env.require, env.load_dotenv, env.all |
| 9 | std.db module | ✅ Done | — | db.connect, db.execute, db.query, db.migrate (SQLite) |

## Future Roadmap (Beyond Issues)

- **Cortex V1:** Replace keyword matcher with small local model (1B param)
- **Evolve V2:** Telemetry-driven retraining + approval workflow
- **std.html:** Templating + static files
- **Test framework:** `aid test` with reason block assertions
- **Package manager:** `aid install <package>`
- **VS Code extension:** Syntax highlighting, autocomplete
- **Generics / traits:** Full type system

## Commercial License Tiers

| Revenue from AID Products | Share |
|---------------------------|-------|
| Under $100K | Free |
| $100K - $1M | 0.1% |
| $1M - $10M | 0.5% |
| Over $10M | 1.0% |

## Key Dates

- 2026-02-18: Language concept & spec created
- 2026-02-19: PoC working, real compiler built, reason blocks, evolve telemetry, contracts, full parser, website, GitHub repo
- 2026-02-19: v0.1.0 released — all 7 roadmap features complete
- 2030-02-19: BSL license converts to Apache 2.0 (Change Date)
