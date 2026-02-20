# AID Package Specification

**Version 0.2.0** — Package Format, Registry & Dependency Resolution

> This is a design document. Implementation will follow in subsequent releases.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Package Format (`aid.toml`)](#2-package-format-aidtoml)
3. [Package Structure](#3-package-structure)
4. [Versioning](#4-versioning)
5. [Registry Design](#5-registry-design)
6. [Dependency Resolution](#6-dependency-resolution)
7. [Lock File (`aid.lock`)](#7-lock-file-aidlock)
8. [CLI Commands](#8-cli-commands)
9. [Standard Library Packages](#9-standard-library-packages)
10. [Security](#10-security)
11. [Design Decisions](#11-design-decisions)

---

## 1. Overview

AID packages are distributable units of AID code — libraries, tools, and applications. The package system is designed around three principles:

- **Simplicity** — A single `aid.toml` manifest, inspired by Cargo.toml
- **Security** — Checksums, signatures, and local-first architecture
- **Compatibility** — Git-based packages (like Go modules) alongside a central registry

Every AID project is a package. The `aid.toml` file at the project root defines its identity, dependencies, and build configuration.

---

## 2. Package Format (`aid.toml`)

### 2.1 Full Manifest Example

```toml
[package]
name = "mycompany/api-toolkit"
version = "1.2.0"
description = "HTTP utilities and middleware for AID APIs"
author = "Danilo Smaldone <danilo@example.com>"
license = "MIT"
repository = "https://github.com/mycompany/api-toolkit"
homepage = "https://api-toolkit.example.com"
keywords = ["http", "middleware", "api", "utilities"]
edition = "2026"
readme = "README.md"
entry = "src/lib.aid"

[dependencies]
"std/json" = "^1.0"
"community/redis" = "~2.3"
"community/logger" = ">=1.0, <3.0"
"github.com/user/cool-lib" = { git = "https://github.com/user/cool-lib.git", tag = "v0.5.0" }

[dev-dependencies]
"std/test" = "^1.0"
"community/mock-http" = "0.9.2"

[build]
target = "native"               # "native" | "wasm" | "both"
features = ["tls", "compression"]
optimize = true
docs = true

[cortex]
mode = "hybrid"
confidence = 0.85

[features]
default = ["tls"]
tls = []
compression = []
full = ["tls", "compression"]
```

### 2.2 Field Reference

#### `[package]` — Required

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `name` | ✅ | string | Package name in `namespace/package` format |
| `version` | ✅ | string | Semantic version (e.g., `"1.2.0"`) |
| `description` | ❌ | string | Short description (max 256 chars) |
| `author` | ❌ | string | Author name and optional email |
| `license` | ❌ | string | SPDX license identifier |
| `repository` | ❌ | string | Source code URL |
| `homepage` | ❌ | string | Project homepage URL |
| `keywords` | ❌ | array | Up to 5 keywords for search |
| `edition` | ❌ | string | AID language edition (default: latest) |
| `readme` | ❌ | string | Path to README file |
| `entry` | ❌ | string | Library entry point (default: `src/lib.aid`) |

#### `[dependencies]` — Optional

Dependencies are declared as `name = "version_constraint"` or as inline tables for git sources:

```toml
[dependencies]
"community/redis" = "^2.0"                                    # Registry package
"github.com/user/lib" = { git = "https://...", tag = "v1.0" } # Git package
"github.com/user/lib" = { git = "https://...", branch = "main" } # Git branch
"local/mylib" = { path = "../mylib" }                          # Local path
```

#### `[dev-dependencies]` — Optional

Same format as `[dependencies]`. Only installed during development and testing. Not included in published packages.

#### `[build]` — Optional

| Field | Default | Description |
|-------|---------|-------------|
| `target` | `"native"` | Compilation target: `native`, `wasm`, or `both` |
| `features` | `[]` | Features to enable |
| `optimize` | `true` | Enable release optimizations |
| `docs` | `true` | Generate documentation on build |

#### `[features]` — Optional

Feature flags for conditional compilation. Dependencies can be feature-gated:

```toml
[features]
default = ["json"]
json = []
xml = []
full = ["json", "xml"]

[dependencies]
"community/xml-parser" = { version = "^1.0", optional = true, features = ["xml"] }
```

---

## 3. Package Structure

### 3.1 Standard Layout

```
my-package/
├── aid.toml              # Package manifest (required)
├── aid.lock              # Dependency lock file (auto-generated)
├── README.md             # Package documentation
├── LICENSE.md            # License file
├── CHANGELOG.md          # Version history
├── src/
│   ├── lib.aid           # Library entry point (for libraries)
│   ├── main.aid          # Application entry point (for binaries)
│   └── models/
│       └── user.aid      # Sub-modules
├── examples/
│   └── basic.aid         # Example programs
├── tests/
│   ├── unit/
│   │   └── models_test.aid
│   └── integration/
│       └── api_test.aid
├── build/                # Compiler output (gitignored)
│   ├── output.wasm
│   └── docs/
├── .cortex/              # Cortex data (gitignored)
│   ├── telemetry/
│   └── cache/
└── .aidpackages/         # Installed dependencies (gitignored)
```

### 3.2 Library vs Application

A package is a **library** if it has `src/lib.aid`, and an **application** if it has `src/main.aid`. A package can be both (lib + binary).

- **Library packages** export entities, functions, reason blocks, and contracts for other packages to use.
- **Application packages** have a `fn main()` entry point and produce an executable binary.

### 3.3 Module Resolution

Modules map to files by path convention:

| Module declaration | File path |
|-------------------|-----------|
| `module main` | `src/main.aid` |
| `module lib` | `src/lib.aid` |
| `module models.user` | `src/models/user.aid` |
| `module handlers.auth` | `src/handlers/auth.aid` |

---

## 4. Versioning

### 4.1 Semantic Versioning

All AID packages follow [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH
```

- **MAJOR** — Breaking changes to the public API
- **MINOR** — New functionality, backward-compatible
- **PATCH** — Bug fixes, backward-compatible

Pre-release versions: `1.0.0-alpha.1`, `1.0.0-beta.2`, `1.0.0-rc.1`

Build metadata: `1.0.0+build.123`

### 4.2 Version Constraints

| Syntax | Meaning | Example | Matches |
|--------|---------|---------|---------|
| `"1.2.3"` | Exact version | `"1.2.3"` | Only 1.2.3 |
| `"^1.2"` | Compatible (same major) | `"^1.2"` | ≥1.2.0, <2.0.0 |
| `"^0.2"` | Compatible (same minor for 0.x) | `"^0.2"` | ≥0.2.0, <0.3.0 |
| `"~1.2"` | Approximate (same minor) | `"~1.2"` | ≥1.2.0, <1.3.0 |
| `">=1.0"` | Minimum version | `">=1.0"` | ≥1.0.0 |
| `">=1.0, <3.0"` | Range | `">=1.0, <3.0"` | ≥1.0.0 and <3.0.0 |
| `"*"` | Any version | `"*"` | Any |

**Default operator** — bare version `"1.2"` is treated as `"^1.2"` (caret / compatible).

### 4.3 Version Resolution Rules

1. **Latest compatible** — the resolver picks the newest version satisfying the constraint
2. **Minimal version selection** (optional flag) — pick the *oldest* version satisfying the constraint, for reproducibility
3. **Pre-release opt-in** — pre-release versions are only considered if explicitly requested (`">=1.0.0-alpha"`)

---

## 5. Registry Design

### 5.1 Architecture

AID supports two package sources:

| Source | Format | Example |
|--------|--------|---------|
| **Central registry** | `namespace/package` | `community/redis`, `std/json` |
| **Git-based** | `host/owner/repo` | `github.com/user/cool-lib` |

The central registry is **`registry.aidlang.dev`** (future). Git-based packages work immediately — no registry account needed.

### 5.2 Package Naming

```
namespace/package-name
```

**Namespaces:**

| Namespace | Owner | Description |
|-----------|-------|-------------|
| `std` | AID core team | Standard library extensions (bundled, no install) |
| `community` | Verified authors | Community-contributed packages |
| `<username>` | Individual | Personal namespace (e.g., `danilo/my-utils`) |
| `<org>` | Organization | Organization namespace (e.g., `telnyx/sip-client`) |

**Naming rules:**
- Lowercase alphanumeric + hyphens
- 2–64 characters
- Must start with a letter
- No reserved words (`std`, `aid`, `cortex`, `reason`, `evolve`)

### 5.3 Registry API

Base URL: `https://registry.aidlang.dev/api/v1`

#### Publish

```
PUT /packages/{namespace}/{name}
Authorization: Bearer <token>
Content-Type: application/octet-stream

Body: .tar.gz of package contents
```

Response:
```json
{
  "name": "community/redis",
  "version": "2.3.1",
  "checksum": "sha256:abc123...",
  "published_at": "2026-02-20T12:00:00Z"
}
```

#### Search

```
GET /search?q=redis&limit=20&offset=0
```

Response:
```json
{
  "results": [
    {
      "name": "community/redis",
      "version": "2.3.1",
      "description": "Redis client for AID",
      "downloads": 12450,
      "updated_at": "2026-02-20T12:00:00Z"
    }
  ],
  "total": 1
}
```

#### Download

```
GET /packages/{namespace}/{name}/{version}
```

Returns: `.tar.gz` archive with `Content-Disposition` header.

#### Package Info

```
GET /packages/{namespace}/{name}
```

Response:
```json
{
  "name": "community/redis",
  "description": "Redis client for AID",
  "author": "Jane Doe",
  "license": "MIT",
  "versions": ["2.3.1", "2.3.0", "2.2.0", "2.1.0", "2.0.0"],
  "latest": "2.3.1",
  "repository": "https://github.com/janedoe/aid-redis",
  "downloads_total": 45230,
  "created_at": "2026-01-15T10:00:00Z"
}
```

#### Version Info

```
GET /packages/{namespace}/{name}/{version}/info
```

Response:
```json
{
  "version": "2.3.1",
  "checksum": "sha256:abc123...",
  "signature": "ed25519:xyz789...",
  "dependencies": {
    "std/json": "^1.0"
  },
  "size_bytes": 24576,
  "aid_edition": "2026",
  "published_at": "2026-02-20T12:00:00Z"
}
```

### 5.4 Git-Based Packages

For packages hosted on Git (no registry account required):

```toml
[dependencies]
"github.com/user/my-lib" = { git = "https://github.com/user/my-lib.git", tag = "v1.0.0" }
```

Resolution:
1. Clone/fetch the repository
2. Checkout the specified tag/branch/commit
3. Read `aid.toml` from the repo root
4. Cache in `.aidpackages/git/`

Supported hosts: any Git URL (GitHub, GitLab, Bitbucket, self-hosted).

---

## 6. Dependency Resolution

### 6.1 Algorithm

AID uses a **greedy version resolution** algorithm with backtracking, inspired by PubGrub (used by Dart/pub):

1. **Build dependency graph** — read `aid.toml` from the root package and all transitive dependencies
2. **Select versions** — for each dependency, select the newest version satisfying all constraints
3. **Detect conflicts** — if two packages require incompatible versions of the same dependency, backtrack
4. **Report or resolve** — if resolution succeeds, write `aid.lock`; if it fails, report the conflict with actionable diagnostics

### 6.2 Conflict Resolution Strategy

When two dependencies require incompatible versions of the same package:

```
Package A requires "community/json" ^1.0
Package B requires "community/json" ^2.0
```

**Resolution order:**
1. **Check compatibility** — if both ranges have overlap, pick from the overlap
2. **Suggest upgrade** — if one package has a newer version that relaxes the constraint, suggest it
3. **Fail with diagnostic** — show the full dependency chain and conflict

```
DEPENDENCY ERROR: version conflict for "community/json"

  "community/redis" v2.3.1 requires "community/json" ^1.0
    via: my-package → community/redis → community/json
  
  "community/graphql" v3.0.0 requires "community/json" ^2.0
    via: my-package → community/graphql → community/json

  Suggestion: "community/redis" v2.4.0 supports "community/json" ^2.0
              Run: aid update community/redis
```

### 6.3 Duplicate Handling

AID does **not** allow multiple versions of the same package in one build (unlike npm). One version per dependency, resolved globally. This ensures:

- Smaller binaries
- No type incompatibility between versions
- Predictable behavior

---

## 7. Lock File (`aid.lock`)

The lock file records the exact versions and checksums of all resolved dependencies. It is auto-generated by `aid install` and should be committed to version control.

### 7.1 Format

```toml
# This file is auto-generated by aid. Do not edit manually.
# aid.lock v1

[[package]]
name = "community/redis"
version = "2.3.1"
source = "registry"
checksum = "sha256:a1b2c3d4e5f6..."
dependencies = ["std/json"]

[[package]]
name = "std/json"
version = "1.2.0"
source = "bundled"
checksum = "sha256:f6e5d4c3b2a1..."
dependencies = []

[[package]]
name = "github.com/user/cool-lib"
version = "0.5.0"
source = "git"
git = "https://github.com/user/cool-lib.git"
rev = "abc123def456"
checksum = "sha256:1a2b3c4d5e6f..."
dependencies = []
```

### 7.2 Lock File Behavior

| Command | Lock file behavior |
|---------|-------------------|
| `aid install` | Creates lock file if missing; installs from lock file if present |
| `aid update` | Re-resolves all dependencies, updates lock file |
| `aid update <pkg>` | Re-resolves only the specified package |
| `aid install <pkg>` | Adds to aid.toml and updates lock file |

---

## 8. CLI Commands

### 8.1 Package Management

```bash
# Install all dependencies from aid.toml (uses aid.lock if present)
aid install

# Add a dependency (updates aid.toml + aid.lock)
aid install community/redis
aid install community/redis@2.3
aid install community/redis@^2.0

# Add a dev dependency
aid install --dev community/mock-http

# Add a git dependency
aid install --git https://github.com/user/cool-lib.git --tag v1.0.0

# Remove a dependency
aid remove community/redis

# Update all dependencies to latest compatible versions
aid update

# Update a specific dependency
aid update community/redis

# List installed dependencies
aid list

# Show dependency tree
aid tree
```

### 8.2 Publishing

```bash
# Login to registry
aid login

# Publish current package to registry
aid publish

# Publish with dry run (validate only)
aid publish --dry-run

# Yank a version (mark as unsuitable, existing users unaffected)
aid yank community/my-package@1.2.3

# Un-yank
aid yank --undo community/my-package@1.2.3
```

### 8.3 Search & Discovery

```bash
# Search the registry
aid search redis

# Show package info
aid info community/redis

# Show specific version info
aid info community/redis@2.3.1
```

### 8.4 Project Initialization

```bash
# Create a new library package
aid init my-lib --lib

# Create a new application package
aid init my-app --bin

# Create in current directory
aid init . --lib
```

Generates:
```
my-lib/
├── aid.toml
├── src/
│   └── lib.aid        # or main.aid for --bin
├── tests/
│   └── lib_test.aid
└── README.md
```

---

## 9. Standard Library Packages

Standard library modules are bundled with the AID compiler and require no installation. They are always available via `use std.*`.

| Module | Import | Description |
|--------|--------|-------------|
| `std.http` | `use std.http` | HTTP server, client, request/response |
| `std.db` | `use std.db` | Database connectivity (SQLite, PostgreSQL*) |
| `std.env` | `use std.env` | Environment variables, .env files |
| `std.auth` | `use std.auth` | JWT, bcrypt, API keys, auth middleware |
| `std.html` | `use std.html` | HTML templates, static files |
| `std.json` | `use std.json` | JSON parsing and serialization |
| `std.crypto` | `use std.crypto` | Hashing, encryption, random |
| `std.fs` | `use std.fs` | File system operations |
| `std.time` | `use std.time` | Date, time, duration, timers |
| `std.log` | `use std.log` | Structured logging |
| `std.test` | `use std.test` | Testing framework |

\* *PostgreSQL support planned for future release.*

Standard library modules follow the same versioning as the compiler — they are always compatible with the installed AID version. They do **not** appear in `aid.toml` or `aid.lock`.

Community packages extend beyond the standard library:

```toml
[dependencies]
"community/redis" = "^2.0"        # Redis client
"community/graphql" = "^3.0"      # GraphQL server
"community/websocket" = "^1.0"    # WebSocket support
"community/smtp" = "^1.0"         # Email sending
"community/queue" = "^1.0"        # Message queues
```

---

## 10. Security

### 10.1 Checksum Verification

Every published package includes a SHA-256 checksum. On install:

1. Download the package archive
2. Compute SHA-256 of the archive
3. Compare against the checksum in the registry metadata
4. Compare against the checksum in `aid.lock` (if present)
5. Fail if any mismatch

### 10.2 Package Signing

Authors can sign packages with Ed25519 keys:

```bash
# Generate a signing key
aid key generate

# Publish with signature
aid publish --sign
```

Verification:
- The registry stores the author's public key
- On install, if a signature is present, it is verified against the registered public key
- `aid install --require-signatures` enforces that all dependencies must be signed

### 10.3 Supply Chain Safety

- **Yank, not delete** — published versions can be yanked (hidden from new installs) but never deleted. Existing lock files continue to work.
- **Immutable versions** — once published, a version cannot be overwritten. Publish a new version instead.
- **Audit command** — `aid audit` checks all dependencies against a vulnerability database
- **Lock file pinning** — committing `aid.lock` ensures reproducible builds

### 10.4 Cortex & Packages

Packages that include `reason` blocks run through the local Cortex during compilation — **no remote inference**. A package cannot execute arbitrary code during install (no build scripts unless explicitly enabled in `aid.toml`).

---

## 11. Design Decisions

### Why `aid.toml` instead of JSON/YAML?
TOML is human-readable, has clear section semantics, and is the standard for Rust (Cargo.toml), Python (pyproject.toml), and other modern tools. It avoids YAML's pitfalls and JSON's verbosity.

### Why namespace/package naming?
Prevents name squatting. Organizations own their namespace. `std/` is reserved for the standard library. Inspired by Go modules and npm scopes.

### Why no multiple versions of the same package?
Multiple versions (like npm's node_modules tree) cause type incompatibilities and binary bloat. One version per package, resolved globally, keeps things simple and predictable — aligned with Rust/Go's approach.

### Why PubGrub-style resolution?
Greedy + backtracking gives clear error messages when conflicts occur. SAT solvers are powerful but produce opaque errors. PubGrub is proven in Dart/pub and is being adopted by Cargo.

### Why lock files committed to version control?
Reproducible builds. CI/CD, teammates, and production all use the exact same dependency versions. `aid install` without a lock file resolves fresh; with a lock file, it installs exactly what's recorded.

### Why Ed25519 for signing?
Small keys, fast verification, quantum-resistant consideration in future. Same choice as minisign, signify, and SSH.

---

*This specification is part of AID v0.2.0. For the language documentation, see [AID-Language-Documentation.md](AID-Language-Documentation.md).*
