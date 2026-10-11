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

const COMMON_PRELUDE_JS: &str = r#"
// Embedded Zero-Dependency Faker Implementation
(function() {
    let seed = Date.now() ^ (Math.random() * 0x10000000);
    function pseudo() {
        seed = (seed * 9301 + 49297) % 233280;
        return seed / 233280;
    }
    function pick(arr) {
        return arr[Math.floor(pseudo() * arr.length)];
    }
    function randInt(min, max) {
        return Math.floor(pseudo() * (max - min + 1)) + min;
    }

    const firstNames = ["James", "Mary", "Robert", "Patricia", "John", "Jennifer", "Michael", "Linda", "David", "Elizabeth", "William", "Barbara", "Richard", "Susan", "Joseph", "Jessica", "Thomas", "Sarah"];
    const lastNames = ["Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis", "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson"];
    const domains = ["example.com", "test.org", "demo.net", "yarac.dev", "mail.com"];
    const cities = ["New York", "London", "Tokyo", "Paris", "Berlin", "Sydney", "Toronto", "Singapore", "San Francisco", "Dublin"];
    const countries = ["United States", "United Kingdom", "Canada", "Germany", "France", "Japan", "Australia", "Singapore"];
    const streets = ["Main St", "High St", "Maple Ave", "Oak St", "Park Blvd", "Broadway", "Market St", "Lakeview Dr"];
    const words = ["agile", "robust", "scalable", "ephemeral", "secure", "distributed", "protocol", "payload", "stream", "pipeline", "cluster", "gateway", "latency"];
    const companies = ["Acme Corp", "Globex", "Initech", "Umbrella", "Cyberdyne", "Soylent", "Massive Dynamic", "Hooli", "Pied Piper"];
    const jobTitles = ["Software Engineer", "DevOps Specialist", "Product Manager", "QA Architect", "Systems Analyst", "SRE"];

    function genUuid() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            const r = Math.floor(pseudo() * 16);
            const v = c === 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    }

    globalThis.faker = {
        person: {
            firstName: () => pick(firstNames),
            lastName: () => pick(lastNames),
            fullName: () => `${pick(firstNames)} ${pick(lastNames)}`,
            jobTitle: () => pick(jobTitles),
        },
        name: {
            firstName: () => pick(firstNames),
            lastName: () => pick(lastNames),
            fullName: () => `${pick(firstNames)} ${pick(lastNames)}`,
            findName: () => `${pick(firstNames)} ${pick(lastNames)}`,
            jobTitle: () => pick(jobTitles),
        },
        internet: {
            email: () => `${pick(firstNames).toLowerCase()}.${pick(lastNames).toLowerCase()}${randInt(10,999)}@${pick(domains)}`,
            exampleEmail: () => `${pick(firstNames).toLowerCase()}${randInt(1,99)}@example.com`,
            userName: () => `${pick(firstNames).toLowerCase()}_${randInt(100,999)}`,
            password: (len = 12) => {
                const chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%";
                return Array.from({ length: len }, () => chars[Math.floor(pseudo() * chars.length)]).join('');
            },
            url: () => `https://${pick(words)}.${pick(domains)}/api/v1`,
            ip: () => `${randInt(1,254)}.${randInt(1,254)}.${randInt(1,254)}.${randInt(1,254)}`,
            ipv6: () => `2001:db8:${randInt(1000,9999).toString(16)}:${randInt(1000,9999).toString(16)}::1`,
            color: () => `#${Math.floor(pseudo()*16777215).toString(16).padStart(6, '0')}`,
        },
        number: {
            int: (opts) => {
                const max = typeof opts === 'number' ? opts : (opts?.max ?? 10000);
                const min = opts?.min ?? 0;
                return randInt(min, max);
            },
            float: (opts) => {
                const min = opts?.min ?? 0;
                const max = opts?.max ?? 100;
                const prec = opts?.precision ?? 0.01;
                const val = min + pseudo() * (max - min);
                return Math.round(val / prec) * prec;
            }
        },
        datatype: {
            uuid: () => genUuid(),
            boolean: () => pseudo() > 0.5,
            number: (max = 1000) => randInt(0, max),
        },
        random: {
            uuid: () => genUuid(),
            boolean: () => pseudo() > 0.5,
            number: (max = 1000) => randInt(0, max),
            word: () => pick(words),
            words: (count = 3) => Array.from({ length: count }, () => pick(words)).join(' '),
            alphaNumeric: (len = 8) => {
                const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
                return Array.from({ length: len }, () => chars[Math.floor(pseudo() * chars.length)]).join('');
            }
        },
        string: {
            uuid: () => genUuid(),
            alphanumeric: (len = 8) => {
                const chars = "abcdefghijklmnopqrstuvwxyz0123456789";
                return Array.from({ length: len }, () => chars[Math.floor(pseudo() * chars.length)]).join('');
            },
            sample: (len = 10) => Array.from({ length: len }, () => pick(words)).join(' '),
        },
        location: {
            city: () => pick(cities),
            country: () => pick(countries),
            countryCode: () => "US",
            streetAddress: () => `${randInt(100, 9999)} ${pick(streets)}`,
            street: () => pick(streets),
        },
        address: {
            city: () => pick(cities),
            country: () => pick(countries),
            streetAddress: () => `${randInt(100, 9999)} ${pick(streets)}`,
        },
        company: {
            name: () => pick(companies),
            companyName: () => pick(companies),
        },
        commerce: {
            price: () => `${randInt(5, 500)}.${randInt(10, 99)}`,
            productName: () => `${pick(words)} ${pick(words)}`,
        },
        lorem: {
            word: () => pick(words),
            words: (count = 3) => Array.from({ length: count }, () => pick(words)).join(' '),
            sentence: () => `${pick(words)} ${pick(words)} ${pick(words)} ${pick(words)}.`,
        },
        date: {
            recent: () => new Date(),
            past: () => new Date(Date.now() - randInt(86400000, 86400000 * 30)),
            future: () => new Date(Date.now() + randInt(86400000, 86400000 * 30)),
            now: () => Date.now(),
        }
    };
})();

// Jasmine / Chai Style Testing Environment
(function() {
    let currentSuite = "";

    function fail(msg) {
        throw new Error(msg);
    }

    function deepEqual(a, b) {
        if (a === b) return true;
        if (a == null || b == null) return false;
        if (typeof a !== 'object' || typeof b !== 'object') return false;
        const keysA = Object.keys(a);
        const keysB = Object.keys(b);
        if (keysA.length !== keysB.length) return false;
        for (const k of keysA) {
            if (!keysB.includes(k) || !deepEqual(a[k], b[k])) return false;
        }
        return true;
    }

    function createExpectation(actual, isNegated = false) {
        const check = (passed, msg) => {
            const result = isNegated ? !passed : passed;
            if (!result) {
                fail(msg);
            }
        };

        return {
            get not() {
                return createExpectation(actual, !isNegated);
            },
            toBe(expected) {
                check(actual === expected, `Expected ${JSON.stringify(actual)} to be ${JSON.stringify(expected)}`);
            },
            toEqual(expected) {
                check(deepEqual(actual, expected), `Expected ${JSON.stringify(actual)} to equal ${JSON.stringify(expected)}`);
            },
            toBeDefined() {
                check(actual !== undefined, `Expected value to be defined, but was undefined`);
            },
            toBeUndefined() {
                check(actual === undefined, `Expected value to be undefined, but was ${JSON.stringify(actual)}`);
            },
            toBeNull() {
                check(actual === null, `Expected value to be null, but was ${JSON.stringify(actual)}`);
            },
            toBeTruthy() {
                check(Boolean(actual), `Expected ${JSON.stringify(actual)} to be truthy`);
            },
            toBeFalsy() {
                check(!Boolean(actual), `Expected ${JSON.stringify(actual)} to be falsy`);
            },
            toBeGreaterThan(n) {
                check(actual > n, `Expected ${actual} to be greater than ${n}`);
            },
            toBeLessThan(n) {
                check(actual < n, `Expected ${actual} to be less than ${n}`);
            },
            toBeGreaterThanOrEqual(n) {
                check(actual >= n, `Expected ${actual} to be >= ${n}`);
            },
            toBeLessThanOrEqual(n) {
                check(actual <= n, `Expected ${actual} to be <= ${n}`);
            },
            toContain(item) {
                if (typeof actual === 'string' || Array.isArray(actual)) {
                    check(actual.includes(item), `Expected ${JSON.stringify(actual)} to contain ${JSON.stringify(item)}`);
                } else {
                    fail(`Expected string or array for toContain, got ${typeof actual}`);
                }
            },
            toMatch(pattern) {
                const regex = pattern instanceof RegExp ? pattern : new RegExp(pattern);
                check(regex.test(String(actual)), `Expected ${JSON.stringify(actual)} to match ${pattern}`);
            },
            toThrow() {
                if (typeof actual !== 'function') fail('Expected function for toThrow assertion');
                let threw = false;
                try {
                    actual();
                } catch {
                    threw = true;
                }
                check(threw, `Expected function to throw error`);
            },
            // Chai BDD compatibility
            to: {
                equal(expected) {
                    check(actual === expected, `Expected ${JSON.stringify(actual)} to equal ${JSON.stringify(expected)}`);
                },
                deep: {
                    equal(expected) {
                        check(deepEqual(actual, expected), `Expected ${JSON.stringify(actual)} to deeply equal ${JSON.stringify(expected)}`);
                    }
                },
                be: {
                    a(expectedType) {
                        const actualType = typeof actual;
                        check(actualType === expectedType, `Expected type '${actualType}' to be '${expectedType}'`);
                    },
                    get true() {
                        check(actual === true, `Expected true, got ${JSON.stringify(actual)}`);
                    },
                    get false() {
                        check(actual === false, `Expected false, got ${JSON.stringify(actual)}`);
                    }
                },
                have: {
                    property(prop) {
                        check(actual != null && Object.prototype.hasOwnProperty.call(actual, prop), `Expected object to have property '${prop}'`);
                    }
                }
            }
        };
    }

    globalThis.expect = function(actual) {
        return createExpectation(actual, false);
    };

    globalThis.describe = function(suiteName, fn) {
        const prev = currentSuite;
        currentSuite = prev ? `${prev} > ${suiteName}` : String(suiteName);
        try {
            fn();
        } finally {
            currentSuite = prev;
        }
    };

    globalThis.it = function(testName, fn) {
        const fullTitle = currentSuite ? `${currentSuite} > ${testName}` : String(testName);
        try {
            fn();
            globalThis.__yarac_internal.assertions.push({
                name: fullTitle,
                passed: true,
                error_message: null
            });
        } catch (err) {
            globalThis.__yarac_internal.assertions.push({
                name: fullTitle,
                passed: false,
                error_message: err?.message || String(err)
            });
        }
    };

    globalThis.test = globalThis.it;
})();
"#;

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
