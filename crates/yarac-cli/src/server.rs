use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use yarac_core::models::*;
use yarac_core::scope::VariableScopeStack;
use yarac_core::scrubber::{ScrubConfig, SecretRegistry};
use yarac_core::watcher::{ChangeKind, WorkspaceWatcher};
use yarac_importer::{BrunoImporter, OpenApiExporter, OpenApiImporter, PostmanImporter};
use yarac_net::YaracHttpClient;
use yarac_parser::parse;
use yarac_scripting::ScriptRunner;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequestPayload {
    pub dsl: Option<String>,
    pub request: Option<YaracRequest>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunnerPayload {
    pub requests: Vec<YaracRequest>,
    #[serde(default = "default_one")]
    pub iterations: usize,
    #[serde(default = "default_one")]
    pub concurrency: usize,
    #[serde(default)]
    pub delay_ms: u64,
    pub data: Option<Vec<HashMap<String, String>>>,
}

fn default_one() -> usize {
    1
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteropImportPayload {
    pub format: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteropExportPayload {
    pub requests: Vec<YaracRequest>,
    #[serde(default = "default_export_title")]
    pub title: String,
    #[serde(default = "default_export_version")]
    pub version: String,
}

fn default_export_title() -> String {
    "Yarac API Specification".into()
}

fn default_export_version() -> String {
    "1.0.0".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRequestItem {
    pub id: String,
    #[serde(rename = "collectionId", default)]
    pub collection_id: String,
    pub name: String,
    #[serde(rename = "dslContent", default)]
    pub dsl_content: String,
    pub request: Option<YaracRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSavePayload {
    pub manifest: WorkspaceManifest,
    pub requests: Vec<WorkspaceRequestItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SseFileEvent {
    pub path: String,
    pub kind: String,
    pub content: Option<String>,
}

pub struct YaracLocalServer {
    workspace_dir: PathBuf,
    watcher: Option<Arc<WorkspaceWatcher>>,
    event_tx: broadcast::Sender<SseFileEvent>,
}

impl YaracLocalServer {
    pub fn new(workspace_dir: PathBuf) -> Self {
        let (tx, _) = broadcast::channel(128);
        let watcher = WorkspaceWatcher::new(&workspace_dir).ok().map(Arc::new);

        Self {
            workspace_dir,
            watcher,
            event_tx: tx,
        }
    }

    pub async fn run(self: Arc<Self>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        println!("🚀 Yarac Local Daemon running at http://{}", addr);

        // Background task polling the watcher and broadcasting to SSE clients
        if let Some(watcher) = &self.watcher {
            let watcher_clone = Arc::clone(watcher);
            let tx_clone = self.event_tx.clone();
            tokio::spawn(async move {
                loop {
                    while let Some(event) = watcher_clone.try_recv() {
                        let kind_str = match event.kind {
                            ChangeKind::Created => "created",
                            ChangeKind::Modified => "modified",
                            ChangeKind::Deleted => "deleted",
                        };
                        let _ = tx_clone.send(SseFileEvent {
                            path: event.path.to_string_lossy().to_string(),
                            kind: kind_str.into(),
                            content: event.content,
                        });
                    }
                    tokio::time::sleep(Duration::from_millis(150)).await;
                }
            });
        }

        loop {
            let (mut socket, _) = listener.accept().await?;
            let server = Arc::clone(&self);

            tokio::spawn(async move {
                let mut buf = vec![0u8; 65536];
                let n = match socket.read(&mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };

                let raw = String::from_utf8_lossy(&buf[..n]);
                let (head, body) = match raw.split_once("\r\n\r\n") {
                    Some((h, b)) => (h, b),
                    None => (raw.as_ref(), ""),
                };

                let mut lines = head.lines();
                let request_line = lines.next().unwrap_or("");
                let mut parts = request_line.split_whitespace();
                let method = parts.next().unwrap_or("GET");
                let path = parts.next().unwrap_or("/");

                // Handle CORS preflight
                if method == "OPTIONS" {
                    let resp = "HTTP/1.1 204 No Content\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: GET, POST, OPTIONS, PUT, DELETE\r\n\
Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
Access-Control-Max-Age: 86400\r\n\
Content-Length: 0\r\n\r\n";
                    let _ = socket.write_all(resp.as_bytes()).await;
                    return;
                }

                if method == "GET" && path == "/api/health" {
                    let resp_body = json!({ "status": "ok", "version": "0.1.0" }).to_string();
                    Self::send_json(&mut socket, 200, &resp_body).await;
                    return;
                }

                if method == "GET" && path == "/api/workspace/events" {
                    let mut rx = server.event_tx.subscribe();
                    let header = "HTTP/1.1 200 OK\r\n\
Content-Type: text/event-stream\r\n\
Cache-Control: no-cache\r\n\
Connection: keep-alive\r\n\
Access-Control-Allow-Origin: *\r\n\r\n";
                    if socket.write_all(header.as_bytes()).await.is_err() {
                        return;
                    }

                    while let Ok(event) = rx.recv().await {
                        let data = serde_json::to_string(&event).unwrap_or_default();
                        let msg = format!("event: file-changed\ndata: {}\n\n", data);
                        if socket.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                    return;
                }

                if method == "POST" && path == "/api/request/execute" {
                    let res = server.handle_execute_request(body).await;
                    Self::send_json(&mut socket, 200, &serde_json::to_string(&res).unwrap_or_default()).await;
                    return;
                }

                if method == "POST" && path == "/api/runner/execute" {
                    let res = server.handle_runner_execute(body).await;
                    Self::send_json(&mut socket, 200, &serde_json::to_string(&res).unwrap_or_default()).await;
                    return;
                }

                if method == "POST" && path == "/api/interop/import" {
                    let res = server.handle_interop_import(body).await;
                    Self::send_json(&mut socket, 200, &serde_json::to_string(&res).unwrap_or_default()).await;
                    return;
                }

                if method == "POST" && path == "/api/interop/export" {
                    let res = server.handle_interop_export(body).await;
                    Self::send_json(&mut socket, 200, &serde_json::to_string(&res).unwrap_or_default()).await;
                    return;
                }

                if method == "GET" && path == "/api/workspace" {
                    let res = server.handle_load_workspace().await;
                    Self::send_json(&mut socket, 200, &serde_json::to_string(&res).unwrap_or_default()).await;
                    return;
                }

                if method == "POST" && path == "/api/workspace/save" {
                    let res = server.handle_save_workspace(body).await;
                    Self::send_json(&mut socket, 200, &serde_json::to_string(&res).unwrap_or_default()).await;
                    return;
                }

                let not_found = json!({ "error": "Not Found" }).to_string();
                Self::send_json(&mut socket, 404, &not_found).await;
            });
        }
    }

    async fn send_json(socket: &mut tokio::net::TcpStream, status: u16, body: &str) {
        let status_text = match status {
            200 => "OK",
            400 => "Bad Request",
            404 => "Not Found",
            _ => "Error",
        };
        let response = format!(
            "HTTP/1.1 {} {}\r\n\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
Content-Type: application/json\r\n\
Content-Length: {}\r\n\
Connection: close\r\n\r\n{}",
            status,
            status_text,
            body.len(),
            body
        );
        let _ = socket.write_all(response.as_bytes()).await;
    }

    async fn handle_execute_request(&self, body: &str) -> Value {
        let payload: ExecuteRequestPayload = match serde_json::from_str(body) {
            Ok(p) => p,
            Err(e) => return json!({ "error": format!("Invalid payload: {}", e) }),
        };

        let mut req = if let Some(r) = payload.request {
            r
        } else if let Some(dsl) = payload.dsl {
            match parse(&dsl).and_then(|cst| cst.to_request()) {
                Ok(r) => r,
                Err(e) => return json!({ "error": format!("DSL parse error: {}", e) }),
            }
        } else {
            return json!({ "error": "Missing request or DSL content" });
        };

        let mut scope = VariableScopeStack::new();
        for (k, v) in payload.env_vars {
            scope.environment.insert(k, VariableValue::new(v));
        }

        let secrets: HashMap<String, String> = HashMap::new();
        let runner = ScriptRunner::new(Duration::from_millis(5000));

        if let Some(pre_script) = req.scripts.pre_request.clone() {
            let _ = runner.run_pre_request(&pre_script, &mut req, &secrets).await;
        }

        let interpolated_url = scope.interpolate(&req.url);

        let client = match YaracHttpClient::new(req.settings.verify_ssl, req.settings.follow_redirects, req.settings.timeout_ms, None) {
            Ok(c) => c,
            Err(e) => return json!({ "error": format!("HTTP client initialization failed: {}", e) }),
        };

        let client_res = client.execute(&req, &self.workspace_dir, &interpolated_url).await;

        let mut report = match client_res {
            Ok(rep) => rep,
            Err(e) => ExecutionReport {
                request_name: req.name.clone(),
                method: req.method.clone(),
                url: req.url.clone(),
                status_code: 0,
                duration_ms: 0,
                response_headers: indexmap::IndexMap::new(),
                response_body: String::new(),
                assertions: Vec::new(),
                logs: Vec::new(),
                error: Some(e.to_string()),
            },
        };

        if let Some(post_script) = &req.scripts.post_response {
            let _ = runner.run_post_response(post_script, &mut report, &secrets).await;
        }

        let mut registry = SecretRegistry::new();
        for sec in secrets.values() {
            registry.register(sec);
        }
        report.response_body = registry.scrub(&report.response_body, &ScrubConfig::default());

        json!({ "report": report })
    }

    async fn handle_runner_execute(&self, body: &str) -> Value {
        let payload: RunnerPayload = match serde_json::from_str(body) {
            Ok(p) => p,
            Err(e) => return json!({ "error": format!("Invalid runner payload: {}", e) }),
        };

        let mut histogram = match Histogram::<u64>::new_with_bounds(1, 60_000, 3) {
            Ok(h) => h,
            Err(e) => return json!({ "error": format!("Histogram init failed: {}", e) }),
        };

        let mut logs = Vec::new();
        let start_time = Instant::now();
        let total_runs = payload.iterations * payload.requests.len();

        let client = match YaracHttpClient::new(true, true, 15000, None) {
            Ok(c) => c,
            Err(e) => return json!({ "error": format!("HTTP client initialization failed: {}", e) }),
        };

        for iter in 0..payload.iterations {
            for req in &payload.requests {
                let req_start = Instant::now();
                let res = client.execute(req, &self.workspace_dir, &req.url).await;
                let duration = req_start.elapsed().as_millis() as u64;

                let _ = histogram.record(duration.max(1));

                let (status, passed) = match res {
                    Ok(r) => (r.status_code, r.all_passed()),
                    Err(_) => (0, false),
                };

                logs.push(json!({
                    "iteration": iter + 1,
                    "requestName": req.name,
                    "method": req.method,
                    "status": status,
                    "durationMs": duration,
                    "passed": passed,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));

                if payload.delay_ms > 0 {
                    tokio::time::sleep(Duration::from_millis(payload.delay_ms)).await;
                }
            }
        }

        let elapsed_sec = start_time.elapsed().as_secs_f64().max(0.01);
        let failures = logs.iter().filter(|l| l.get("passed") == Some(&Value::Bool(false))).count();
        let error_rate = if total_runs > 0 {
            (failures as f64 / total_runs as f64) * 100.0
        } else {
            0.0
        };

        json!({
            "stats": {
                "count": histogram.len(),
                "min": histogram.min(),
                "max": histogram.max(),
                "mean": histogram.mean() as u64,
                "p50": histogram.value_at_quantile(0.50),
                "p90": histogram.value_at_quantile(0.90),
                "p95": histogram.value_at_quantile(0.95),
                "p99": histogram.value_at_quantile(0.99),
                "rps": (total_runs as f64 / elapsed_sec).round(),
                "errorRate": error_rate.round(),
                "successCount": total_runs - failures,
                "failureCount": failures,
            },
            "logs": logs
        })
    }

    async fn handle_interop_import(&self, body: &str) -> Value {
        let payload: InteropImportPayload = match serde_json::from_str(body) {
            Ok(p) => p,
            Err(e) => return json!({ "error": format!("Invalid import payload: {}", e) }),
        };

        match payload.format.to_lowercase().as_str() {
            "bruno" => match BrunoImporter::import_bru(&payload.content) {
                Ok(req) => json!({ "requests": [req] }),
                Err(e) => json!({ "error": format!("Bruno import error: {}", e) }),
            },
            "postman" => match PostmanImporter::import_collection(&payload.content) {
                Ok(reqs) => json!({ "requests": reqs }),
                Err(e) => json!({ "error": format!("Postman import error: {}", e) }),
            },
            "openapi" => match OpenApiImporter::import_spec(&payload.content) {
                Ok(reqs) => json!({ "requests": reqs }),
                Err(e) => json!({ "error": format!("OpenAPI import error: {}", e) }),
            },
            fmt => json!({ "error": format!("Unsupported format: {}", fmt) }),
        }
    }

    async fn handle_interop_export(&self, body: &str) -> Value {
        let payload: InteropExportPayload = match serde_json::from_str(body) {
            Ok(p) => p,
            Err(e) => return json!({ "error": format!("Invalid export payload: {}", e) }),
        };

        match OpenApiExporter::export_openapi_v3_1(&payload.requests, &payload.title, &payload.version) {
            Ok(spec) => json!({ "spec": spec }),
            Err(e) => json!({ "error": format!("Export failed: {}", e) }),
        }
    }

    async fn handle_load_workspace(&self) -> Value {
        let manifest_path = self.workspace_dir.join("workspace.json");
        let manifest: Option<WorkspaceManifest> = if manifest_path.is_file() {
            std::fs::read_to_string(&manifest_path)
                .ok()
                .and_then(|c| serde_json::from_str(&c).ok())
        } else {
            None
        };

        // Scan directory for all .yarac files
        let mut loaded_requests = Vec::new();
        Self::scan_yarac_files(&self.workspace_dir, &mut loaded_requests);

        json!({
            "manifest": manifest,
            "requests": loaded_requests
        })
    }

    async fn handle_save_workspace(&self, body: &str) -> Value {
        let payload: WorkspaceSavePayload = match serde_json::from_str(body) {
            Ok(p) => p,
            Err(e) => return json!({ "error": format!("Invalid workspace payload: {}", e) }),
        };

        let _ = std::fs::create_dir_all(&self.workspace_dir);

        // Save workspace.json manifest
        let manifest_path = self.workspace_dir.join("workspace.json");
        if let Some(w) = &self.watcher {
            w.mark_self_write(&manifest_path, Duration::from_millis(1000));
        }
        let manifest_json = serde_json::to_string_pretty(&payload.manifest).unwrap_or_default();
        if let Err(e) = std::fs::write(&manifest_path, manifest_json) {
            return json!({ "error": format!("Failed to write manifest: {}", e) });
        }

        // Save each request to disk in collection folders
        for item in &payload.requests {
            let col_name = payload
                .manifest
                .collections
                .iter()
                .find(|c| c.id == item.collection_id)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "General".into());

            let safe_col = col_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            let safe_name = item.name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");

            let col_dir = self.workspace_dir.join(&safe_col);
            let _ = std::fs::create_dir_all(&col_dir);

            let req_path = col_dir.join(format!("{}.yarac", safe_name));
            if let Some(w) = &self.watcher {
                w.mark_self_write(&req_path, Duration::from_millis(1000));
            }

            let dsl = if !item.dsl_content.trim().is_empty() {
                item.dsl_content.clone()
            } else if let Some(req) = &item.request {
                yarac_importer::serialize_to_yarac(req)
            } else {
                format!("version = \"1.0\"\nname = \"{}\"\nmethod = \"GET\"\nurl = \"http://localhost\"\n", item.name)
            };
            let _ = std::fs::write(&req_path, dsl);
        }

        json!({ "status": "ok" })
    }

    fn scan_yarac_files(dir: &Path, out: &mut Vec<Value>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::scan_yarac_files(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("yarac") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let col_name = path
                            .parent()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("General");
                        let req_name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Request");
                        out.push(json!({
                            "collectionName": col_name,
                            "fileName": format!("{}.yarac", req_name),
                            "name": req_name,
                            "path": path.to_string_lossy(),
                            "dslContent": content
                        }));
                    }
                }
            }
        }
    }
}
