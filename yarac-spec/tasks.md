# Implementation Tasks

### Phase 1: MVP Core Architecture
- [ ] Initialize Tauri v2 + Svelte 5 + Tailwind CSS project skeleton.
- [ ] Build Lossless CST Parser/Serializer in Rust using `nom` or `pest`.
  - [ ] Implement CRLF/LF newline preservation.
  - [ ] Unit test suite verifying `serialize(parse(x)) == x`.
  - [ ] Raw CodeMirror fallback reader for Git merge conflict markers.
- [ ] Build Postman Collection v2.1 Importer (direct translation to `.yarac` without runtime polyfill layers).
- [ ] Implement Rust networking core with `reqwest`.
  - [ ] Integrate `reqwest_cookie_store` with environment isolation.
  - [ ] Add global/collection SSL bypass and mTLS cert loaders.
  - [ ] Implement chunked streaming/IPC virtualization for payloads > 2MB.
- [ ] Integrate `deno_core` engine for script execution.
  - [ ] Implement ephemeral execution contexts with explicit garbage collection per request.
  - [ ] Implement `5000ms` hard execution interrupt handler.
  - [ ] Build Token-Boundary Secret Scrubber (>= 6 chars entropy threshold).
- [ ] Implement storage & file handling:
  - [ ] Integrate `notify` crate for background filesystem change detection.
  - [ ] Implement Workspace Relative Path Anchoring for file uploads and certs.
  - [ ] Wire variable store to `.yarac.local` and OS `keyring` storage.
- [ ] Build Svelte 5 + CodeMirror 6 UI:
  - [ ] Add CodeMirror Hover Tooltip extension for variable resolution.
  - [ ] Build 3-way external conflict resolver modal.
- [ ] Build Hybrid Git Engine:
  - [ ] Implement `gitoxide` for local status, diff, branch, stage, and commit.
  - [ ] Implement system `git` CLI fallback for complex remote push/pull.
- [ ] Build Model Context Protocol (MCP) interface:
  - [ ] Implement `stdio` transport in CLI (`yarac-cli mcp`).
  - [ ] Implement HTTP/SSE transport with Keyring-backed token auth and UI toggle.
- [ ] Implement `yarac-cli` headless runner with JUnit XML, TAP, and deterministic exit codes (`0`, `1`, `2`, `3`).

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
