# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.2] - 2026-10-10

### Added
- **Visual Editor Body Tab Full-Stack Parity & File Upload**:
  - Full bidirectional parity between Visual Editor Body tab and `.yarac` DSL for all 7 body types: `none`, `json`, `xml`, `text`, `form-urlencoded` (`[body.form]`), `multipart` (`[body.multipart]`), and `binary` (`[body.binary]`).
  - Rust parser CST and AST support for XML, Text, Multipart, and Binary request bodies with 100% round-trip fidelity (`assert_eq!(serialize(parse(input)), input)`).
  - Variable interpolation and execution dispatching for XML, Text, and Binary payloads in `yarac-core` and `yarac-net`.
  - Smart body formatting and compression tools:
    - Dedicated "Format" and "Compress" actions for JSON, XML (comment stripping, inter-tag collapse, CDATA preservation), and Plain Text.
    - Cleaned up redundant UI controls (removed duplicate "Pretty JSON" button).
  - Real-world file upload and browsing experience:
    - Interactive file picker (`<input type="file">`) dialog for Binary payloads and Multipart form file rows.
    - Drag-and-drop dropzone for binary files with file metadata inspection card (filename, formatted byte size, auto-detected MIME type, and clear action).
    - Enforced Relative Path Anchoring (`./fixtures/<file>`) across UI and backend, preventing absolute machine paths from persisting.

## [0.3.1] - 2026-10-10

### Added
- **Inactive & Deleted Key-Value Suffix Tilde (`~`) Notation**:
  - Unchecking headers, query parameters, variables, or environment variables appends `~` to the key name (e.g. `X-Request-Id~ = "{{$uuid}}"`) in `.yarac` DSL instead of deleting it.
  - Lossless bidirectional synchronization between Visual Editor and `.yarac` code views preserving inactive items.
  - Rust parser CST recognition of `~` in key identifiers with 100% round-trip fidelity (`assert_eq!(serialize(parse(input)), input)`).
  - Strict exclusion of inactive keys (`key.ends_with('~')`) from HTTP request execution across CLI, daemon, and client fallback runners.

## [0.3.0] - 2026-10-10

### Added
- **Request Tab Enhancements**:
  - **Request-Level Variables (`[variables]`)**: Scoped variables with secret toggles, prioritized over collection and environment scopes, interpolable anywhere in URLs, headers, and request bodies via `{{variable_name}}`.
  - **Embedded Faker Engine & Bruno Dynamic Variables**:
    - Comprehensive embedded zero-dependency faker library (`faker.internet`, `faker.person`, `faker.string`, `faker.location`, `faker.number`, `faker.commerce`, `faker.company`, `faker.datatype`, `faker.lorem`, `faker.date`).
    - Complete Bruno dynamic macro evaluation (`$uuid`, `$guid`, `$timestamp`, `$isoTimestamp`, `$randomInt`, `$randomFloat`, `$randomEmail`, `$randomFullName`, `$randomPassword`, `$randomCity`, `$randomCountry`, `$randomPrice`, etc.).
  - **Pre-Request & Post-Response Scripting (`[scripts.pre-request]`, `[scripts.post-response]`)**:
    - Ephemeral sandboxed `deno_core` runtime with 5000ms interrupt timeout.
    - Script global APIs: `req`, `res`, `yarac.getVar()`, `yarac.setVar()`, `yarac.getEnvVar()`, `yarac.setEnvVar()`, `faker`, and `console.log`.
  - **JavaScript Assertions (`[assert]`)**:
    - Lightweight single-line or multi-line assertion blocks evaluated against responses.
  - **Jasmine / Bruno BDD Test Suite (`[tests]`)**:
    - Full `describe`, `it`, `test`, `expect` matchers (`toBe`, `toEqual`, `toBeDefined`, `toBeNull`, `toBeGreaterThan`, `toContain`, `toMatch`, `toThrow`).
  - **Request Execution Settings (`[settings]`)**:
    - `follow_redirects` (bool), `max_redirects` (u32 limit, default 5), `timeout_ms` (u64, default 0 for indefinite/no timeout), `verify_ssl`, `inherit_auth`.
- **Response Tab Enhancements**:
  - **Test Results View**: Real-time pass/fail breakdown with duration, error messages, and script console output logs.
  - **Execution History Timeline**: Chronological run log capturing execution timestamps, status badges, latencies, and payload sizes.
  - **Enhanced Response Body Views**:
    - JSON view with pretty formatting and syntax rendering.
    - Flattened Data Table with multi-column text search, column filtering, and interactive column sorting (ascending/descending).
    - Rendered HTML preview with secure sandboxed frame.
    - XML formatted and syntax-highlighted document viewer.
    - Raw text body viewer.
    - Visualizer charts (Pie Chart & Bar Chart) for numerical response payloads.
  - **Response Tools Menu (`...`)**:
    - Copy response body, save/download as JSON or raw text, clear response.
    - Split layout toggle between vertical (stacked) and horizontal (side-by-side) orientations.

## [0.2.4] - 2026-10-10

### Added
- **Collapsible Tree View & Collection Fold Management (`frontend/src/components/WorkspaceNav.svelte`)**:
  - **Individual Expand / Collapse Toggles**: Interactive folder header clicking and dedicated rotating chevron buttons (`▶` / `▼`) to fold and unfold collection contents.
  - **Expand All & Collapse All Controls**: Toolbar action buttons in the Collections header bar to expand or collapse all collections at once with animated status toasts.
  - **Context Menu Integration**: Added "Expand/Collapse Collection", "Expand All", and "Collapse All" actions to the collection right-click menu.
  - **Keyboard Navigation & Accessibility**: Added `aria-expanded` attributes and keyboard shortcuts (`Enter`/`Space` to toggle, `ArrowRight` to expand, `ArrowLeft` to collapse).
  - **Context-Aware Auto-Expansion**: Automatically expands collapsed collections when adding a new request (`+req`) or dropping a dragged request into them.
  - **Automated Verification**: Added comprehensive unit test suite (`frontend/src/utils/testCollapsibleTreeView.ts`) verifying state transitions and edge cases.

## [0.2.3] - 2026-10-10

### Added
- **Full-Stack REST Client Synchronization & Auto-Daemon Execution**:
  - **Full-Spectrum Variable Interpolation Engine (`yarac-core`)**:
    - Implemented `VariableScopeStack::interpolate_request` recursively resolving template variables `{{var}}` and dynamic macros (`{{$uuid}}`) across URLs, headers, query parameters, authentication configurations, and request payloads (JSON, raw, urlencoded, multipart).
  - **Offline Mock Echo Engine & Request Dispatcher (`yarac-cli/src/server.rs`)**:
    - Integrated request scope interpolation into live request and runner execution endpoints.
    - Added built-in loopback mock routes (`/api/v1/users/*`, `/credentials`, `/api/echo*`) supporting 100% offline self-contained usability without external network dependencies.
    - Filtered `.env.yarac` and hidden system files from workspace scan and file collectors.
  - **Automated Daemon Lifecycle (`frontend/vite.config.ts`)**:
    - Added health probe for `127.0.0.1:9475` on dev server startup, automatically spawning `yarac-cli.exe serve` if not running.
  - **Query Parameter Synchronization & Tab State Persistence (`frontend/stores/workspace.svelte.ts`)**:
    - Added bidirectional query parameter parsing and serialization (`[params.query]`) matching the `.yarac` CST specification.
    - Preserved active request tabs and selected tab across page reloads.
  - **Headless CLI Runner Environment Auto-Discovery (`yarac-cli`)**:
    - Auto-discovers `.env.yarac` and defaults `base_url` to local daemon port for headless test runs.
  - **VS Code Debugging Profiles (`.vscode/launch.json`)**:
    - Corrected workspace directory paths and added dedicated daemon debugging target.

## [0.2.2] - 2026-10-10

### Added
- **File-System Synchronized Workspace & Explorer Integration**:
  - **Immediate Disk Persistence**: Creating, duplicating, renaming, or deleting requests immediately synchronizes changes to disk as `.yarac` files in their respective collection directories with updated `workspace.json` manifest.
  - **Collection Disk Synchronization**: Creating, renaming, or deleting collections automatically creates/updates/removes directories on disk.
  - **Atomic File Reconciliation**: Saves clean up orphaned or renamed `.yarac` files and empty removed directories automatically with 1500ms watcher echo suppression.
  - **Native Reveal in Explorer**: Integrated `POST /api/workspace/reveal` across Vite dev server and Rust daemon. Highlights files in Windows File Explorer (`explorer.exe /select`) or opens collection folders, with cross-platform macOS/Linux support.
  - **Collection Actions Context Menu**: Added right-click context menu and hover action buttons for collections (New Request, Reveal in Explorer, Rename Collection, Delete Collection).

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
