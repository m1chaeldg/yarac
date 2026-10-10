# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-10-10

### Added
- **Frontend Studio Suite & Visual Request Workbench (`frontend/`)**:
  - **Multi-Workspace & Collection State Store (`stores/workspace.svelte.ts`)**: Svelte 5 reactive runes store managing multi-workspace hierarchy, collections, request tabs, and environment profiles with automatic `localStorage` persistence.
  - **Visual Request Editor (`components/VisualEditor.svelte`)**: Bruno/Postman-grade visual forms for query parameters, headers with common presets, authentication profiles (Bearer, Basic Auth, API Key in header/query, Inherit), and multi-type body payloads (JSON, URL-encoded, multipart, raw text).
  - **Lossless Bidirectional DSL Sync**: Real-time two-way synchronization between visual forms and `.yarac` CodeMirror 6 code editor without loss of formatting or comments.
  - **Multi-Tab Request Interface (`components/TabBar.svelte`)**: Browser-style tab strip with HTTP method pills, unsaved dirty indicators, and tab switching/closing.
  - **Response Analyzer & Visualizer (`components/ResponseViewer.svelte`)**:
    - Formatted and syntax-highlighted JSON viewer.
    - Automatic JSON-flattening data table view for structured array and nested object inspection.
    - Native responsive SVG Data Visualizer generating interactive Pie Charts and Bar Charts from numerical response metrics.
    - Sandboxed HTML render viewport and header inspector.
  - **Bruno-Style Environment Manager (`components/EnvironmentModal.svelte`)**: Multi-profile management with 1-click profile duplication, cross-environment variable copying, and secret masking.
  - **Smooth Resizable Split Panes (`components/Resizer.svelte`)**: Fully accessible keyboard- and mouse-resizable divider handles for sidebar width and Request/Response viewport height with double-click reset.

## [0.2.0] - 2026-10-10

### Added
- **Declarative Flow Controls (`yarac-core`, `yarac-parser`, `yarac-scripting`)**:
  - `flow.once_per_run`: Single execution gating per collection run for authentication or setup tasks.
  - `flow.retry_until`: Built-in endpoint polling with configurable `retry_interval_ms`, `max_retries` cap, and ephemeral `deno_core` JavaScript predicate evaluation against `response.status`, `response.body`, and `response.headers`.
  - Pacing and randomized jitter: `delay_ms`, `jitter_min_ms`, and `jitter_max_ms` support in CST parser and serializer with preserved round-trip invariant.
- **High-Concurrency Runner & Statistical Profiling (`yarac-cli`)**:
  - Tokio-based worker pool supporting `--concurrency`, `--ramp-up`, `--duration`, and `--iterations`.
  - External CSV and JSON iteration data loader (`-d` / `--data`).
  - Integrated `hdrhistogram` for in-memory tracking of `p50`, `p90`, `p95`, `p99`, RPS throughput, and error rates while strictly respecting the <100MB RAM runtime target.
  - Terminal ASCII histogram distribution chart rendering latency buckets and standard percentile statistics.
- **Interoperability & Schema Conversion (`yarac-importer`, `yarac-cli`)**:
  - OpenAPI v3.1 Exporter translating `.yarac` collections to OpenAPI specification files with `x-yarac-*` vendor extensions.
  - Bruno `.bru` Importer converting dictionary/body blocks to `.yarac` request structures.
  - OpenAPI v3 and v3.1 Importer parsing JSON and YAML specifications into collections.
  - `yarac-cli export openapi` and `yarac-cli import --format <postman|bruno|openapi>` subcommands.
- **Frontend GUI Client Enhancements (`frontend/`)**:
  - **Collection Runner & Profiler View (`RunnerView.svelte`, `RunnerModal.svelte`)**: Dedicated Svelte 5 dashboard with concurrency/iteration controls, CSV/JSON data input, live throughput/percentile metric cards, and animated HDR percentile distribution bar charts.
  - **Flow Controls Panel (`FlowPanel.svelte`)**: Visual GUI builder for `once_per_run`, `retry_until`, interval, max retries, base delay, and jitter sliders with real-time bi-directional `.yarac` CST DSL synchronization.
  - **Interoperability Hub Modal (`InteropModal.svelte`)**: Interactive dialog for importing Bruno, OpenAPI (JSON/YAML), and Postman v2.1 collections, and exporting to OpenAPI v3.1 with metadata overlays.
  - Top-level mode navigation switcher between ⚡ *Request Studio* and 📊 *Collection Runner* in `App.svelte`.

## [0.1.0] - 2026-10-10

### Added
- **Lossless CST Parser & Serializer (`yarac-parser`)**: Built with parser-combinator `nom`, preserving raw formatting, indentation, comments, CRLF/LF newlines, and multi-line fenced code blocks. Verified with round-trip invariant `assert_eq!(serialize(parse(input)), input)`.
- **Core Domain & Security Engine (`yarac-core`)**: Domain models, scope variable resolver (Local $\to$ Collection $\to$ Environment $\to$ Workspace $\to$ Global), dynamic template generation (`{{$uuid}}`, `{{$timestamp}}`, `{{$randomInt}}`), and workspace-relative path anchoring.
- **Token-Boundary Secret Scrubber**: Substring scrubbing for secrets $\ge 6$ characters and lexical token-boundary matching for secrets $< 6$ characters per `AGENTS.md`.
- **HTTP Engine (`yarac-net`)**: Asynchronous HTTP/1.1 and HTTP/2 execution backed by `reqwest`, with isolated cookie store per environment, multipart uploads, SSL controls, and redirect policies.
- **Ephemeral Script Sandbox (`yarac-scripting`)**: Embedded standard `deno_core` V8 runtime with ephemeral execution context dropped per request, 5000ms hard timeout interrupt, and BDD assertions (`expect(val).to.equal(...)`).
- **Postman v2.1 Importer (`yarac-importer`)**: Direct conversion of Postman v2.1 collection JSON files to `.yarac` request structures with auth, query params, and workspace file serialization.
- **Hybrid Git Engine (`yarac-git`) & Conflict Reader**: Lossless parsing and roundtrip serialization of Git merge conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`).
- **Model Context Protocol (`yarac-mcp`)**: JSON-RPC 2.0 MCP server with live tool execution and Bearer-authenticated loopback HTTP/SSE daemon.
- **Headless CLI Runner (`yarac-cli`)**: Recursive directory collection runner with `--env` file loading, secret scrubbing, JUnit XML (`--reporter junit`), TAP (`--reporter tap`), and deterministic exit codes (`0`, `1`, `2`, `3`).
- **Frontend & Tauri Skeleton (`frontend/`, `src-tauri/`)**: Tauri v2 + Svelte 5 + Tailwind CSS project skeleton with CodeMirror 6 Variable Hover Tooltip extension, 3-way conflict resolver modal, and large payload virtualization.
- **Test Suite**: Comprehensive unit and integration test suite across the entire workspace.
