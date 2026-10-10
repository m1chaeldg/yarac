# Proposal: Yarac (Yet Another REST API Client)

**Mission:** Deliver a blazingly fast, low-memory (<100MB RAM), local-first API testing client that bridges developer-native workflows with enterprise security and AI extensibility.

## Core Philosophy
- **Local-First & Offline-Native:** Zero telemetry. Zero mandatory external network calls. All collections, workspaces, and environments reside as plain text on the local filesystem.
- **Git-Native:** Collections are stored using the custom `.yarac` DSL, fully compatible with standard Git workflows and clean diff viewers.
- **Standards-Driven Scripting:** Uses embedded `deno_core` to provide a standard ECMAScript/Web standards runtime. No custom compatibility bloat; users write modern, standard JavaScript.
- **Relative Portability:** All file references (payloads, attachments, certs) use workspace-relative paths, ensuring multi-platform collaboration across Git.
- **Secure AI Extensibility:** Integrates with local LLMs and AI agents via dual-transport Model Context Protocol (MCP) support.

## MVP Scope (Phase 1)
- Core HTTP/1.1 & HTTP/2 execution with environment-isolated Cookie Jar.
- Lossless CST parser and round-trip serializer for `.yarac` files.
- Embedded `deno_core` runtime with strict execution timeouts and ephemeral context isolation.
- Basic Postman v2.1 import (schema conversion only, without legacy compatibility shims).
- Hybrid Git engine: `gitoxide` for fast local status/diff/commit with system `git` CLI fallback.
- Dual-transport MCP server (`stdio` + authenticated local HTTP/SSE).
- CI/CD headless runner (`yarac-cli`) with deterministic exit codes.
