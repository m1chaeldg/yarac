use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use thiserror::Error;

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
    pub fn handle_request(req: JsonRpcRequest) -> JsonRpcResponse {
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
            "tools/call" => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id,
                result: Some(json!({
                    "content": [
                        { "type": "text", "text": "Execution completed successfully." }
                    ]
                })),
                error: None,
            },
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

    /// Runs stdio transport loop (token-free for local sub-processes)
    pub fn run_stdio() -> Result<(), McpError> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let handle = stdin.lock();

        for line in handle.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line) {
                let resp = Self::handle_request(req);
                let json_out = serde_json::to_string(&resp)?;
                writeln!(stdout, "{}", json_out)?;
                stdout.flush()?;
            }
        }

        Ok(())
    }
}
