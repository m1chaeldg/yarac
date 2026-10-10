# System Architecture & Design

Yarac uses a decoupled architecture: a lean presentation layer coupled with an asynchronous Rust core handling networking, parsing, version control, and sandboxed script execution.

```
+--------------------------------------------------------------------------+
|                       Presentation Layer (Svelte 5)                      |
|  - CodeMirror 6 Editor (Syntax Highlighting, Hover Tooltip Extension)   |
|  - Virtualized Viewport (Chunked streaming for payloads > 2MB)          |
|  - Request Config Panels, Runner UI, 3-Way Conflict Resolver             |
+--------------------------------------------------------------------------+
                                     |
               Tauri v2 IPC (Binary Streams / Chunked IPC)
                                     |
+--------------------------------------------------------------------------+
|                           Rust Core Engine                               |
|                                                                          |
|  +-----------------------+  +-----------------------------------------+  |
|  | Lossless CST Parser   |  | deno_core Runtime (V8 Isolate)          |  |
|  | - Line-ending aware   |  | - Ephemeral context per request cycle   |  |
|  | - Round-trip invariant|  | - Native Web standards (crypto, fetch)  |  |
|  +-----------------------+  | - Token-Boundary Secret Scrubber        |  |
|                             +-----------------------------------------+  |
|  +-----------------------+                                               |
|  | reqwest Engine        |  +-----------------------------------------+  |
|  | - Cookie Jar Store    |  | Storage & File Watcher                  |  |
|  | - AWS SigV4 Signer    |  | - notify (Background FS watcher)        |  |
|  +-----------------------+  | - Hybrid Git (gitoxide + CLI fallback)  |  |
|                             | - keyring (OS Secret Enclave)           |  |
|  +-----------------------+  | - Relative Path Resolver                |  |
|  | MCP Dual Transport    |  +-----------------------------------------+  |
|  | - stdio (CLI Subproc) |                                               |
|  | - HTTP/SSE (Token)    |                                               |
|  +-----------------------+                                               |
+--------------------------------------------------------------------------+
```

## 1. Frontend Layer
* **Framework:** Svelte 5 (runes reactivity), TypeScript, Tailwind CSS.
* **Code Editor:** CodeMirror 6 with custom extensions:
  - `.yarac` grammar and fenced code block syntax highlight.
  - **Variable Hover Tooltip:** Displays resolved values, active scope origin (`Global`, `Workspace`, `Environment: <name>`, `Collection`, `Local`), and OS Keyring indicator badges.
  - **Large Payload Virtualization:** Payloads > 2MB stream in chunks; CodeMirror windows visible lines to prevent DOM lockup.
* **External File Changes:** Integrated with the Rust file watcher (`notify` crate). If external tools (e.g., `git checkout`) modify files while unsaved changes exist in the editor, a 3-way diff modal prompts the user to resolve the divergence.

## 2. Parser Architecture (Lossless Concrete Syntax Tree)
* Built using Rust parser combinators (`nom` or `pest`).
* Implements a **Concrete Syntax Tree (CST)** preserving comments, key order, indentation, and detected line-ending conventions (CRLF vs. LF).
* Multi-line fence blocks escape nested backtick sequences using dynamic fence length rules.
* Invariant: `serialize(parse(input)) == input`.

## 3. Script Execution Runtime (`deno_core`)
* **Engine:** Embedded `deno_core` utilizing a single reusable V8 isolate.
* **Ephemeral Contexts:** To prevent memory leaks during long-running collection iterations, each request execution runs inside a newly created execution scope that is dropped and garbage-collected immediately upon test completion.
* **Execution Timeout:** Native hard execution interrupt set to `5000ms` (configurable).
* **Standard Web APIs:** Utilizes modern ECMAScript standards natively supported by Deno (`console`, `crypto.subtle`, `atob`/`btoa`, `fetch`).
* **No Legacy Postman Polyfills:** Yarac does not inject legacy shims (`CryptoJS`, Node `Buffer`). Modern standard JavaScript APIs must be used.

## 4. Token-Boundary Secret Scrubbing Engine
* Secrets are tagged with an in-memory `Tainted` bit.
* Controlled by `security.scrub_tainted_secrets` (default: `true`).
* **Scrubbing Rules:**
  - **Entropy/Length Threshold:** Only secrets with >= 6 characters are candidates for substring scrubbing.
  - **Token Boundaries:** Secrets < 6 characters are redacted only when appearing as distinct lexical tokens or specific header values (e.g., inside `Authorization`).
  - Redaction replaces matched values with `[REDACTED]`.

## 5. Storage Layout & Relative Path Anchoring
* Workspaces enforce clean directory segregation:
  ```text
  my-workspace/
  ├── yarac.workspace
  ├── environments/
  │   ├── dev.env.yarac
  │   └── staging.env.yarac
  └── collections/
      └── users/
          └── create_user.yarac
  ```
* **Relative Path Anchoring:** All file attachments, certificates, and multipart uploads resolve relative to the workspace root (e.g., `./fixtures/file.csv`). Absolute system paths are prohibited.

## 6. Hybrid Git Engine
* **Local Operations:** `gitoxide` handles repository reading, staging, committing, local branching, and diff calculations in pure Rust.
* **Remote Sync Fallback:** Remote `push` and `pull` operations fall back to the host system's `git` CLI executable when complex corporate SSH agents or credential helpers are encountered.

## 7. Model Context Protocol (MCP) Dual-Transport Model
* **Transport 1 — `stdio`:** Default CLI subprocess transport (`yarac-cli mcp`) for local agent hosts (Cursor, Claude Desktop). Token-free.
* **Transport 2 — HTTP/SSE:** Optional daemon on loopback `127.0.0.1`. Gated behind an OS Keyring-backed Bearer token.
