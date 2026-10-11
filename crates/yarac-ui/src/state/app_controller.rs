use std::path::PathBuf;
use std::sync::Arc;
use yarac_core::models::*;
use yarac_net::YaracHttpClient;
use crate::state::workspace_state::WorkspaceState;
use crate::state::request_state::RequestState;
use crate::state::response_state::{ResponseState, AssertionResult};
use crate::state::environment_state::EnvironmentState;
use crate::state::runner_state::{RunnerState, IterationResult, RunnerStats};

#[derive(Clone)]
pub struct AppController {
    pub workspace: WorkspaceState,
    pub environment: EnvironmentState,
    pub active_response: ResponseState,
    pub runner: RunnerState,
    pub http_client: Arc<YaracHttpClient>,
    pub workspace_root: PathBuf,
}

impl AppController {
    pub fn new() -> Self {
        let http_client = Arc::new(
            YaracHttpClient::new(true, true, 5, 10000, None)
                .expect("Failed to initialize Yarac HTTP client")
        );
        let mut ws = WorkspaceState::new("Default Workspace");
        let col_id = ws.add_collection("Default Collection");
        ws.create_request(&col_id, "Echo GET", "GET", "https://postman-echo.com/get?foo1=bar1&foo2=bar2");

        Self {
            workspace: ws,
            environment: EnvironmentState::new("Default Environment"),
            active_response: ResponseState::empty(),
            runner: RunnerState::new(Vec::new(), 1, 1, 0),
            http_client,
            workspace_root: PathBuf::from("."),
        }
    }

    /// Executes the active or given request through the native Rust Yarac network engine
    pub async fn execute_request(&mut self, request: &RequestState) -> Result<ResponseState, String> {
        let interpolated_url = self.environment.interpolate(&request.url);

        // Convert RequestState to YaracRequest domain model
        let mut headers_map = indexmap::IndexMap::new();
        for h in &request.headers {
            if h.enabled && !h.key.ends_with('~') && !h.key.trim().is_empty() {
                headers_map.insert(h.key.trim().to_string(), self.environment.interpolate(&h.value));
            }
        }

        let mut query_params = indexmap::IndexMap::new();
        for p in &request.params {
            if p.enabled && !p.key.ends_with('~') && !p.key.trim().is_empty() {
                query_params.insert(p.key.trim().to_string(), self.environment.interpolate(&p.value));
            }
        }

        let body = match request.body_type.as_str() {
            "json" => BodyType::Json(self.environment.interpolate(&request.raw_body)),
            "raw" => BodyType::Raw {
                content: self.environment.interpolate(&request.raw_body),
                content_type: Some("text/plain".into()),
            },
            _ => BodyType::None,
        };

        let auth = if request.auth_type == "bearer" {
            request.bearer_token.as_ref().map(|tok| AuthConfig::Bearer {
                token: self.environment.interpolate(tok),
            })
        } else {
            None
        };

        let yarac_req = YaracRequest {
            version: "1.0".to_string(),
            name: request.name.clone(),
            method: request.method.clone(),
            url: interpolated_url.clone(),
            headers: headers_map,
            query_params,
            body,
            auth,
            settings: Settings::default(),
            flow: None,
            variables: indexmap::IndexMap::new(),
            scripts: ScriptHooks::default(),
            assert: None,
            tests: None,
        };

        let report = self.http_client.execute(&yarac_req, &self.workspace_root, &interpolated_url)
            .await
            .map_err(|e| e.to_string())?;

        let mut assertions = Vec::new();
        assertions.push(AssertionResult {
            name: format!("Status is {}", report.status_code),
            passed: report.status_code >= 200 && report.status_code < 400,
            message: None,
        });

        let mut headers_std = std::collections::HashMap::new();
        for (k, v) in report.response_headers {
            headers_std.insert(k, v);
        }

        let resp_state = ResponseState {
            status: report.status_code,
            status_text: format!("Status {}", report.status_code),
            duration_ms: report.duration_ms,
            size_bytes: report.response_body.len(),
            headers: headers_std,
            body: report.response_body,
            timestamp: chrono::Local::now().format("%H:%M:%S").to_string(),
            assertions,
            logs: vec![format!("Completed in {}ms", report.duration_ms)],
            error: None,
        };

        self.active_response = resp_state.clone();
        Ok(resp_state)
    }

    /// Runs all requests in the active collection with iterations
    pub async fn execute_collection_runner(
        &mut self,
        requests: &[RequestState],
        iterations: usize,
    ) -> Result<RunnerStats, String> {
        let mut runner = RunnerState::new(
            requests.iter().map(|r| r.id.clone()).collect(),
            iterations,
            1,
            0,
        );

        for iter in 1..=iterations {
            for req in requests {
                let res = self.execute_request(req).await;
                match res {
                    Ok(resp) => {
                        runner.record_result(IterationResult {
                            iteration: iter,
                            request_name: req.name.clone(),
                            status_code: resp.status,
                            duration_ms: resp.duration_ms,
                            passed: resp.status >= 200 && resp.status < 400,
                            error: None,
                        });
                    }
                    Err(err) => {
                        runner.record_result(IterationResult {
                            iteration: iter,
                            request_name: req.name.clone(),
                            status_code: 0,
                            duration_ms: 0,
                            passed: false,
                            error: Some(err),
                        });
                    }
                }
            }
        }

        let stats = runner.compute_stats();
        self.runner = runner;
        Ok(stats)
    }
}
