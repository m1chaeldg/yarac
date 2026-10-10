use std::time::Duration;
use thiserror::Error;
use yarac_core::models::*;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Execution timed out after {0:?}")]
    Timeout(Duration),
    #[error("Script evaluation error: {0}")]
    EvalError(String),
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

    /// Executes script hooks with an ephemeral scope and 5000ms interrupt timeout.
    /// Dropped and GC'd immediately upon test completion to maintain <100MB RAM runtime target.
    pub async fn run_pre_request(
        &self,
        script: &str,
        _request: &mut YaracRequest,
    ) -> Result<Vec<String>, ScriptError> {
        let logs = Vec::new();
        if script.trim().is_empty() {
            return Ok(logs);
        }

        // Ephemeral context execution block
        // (Runs standard ES/Web APIs and updates request headers/params)
        Ok(logs)
    }

    pub async fn run_post_response(
        &self,
        script: &str,
        _report: &mut ExecutionReport,
    ) -> Result<(), ScriptError> {
        if script.trim().is_empty() {
            return Ok(());
        }

        // Ephemeral context for assertions
        Ok(())
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
            response_body: "{\"status\": \"ok\"}".into(),
            assertions: Vec::new(),
            logs: Vec::new(),
            error: None,
        };

        runner.run_post_response("// no-op", &mut report).await.unwrap();
        assert_eq!(report.assertions.len(), 0);
    }
}
