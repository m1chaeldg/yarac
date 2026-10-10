# Yarac Feature Specifications

## 1. Protocols & Network Engine
* **Supported Protocols:** REST (HTTP/1.1, HTTP/2), GraphQL (schema introspection and variables), WebSockets, gRPC, Server-Sent Events (SSE).
* **Cookie Management:** Built-in automatic Cookie Jar backed by `reqwest_cookie_store`. Isolated per environment.
* **Privacy & Telemetry:** Absolutely zero telemetry, analytics, or background update pings. 100% offline-first.

## 2. Environments, Scopes & Secrets
* **Scope Hierarchy:** Local -> Collection -> Environment -> Workspace -> Global.
* **Hover Tooltip Inspection:** CodeMirror tooltip showing evaluated value, scope origin, and storage source (Keyring vs. local file).
* **Relative Path Anchoring:** All file upload references, client cert paths, and payload fixtures resolve strictly relative to the workspace root.
* **Untracked Local Overrides:** Automatic parsing of root `.yarac.local` files for uncommitted local keys.

## 3. Security & Secret Scrubbing
* **Keyring Storage:** Secrets write directly to Windows Credential Manager, macOS Keychain, or Linux Secret Service (`keyring` crate).
* **Scrubbing Configuration:**
  - Setting: `security.scrub_tainted_secrets` (Boolean, default: `true`).
  - Active secrets >= 6 chars are scrubbed across logs and console outputs as `[REDACTED]`.
  - Active secrets < 6 chars require strict token or header-boundary matches to avoid over-redacting common strings.

## 4. Scripting & Test Assertions
* **Runtime:** Sandboxed modern ECMAScript via `deno_core`.
* **Execution Timeout:** Hard interrupt at `5000ms` to prevent runaway loops.
* **Test Assertions:** Standard BDD assertion syntax:
  ```javascript
  yarac.test("User created successfully", () => {
    expect(yarac.response.status).to.equal(201);
    expect(yarac.response.json().id).to.be.a("string");
  });
  ```
* **No Over-Engineered Postman Shims:** Standard JavaScript only. Legacy Postman-specific libraries (`CryptoJS`, `pm.*` globals) are not polyfilled; script errors are surfaced directly to the user to fix natively.

## 5. File Format, Git Sync & Interoperability
* **`.yarac` Format:** Lossless hybrid DSL combining TOML metadata and fenced code blocks.
* **In-App Git Client:** Status, staging, commits, and diffs via `gitoxide`, with system CLI fallback for complex remote pushes/pulls.
* **Importers:** Postman Collection v2.1 (converts endpoints, headers, params, and bodies directly to `.yarac` files without synthetic runtime compatibility layers).
* **Export to OpenAPI v3.1:** Exports standard endpoints and schemas, retaining execution metadata in standard vendor extensions (`x-yarac-*`).

## 6. Declarative Flow Controls & Pacing (JMeter-Inspired)
* **Once-Per-Run (`flow.once_per_run = true`):** Runs the request only on the initial iteration of a test or load run.
* **Declarative Polling (`flow.retry_until`):** Repeats an endpoint automatically until an expression is satisfied or timeouts occur:
  ```toml
  [flow]
  retry_until = "response.status == 200 && response.body.status == 'ready'"
  retry_interval_ms = 1000
  max_retries = 10
  ```
* **Pacing & Timers (Jitter):** Simulates natural client think-time between calls:
  - Fixed delay: e.g., `delay_ms = 250`
  - Randomized jitter: e.g., `jitter_min_ms = 100`, `jitter_max_ms = 500` to prevent unnatural traffic spikes.

## 7. Collection Runner, Concurrency & CI/CD (`yarac-cli`)
* **Functional & Data-Driven Runs:**
  - Execute sequential folders/collections with iteration counts or data files (CSV/JSON).
  - Deterministic Exit Codes: `0` (Success), `1` (Assertion failure), `2` (Config error), `3` (Network error).
* **Local Concurrency Mode (Micro-Load Testing):**
  - Uses Tokio asynchronous worker pools in Rust to simulate concurrent users with minimal RAM footprint:
    ```bash
    yarac-cli run ./collections/checkout --concurrency 50 --ramp-up 10s --duration 60s
    ```
* **Statistical Performance Profiling:**
  - Employs an in-memory HDR histogram (`hdrhistogram` crate) to calculate real-time performance profiles without GC pauses:
    - Min, Max, Mean, p50, p90, p95, and p99 latencies.
    - Requests per second (RPS / Throughput) and error percentage.
* **Reporting Formats:** Terminal interactive summary table, JUnit XML, and TAP (Test Anything Protocol).

## 8. Model Context Protocol (MCP) Integration
* **`stdio` Transport:** Invoked via `yarac-cli mcp` for local agent integration.
* **HTTP/SSE Transport:** Local daemon on configurable loopback port (default: `127.0.0.1:9472`) with Bearer token authentication.
