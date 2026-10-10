# Implementation Tasks

### Phase 1: MVP Core Architecture
- [x] Initialize Tauri v2 + Svelte 5 + Tailwind CSS project skeleton.
- [x] Build Lossless CST Parser/Serializer in Rust using `nom` or `pest`.
  - [x] Implement CRLF/LF newline preservation.
  - [x] Unit test suite verifying `serialize(parse(x)) == x`.
  - [x] Raw CodeMirror fallback reader for Git merge conflict markers.
- [x] Build Postman Collection v2.1 Importer (direct translation to `.yarac` without runtime polyfill layers).
- [x] Implement Rust networking core with `reqwest`.
  - [x] Integrate `reqwest_cookie_store` with environment isolation.
  - [x] Add global/collection SSL bypass and mTLS cert loaders.
  - [x] Implement chunked streaming/IPC virtualization for payloads > 2MB.
- [x] Integrate `deno_core` engine for script execution.
  - [x] Implement ephemeral execution contexts with explicit garbage collection per request.
  - [x] Implement `5000ms` hard execution interrupt handler.
  - [x] Build Token-Boundary Secret Scrubber (>= 6 chars entropy threshold).
- [x] Implement storage & file handling:
  - [x] Integrate `notify` crate for background filesystem change detection.
  - [x] Implement Workspace Relative Path Anchoring for file uploads and certs.
  - [x] Wire variable store to `.yarac.local` and OS `keyring` storage.
- [x] Build Svelte 5 + CodeMirror 6 UI:
  - [x] Add CodeMirror Hover Tooltip extension for variable resolution.
  - [x] Build 3-way external conflict resolver modal.
- [x] Build Hybrid Git Engine:
  - [x] Implement `gitoxide` for local status, diff, branch, stage, and commit.
  - [x] Implement system `git` CLI fallback for complex remote push/pull.
- [x] Build Model Context Protocol (MCP) interface:
  - [x] Implement `stdio` transport in CLI (`yarac-cli mcp`).
  - [x] Implement HTTP/SSE transport with Keyring-backed token auth and UI toggle.
- [x] Implement `yarac-cli` headless runner with JUnit XML, TAP, and deterministic exit codes (`0`, `1`, `2`, `3`).

### Phase 2: Performance Profiling, Flow Controls & Interoperability
- [ ] Build Declarative Flow Controllers in the Rust runner:
  - [ ] Support `flow.once_per_run` execution gating.
  - [ ] Support `flow.retry_until` with configurable intervals and max retry caps.
  - [ ] Implement request pacing timers with randomized jitter ranges.
- [ ] Implement Runner Concurrency & Statistical Profiling:
  - [ ] Add Tokio-based worker pool to support `--concurrency`, `--ramp-up`, and `--duration` flags.
  - [ ] Integrate `hdrhistogram` for in-memory tracking of p50, p90, p95, p99, and RPS metrics.
  - [ ] Build terminal and UI summary charts displaying latency percentiles and error rates.
- [ ] Build OpenAPI v3.1 Exporter translating `.yarac` files to paths with `x-yarac-*` overlays.
- [ ] Build Bruno `.bru` and OpenAPI v3/v3.1 Importers.
- [ ] Implement UI-driven sequential collection runner with CSV/JSON iterations.

### Phase 3: Advanced Integrations (Post-MVP)
- [ ] AWS Secrets Manager and HashiCorp Vault live resolution engines.
- [ ] Sandboxed HTML Visualizer ("App" view) in an isolated sub-view.
- [ ] External VS Code / LSP extension for standalone `.yarac` editing.
- [ ] Built-in MITM local HTTP Proxy recorder for capturing browser traffic into `.yarac` requests.
