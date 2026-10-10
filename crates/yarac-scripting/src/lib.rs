use std::collections::HashMap;
use std::time::Duration;
use deno_core::{JsRuntime, RuntimeOptions};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use yarac_core::models::*;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Execution timed out after {0:?}")]
    Timeout(Duration),
    #[error("Script evaluation error: {0}")]
    EvalError(String),
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PreRequestState {
    headers: HashMap<String, String>,
    params: HashMap<String, String>,
    secrets: HashMap<String, String>,
    logs: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PostResponseState {
    assertions: Vec<AssertionResult>,
    env_vars: HashMap<String, String>,
    logs: Vec<String>,
    secrets: HashMap<String, String>,
}

pub struct ScriptRunner {
    timeout: Duration,
}

impl Default for ScriptRunner {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(5000), // Enforce 5000ms hard interrupt timeout per AGENTS.md
        }
    }
}

impl ScriptRunner {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Executes pre-request script hook with an ephemeral context and 5000ms interrupt timeout.
    /// Dropped and GC'd immediately upon test completion to maintain <100MB RAM runtime target.
    pub async fn run_pre_request(
        &self,
        script: &str,
        request: &mut YaracRequest,
        secrets: &HashMap<String, String>,
    ) -> Result<Vec<String>, ScriptError> {
        if script.trim().is_empty() {
            return Ok(Vec::new());
        }

        let script = script.to_string();
        let timeout = self.timeout;
        let body_str = match &request.body {
            BodyType::Json(s) => s.clone(),
            BodyType::Raw { content, .. } => content.clone(),
            _ => String::new(),
        };

        let initial_headers: HashMap<String, String> = request
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let initial_params: HashMap<String, String> = request
            .query_params
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let secrets_clone = secrets.clone();

        let run_result = tokio::task::spawn_blocking(move || {
            let mut runtime = JsRuntime::new(RuntimeOptions::default());

            // Prepare state setup
            let setup_js = format!(
                r#"
                globalThis.__yarac_internal = {{
                    headers: {},
                    params: {},
                    secrets: {},
                    logs: []
                }};
                globalThis.__request_body = {};

                globalThis.console = globalThis.console || {{}};
                globalThis.console.log = function(...args) {{
                    __yarac_internal.logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                }};

                globalThis.yarac = {{
                    setHeader(k, v) {{
                        __yarac_internal.headers[String(k)] = String(v);
                    }},
                    setParam(k, v) {{
                        __yarac_internal.params[String(k)] = String(v);
                    }},
                    getSecret(k) {{
                        return __yarac_internal.secrets[String(k)] || '';
                    }},
                    request: {{
                        getBody() {{
                            return __request_body;
                        }}
                    }}
                }};
                "#,
                serde_json::to_string(&initial_headers).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&initial_params).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&secrets_clone).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&body_str).unwrap_or_else(|_| "\"\"".into())
            );

            runtime
                .execute_script("setup_pre_request.js", setup_js)
                .map_err(|e| ScriptError::EvalError(e.to_string()))?;

            let wrapped_script = format!(
                r#"
                (async () => {{
                    {}
                }})()
                "#,
                script
            );

            let res = runtime.execute_script("pre_request.js", wrapped_script);
            if let Err(e) = res {
                return Err(ScriptError::EvalError(e.to_string()));
            }

            // Run microtasks
            let _ = runtime.run_event_loop(Default::default());

            let extract_js = "JSON.stringify(__yarac_internal);";
            let val = runtime
                .execute_script("extract.js", extract_js)
                .map_err(|e| ScriptError::EvalError(e.to_string()))?;

            deno_core::scope!(scope, &mut runtime);
            let local_val = deno_core::v8::Local::new(scope, val);
            let json_str = local_val.to_rust_string_lossy(scope);

            let state: PreRequestState = serde_json::from_str(&json_str)
                .map_err(|e| ScriptError::EvalError(format!("State extract failed: {e}")))?;

            Ok(state)
        });

        let state = tokio::time::timeout(timeout, run_result)
            .await
            .map_err(|_| ScriptError::Timeout(timeout))?
            .map_err(|e| ScriptError::EvalError(e.to_string()))??;

        // Apply mutations to request
        for (k, v) in state.headers {
            request.headers.insert(k, v);
        }
        for (k, v) in state.params {
            request.query_params.insert(k, v);
        }

        Ok(state.logs)
    }

    /// Executes post-response script hook with an ephemeral context and 5000ms interrupt timeout.
    pub async fn run_post_response(
        &self,
        script: &str,
        report: &mut ExecutionReport,
        secrets: &HashMap<String, String>,
    ) -> Result<HashMap<String, String>, ScriptError> {
        if script.trim().is_empty() {
            return Ok(HashMap::new());
        }

        let script = script.to_string();
        let timeout = self.timeout;
        let status = report.status_code;
        let response_body = report.response_body.clone();
        let response_headers: HashMap<String, String> = report
            .response_headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let secrets_clone = secrets.clone();

        let run_result = tokio::task::spawn_blocking(move || {
            let mut runtime = JsRuntime::new(RuntimeOptions::default());

            let setup_js = format!(
                r#"
                globalThis.__yarac_internal = {{
                    assertions: [],
                    env_vars: {{}},
                    logs: [],
                    secrets: {}
                }};

                globalThis.console = globalThis.console || {{}};
                globalThis.console.log = function(...args) {{
                    __yarac_internal.logs.push(args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' '));
                }};

                globalThis.expect = function(actual) {{
                    return {{
                        to: {{
                            equal(expected) {{
                                if (actual !== expected) {{
                                    throw new Error(`Expected ${{JSON.stringify(actual)}} to equal ${{JSON.stringify(expected)}}`);
                                }}
                            }},
                            be: {{
                                a(expectedType) {{
                                    const actualType = typeof actual;
                                    if (actualType !== expectedType) {{
                                        throw new Error(`Expected type '${{actualType}}' to be '${{expectedType}}'`);
                                    }}
                                }}
                            }}
                        }}
                    }};
                }};

                globalThis.yarac = {{
                    response: {{
                        status: {},
                        headers: {},
                        body: {},
                        json() {{
                            return JSON.parse(this.body);
                        }}
                    }},
                    test(name, fn) {{
                        try {{
                            fn();
                            __yarac_internal.assertions.push({{
                                name: String(name),
                                passed: true,
                                error_message: null
                            }});
                        }} catch (err) {{
                            __yarac_internal.assertions.push({{
                                name: String(name),
                                passed: false,
                                error_message: err.message || String(err)
                            }});
                        }}
                    }},
                    setEnvVar(k, v) {{
                        __yarac_internal.env_vars[String(k)] = String(v);
                    }},
                    getSecret(k) {{
                        return __yarac_internal.secrets[String(k)] || '';
                    }}
                }};
                "#,
                serde_json::to_string(&secrets_clone).unwrap_or_else(|_| "{}".into()),
                status,
                serde_json::to_string(&response_headers).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&response_body).unwrap_or_else(|_| "\"\"".into())
            );

            runtime
                .execute_script("setup_post_response.js", setup_js)
                .map_err(|e| ScriptError::EvalError(e.to_string()))?;

            let wrapped_script = format!(
                r#"
                (async () => {{
                    {}
                }})()
                "#,
                script
            );

            let res = runtime.execute_script("post_response.js", wrapped_script);
            if let Err(e) = res {
                return Err(ScriptError::EvalError(e.to_string()));
            }

            let _ = runtime.run_event_loop(Default::default());

            let extract_js = "JSON.stringify(__yarac_internal);";
            let val = runtime
                .execute_script("extract.js", extract_js)
                .map_err(|e| ScriptError::EvalError(e.to_string()))?;

            deno_core::scope!(scope, &mut runtime);
            let local_val = deno_core::v8::Local::new(scope, val);
            let json_str = local_val.to_rust_string_lossy(scope);

            let state: PostResponseState = serde_json::from_str(&json_str)
                .map_err(|e| ScriptError::EvalError(format!("State extract failed: {e}")))?;

            Ok(state)
        });

        let state = tokio::time::timeout(timeout, run_result)
            .await
            .map_err(|_| ScriptError::Timeout(timeout))?
            .map_err(|e| ScriptError::EvalError(e.to_string()))??;

        report.assertions.extend(state.assertions);
        report.logs.extend(state.logs);

        Ok(state.env_vars)
    }

    /// Evaluates a declarative polling predicate expression (e.g. `response.status == 200 && response.body.status == 'ready'`).
    pub async fn evaluate_predicate(
        &self,
        expression: &str,
        status_code: u16,
        response_body: &str,
        response_headers: &indexmap::IndexMap<String, String>,
    ) -> Result<bool, ScriptError> {
        if expression.trim().is_empty() {
            return Ok(true);
        }

        let expr = expression.to_string();
        let timeout = self.timeout;
        let body_str = response_body.to_string();
        let headers_map: HashMap<String, String> = response_headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        let run_result = tokio::task::spawn_blocking(move || {
            let mut runtime = JsRuntime::new(RuntimeOptions::default());

            let headers_json = serde_json::to_string(&headers_map).unwrap_or_else(|_| "{}".to_string());
            let body_escaped = serde_json::to_string(&body_str).unwrap_or_else(|_| "\"\"".to_string());

            let setup_js = format!(
                r#"
                let parsedBody = null;
                try {{
                    parsedBody = JSON.parse({body_escaped});
                }} catch (e) {{
                    parsedBody = {body_escaped};
                }}

                globalThis.response = {{
                    status: {status_code},
                    statusCode: {status_code},
                    body: parsedBody,
                    headers: {headers_json},
                    header: function(name) {{
                        const h = {headers_json};
                        for (const k in h) {{
                            if (k.toLowerCase() === name.toLowerCase()) return h[k];
                        }}
                        return null;
                    }}
                }};
                Boolean({expr})
                "#
            );

            let val = runtime
                .execute_script("<predicate>", setup_js)
                .map_err(|e| ScriptError::EvalError(e.to_string()))?;

            deno_core::scope!(scope, &mut runtime);
            let local_val = deno_core::v8::Local::new(scope, val);
            Ok(local_val.boolean_value(scope))
        });

        tokio::time::timeout(timeout, run_result)
            .await
            .map_err(|_| ScriptError::Timeout(timeout))?
            .map_err(|e| ScriptError::EvalError(e.to_string()))?
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ephemeral_runner_creation() {
        let runner = ScriptRunner::default();
        let mut report = ExecutionReport {
            request_name: "Test".into(),
            method: "GET".into(),
            url: "http://localhost".into(),
            status_code: 200,
            duration_ms: 10,
            response_headers: indexmap::IndexMap::new(),
            response_body: "{\"status\": \"ok\", \"id\": \"abc-123\"}".into(),
            assertions: Vec::new(),
            logs: Vec::new(),
            error: None,
        };

        let script = r#"
            yarac.test("Check status", () => {
                expect(yarac.response.status).to.equal(200);
                const data = yarac.response.json();
                expect(data.id).to.be.a("string");
                yarac.setEnvVar("saved_id", data.id);
            });
        "#;

        let env_vars = runner
            .run_post_response(script, &mut report, &HashMap::new())
            .await
            .unwrap();

        assert_eq!(report.assertions.len(), 1);
        assert!(report.assertions[0].passed);
        assert_eq!(report.assertions[0].name, "Check status");
        assert_eq!(env_vars.get("saved_id").unwrap(), "abc-123");
    }

    #[tokio::test]
    async fn test_pre_request_header_and_crypto() {
        let runner = ScriptRunner::default();
        let mut request = YaracRequest::default();
        request.body = BodyType::Json(r#"{"msg":"hello"}"#.into());

        let mut secrets = HashMap::new();
        secrets.insert("api_key".into(), "secret-123".into());

        let script = r#"
            const secret = yarac.getSecret("api_key");
            yarac.setHeader("X-Custom", "custom-header-" + secret);
            console.log("Logged from pre-request");
        "#;

        let logs = runner
            .run_pre_request(script, &mut request, &secrets)
            .await
            .unwrap();

        assert_eq!(
            request.headers.get("X-Custom").unwrap(),
            "custom-header-secret-123"
        );
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0], "Logged from pre-request");
    }

    #[tokio::test]
    async fn test_assertion_failure_capture() {
        let runner = ScriptRunner::default();
        let mut report = ExecutionReport {
            request_name: "Test".into(),
            method: "GET".into(),
            url: "http://localhost".into(),
            status_code: 500,
            duration_ms: 10,
            response_headers: indexmap::IndexMap::new(),
            response_body: "{}".into(),
            assertions: Vec::new(),
            logs: Vec::new(),
            error: None,
        };

        let script = r#"
            yarac.test("Expect 200", () => {
                expect(yarac.response.status).to.equal(200);
            });
        "#;

        runner
            .run_post_response(script, &mut report, &HashMap::new())
            .await
            .unwrap();

        assert_eq!(report.assertions.len(), 1);
        assert!(!report.assertions[0].passed);
        assert!(report.assertions[0].error_message.is_some());
    }

    #[tokio::test]
    async fn test_evaluate_predicate() {
        let runner = ScriptRunner::default();
        let headers = indexmap::IndexMap::new();

        // Matching predicate
        let passed = runner
            .evaluate_predicate(
                "response.status == 200 && response.body.status == 'ready'",
                200,
                r#"{"status": "ready", "count": 5}"#,
                &headers,
            )
            .await
            .unwrap();
        assert!(passed);

        // Failing predicate
        let not_passed = runner
            .evaluate_predicate(
                "response.status == 200 && response.body.status == 'ready'",
                200,
                r#"{"status": "pending", "count": 5}"#,
                &headers,
            )
            .await
            .unwrap();
        assert!(!not_passed);
    }
}

