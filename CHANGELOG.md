# Changelog

## v0.1.0-alpha — 2026-02-19

### Added
- Language specification v1.0 (25 sections)
- Rust compiler with pest parser (~5,000 lines)
- Reason blocks: AI-powered decision functions (V1 keyword matching)
- Evolve blocks: runtime telemetry logging and /telemetry endpoint
- Contract validation: natural language rules → type-safe validators
- Full parser: entities, async, pattern matching, loops, error handling
- CLI: build, run, clean, docs, fmt, lint, cortex, rollback, evolve
- Auto-documentation generated at every build
- std.http → Axum mapping with routes, middleware, JSON support
- GitHub Wiki with 19 documentation pages
- Project website (GitHub Pages)
- Intent routing: compiler auto-discovers handlers, builds route tables, /api/routes endpoint
- 5 working examples: hello, tickets, full-demo, contracts, intent
- BSL 1.1 license with commercial revenue share terms

### Architecture
- Parser: pest PEG grammar
- AST: Full type definitions for all AID constructs
- Transpiler: AST → Rust/Axum code generation
- Codegen: HTTP mapping, reason block keyword extraction, contract rule parsing
- Target: Native binary (WASM planned)
