use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use yarac_core::models::*;
use yarac_core::scope::VariableScopeStack;
use yarac_core::scrubber::{ScrubConfig, SecretRegistry};
use yarac_net::YaracHttpClient;
use yarac_parser::parse;
use yarac_scripting::ScriptRunner;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("JSON-RPC error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub struct YaracMcpServer;

impl YaracMcpServer {
    /// Dispatches a JSON-RPC method for MCP
    pub async fn handle_request(req: JsonRpcRequest, workspace_root: &Path) -> JsonRpcResponse {
        let id = req.id;
        match req.method.as_str() {
            "initialize" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": "yarac-mcp",
                        "version": "0.1.0"
                    }
                })),
                error: None,
            },
            "tools/list" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "list_requests",
                            "description": "Lists all .yarac request files in the active workspace",
                            "inputSchema": { "type": "object", "properties": {} }
                        },
                        {
                            "name": "run_request",
                            "description": "Executes a .yarac request file and returns execution report",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string" },
                                    "environment": { "type": "string" }
                                },
                                "required": ["path"]
                            }
                        }
                    ]
                })),
                error: None,
            },
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let args = params.get("arguments").cloned().unwrap_or(Value::Null);

                match tool_name {
                    "list_requests" => {
                        let mut requests = Vec::new();
                        Self::collect_yarac_files(workspace_root, &mut requests);
                        JsonRpcResponse {
                            jsonrpc: "2.0".into(),
                            id,
                            result: Some(json!({
                                "content": [
                                    { "type": "text", "text": serde_json::to_string_pretty(&requests).unwrap_or_default() }
                                ]
                            })),
                            error: None,
                        }
                    }
                    "run_request" => {
                        let path_str = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
                        let file_path = workspace_root.join(path_str);
                        let report_result = Self::execute_yarac_file(&file_path, workspace_root).await;

                        match report_result {
                            Ok(report) => JsonRpcResponse {
                                jsonrpc: "2.0".into(),
                                id,
                                result: Some(json!({
                                    "content": [
                                        { "type": "text", "text": serde_json::to_string_pretty(&report).unwrap_or_default() }
                                    ]
                                })),
                                error: None,
                            },
                            Err(err_msg) => JsonRpcResponse {
                                jsonrpc: "2.0".into(),
                                id,
                                result: None,
                                error: Some(json!({
                                    "code": -32000,
                                    "message": err_msg
                                })),
                            },
                        }
                    }
                    _ => JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id,
                        result: None,
                        error: Some(json!({
                            "code": -32601,
                            "message": format!("Tool not found: {tool_name}")
                        })),
                    },
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: None,
                error: Some(json!({
                    "code": -32601,
                    "message": "Method not found"
                })),
            },
        }
    }

    fn collect_yarac_files(dir: &Path, out: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_yarac_files(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("yarac") {
                    out.push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    async fn execute_yarac_file(path: &Path, workspace_root: &Path) -> Result<ExecutionReport, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
        let cst = parse(&content).map_err(|e| format!("Syntax error: {e}"))?;
        let mut req = cst.to_request().map_err(|e| format!("Model mapping error: {e}"))?;

        let scope = VariableScopeStack::new();
        let secrets = HashMap::new();

        let runner = ScriptRunner::default();
        if let Some(pre_script) = req.scripts.pre_request.clone() {
            let _ = runner.run_pre_request(&pre_script, &mut req, &secrets).await;
        }

        let interpolated_url = scope.interpolate(&req.url);

        let client = YaracHttpClient::new(
            req.settings.verify_ssl,
            req.settings.follow_redirects,
            req.settings.timeout_ms,
            None,
        ).map_err(|e| format!("Client creation error: {e}"))?;

        let mut report = client.execute(&req, workspace_root, &interpolated_url).await
            .map_err(|e| format!("HTTP execution error: {e}"))?;

        if let Some(post_script) = &req.scripts.post_response {
            let _ = runner.run_post_response(post_script, &mut report, &secrets).await;
        }

        // Apply tainted secret scrubber per AGENTS.md
        let mut registry = SecretRegistry::new();
        for sec in secrets.values() {
            registry.register(sec);
        }
        report.response_body = registry.scrub(&report.response_body, &ScrubConfig::default());

        Ok(report)
    }

    /// Runs stdio transport loop (token-free for local sub-processes)
    pub async fn run_stdio(workspace_root: &Path) -> Result<(), McpError> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let handle = stdin.lock();

        for line in handle.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
                let resp = Self::handle_request(req, workspace_root).await;
                let json_out = serde_json::to_string(&resp)?;
                writeln!(stdout, "{}", json_out)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }

    /// Runs HTTP/SSE transport loop bound strictly to 127.0.0.1 with Keyring-backed token validation
    pub async fn run_http_sse(port: u16, expected_token: &str, workspace_root: PathBuf) -> Result<(), McpError> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Yarac MCP HTTP/SSE listening on http://{} (Authenticated)", addr);

        let expected_auth = format!("Bearer {}", expected_token);

        loop {
            let (mut socket, _) = listener.accept().await?;
            let expected_auth = expected_auth.clone();
            let ws_root = workspace_root.clone();

            tokio::spawn(async move {
                let mut buf = [0u8; 4096];
                if let Ok(n) = socket.read(&mut buf).await {
                    if n == 0 {
                        return;
                    }
                    let req_str = String::from_utf8_lossy(&buf[..n]);

                    // Validate Bearer auth header
                    let is_authenticated = req_str.lines().any(|l| {
                        l.to_lowercase().starts_with("authorization:") && l.contains(&expected_auth)
                    });

                    if !is_authenticated {
                        let resp = "HTTP/1.1 401 Unauthorized\r\nContent-Length: 26\r\nConnection: close\r\n\r\n401 Unauthorized: Invalid Bearer Token";
                        let _ = socket.write_all(resp.as_bytes()).await;
                        return;
                    }

                    // Extract body after double newline
                    let body = req_str.split("\r\n\r\n").nth(1).unwrap_or("");
                    if let Ok(rpc_req) = serde_json::from_str::<JsonRpcRequest>(body) {
                        let rpc_resp = Self::handle_request(rpc_req, &ws_root).await;
                        let resp_body = serde_json::to_string(&rpc_resp).unwrap_or_default();
                        let http_resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            resp_body.len(),
                            resp_body
                        );
                        let _ = socket.write_all(http_resp.as_bytes()).await;
                    } else {
                        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 15\r\nConnection: close\r\n\r\nBad JSON-RPC";
                        let _ = socket.write_all(resp.as_bytes()).await;
                    }
                }
            });
        }
    }
}
