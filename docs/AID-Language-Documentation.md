# AID — Auto-Intelligent Development Language

**Version 1.0 — Language Documentation**

> *Code that thinks. Software that evolves.*

---

## Implementation Status

> Last updated: 2026-02-19 (v0.1.0-alpha)

| Feature | Spec | Implemented | Notes |
|---------|------|-------------|-------|
| Type system | ✅ | ✅ | Full parser support |
| Entities | ✅ | ✅ | Fields, defaults, methods |
| Functions | ✅ | ✅ | Regular, async, lambda, private |
| Control flow | ✅ | ✅ | if/else, match, for, while |
| Error handling | ✅ | ✅ | result, option, try |
| Modules | ✅ | ✅ | Parsing only |
| Reason blocks | ✅ | ✅ V1 | Keyword matching from examples |
| Evolve blocks | ✅ | ✅ V1 | Runtime telemetry logging |
| Intent routing | ✅ | ✅ V1 | Auto-discovery by naming convention |
| Contracts | ✅ | ✅ V1 | English rules → validators |
| HTTP server | ✅ | ✅ | Via Axum |
| Auto-docs | ✅ | ✅ | Basic generation |
| WASM target | ✅ | ✅ | `--target wasm` → wasm32-wasip1 |
| Cortex engine | ✅ | ⬜ | V1 uses keyword matching |

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Design Philosophy](#2-design-philosophy)
3. [Architecture Overview](#3-architecture-overview)
4. [Getting Started](#4-getting-started)
5. [Type System](#5-type-system)
6. [Variables & Mutability](#6-variables--mutability)
7. [Entities](#7-entities)
8. [Functions](#8-functions)
9. [Control Flow](#9-control-flow)
10. [Error Handling](#10-error-handling)
11. [Modules & Visibility](#11-modules--visibility)
12. [The Cortex Engine](#12-the-cortex-engine)
13. [Reason Blocks](#13-reason-blocks)
14. [Evolve Blocks](#14-evolve-blocks)
15. [Intent Routing](#15-intent-routing)
16. [Contracts](#16-contracts)
17. [HTTP Server](#17-http-server)
18. [Auto-Documentation](#18-auto-documentation)
19. [Compilation & Build Pipeline](#19-compilation--build-pipeline)
20. [Standard Library](#20-standard-library)
21. [CLI Reference](#21-cli-reference)
22. [Complete Example — Ticket API](#22-complete-example--ticket-api)
23. [Appendix A — Reserved Keywords](#appendix-a--reserved-keywords)
24. [Appendix B — Operator Precedence](#appendix-b--operator-precedence)
25. [Appendix C — Grammar (EBNF)](#appendix-c--grammar-ebnf)

---

## 1. Introduction

AID (Auto-Intelligent Development Language) is a statically typed, compiled programming language designed for building intelligent APIs and web services. It introduces a fundamentally new concept: **embedded reasoning** — the ability to declare AI-powered decision logic directly in source code using natural language, compiled and optimized alongside traditional code.

AID compiles to **WebAssembly (WASM)** via Rust transpilation, ensuring portability across processors, operating systems, and deployment environments — from edge devices to cloud servers.

### What Makes AID Different

Traditional programming languages require developers to encode every decision as explicit logic: conditionals, lookup tables, regular expressions, rule engines. AID introduces the `reason` block — a first-class language construct that lets developers declare *what* a decision should achieve, and delegates the *how* to the Cortex, a local AI reasoning engine.

This is not an API call to a cloud service. The Cortex runs locally, processes `reason` blocks at compile time to generate optimized decision logic, and optionally provides runtime inference for edge cases. The result is software that:

- **Thinks** — makes intelligent decisions without hand-coded rules
- **Evolves** — improves its decision accuracy with each deployment
- **Documents itself** — generates human-readable explanations of all logic at every build

### Target Audience

- **Solo developers** building APIs and web services rapidly
- **Small engineering teams** at startups and enterprises who want to ship intelligent software without maintaining separate ML infrastructure
- **Backend engineers** who need classification, routing, validation, or content analysis baked into their applications

### Language Goals

| Goal | Description |
|------|-------------|
| **Simplicity** | Go-like readability. The syntax should be learnable in a day. |
| **Safety** | Rust-like guarantees. Strong static typing, immutability by default, explicit error handling. |
| **Intelligence** | First-class AI reasoning as a language construct, not a library. |
| **Portability** | WASM compilation target. Write once, deploy anywhere. |
| **Self-documentation** | Every build generates complete, accurate documentation automatically. |

---

## 2. Design Philosophy

AID is built on five principles:

### 2.1 Explicit Over Magical

The developer is always in control. AI reasoning happens only when the developer explicitly requests it via a `reason` block. There is no hidden inference, no implicit AI behavior. Every intelligent decision point is visible in the source code.

### 2.2 Compile-Time Over Runtime

Wherever possible, intelligence is resolved at compile time. The Cortex analyzes `reason` blocks during compilation and generates optimized, deterministic code — decision trees, embedding lookups, pattern matchers. Runtime inference is available but opt-in, not default.

### 2.3 Local Over Remote

The Cortex engine runs entirely on the developer's machine. No data leaves the build environment. No cloud API keys. No network dependency. Privacy and speed are guaranteed by architecture, not by policy.

### 2.4 Simple Syntax, Powerful Semantics

The surface syntax borrows from Go (readability, minimal ceremony) and Rust (type safety, pattern matching, explicit error handling). The novel constructs — `reason`, `evolve`, `intent`, `contract` — follow the same syntactic conventions, so they feel native rather than bolted on.

### 2.5 Code Is Documentation

AID rejects the notion that documentation is a separate artifact maintained by humans. Every build produces complete documentation derived directly from the source code, including natural-language explanations of AI decision logic. If the code changes, the documentation changes. They cannot drift.

---

## 3. Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                 AID Source (.aid)                │
├─────────────────────────────────────────────────┤
│                                                 │
│   ┌─────────┐   ┌─────────┐   ┌────────────┐   │
│   │ Parser  │──▶│  AST    │──▶│ Type Check │   │
│   └─────────┘   └─────────┘   └─────┬──────┘   │
│                                     │           │
│                              ┌──────▼───────┐   │
│                              │   Cortex     │   │
│                              │   Engine     │   │
│                              │              │   │
│                              │ • Analyze    │   │
│                              │   reason     │   │
│                              │   blocks     │   │
│                              │ • Generate   │   │
│                              │   decision   │   │
│                              │   logic      │   │
│                              │ • Process    │   │
│                              │   evolve     │   │
│                              │   telemetry  │   │
│                              │ • Build      │   │
│                              │   intent     │   │
│                              │   routes     │   │
│                              │ • Generate   │   │
│                              │   contract   │   │
│                              │   validators │   │
│                              └──────┬───────┘   │
│                                     │           │
│                              ┌──────▼───────┐   │
│                              │ Transpiler   │   │
│                              │ (AID → Rust) │   │
│                              └──────┬───────┘   │
│                                     │           │
│                              ┌──────▼───────┐   │
│                              │ Rust → WASM  │   │
│                              └──────┬───────┘   │
│                                     │           │
│                    ┌────────────────┼────────┐  │
│                    ▼                ▼        ▼  │
│               Binary (.wasm)   Docs (.md)  Telemetry │
│                                             Store    │
└─────────────────────────────────────────────────┘
```

### 3.1 Compilation Pipeline

1. **Parse** — AID source files (`.aid`) are parsed into an Abstract Syntax Tree (AST)
2. **Type Check** — Static type analysis with inference. All types are resolved at this stage.
3. **Cortex Processing** — The Cortex engine processes all AI-related constructs:
   - `reason` blocks → optimized decision logic
   - `evolve` blocks → integrates telemetry from previous runs
   - `intent` routes → compile-time routing tables
   - `contract` rules → validation functions
4. **Transpile** — The enriched AST is transpiled to Rust source code
5. **Compile** — Rust compiles to WebAssembly
6. **Document** — Auto-documentation is generated from the AST + Cortex outputs

### 3.2 The Cortex Engine

The Cortex is a local AI reasoning engine embedded in the AID compiler. It is **not** a general-purpose LLM. It is a specialized engine optimized for:

- **Classification** — mapping inputs to categories
- **Pattern recognition** — identifying structures in text and data
- **Decision optimization** — generating efficient branching logic from examples and constraints
- **Validation generation** — converting natural language rules to type-safe code

The Cortex v1 implementation uses a small quantized model (≈1B parameters) or a vector similarity engine, depending on the complexity of the `reason` block. It runs entirely locally with no network access.

---

## 4. Getting Started

### 4.1 Installation

```bash
# Install the AID toolchain
curl -sSf https://install.aidlang.dev | sh

# Verify installation
aid --version
```

The installer provides:
- `aid` — compiler and build tool
- `cortex` — the local reasoning engine
- `aid-doc` — documentation generator (also runs automatically on build)

### 4.2 Project Structure

```
my-project/
├── aid.toml            # Project configuration
├── src/
│   ├── main.aid        # Entry point
│   └── models/
│       └── user.aid    # Entity definitions
├── build/
│   ├── output.wasm     # Compiled binary
│   └── docs/           # Auto-generated documentation
└── .cortex/
    ├── telemetry/      # Evolve tracking data
    └── cache/          # Cortex compilation cache
```

### 4.3 Project Configuration (`aid.toml`)

```toml
[project]
name = "my-api"
version = "0.1.0"
entry = "src/main.aid"

[cortex]
mode = "hybrid"         # static | dynamic | hybrid
confidence = 0.85       # minimum confidence threshold
model = "cortex-1b"     # local model to use

[build]
target = "wasm"
optimize = true
docs = true             # generate docs on every build

[evolve]
storage = ".cortex/telemetry"
auto_approve = false    # require human approval for logic changes
```

### 4.4 Hello World

```aid
// src/main.aid
module main

use std.http

fn main() {
    server := http.new(port: 8080)

    server.get("/") => fn(req) -> Response {
        Response.text("Hello from AID")
    }

    server.start()
}
```

```bash
# Build and run
aid build
aid run

# Output:
# ✓ Compiled to build/output.wasm (42 KB)
# ✓ Documentation generated at build/docs/
# → Server listening on http://localhost:8080
```

---

## 5. Type System

AID uses a **strong, static type system with local type inference**. Every value has a known type at compile time. The compiler infers types where possible, but the developer can always annotate explicitly.

### 5.1 Scalar Types

| Type | Description | Example |
|------|-------------|---------|
| `int` | 64-bit signed integer | `42`, `-1`, `0` |
| `float` | 64-bit floating point (IEEE 754) | `3.14`, `-0.5` |
| `bool` | Boolean | `true`, `false` |
| `string` | UTF-8 string | `"hello"`, `""` |
| `byte` | Unsigned 8-bit integer | `0xFF`, `255` |

### 5.2 Composite Types

| Type | Description | Example |
|------|-------------|---------|
| `array<T>` | Ordered, variable-length collection | `[1, 2, 3]` |
| `map<K, V>` | Key-value hash map | `{ "a": 1, "b": 2 }` |
| `option<T>` | Value that may or may not exist | `Some(42)`, `None` |
| `result<T, E>` | Success or error | `Ok(data)`, `Err(e)` |
| `stream<T>` | Asynchronous data stream | *(see §8 Async)* |
| `entity` | Named structured type | *(see §7 Entities)* |

### 5.3 Type Inference

The walrus operator `:=` triggers type inference. The compiler deduces the type from the right-hand expression.

```aid
name := "AID"               // inferred: string
port := 8080                 // inferred: int
ratio := 0.75                // inferred: float
active := true               // inferred: bool
tags := ["api", "fast"]      // inferred: array<string>
config := { "port": 8080 }   // inferred: map<string, int>
```

Explicit annotation uses the `name: type = value` syntax:

```aid
port: int = 8080
name: string = "AID"
verbose: bool = false
```

### 5.4 Type Aliases

```aid
type UserID = int
type Headers = map<string, string>
type Handler = fn(Request) -> Response
```

### 5.5 Nullability

AID has **no null**. Absence of a value is expressed through `option<T>`:

```aid
fn find_user(id: int) -> option<User> {
    // returns Some(user) or None
}

// Caller must handle both cases
match find_user(42) {
    Some(user) => greet(user)
    None => log("User not found")
}
```

---

## 6. Variables & Mutability

All bindings in AID are **immutable by default**. This design choice supports:

- Predictable behavior in concurrent and async code
- Safer reasoning by the Cortex (immutable data has no hidden state changes)
- Fewer bugs in API handlers where data flows through a pipeline

### 6.1 Immutable Bindings (Default)

```aid
name := "AID"
name = "changed"        // ❌ COMPILE ERROR: cannot mutate immutable binding
```

### 6.2 Mutable Bindings

Use the `mut` keyword to opt into mutability:

```aid
mut counter := 0
counter = counter + 1   // ✅ OK

mut user := User { id: 1, name: "Dan", email: "dan@x.com" }
user.name = "Danilo"    // ✅ OK — entity fields are mutable when binding is mut
```

### 6.3 Constants

Compile-time constants use `const`. They must be deterministic — no function calls, no `reason` blocks.

```aid
const MAX_RETRIES = 3
const API_VERSION = "v1"
const TIMEOUT_MS = 5000
```

### 6.4 Shadowing

Immutable bindings can be shadowed (re-declared) in the same scope. This is preferred over mutation for transformations:

```aid
input := read_body(req)
input := trim(input)           // shadows the previous binding
input := parse_json(input)     // shadows again — each step is immutable
```

---

## 7. Entities

Entities are AID's structured data types, equivalent to structs in Rust or Go. They define the shape of data.

### 7.1 Declaration

```aid
entity User {
    id: int
    name: string
    email: string
    role: string = "viewer"         // default value
    created_at: string = now()      // default from function (evaluated at instantiation)
}
```

### 7.2 Instantiation

```aid
user := User {
    id: 1,
    name: "Danilo",
    email: "danilo@example.com"
    // role defaults to "viewer"
    // created_at defaults to now()
}
```

All fields without defaults are required. The compiler reports missing fields as errors.

### 7.3 Nested Entities

```aid
entity Address {
    street: string
    city: string
    country: string
}

entity Company {
    name: string
    address: Address
    employees: array<User>
}
```

### 7.4 Entity Methods

Entities can have associated methods:

```aid
entity Circle {
    radius: float

    fn area() -> float => 3.14159 * radius * radius

    fn scale(factor: float) -> Circle {
        return Circle { radius: radius * factor }
    }
}

c := Circle { radius: 5.0 }
a := c.area()              // 78.539...
bigger := c.scale(2.0)     // Circle { radius: 10.0 }
```

Note: `scale` returns a **new** Circle (immutable by default). It does not modify the original.

---

## 8. Functions

### 8.1 Declaration

```aid
fn greet(name: string) -> string {
    return "Hello, " + name
}
```

### 8.2 Single-Expression Shorthand

When a function body is a single expression, use `=>`:

```aid
fn double(x: int) -> int => x * 2
fn is_admin(user: User) -> bool => user.role == "admin"
```

### 8.3 Multiple Return Values

```aid
fn divide(a: float, b: float) -> (float, bool) {
    if b == 0.0 {
        return (0.0, false)
    }
    return (a / b, true)
}

(result, ok) := divide(10.0, 3.0)
```

### 8.4 Named Parameters

For clarity at the call site, parameters can be passed by name:

```aid
fn create_server(host: string, port: int, tls: bool) -> Server {
    // ...
}

server := create_server(host: "0.0.0.0", port: 443, tls: true)
```

### 8.5 Default Parameters

```aid
fn connect(host: string, port: int = 5432, timeout: int = 30) -> Connection {
    // ...
}

conn := connect("localhost")                    // port=5432, timeout=30
conn := connect("localhost", timeout: 10)       // port=5432, timeout=10
```

### 8.6 Anonymous Functions (Lambdas)

```aid
numbers := [1, 2, 3, 4, 5]
evens := numbers.filter(fn(n) => n % 2 == 0)
doubled := numbers.map(fn(n) => n * 2)
```

### 8.7 Async Functions

Asynchronous functions are declared with `async` and return a future that must be awaited:

```aid
async fn fetch(url: string) -> result<string, HttpError> {
    response := await http.get(url)
    return Ok(response.body)
}

// Usage
data := await fetch("https://api.example.com/data")
```

### 8.8 Visibility

Functions are **public by default** within their module. Use `private` to restrict:

```aid
fn public_handler(req: Request) -> Response { ... }         // accessible from other modules

private fn internal_logic(data: string) -> string { ... }   // only accessible in this module
```

---

## 9. Control Flow

### 9.1 Conditionals

```aid
if temperature > 30 {
    alert("Too hot")
} else if temperature < 0 {
    alert("Freezing")
} else {
    log("Normal")
}
```

Conditionals are expressions — they return values:

```aid
label := if score > 90 { "excellent" } else if score > 70 { "good" } else { "needs work" }
```

### 9.2 Pattern Matching

`match` is AID's primary tool for branching on values. It must be **exhaustive** — all possible values must be covered (use `_` as a wildcard).

```aid
match status_code {
    200 => handle_success(body)
    201 => handle_created(body)
    400 => handle_bad_request()
    401 | 403 => handle_unauthorized()      // multiple values
    404 => handle_not_found()
    500..599 => handle_server_error()       // range
    _ => handle_unknown(status_code)        // wildcard (required)
}
```

Match on entity types:

```aid
match event {
    UserCreated(user) => send_welcome(user)
    UserDeleted(id) => cleanup(id)
    _ => log("Unknown event")
}
```

Match is an expression:

```aid
message := match role {
    "admin" => "Full access granted"
    "editor" => "Edit access granted"
    _ => "Read-only access"
}
```

### 9.3 Loops

**For-in loops** iterate over collections:

```aid
for user in users {
    send_notification(user)
}

// With index
for (i, user) in users.enumerate() {
    log(i.to_string() + ": " + user.name)
}
```

**While loops** for condition-based iteration:

```aid
mut attempts := 0
while attempts < 3 {
    if try_connect() {
        break
    }
    attempts = attempts + 1
}
```

**Loop control:**

```aid
for item in items {
    if item.skip { continue }     // skip to next iteration
    if item.stop { break }        // exit loop
    process(item)
}
```

---

## 10. Error Handling

AID uses the `result<T, E>` type for explicit error handling. There are no exceptions. Every function that can fail declares it in its return type.

### 10.1 The `result` Type

```aid
result<T, E>
├── Ok(T)       // success, contains value of type T
└── Err(E)      // failure, contains error of type E
```

### 10.2 Returning Errors

```aid
fn parse_port(input: string) -> result<int, ParseError> {
    port := try_parse_int(input)
    if port < 0 || port > 65535 {
        return Err(ParseError { message: "Port out of range: " + input })
    }
    return Ok(port)
}
```

### 10.3 The `try` Keyword

`try` propagates errors upward. If the expression evaluates to `Err`, the enclosing function immediately returns that error. If it evaluates to `Ok`, the value is unwrapped.

```aid
fn load_config(path: string) -> result<Config, Error> {
    content := try read_file(path)          // returns Err if read fails
    data := try parse_json(content)         // returns Err if parse fails
    config := try validate(data)            // returns Err if validation fails
    return Ok(config)
}
```

Without `try`, the equivalent code would be:

```aid
fn load_config(path: string) -> result<Config, Error> {
    match read_file(path) {
        Ok(content) => {
            match parse_json(content) {
                Ok(data) => {
                    match validate(data) {
                        Ok(config) => return Ok(config)
                        Err(e) => return Err(e)
                    }
                }
                Err(e) => return Err(e)
            }
        }
        Err(e) => return Err(e)
    }
}
```

### 10.4 Custom Error Types

```aid
entity ApiError {
    code: int
    message: string
    details: option<string> = None
}

fn handle_request(req: Request) -> result<Response, ApiError> {
    if !req.has_auth() {
        return Err(ApiError { code: 401, message: "Unauthorized" })
    }
    // ...
}
```

### 10.5 The `option` Type for Absence

For values that may not exist (but are not errors):

```aid
fn find_user(id: int) -> option<User> {
    // returns Some(user) or None
}

// Unwrap with default
user := find_user(42).unwrap_or(default_user)

// Unwrap or return early
fn get_user_name(id: int) -> result<string, Error> {
    user := try find_user(id).ok_or(Error { message: "Not found" })
    return Ok(user.name)
}
```

---

## 11. Modules & Visibility

### 11.1 Module Declaration

Every `.aid` file begins with a module declaration. The module name defines its namespace.

```aid
// src/models/user.aid
module models.user

entity User {
    id: int
    name: string
    email: string
}
```

### 11.2 Imports

```aid
module main

use std.http                     // standard library module
use std.json
use models.user.User             // specific entity from a module
use models.user.*                // all exports from a module
use utils.{ validate, sanitize } // multiple specific imports
```

### 11.3 Visibility Rules

| Modifier | Scope |
|----------|-------|
| *(none)* | Public — accessible from any module that imports this one |
| `private` | Private — accessible only within the declaring module |

```aid
module auth

fn verify_token(token: string) -> result<Claims, AuthError> {
    // Public: other modules can call this
    decoded := try private_decode(token)
    return validate_claims(decoded)
}

private fn private_decode(token: string) -> result<Decoded, Error> {
    // Private: only this module can call this
}
```

### 11.4 Module Organization Convention

```
src/
├── main.aid                    // module main
├── models/
│   ├── user.aid                // module models.user
│   └── ticket.aid              // module models.ticket
├── handlers/
│   ├── auth.aid                // module handlers.auth
│   └── tickets.aid             // module handlers.tickets
└── reasoning/
    ├── classifier.aid          // module reasoning.classifier
    └── router.aid              // module reasoning.router
```

---

## 12. The Cortex Engine

The Cortex is AID's embedded reasoning engine. It is the core technology that enables `reason`, `evolve`, `intent`, and `contract` constructs. This section describes its architecture and behavior in detail.

### 12.1 What the Cortex Is

The Cortex is a **compile-time and optional runtime AI engine** that:

- Analyzes natural language directives in AID source code
- Generates optimized, deterministic code from those directives
- Runs locally with no network access
- Is specialized for classification, pattern matching, and validation — not general conversation

### 12.2 What the Cortex Is Not

- It is **not** a general-purpose LLM
- It does **not** connect to cloud services
- It does **not** generate AID code for the developer to edit
- It does **not** make decisions that aren't explicitly declared in `reason` blocks

### 12.3 Cortex Modes

The Cortex operates in three modes, configurable per project in `aid.toml` and overridable per `reason` block:

| Mode | Compile Time | Runtime | Use Case |
|------|-------------|---------|----------|
| `static` | ✅ Full analysis | ❌ None | Deterministic decisions, max performance |
| `dynamic` | ❌ Minimal | ✅ Full inference | Unpredictable inputs, complex reasoning |
| `hybrid` (default) | ✅ Full analysis | ✅ Fallback only | Best balance of speed and intelligence |

### 12.4 Confidence Threshold

Every Cortex decision has an associated confidence score (0.0 to 1.0). The threshold is configured in `aid.toml`:

```toml
[cortex]
confidence = 0.85
```

- **In `static` mode:** If confidence at compile time is below threshold, the build **fails** with a diagnostic explaining why the Cortex cannot generate reliable logic. The developer must provide more examples or constraints.
- **In `hybrid` mode:** Compile-time logic is generated at whatever confidence is achievable. At runtime, if a specific input falls below the threshold, the Cortex performs live inference.
- **In `dynamic` mode:** All decisions are made at runtime. Confidence affects logging, not behavior.

### 12.5 Cortex v1 Implementation

The initial Cortex implementation supports two backends:

1. **Quantized model (default)** — A small (~1B parameter) language model, quantized for CPU inference. Suitable for classification, routing, and validation tasks.
2. **Vector similarity engine** — An embedding-based approach using vector distance for classification. Faster but less flexible than the model backend.

The backend is selected automatically based on the complexity of the `reason` block, or can be forced in `aid.toml`:

```toml
[cortex]
backend = "model"       # "model" | "vector" | "auto"
```

### 12.6 Future: Plugin System

A future version of AID will support Cortex plugins, allowing external models to serve as fallback reasoning backends. The plugin interface will be standardized but the core Cortex will always remain local-first.

---

## 13. Reason Blocks

The `reason` block is AID's signature construct. It declares a function whose logic is generated by the Cortex based on natural language directives rather than hand-written code.

### 13.1 Syntax

```aid
reason <name>(<parameters>) -> <return_type> {
    goal: "<natural language description of what this function should do>"
    constraints: [
        "<rule 1>",
        "<rule 2>",
        ...
    ]
    context: [<optional variable references>]
    examples: [
        (<input_1>, <expected_output_1>),
        (<input_2>, <expected_output_2>),
        ...
    ]
    fallback: <expression>
}
```

### 13.2 Fields

| Field | Required | Description |
|-------|----------|-------------|
| `goal` | ✅ Yes | Natural language description of the function's purpose. This is the primary directive for the Cortex. |
| `constraints` | ✅ Yes | Array of rules that the generated logic must obey. These are hard requirements — violations cause build errors. |
| `examples` | ❌ No | Input/output pairs for training. More examples yield higher confidence and more accurate logic. |
| `context` | ❌ No | References to variables or entities in scope that the Cortex should consider when generating logic. |
| `fallback` | ❌ No | A default return value used when confidence is below threshold and runtime inference is unavailable. If omitted, the function returns an error on low confidence. |

### 13.3 Execution Modes

Override the project-level Cortex mode per block:

```aid
// Compile-time only: zero runtime cost, fully baked into the binary
reason(static) categorize(text: string) -> string {
    goal: "Categorize text into predefined buckets"
    constraints: ["Return one of: news, opinion, tutorial, other"]
    examples: [
        ("Breaking: earthquake hits...", "news"),
        ("I think we should...", "opinion"),
        ("Step 1: install the package...", "tutorial")
    ]
}

// Runtime only: Cortex inference on every call
reason(dynamic) analyze_sentiment(text: string) -> float {
    goal: "Return a sentiment score from -1.0 (negative) to 1.0 (positive)"
    constraints: [
        "Neutral text should return between -0.1 and 0.1",
        "Must handle sarcasm conservatively (lean toward neutral)"
    ]
    fallback: 0.0
}

// Hybrid (default): baked logic with runtime fallback
reason suggest_handler(req: Request) -> string {
    goal: "Suggest the best handler function name for this request"
    constraints: ["Return a valid function name from the current module"]
    fallback: "default_handler"
}
```

### 13.4 Using Reason Blocks

Once declared, a `reason` block is called like any other function:

```aid
category := categorize("Step 1: install the package")
// category == "tutorial"

score := analyze_sentiment("This product is amazing!")
// score ≈ 0.85

handler := suggest_handler(incoming_request)
// handler == "handle_create_user"
```

### 13.5 Constraints as Guarantees

Constraints are not suggestions — they are **compile-time guarantees**. The Cortex must prove that its generated logic satisfies every constraint, or the build fails.

Example of a build failure:

```aid
reason pick_color(mood: string) -> string {
    goal: "Pick a color based on mood"
    constraints: [
        "Return a valid hex color code",
        "Always start with #",
        "Exactly 7 characters"
    ]
    // No examples provided
}
```

```
BUILD ERROR: reason block 'pick_color'
  Constraint "Return a valid hex color code" cannot be guaranteed.
  The Cortex has insufficient examples to generate reliable hex codes.
  Suggestion: Add at least 5 examples or reduce constraint specificity.
```

### 13.6 Best Practices

1. **Be specific in goals.** "Classify text" is weaker than "Classify a customer support ticket into one of four categories."
2. **Provide 5+ examples for critical blocks.** More examples = higher confidence = better compile-time optimization.
3. **Constraints should be verifiable.** "Be smart about it" is not a constraint. "Return exactly one of: A, B, C" is.
4. **Use `static` for hot paths.** If a `reason` block is called thousands of times per second, force static compilation for zero runtime overhead.
5. **Always set a `fallback` for user-facing blocks.** You don't want an error surfacing to a customer because of a low-confidence edge case.

---

## 14. Evolve Blocks

The `evolve` construct is AID's mechanism for **self-improving code**. It attaches to a `reason` block and enables a feedback loop: runtime telemetry feeds back into the next compilation cycle, allowing the Cortex to generate increasingly accurate decision logic over time.

### 14.1 Syntax

```aid
evolve <reason_block_name> {
    track: <bool>                   // enable/disable telemetry logging
    retrain_every: <int>            // re-optimize after N invocations
    min_accuracy: <float>           // alert if accuracy drops below this (0.0 - 1.0)
    storage: "<path>"               // telemetry storage location (default: ".cortex/telemetry")
    approve: <bool>                 // require human approval before applying changes (default: false)
}
```

### 14.2 Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `track` | ✅ Yes | — | Enables runtime logging of inputs, outputs, and correctness signals |
| `retrain_every` | ❌ No | `1000` | Number of invocations before the Cortex re-analyzes and proposes new logic |
| `min_accuracy` | ❌ No | `0.90` | Minimum acceptable accuracy. Below this, the build emits a warning. |
| `storage` | ❌ No | `".cortex/telemetry"` | Path to store telemetry data. Supports `"local"` (filesystem) or a custom path. |
| `approve` | ❌ No | `false` | When `true`, the build pauses before applying Cortex-proposed changes and requires explicit developer approval. |

### 14.3 Complete Example

```aid
reason classify_ticket(text: string) -> string {
    goal: "Classify a customer support ticket into a category"
    constraints: [
        "Return one of: billing, technical, general, urgent",
        "If uncertain, return 'general'",
        "Tickets mentioning 'outage' or 'down' are always 'urgent'"
    ]
    examples: [
        ("My credit card was charged twice", "billing"),
        ("The server is completely down", "urgent"),
        ("How do I reset my password?", "technical"),
        ("I have a question about your product", "general")
    ]
    fallback: "general"
}

evolve classify_ticket {
    track: true
    retrain_every: 500
    min_accuracy: 0.95
    approve: true
}
```

### 14.4 The Evolution Lifecycle

```
              ┌──────────────────────────┐
              │     First Build (v1)     │
              │                          │
              │  Cortex generates logic  │
              │  from goal + constraints │
              │  + examples              │
              └────────────┬─────────────┘
                           │
                           ▼
              ┌──────────────────────────┐
              │     Production Run       │
              │                          │
              │  Runtime logs:           │
              │  (input, output, ✓/✗)    │
              │  every invocation        │
              └────────────┬─────────────┘
                           │  after N calls
                           ▼
              ┌──────────────────────────┐
              │     Next Build (v2)      │
              │                          │
              │  Cortex reads telemetry  │
              │  Identifies patterns     │
              │  Proposes new logic      │
              │                          │
              │  [approve: true]         │
              │  → Developer reviews     │
              │  → Accepts or rejects    │
              └────────────┬─────────────┘
                           │
                           ▼
              ┌──────────────────────────┐
              │  Updated Binary (v2)     │
              │                          │
              │  More patterns           │
              │  Higher accuracy         │
              │  Better edge case        │
              │  handling                │
              └──────────────────────────┘
```

### 14.5 Correctness Signals

The Cortex needs to know whether a decision was correct. AID provides three mechanisms:

**1. Explicit feedback (recommended for critical paths):**

```aid
result := classify_ticket(ticket.text)
// Later, when a human reviews:
cortex.feedback(classify_ticket, ticket.text, result, correct: true)
```

**2. Implicit feedback from downstream behavior:**

If a classified ticket is immediately reclassified by a human agent, the Cortex infers the original classification was wrong.

**3. Statistical inference:**

When no explicit feedback is available, the Cortex uses output distribution analysis to detect drift and anomalies.

### 14.6 Approval Workflow

When `approve: true`, the build process pauses after the Cortex proposes changes:

```
$ aid build

✓ Parsing complete
✓ Type checking complete
⚡ Cortex evolution detected for 'classify_ticket'

  PROPOSED CHANGES (v1 → v2):
  ────────────────────────────
  + New pattern: "invoice" → billing (was: general)
    Source: 847 samples, 94.2% correlation
  
  + New pattern: "can't login" → technical (was: general)
    Source: 231 samples, 89.7% correlation
  
  ~ Adjusted threshold: "urgent" confidence raised from 0.70 → 0.82
    Source: 12 false positives in 500 samples
  
  Projected accuracy: 91.3% → 96.8%
  
  Accept changes? [y/n/diff]:
```

### 14.7 Rollback

Every evolution version is stored. Roll back with:

```bash
aid rollback classify_ticket           # reverts to previous version
aid rollback classify_ticket --to v1   # reverts to specific version
```

### 14.8 Evolution History

The auto-documentation includes a full evolution log:

```markdown
## classify_ticket — Evolution History

| Version | Date       | Patterns | Samples | Accuracy | Status   |
|---------|------------|----------|---------|----------|----------|
| v1      | 2026-02-18 | 3        | 4       | 87.2%    | archived |
| v2      | 2026-02-20 | 8        | 1,247   | 94.1%    | archived |
| v3      | 2026-02-25 | 12       | 3,891   | 97.3%    | active ✓ |
```

---

## 15. Intent Routing

Intent routing is an AI-native approach to HTTP request routing. Instead of manually mapping URL patterns to handlers, the Cortex analyzes your handler functions and builds an optimized routing table at compile time.

### 15.1 Syntax

```aid
server.intent(<base_path>) => fn(req) -> Response { ... }
```

### 15.2 How It Works

1. **Discovery:** The Cortex scans all functions in scope that match the handler signature pattern
2. **Analysis:** It examines function names, parameter types, entity types, and any `reason` blocks to understand each handler's purpose
3. **Routing table:** At compile time, it generates a deterministic routing table mapping request characteristics (method, path, body shape, headers) to handlers
4. **Explicit priority:** Routes declared with `server.get()`, `server.post()`, etc. always take priority over intent routes

### 15.3 Example

```aid
module main

use std.http
use models.{ User, Ticket, TicketFilter }

fn main() {
    server := http.new(port: 8080)

    // Explicit routes — these always win
    server.get("/health") => fn(req) -> Response {
        Response.json({ status: "ok" })
    }

    // Intent routing for /api/*
    server.intent("/api") => fn(req) -> Response {
        // Cortex routes to the appropriate handler
    }

    server.start()
}

// ── Handlers (auto-discovered by Cortex) ────────────

fn create_user(user: User) -> result<User, ApiError> {
    // Cortex maps: POST /api/users → create_user
    // Inferred from: function name + User parameter type
}

fn get_user(id: int) -> option<User> {
    // Cortex maps: GET /api/users/:id → get_user
    // Inferred from: function name + int parameter (path param)
}

fn list_tickets(filter: TicketFilter) -> array<Ticket> {
    // Cortex maps: GET /api/tickets?... → list_tickets
    // Inferred from: function name + filter parameter (query params)
}

fn update_ticket(id: int, changes: TicketPatch) -> result<Ticket, ApiError> {
    // Cortex maps: PATCH /api/tickets/:id → update_ticket
    // Inferred from: function name + id param + patch entity
}
```

### 15.4 Generated Routing Table (Visible in Auto-Docs)

```markdown
## API Routes (auto-generated)

| Method | Path               | Handler          | Source         |
|--------|--------------------|------------------|----------------|
| GET    | /health            | (inline)         | explicit       |
| POST   | /api/users         | create_user      | intent/cortex  |
| GET    | /api/users/:id     | get_user         | intent/cortex  |
| GET    | /api/tickets       | list_tickets     | intent/cortex  |
| PATCH  | /api/tickets/:id   | update_ticket    | intent/cortex  |
```

### 15.5 Conflict Resolution

If the Cortex cannot determine a unique handler for a request pattern, the build fails with a diagnostic:

```
BUILD ERROR: intent routing conflict at '/api'
  Both 'get_user' and 'find_user' match pattern GET /api/users/:id
  Suggestion: Rename one function, or use explicit routing for this path.
```

### 15.6 Fallback

When no handler matches at runtime:

```aid
server.intent("/api") => fn(req) -> Response {
    // If Cortex can't match, this body executes
    Response.json({ error: "No handler found for " + req.path }, status: 404)
}
```

### 15.7 When to Use Intent Routing

| Scenario | Recommendation |
|----------|----------------|
| CRUD APIs with predictable patterns | ✅ Intent routing saves boilerplate |
| Performance-critical paths (< 1ms) | ❌ Use explicit routes |
| Complex middleware chains | ❌ Use explicit routes with middleware |
| Rapid prototyping | ✅ Intent routing accelerates development |
| Public APIs with strict contracts | ⚠️ Use with `contract` for validation |

---

## 16. Contracts

Contracts combine interfaces with AI-generated validation. They allow developers to describe validation rules in natural language, and the Cortex generates type-safe, efficient validation code at compile time.

### 16.1 Syntax

```aid
contract <Name> {
    "<validation rule in natural language>"
    "<another rule>"
    ...

    fn <method>(<params>) -> <return_type>
    fn <method>(<params>) -> <return_type>
    ...
}
```

### 16.2 Fields

A contract contains two types of declarations:

1. **Rules** — Quoted strings describing validation requirements in natural language
2. **Methods** — Function signatures that the contract enforces

### 16.3 Example

```aid
contract UserAPI {
    "User ID must be a positive integer"
    "Email must contain exactly one @ symbol followed by a valid domain"
    "Name must be between 2 and 100 characters"
    "Name must not contain HTML tags or script injections"
    "Role must be one of: admin, editor, viewer"
    "If role is admin, email domain must be @company.com"

    fn create(user: User) -> result<User, ValidationError>
    fn get(id: int) -> option<User>
    fn update(id: int, changes: UserPatch) -> result<User, ValidationError>
    fn delete(id: int) -> result<bool, ValidationError>
}
```

### 16.4 How the Cortex Processes Contracts

1. **Parse rules:** Each quoted string is analyzed as a validation requirement
2. **Map to types:** The Cortex identifies which fields of which entities each rule applies to
3. **Generate validators:** For each rule, the Cortex generates a type-safe validation function
4. **Wrap methods:** Every method in the contract is wrapped with automatic validation of its inputs

### 16.5 Generated Code (Visible in Auto-Docs)

The developer never writes this, but it's visible in the generated documentation for transparency:

```aid
// AUTO-GENERATED by Cortex — do not edit
private fn __validate_user_for_create(user: User) -> result<(), ValidationError> {
    if user.id <= 0 {
        return Err(ValidationError {
            field: "id",
            rule: "User ID must be a positive integer",
            value: user.id.to_string()
        })
    }
    if !is_valid_email(user.email) {
        return Err(ValidationError {
            field: "email",
            rule: "Email must contain exactly one @ symbol followed by a valid domain",
            value: user.email
        })
    }
    if user.name.length() < 2 || user.name.length() > 100 {
        return Err(ValidationError {
            field: "name",
            rule: "Name must be between 2 and 100 characters",
            value: user.name
        })
    }
    if contains_html(user.name) {
        return Err(ValidationError {
            field: "name",
            rule: "Name must not contain HTML tags or script injections",
            value: user.name
        })
    }
    if !["admin", "editor", "viewer"].contains(user.role) {
        return Err(ValidationError {
            field: "role",
            rule: "Role must be one of: admin, editor, viewer",
            value: user.role
        })
    }
    if user.role == "admin" && !user.email.ends_with("@company.com") {
        return Err(ValidationError {
            field: "email",
            rule: "If role is admin, email domain must be @company.com",
            value: user.email
        })
    }
    return Ok(())
}
```

### 16.6 Implementing a Contract

```aid
module handlers.users

use contracts.UserAPI

// Implement the contract
implement UserAPI {
    fn create(user: User) -> result<User, ValidationError> {
        // Validation runs automatically before this code executes
        saved := try db.insert(user)
        return Ok(saved)
    }

    fn get(id: int) -> option<User> {
        // ID validation runs automatically (must be positive)
        return db.find_by_id(id)
    }

    fn update(id: int, changes: UserPatch) -> result<User, ValidationError> {
        // Both id and changes are validated
        user := try db.update(id, changes)
        return Ok(user)
    }

    fn delete(id: int) -> result<bool, ValidationError> {
        return db.delete(id)
    }
}
```

### 16.7 ValidationError Entity

AID provides a built-in `ValidationError` entity:

```aid
entity ValidationError {
    field: string           // which field failed
    rule: string            // the natural language rule that was violated
    value: string           // the offending value (as string)
    code: string = ""       // optional machine-readable code
}
```

### 16.8 Contracts and Auto-Docs

Contracts produce particularly rich documentation because the rules are already in natural language:

```markdown
## UserAPI Contract

### Validation Rules
1. User ID must be a positive integer
2. Email must contain exactly one @ symbol followed by a valid domain
3. Name must be between 2 and 100 characters
4. Name must not contain HTML tags or script injections
5. Role must be one of: admin, editor, viewer
6. If role is admin, email domain must be @company.com

### Methods
| Method | Parameters | Returns | Validations Applied |
|--------|-----------|---------|---------------------|
| create | user: User | result<User, ValidationError> | Rules 1-6 |
| get    | id: int    | option<User>                  | Rule 1     |
| update | id: int, changes: UserPatch | result<User, ValidationError> | Rules 1-6 |
| delete | id: int    | result<bool, ValidationError> | Rule 1     |
```

---

## 17. HTTP Server

AID is an **API-first language**. HTTP server capabilities are built into the standard library as first-class constructs, not third-party dependencies.

### 17.1 Creating a Server

```aid
use std.http

server := http.new(port: 8080)
server := http.new(port: 443, host: "0.0.0.0", tls: true)
```

### 17.2 Route Registration

```aid
server.get("/path")    => fn(req) -> Response { ... }
server.post("/path")   => fn(req) -> Response { ... }
server.put("/path")    => fn(req) -> Response { ... }
server.patch("/path")  => fn(req) -> Response { ... }
server.delete("/path") => fn(req) -> Response { ... }
```

### 17.3 Path Parameters

```aid
server.get("/users/:id") => fn(req) -> Response {
    id := req.param("id")      // string
    id := req.param_int("id")  // int (returns result)
    // ...
}

server.get("/files/*path") => fn(req) -> Response {
    path := req.param("path")  // captures everything after /files/
    // ...
}
```

### 17.4 Request Object

```aid
entity Request {
    method: string                     // "GET", "POST", etc.
    path: string                       // "/users/42"
    headers: map<string, string>       // request headers
    query: map<string, string>         // query parameters
    body: Body                         // request body

    fn param(name: string) -> string
    fn param_int(name: string) -> result<int, ParseError>
    fn header(name: string) -> option<string>
    fn has_auth() -> bool
}

entity Body {
    raw: string                        // raw body text
    fn json<T>() -> result<T, ParseError>
    fn text() -> string
    fn bytes() -> array<byte>
}
```

### 17.5 Response Object

```aid
entity Response {
    status: int = 200
    headers: map<string, string> = {}
    body: string = ""
}

// Factory methods
Response.json(data)                           // 200 + JSON body
Response.json(data, status: 201)              // custom status
Response.text("Hello")                        // 200 + plain text
Response.error(message: "Not found", status: 404)
Response.redirect(url: "/login")              // 302 redirect
Response.empty(status: 204)                   // no body
```

### 17.6 Middleware

```aid
fn logger(next: fn(Request) -> Response) -> fn(Request) -> Response {
    return fn(req) -> Response {
        start := time.now()
        response := next(req)
        duration := time.since(start)
        log(req.method + " " + req.path + " → " + response.status.to_string() + " (" + duration + ")")
        return response
    }
}

fn cors(next: fn(Request) -> Response) -> fn(Request) -> Response {
    return fn(req) -> Response {
        response := next(req)
        response.headers["Access-Control-Allow-Origin"] = "*"
        return response
    }
}

// Apply middleware
server.use(logger)
server.use(cors)
```

### 17.7 Grouping Routes

```aid
api := server.group("/api/v1")
api.use(auth_middleware)

api.get("/users")      => list_users
api.post("/users")     => create_user
api.get("/users/:id")  => get_user
```

---

## 18. Auto-Documentation

Every AID build generates comprehensive documentation. This is not optional — documentation is a compiler output, as fundamental as the binary itself.

### 18.1 What Is Generated

| Section | Source | Contents |
|---------|--------|----------|
| **API Reference** | Public functions, entities | Signatures, parameters, return types, descriptions |
| **Route Map** | HTTP routes + intent routing | Full URL → handler mapping with methods |
| **Reason Logic** | `reason` blocks | Goal, constraints, decision logic explanation, confidence |
| **Evolution Log** | `evolve` blocks | Version history, accuracy trends, pattern changes |
| **Contract Rules** | `contract` blocks | Validation rules, generated code, field mappings |
| **Entity Schema** | Entity definitions | Fields, types, defaults, relationships |
| **Dependency Graph** | Module imports | Visual module relationship map |
| **Error Catalog** | Custom error types | All error types, codes, and descriptions |

### 18.2 Configuration

```toml
[build]
docs = true                         # enable/disable (default: true)
docs_format = "markdown"            # "markdown" | "html" | "both"
docs_output = "build/docs"          # output directory
docs_include_private = false        # include private functions
docs_include_generated = true       # include Cortex-generated code
```

### 18.3 Doc Comments

AID supports doc comments that are included in generated documentation:

```aid
/// Creates a new user account.
///
/// Validates the user data against the UserAPI contract,
/// then persists to the database.
///
/// Returns the created user with a generated ID.
fn create_user(user: User) -> result<User, ValidationError> {
    // ...
}
```

### 18.4 Output Structure

```
build/docs/
├── index.md                    # Overview + table of contents
├── api/
│   ├── routes.md               # Full route map
│   ├── handlers/
│   │   ├── auth.md
│   │   └── users.md
│   └── models/
│       ├── user.md
│       └── ticket.md
├── reasoning/
│   ├── classify_ticket.md      # Reason block documentation
│   └── evolution-log.md        # Full evolution history
├── contracts/
│   └── user-api.md             # Contract rules + generated validators
└── dependencies.md             # Module dependency graph
```

---

## 19. Compilation & Build Pipeline

### 19.1 Build Command

```bash
aid build                          # compile + docs
aid build --release                # optimized release build
aid build --no-docs                # skip documentation
aid build --verbose                # detailed Cortex output
```

### 19.2 Build Stages

```
Stage 1: Parse           → AST
Stage 2: Type Check      → Typed AST
Stage 3: Cortex Process  → Enriched AST (reason/evolve/intent/contract)
Stage 4: Transpile       → Rust source
Stage 5: Compile         → WASM binary
Stage 6: Document        → Markdown/HTML docs
Stage 7: Package         → Distributable artifact
```

### 19.3 Build Output

```
$ aid build

  AID Compiler v1.0.0
  ────────────────────
  ✓ Parse          12 files, 0 errors
  ✓ Type Check     847 symbols resolved
  ⚡ Cortex         3 reason blocks, 1 evolve, 1 intent, 1 contract
  ✓ Transpile      Generated 2,341 lines of Rust
  ✓ Compile        build/output.wasm (148 KB)
  ✓ Docs           build/docs/ (14 files)
  
  Build complete in 3.2s
```

### 19.4 Run Command

```bash
aid run                            # build + execute
aid run --watch                    # rebuild on file changes
aid run --port 3000                # override port
```

### 19.5 Test Command

```bash
aid test                           # run all tests
aid test --reason                  # test only reason blocks
aid test --coverage                # with coverage report
```

---

## 20. Standard Library

AID ships with a focused standard library designed for API development.

### 20.1 Core Modules

| Module | Description |
|--------|-------------|
| `std.http` | HTTP server and client |
| `std.json` | JSON parsing and serialization |
| `std.io` | File I/O, stdin/stdout |
| `std.time` | Time, duration, formatting |
| `std.crypto` | Hashing, encryption, JWT |
| `std.log` | Structured logging |
| `std.env` | Environment variables |
| `std.fmt` | String formatting |
| `std.math` | Mathematical operations |
| `std.collections` | Advanced data structures (set, queue, deque) |
| `std.regex` | Regular expressions |
| `std.uuid` | UUID generation |
| `std.base64` | Base64 encoding/decoding |

### 20.2 Database (Planned)

```aid
use std.db

conn := db.connect("postgres://localhost/mydb")
users := try conn.query<User>("SELECT * FROM users WHERE role = $1", ["admin"])
```

### 20.3 WebSocket (Planned)

```aid
use std.ws

server.ws("/live") => fn(socket) {
    for msg in socket.messages() {
        socket.send("Echo: " + msg)
    }
}
```

---

## 21. CLI Reference

| Command | Description |
|---------|-------------|
| `aid new <name>` | Create a new project |
| `aid build` | Compile the project |
| `aid build --release` | Optimized release build |
| `aid run` | Build and execute |
| `aid run --watch` | Build, execute, and watch for changes |
| `aid test` | Run tests |
| `aid test --reason` | Test reason blocks specifically |
| `aid docs` | Generate documentation only |
| `aid docs --serve` | Generate and serve docs locally |
| `aid rollback <name>` | Revert an evolved reason block |
| `aid rollback <name> --to <version>` | Revert to a specific version |
| `aid cortex status` | Show Cortex engine status and model info |
| `aid cortex test <block>` | Test a specific reason block interactively |
| `aid evolve status` | Show evolution status for all tracked blocks |
| `aid evolve history <block>` | Show evolution history for a specific block |
| `aid clean` | Remove build artifacts |
| `aid fmt` | Format source code |
| `aid lint` | Run linter |

---

## 22. Complete Example — Ticket API

A full, working example that demonstrates all major AID features together.

```aid
// src/main.aid
module main

use std.http
use std.json
use std.log
use models.{ Ticket, TicketFilter }
use handlers.tickets

fn main() {
    log.info("Starting Ticket API server")

    server := http.new(port: 8080)
    server.use(request_logger)

    // Explicit critical routes
    server.get("/health") => fn(req) -> Response {
        Response.json({ status: "ok", version: "1.0.0" })
    }

    // Intent routing for the API
    server.intent("/api/v1") => fn(req) -> Response {
        Response.error(message: "Route not found", status: 404)
    }

    server.start()
}

private fn request_logger(next: fn(Request) -> Response) -> fn(Request) -> Response {
    return fn(req) -> Response {
        start := time.now()
        response := next(req)
        log.info(req.method + " " + req.path + " " + response.status.to_string() + " " + time.since(start))
        return response
    }
}
```

```aid
// src/models/ticket.aid
module models

entity Ticket {
    id: int
    title: string
    description: string
    category: string = ""
    priority: string = "normal"
    status: string = "open"
    created_at: string = now()
}

entity TicketFilter {
    category: option<string> = None
    priority: option<string> = None
    status: option<string> = None
    limit: int = 50
}

entity TicketPatch {
    title: option<string> = None
    description: option<string> = None
    priority: option<string> = None
    status: option<string> = None
}
```

```aid
// src/contracts/ticket_contract.aid
module contracts

use models.{ Ticket, TicketPatch, TicketFilter }

contract TicketAPI {
    "Ticket ID must be a positive integer"
    "Title must be between 5 and 200 characters"
    "Description must not exceed 5000 characters"
    "Priority must be one of: low, normal, high, critical"
    "Status must be one of: open, in_progress, resolved, closed"
    "Category is assigned automatically and must not be set manually on create"
    "Filter limit must be between 1 and 100"

    fn create_ticket(ticket: Ticket) -> result<Ticket, ValidationError>
    fn get_ticket(id: int) -> option<Ticket>
    fn list_tickets(filter: TicketFilter) -> array<Ticket>
    fn update_ticket(id: int, changes: TicketPatch) -> result<Ticket, ValidationError>
}
```

```aid
// src/handlers/tickets.aid
module handlers.tickets

use std.log
use models.{ Ticket, TicketFilter, TicketPatch }
use contracts.TicketAPI
use reasoning.classifier.classify_ticket

implement TicketAPI {
    fn create_ticket(ticket: Ticket) -> result<Ticket, ValidationError> {
        // Category is auto-assigned by the reason block
        mut new_ticket := ticket
        new_ticket.category = classify_ticket(ticket.title + " " + ticket.description)
        
        saved := try db.insert(new_ticket)
        log.info("Created ticket #" + saved.id.to_string() + " [" + saved.category + "]")
        return Ok(saved)
    }

    fn get_ticket(id: int) -> option<Ticket> {
        return db.find_by_id<Ticket>(id)
    }

    fn list_tickets(filter: TicketFilter) -> array<Ticket> {
        return db.query<Ticket>(filter)
    }

    fn update_ticket(id: int, changes: TicketPatch) -> result<Ticket, ValidationError> {
        updated := try db.update<Ticket>(id, changes)
        return Ok(updated)
    }
}
```

```aid
// src/reasoning/classifier.aid
module reasoning.classifier

reason classify_ticket(text: string) -> string {
    goal: "Classify a customer support ticket into the most appropriate category"
    constraints: [
        "Return exactly one of: billing, technical, account, general, urgent",
        "Tickets mentioning payment, charge, invoice, or refund → billing",
        "Tickets mentioning crash, error, bug, or down → technical",
        "Tickets mentioning password, login, or access → account",
        "Tickets mentioning outage, emergency, or data loss → urgent",
        "If no clear category, return general"
    ]
    examples: [
        ("I was double charged on my last invoice", "billing"),
        ("The API returns a 500 error on POST requests", "technical"),
        ("I can't log into my account", "account"),
        ("All our data seems to be missing since the update", "urgent"),
        ("What integrations do you support?", "general"),
        ("Need a refund for last month", "billing"),
        ("App crashes when uploading files over 10MB", "technical")
    ]
    fallback: "general"
}

evolve classify_ticket {
    track: true
    retrain_every: 500
    min_accuracy: 0.95
    approve: true
}
```

### Build Output

```
$ aid build

  AID Compiler v1.0.0
  ────────────────────
  ✓ Parse          5 files, 0 errors
  ✓ Type Check     42 symbols resolved
  ⚡ Cortex
    • reason 'classify_ticket': 5 categories, 7 examples → 91.4% confidence (static)
    • evolve 'classify_ticket': no prior telemetry (first build)
    • intent '/api/v1': 4 handlers mapped
    • contract 'TicketAPI': 7 rules → 12 validators generated
  ✓ Transpile      Generated 1,847 lines of Rust
  ✓ Compile        build/output.wasm (112 KB)
  ✓ Docs           build/docs/ (11 files)

  Build complete in 2.8s
```

---

## Appendix A — Reserved Keywords

```
async       await       bool        break       byte
const       continue    contract    dynamic     else
entity      evolve      false       float       fn
for         goal        hybrid      if          implement
import      in          int         intent      match
module      mut         None        option      private
reason      result      return      Some        static
stream      string      struct      test        true
try         type        use         while
```

---

## Appendix B — Operator Precedence

| Precedence | Operator | Description | Associativity |
|------------|----------|-------------|---------------|
| 1 (highest) | `()` `.` `[]` | Call, member access, index | Left |
| 2 | `!` `-` (unary) | Logical NOT, negation | Right |
| 3 | `*` `/` `%` | Multiply, divide, modulo | Left |
| 4 | `+` `-` | Add, subtract | Left |
| 5 | `..` | Range | Left |
| 6 | `==` `!=` `<` `>` `<=` `>=` | Comparison | Left |
| 7 | `&&` | Logical AND | Left |
| 8 | `\|\|` | Logical OR | Left |
| 9 | `:=` `=` | Assignment | Right |
| 10 (lowest) | `=>` | Arrow (match, lambda) | Right |

---

## Appendix C — Grammar (EBNF)

```ebnf
program        = module_decl { import } { declaration } ;
module_decl    = "module" qualified_name ;
import         = "use" qualified_name [ ".{" ident_list "}" | ".*" ] ;

declaration    = entity_decl
               | function_decl
               | reason_decl
               | evolve_decl
               | contract_decl
               | implement_decl
               | const_decl
               | type_alias ;

entity_decl    = "entity" IDENT "{" { field_decl } { method_decl } "}" ;
field_decl     = IDENT ":" type [ "=" expression ] ;
method_decl    = ["private"] "fn" IDENT "(" param_list ")" "->" type block ;

function_decl  = ["private"] ["async"] "fn" IDENT "(" param_list ")" "->" type ( block | "=>" expression ) ;
param_list     = [ param { "," param } ] ;
param          = IDENT ":" type [ "=" expression ] ;

reason_decl    = "reason" [ "(" reason_mode ")" ] IDENT "(" param_list ")" "->" type "{" reason_body "}" ;
reason_mode    = "static" | "dynamic" ;
reason_body    = "goal:" STRING
                 "constraints:" "[" string_list "]"
                 [ "context:" "[" expr_list "]" ]
                 [ "examples:" "[" example_list "]" ]
                 [ "fallback:" expression ] ;
example_list   = { "(" expression "," expression ")" "," } ;

evolve_decl    = "evolve" IDENT "{" { evolve_field } "}" ;
evolve_field   = IDENT ":" expression ;

contract_decl  = "contract" IDENT "{" { STRING } { fn_signature } "}" ;
fn_signature   = "fn" IDENT "(" param_list ")" "->" type ;

implement_decl = "implement" IDENT "{" { function_decl } "}" ;

const_decl     = "const" IDENT "=" expression ;
type_alias     = "type" IDENT "=" type ;

type           = scalar_type | composite_type | IDENT ;
scalar_type    = "int" | "float" | "bool" | "string" | "byte" ;
composite_type = "array" "<" type ">"
               | "map" "<" type "," type ">"
               | "option" "<" type ">"
               | "result" "<" type "," type ">"
               | "stream" "<" type ">"
               | "fn" "(" type_list ")" "->" type ;

block          = "{" { statement } "}" ;
statement      = var_decl | assignment | expression | return_stmt | if_stmt | match_stmt | for_stmt | while_stmt ;
var_decl       = ["mut"] IDENT ":=" expression
               | ["mut"] IDENT ":" type "=" expression ;
assignment     = lvalue "=" expression ;
return_stmt    = "return" expression ;
if_stmt        = "if" expression block [ "else" ( if_stmt | block ) ] ;
match_stmt     = "match" expression "{" { match_arm } "}" ;
match_arm      = pattern "=>" ( expression | block ) ;
for_stmt       = "for" pattern "in" expression block ;
while_stmt     = "while" expression block ;

expression     = literal | IDENT | call | member | index | binary | unary | lambda | try_expr ;
try_expr       = "try" expression ;
lambda         = "fn" "(" param_list ")" [ "->" type ] "=>" expression ;
```

---

*AID Language Documentation v1.0*
*Last updated: 2026-02-19*

*© AID Project*
