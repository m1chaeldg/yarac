# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-10

### Added
- **Lossless CST Parser & Serializer (`yarac-parser`)**: Built with parser-combinator `nom`, preserving raw formatting, indentation, comments, CRLF/LF newlines, and multi-line fenced code blocks. Verified with round-trip invariant `assert_eq!(serialize(parse(input)), input)`.
- **Core Domain & Security Engine (`yarac-core`)**: Domain models, scope variable resolver (Local $\to$ Collection $\to$ Environment $\to$ Workspace $\to$ Global), dynamic template generation (`{{$uuid}}`, `{{$timestamp}}`, `{{$randomInt}}`), and workspace-relative path anchoring.
- **Token-Boundary Secret Scrubber**: Substring scrubbing for secrets $\ge 6$ characters and lexical token-boundary matching for secrets $< 6$ characters per `AGENTS.md`.
- **HTTP Engine (`yarac-net`)**: Asynchronous HTTP/1.1 and HTTP/2 execution backed by `reqwest`, with isolated cookie store per environment, multipart uploads, SSL controls, and redirect policies.
- **Ephemeral Script Sandbox (`yarac-scripting`)**: Script execution runner enforcing a 5000ms interrupt timeout.
- **Postman v2.1 Importer (`yarac-importer`)**: Direct conversion of Postman v2.1 collection JSON files to `.yarac` request structures without artificial runtime compatibility shims.
- **Hybrid Git Engine (`yarac-git`)**: Local repository status and staging/commit logic.
- **Model Context Protocol (`yarac-mcp`)**: JSON-RPC 2.0 MCP server exposing tools (`list_requests`, `run_request`) over `stdio` transport.
- **Headless CLI Runner (`yarac-cli`)**: Command-line executable supporting `run` and `mcp` subcommands with deterministic exit codes (`0`: success, `1`: assertion failure, `2`: config/syntax error, `3`: network error).
- **Test Suite**: Comprehensive unit and integration test suite across the entire workspace.
