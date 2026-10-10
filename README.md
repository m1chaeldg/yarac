# Yarac (Yet Another REST API Client)

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Local-First](https://img.shields.io/badge/architecture-local--first-success.svg)](#core-philosophy)
[![Telemetry](https://img.shields.io/badge/telemetry-zero-brightgreen.svg)](#core-philosophy)

**Yarac** is a blazingly fast, low-memory (<100MB RAM runtime target), local-first API testing and execution client. It bridges developer-native Git workflows with enterprise-grade security and AI agent extensibility via the Model Context Protocol (MCP).

---

## Table of Contents

- [Core Philosophy](#core-philosophy)
- [Key Features](#key-features)
- [Architecture & Crates](#architecture--crates)
- [The `.yarac` File Format](#the-yarac-file-format)
- [CLI Usage (`yarac-cli`)](#cli-usage-yarac-cli)
  - [Headless Test Runner](#headless-test-runner)
  - [Deterministic Exit Codes](#deterministic-exit-codes)
  - [Model Context Protocol (MCP) Server](#model-context-protocol-mcp-server)
- [Developer Instructions](#developer-instructions)
  - [Prerequisites](#prerequisites)
  - [Getting Started](#getting-started)
  - [Running Tests](#running-tests)
  - [Code Quality & Formatting](#code-quality--formatting)
- [Building & Creating Releases](#building--creating-releases)
  - [Compiling Production Binaries](#compiling-production-binaries)
  - [Cross-Platform Compilation Targets](#cross-platform-compilation-targets)
  - [Release Packaging Checklist](#release-packaging-checklist)
- [Architectural Constraints & Rules](#architectural-constraints--rules)
- [License](#license)

---

## Core Philosophy

- **Local-First & Offline-Native:** Zero telemetry. Zero analytics. Zero background update pings. All collections, environments, and requests live as human-readable plain text files on your local filesystem.
- **Git-Native:** Collections are formatted using a clean, diff-friendly DSL (`.yarac`), eliminating merge conflicts and vendor lock-in.
- **Standards-Driven Scripting:** Powered by embedded [`deno_core`](https://crates.io/crates/deno_core) (V8 isolate). Scripts run standard modern ECMAScript/Web APIs (`crypto.subtle`, `fetch`, `TextEncoder`, `atob`/`btoa`). No legacy synthetic polyfills (`CryptoJS`, `Buffer`).
- **Relative Portability:** All file references (payloads, multipart uploads, certificates) resolve strictly relative to the workspace root. Machine-specific absolute paths are strictly prohibited.
- **AI Extensibility (MCP):** First-class integration for autonomous coding agents and LLMs via dual-transport Model Context Protocol (MCP).

---

## Key Features

### 1. Protocols & High-Performance Networking
- **Protocols Supported:** REST (HTTP/1.1 and HTTP/2), GraphQL, WebSockets, and Server-Sent Events (SSE).
- **Environment-Isolated Cookie Jar:** Backed by `reqwest_cookie_store`, keeping session cookies segregated per environment.
- **Authentication Handlers:** Bearer tokens, Basic Auth, API Key (Header or Query), and AWS SigV4 signer.
- **SSL & Certificates:** Granular SSL verification bypass and client certificate support.

### 2. Lossless CST Parser & Serializer
- **Lossless Round-Trip:** Parser-combinator engine (`nom`) that parses into a Concrete Syntax Tree (CST) and serializes back with 100% fidelity: `assert_eq!(serialize(parse(input)), input)`.
- **Line Ending Preservation:** Detects and maintains LF vs. CRLF formatting across editing cycles.
- **Dynamic Fenced Blocks:** Escapes nested backtick code blocks using adaptive fence delimiters.

### 3. Hierarchical Scope & Dynamic Variables
- **5-Tier Scope Hierarchy:** Evaluates variables across `Local` &rarr; `Collection` &rarr; `Environment` &rarr; `Workspace` &rarr; `Global`.
- **Dynamic Generators:** Out-of-the-box support for `{{$uuid}}`, `{{$timestamp}}`, `{{$randomInt}}`, and more.
- **Local Overrides:** Automatically reads untracked `.yarac.local` files for uncommitted local developer credentials.

### 4. Enterprise Security & Secret Scrubbing
- **OS Enclave Secret Storage:** Integrates with Windows Credential Manager, macOS Keychain, and Linux Secret Service (`keyring`).
- **Token-Boundary Secret Scrubber:** Automatically scrubs tainted secrets from logs, CLI output, and error traces:
  - Secrets &ge; 6 characters: Automated substring redaction to `[REDACTED]`.
  - Secrets < 6 characters: Strict lexical token or header boundary matching to avoid over-redacting common character sequences.

### 5. Declarative Flow Controls & Pacing
- **Once-Per-Run:** Execute setup/login requests only once during collection runs (`flow.once_per_run = true`).
- **Declarative Polling (`flow.retry_until`):** Built-in retry loops that poll endpoints until assertion expressions are satisfied or caps are reached.
- **Pacing & Randomized Jitter:** Configurable delay and jitter (`jitter_min_ms`, `jitter_max_ms`) to simulate natural client think-time.

### 6. Interoperability & Clean Importers
- **Postman v2.1 Importer:** Direct schema translation to native `.yarac` request structures without artificial runtime compatibility shims.
- **OpenAPI v3.1 Export/Import:** Clean bidirectional interop preserving metadata via `x-yarac-*` vendor extensions.

---

## Architecture & Crates

Yarac is organized as a modular Rust Cargo workspace:

```
c:/repo/yarac/
├── crates/
│   ├── yarac-core/         # Domain models, scope hierarchy, relative paths, secret scrubber
│   ├── yarac-parser/       # Lossless CST parser (nom) & round-trip serializer
│   ├── yarac-net/          # Async reqwest client, multipart streamer, cookie jar
│   ├── yarac-scripting/    # Ephemeral deno_core V8 execution engine (5000ms interrupt)
│   ├── yarac-git/          # Hybrid git engine (gitoxide + CLI fallback)
│   ├── yarac-importer/     # Schema converter for Postman Collections v2.1
│   ├── yarac-mcp/          # Model Context Protocol (MCP) server (stdio & HTTP/SSE)
│   └── yarac-cli/          # Headless test runner and MCP daemon executable
├── tests/                  # Integration and round-trip verification tests
└── yarac-spec/             # Specifications, architecture blueprints, and design docs
```

| Crate | Description |
|---|---|
| [`yarac-core`](crates/yarac-core) | Core data structures, path anchoring, secret registry, and scope resolution stack. |
| [`yarac-parser`](crates/yarac-parser) | Lossless CST parser and serializer with CRLF/LF preservation. |
| [`yarac-net`](crates/yarac-net) | High-throughput asynchronous HTTP execution engine. |
| [`yarac-scripting`](crates/yarac-scripting) | Sandboxed `deno_core` V8 runtime with ephemeral contexts and strict timeouts. |
| [`yarac-git`](crates/yarac-git) | Hybrid Git status, diff, branch, stage, and commit engine. |
| [`yarac-importer`](crates/yarac-importer) | Clean collection importer from Postman v2.1. |
| [`yarac-mcp`](crates/yarac-mcp) | Dual-transport Model Context Protocol implementation. |
| [`yarac-cli`](crates/yarac-cli) | Headless CLI test runner and MCP server. |

---

## The `.yarac` File Format

The `.yarac` format is a clean, Git-friendly hybrid specification using TOML headers combined with fenced code blocks:

```toml
# ==============================================================================
# Yarac Request Specification (.yarac)
# ==============================================================================
version = "1.0"
name = "Create User Account"
method = "POST"
url = "{{base_url}}/api/v1/users"

[settings]
inherit_auth = true
verify_ssl = true
follow_redirects = true
timeout_ms = 15000

[headers]
Content-Type = "application/json"
Accept = "application/json"
X-Request-Id = "{{$uuid}}"

[auth]
type = "bearer"
token = "{{api_bearer_token}}"

[params.query]
dry_run = "false"

[flow]
once_per_run = false
retry_until = "response.status == 201"
retry_interval_ms = 1000
max_retries = 3

# ------------------------------------------------------------------------------
# Request Body (File attachments strictly resolve relative to workspace root)
# ------------------------------------------------------------------------------
[body.json]
```json
{
  "email": "user_{{$randomInt}}@example.com",
  "avatar_attachment": "./fixtures/avatar.png"
}
```

# ------------------------------------------------------------------------------
# Pre-Request & Post-Response Scripts (deno_core / Web Standards)
# ------------------------------------------------------------------------------
[scripts.pre-request]
```javascript
const timestamp = Date.now().toString();
yarac.setHeader("X-Timestamp", timestamp);
```

[scripts.post-response]
```javascript
yarac.test("Response status is 201 Created", () => {
  expect(yarac.response.status).to.equal(201);
  const data = yarac.response.json();
  expect(data.id).to.be.a("string");
  yarac.setEnvVar("created_user_id", data.id);
});
```
```

---

## CLI Usage (`yarac-cli`)

### Headless Test Runner

Execute single `.yarac` request files or entire collection directories:

```bash
# Execute a single request
yarac-cli run ./tests/fixtures/sample.yarac

# Execute with an active environment file
yarac-cli run ./collections/users/create_user.yarac --env ./environments/staging.env.yarac

# Generate test reports for CI/CD pipelines (cli, junit, tap)
yarac-cli run ./collections/checkout --reporter junit > results.xml
```

### Deterministic Exit Codes

`yarac-cli run` returns standard, deterministic exit codes designed for CI/CD automation:

| Exit Code | Classification | Description |
|:---:|---|---|
| `0` | **Success** | Request executed and all test assertions passed. |
| `1` | **Assertion Failure** | Request completed, but one or more assertions failed. |
| `2` | **Configuration / Syntax Error** | Invalid `.yarac` syntax, missing environment, or path resolution violation. |
| `3` | **Network / Connectivity Error** | Connection refused, SSL error, DNS failure, or hard timeout. |

### Model Context Protocol (MCP) Server

Yarac provides native Model Context Protocol support for seamless integration with Cursor, Claude Desktop, and autonomous agents:

```bash
# 1. stdio Transport (Default: token-free subprocess for local agent hosts)
yarac-cli mcp --transport stdio

# 2. HTTP/SSE Transport (Binds strictly to loopback with Keyring-backed Bearer authentication)
yarac-cli mcp --transport sse --port 9472
```

#### Claude Desktop Configuration (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "yarac": {
      "command": "yarac-cli",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

---

## Developer Instructions

### Prerequisites
- [Rust](https://rustup.rs/) (v1.75 or higher recommended)
- `git` installed and available on `PATH`
- C/C++ compiler toolchain (MSVC on Windows, Xcode tools on macOS, GCC/Clang on Linux)

### Getting Started

Clone the repository and verify the workspace:

```bash
git clone https://github.com/m1chaeldg/yarac.git
cd yarac

# Fetch dependencies and check workspace
cargo check
```

### Running Tests

Run the full workspace unit test and integration suite:

```bash
# Run all workspace unit tests
cargo test

# Run parser round-trip invariant tests specifically
cargo test -p yarac-parser --test roundtrip_test

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test
```

### Code Quality & Formatting

Ensure code complies with Rust idioms and workspace standards before committing:

```bash
# Check formatting
cargo fmt --all --check

# Format code
cargo fmt --all

# Run Clippy lints
cargo clippy --all-targets --workspace -- -D warnings
```

---

## Building & Creating Releases

### Compiling Production Binaries

Build optimized release binaries with link-time optimization (LTO):

```bash
# Build all release binaries across the workspace
cargo build --release --workspace

# Build the standalone yarac-cli binary
cargo build --release -p yarac-cli
```

Compiled binaries will be available at:
- **Windows:** `target/release/yarac-cli.exe`
- **macOS / Linux:** `target/release/yarac-cli`

### Cross-Platform Compilation Targets

Yarac can be compiled for major deployment targets:

```bash
# Linux (x86_64 glibc)
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu -p yarac-cli

# Linux (musl static binary)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl -p yarac-cli

# macOS (Apple Silicon M-series)
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin -p yarac-cli

# macOS (Intel)
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin -p yarac-cli

# Windows (MSVC 64-bit)
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc -p yarac-cli
```

### Release Packaging Checklist

When preparing a new release:

1. **Version Bump:** Update the version across `Cargo.toml` (`[workspace.package]` version).
2. **Verification Suite:**
   ```bash
   cargo fmt --all --check
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   ```
3. **Round-Trip Invariant Check:** Ensure all `.yarac` fixtures and round-trip tests pass.
4. **Binary Stripping & Compression:**
   - On Linux/macOS: `strip target/release/yarac-cli`
   - Archive with tar/zip: `tar -czvf yarac-cli-v0.1.0-<target>.tar.gz -C target/release yarac-cli`
5. **Git Tagging:**
   ```bash
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

---

## Architectural Constraints & Rules

All code and contributions to Yarac must satisfy the strict requirements specified in [`AGENTS.md`](AGENTS.md):

1. **Memory Budget:** Maintain a lightweight memory footprint target (<100MB RAM runtime target). No unbounded background workers or unneeded thread pools.
2. **JavaScript Engine (`deno_core`):**
   - Strictly use `deno_core`. Never introduce legacy shims (`CryptoJS`, Node `Buffer`).
   - Every request script runs in an ephemeral context dropped and garbage-collected immediately upon test completion.
   - Enforce a `5000ms` hard interrupt timeout on script execution.
3. **100% Offline & Zero Telemetry:** Never introduce analytics, crash pingbacks, or background tracking.
4. **Relative Path Anchoring:** All file uploads, fixtures, and certificates must resolve strictly relative to the workspace root. Never persist absolute machine paths into `.yarac` files.
5. **Lossless CST Parser Invariant:** Every parser change must satisfy `assert_eq!(serialize(parse(input)), input)` and preserve CRLF/LF line endings.
6. **Secret Scrubbing:**
   - Active secrets &ge; 6 characters undergo substring scrubbing.
   - Active secrets < 6 characters require exact token boundaries or header-specific matching.
7. **MCP Network Security:** HTTP/SSE transport must bind strictly to `127.0.0.1` and validate OS Keyring Bearer tokens on every incoming request.

---

## License

This project is licensed under the [MIT License](LICENSE).
