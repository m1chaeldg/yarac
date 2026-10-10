# Agent Instructions for Yarac Development

All AI assistants and autonomous coding agents contributing to this codebase must adhere to the following architectural constraints:

## 1. Rust Backend Constraints
* **Memory & Performance:** Maintain lightweight memory overhead (<100MB RAM runtime target). Avoid background workers or unneeded thread pools.
* **JavaScript Engine:** Strictly use **`deno_core`**.
  - Do NOT introduce legacy polyfill libraries (`CryptoJS`, `buffer`). Let scripts run standard Web/ES APIs.
  - Ensure every request execution uses an ephemeral context dropped immediately after assertions run.
  - Enforce a `5000ms` hard interrupt timeout on script execution.
* **Privacy & Telemetry:**
  - NEVER introduce analytics, telemetry, pingbacks, or background tracking.
  - The application must operate 100% offline.
* **File System & Paths:**
  - Enforce **Relative Path Anchoring**: all file uploads, fixtures, and certificates must resolve relative to the workspace root. Never persist absolute machine paths into `.yarac` files.
* **Parser Rules:**
  - Never use regex or simple string splitting for parsing `.yarac` files.
  - Use parser-combinators (`nom` or `pest`) producing a Concrete Syntax Tree.
  - Preserve original line-ending formats (CRLF vs. LF) across parse/serialize cycles.
  - Every parser pull request must pass the round-trip invariant: `assert_eq!(serialize(parse(input)), input)`.
* **Full-Stack Parity Invariant:**
  - When introducing or modifying features, always assess and deliver both frontend and backend components together.
  - Never leave frontend features backed by mock timeouts or synthetic state when a Rust domain crate or filesystem contract is required.
* **Security & Tainted Scrubbing:**
  - Respect `security.scrub_tainted_secrets` (default: `true`).
  - Apply automated substring scrubbing only to secrets with length >= 6.
  - For secrets < 6 characters, require exact token boundaries or header-specific matching.

## 2. Frontend Constraints (Svelte 5 & CodeMirror 6)
* **Reactivity Model:** Use Svelte 5 runes (`$state`, `$derived`, `$effect`).
* **Large Payloads:** Payloads > 2MB must use chunked streams or virtualized viewports. Never serialize massive raw JSON strings over synchronous IPC.
* **File Conflict Resilience:** When external changes clash with unsaved UI changes, prompt the user via the 3-way conflict modal instead of silently overwriting.

## 3. CLI & CI/CD Runner
* `yarac-cli run` must adhere strictly to deterministic exit codes:
  - `0`: Success (all assertions passed).
  - `1`: Test assertion failure.
  - `2`: Configuration or syntax error.
  - `3`: Network or connectivity error.

## 4. Model Context Protocol (MCP) Rules
* `stdio` transport is the default for local integrations and must remain token-free.
* HTTP/SSE transport must bind strictly to `127.0.0.1` and validate the OS Keyring-backed Bearer token on every incoming request. Reject unauthenticated requests with HTTP 401.
