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
    env_vars: HashMap<String, String>,
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

const COMMON_PRELUDE_JS: &str = include_str!("prelude.js");


impl ScriptRunner {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Executes pre-request script hook with an ephemeral context and 5000ms interrupt timeout.
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
            BodyType::Json(s) | BodyType::Xml(s) | BodyType::Text(s) => s.clone(),
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
        let req_method = request.method.clone();
        let req_url = request.url.clone();

        let run_result = tokio::task::spawn_blocking(move || {
            let mut runtime = JsRuntime::new(RuntimeOptions::default());

            let setup_js = format!(
                r#"
                {}
                globalThis.__yarac_internal = {{
                    headers: {},
                    params: {},
                    env_vars: {{}},
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
                    getEnvVar(k) {{
                        return __yarac_internal.env_vars[String(k)] || '';
                    }},
                    setEnvVar(k, v) {{
                        __yarac_internal.env_vars[String(k)] = String(v);
                    }},
                    getVar(k) {{
                        return this.getEnvVar(k);
                    }},
                    setVar(k, v) {{
                        this.setEnvVar(k, v);
                    }},
                    request: {{
                        method: "{}",
                        url: "{}",
                        headers: {},
                        params: {},
                        getBody() {{
                            return __request_body;
                        }}
                    }}
                }};

                globalThis.req = globalThis.yarac.request;
                globalThis.request = globalThis.yarac.request;
                "#,
                COMMON_PRELUDE_JS,
                serde_json::to_string(&initial_headers).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&initial_params).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&secrets_clone).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&body_str).unwrap_or_else(|_| "\"\"".into()),
                req_method,
                req_url,
                serde_json::to_string(&initial_headers).unwrap_or_else(|_| "{}".into()),
                serde_json::to_string(&initial_params).unwrap_or_else(|_| "{}".into()),
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
                {}
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

                globalThis.yarac = {{
                    response: {{
                        status: {},
                        statusCode: {},
                        headers: {},
                        body: {},
                        json() {{
                            try {{
                                return JSON.parse(this.body);
                            }} catch(e) {{
                                return null;
                            }}
                        }},
                        header(name) {{
                            const h = this.headers;
                            for (const k in h) {{
                                if (k.toLowerCase() === String(name).toLowerCase()) return h[k];
                            }}
                            return null;
                        }},
                        getHeader(name) {{
                            return this.header(name);
                        }}
                    }},
                    test(name, fn) {{
                        globalThis.it(name, fn);
                    }},
                    setEnvVar(k, v) {{
                        __yarac_internal.env_vars[String(k)] = String(v);
                    }},
                    getEnvVar(k) {{
                        return __yarac_internal.env_vars[String(k)] || '';
                    }},
                    setVar(k, v) {{
                        this.setEnvVar(k, v);
                    }},
                    getVar(k) {{
                        return this.getEnvVar(k);
                    }},
                    getSecret(k) {{
                        return __yarac_internal.secrets[String(k)] || '';
                    }}
                }};

                globalThis.res = globalThis.yarac.response;
                globalThis.response = globalThis.yarac.response;
                "#,
                COMMON_PRELUDE_JS,
                serde_json::to_string(&secrets_clone).unwrap_or_else(|_| "{}".into()),
                status,
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

    /// Evaluates user JS assertion expressions and records assertion results
    pub async fn run_assert(
        &self,
        script: &str,
        report: &mut ExecutionReport,
        secrets: &HashMap<String, String>,
    ) -> Result<(), ScriptError> {
        if script.trim().is_empty() {
            return Ok(());
        }

        // If the assert script has lines like "expect(res.status).toBe(200)", run it in post-response test context
        // If it has bare boolean expressions, wrap them in auto-assertions
        let formatted_script = if script.contains("expect(") || script.contains("test(") || script.contains("it(") {
            script.to_string()
        } else {
            // Process lines as assertions
            let mut buf = String::new();
            for line in script.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
                    continue;
                }
                let escaped_title = trimmed.replace('\"', "\\\"");
                buf.push_str(&format!(
                    "test(\"{}\", () => {{ const res_val = ({}); if (res_val === false) throw new Error(\"Assertion evaluated to false\"); }});\n",
                    escaped_title, trimmed
                ));
            }
            buf
        };

        self.run_post_response(&formatted_script, report, secrets).await?;
        Ok(())
    }

    /// Runs Jasmine/Jest BDD tests against the executed response report
    pub async fn run_tests(
        &self,
        script: &str,
        report: &mut ExecutionReport,
        secrets: &HashMap<String, String>,
    ) -> Result<(), ScriptError> {
        if script.trim().is_empty() {
            return Ok(());
        }
        self.run_post_response(script, report, secrets).await?;
        Ok(())
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
    async fn test_pre_request_with_faker() {
        let runner = ScriptRunner::default();
        let mut request = YaracRequest::default();
        request.body = BodyType::Json(r#"{"msg":"hello"}"#.into());

        let mut secrets = HashMap::new();
        secrets.insert("api_key".into(), "secret-123".into());

        let script = r#"
            const secret = yarac.getSecret("api_key");
            const email = faker.internet.email();
            const name = faker.person.fullName();
            yarac.setHeader("X-Custom", "custom-header-" + secret);
            yarac.setHeader("X-Faker-Email", email);
            yarac.setHeader("X-Faker-Name", name);
            console.log("Logged faker email: " + email);
        "#;

        let logs = runner
            .run_pre_request(script, &mut request, &secrets)
            .await
            .unwrap();

        assert_eq!(
            request.headers.get("X-Custom").unwrap(),
            "custom-header-secret-123"
        );
        assert!(request.headers.get("X-Faker-Email").unwrap().contains('@'));
        assert!(!request.headers.get("X-Faker-Name").unwrap().is_empty());
        assert_eq!(logs.len(), 1);
    }

    #[tokio::test]
    async fn test_jasmine_suite_and_expectations() {
        let runner = ScriptRunner::default();
        let mut report = ExecutionReport {
            request_name: "Test".into(),
            method: "POST".into(),
            url: "http://localhost".into(),
            status_code: 201,
            duration_ms: 15,
            response_headers: [("content-type".into(), "application/json".into())].into_iter().collect(),
            response_body: "{\"user\": {\"name\": \"Alice\", \"age\": 30, \"active\": true}}".into(),
            assertions: Vec::new(),
            logs: Vec::new(),
            error: None,
        };

        let tests_script = r#"
            describe("User API", () => {
                it("verifies status code", () => {
                    expect(response.status).toBe(201);
                    expect(response.status).not.toBe(400);
                });

                it("verifies user payload structure", () => {
                    const data = response.json();
                    expect(data.user.name).toBe("Alice");
                    expect(data.user.age).toBeGreaterThan(18);
                    expect(data.user.active).toBeTruthy();
                    expect(data.user.missing).toBeUndefined();
                });
            });
        "#;

        runner
            .run_tests(tests_script, &mut report, &HashMap::new())
            .await
            .unwrap();

        assert_eq!(report.assertions.len(), 2);
        assert!(report.assertions.iter().all(|a| a.passed));
        assert_eq!(report.assertions[0].name, "User API > verifies status code");
        assert_eq!(report.assertions[1].name, "User API > verifies user payload structure");
    }

    #[tokio::test]
    async fn test_run_assert_expression() {
        let runner = ScriptRunner::default();
        let mut report = ExecutionReport {
            request_name: "Test".into(),
            method: "GET".into(),
            url: "http://localhost".into(),
            status_code: 200,
            duration_ms: 10,
            response_headers: indexmap::IndexMap::new(),
            response_body: "{\"ok\": true}".into(),
            assertions: Vec::new(),
            logs: Vec::new(),
            error: None,
        };

        let assert_script = "response.status === 200\nresponse.json().ok === true";
        runner
            .run_assert(assert_script, &mut report, &HashMap::new())
            .await
            .unwrap();

        assert_eq!(report.assertions.len(), 2);
        assert!(report.assertions[0].passed);
        assert!(report.assertions[1].passed);
    }
}
