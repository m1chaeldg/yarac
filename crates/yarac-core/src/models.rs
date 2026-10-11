use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YaracRequest {
    pub version: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub settings: Settings,
    pub headers: IndexMap<String, String>,
    pub auth: Option<AuthConfig>,
    pub query_params: IndexMap<String, String>,
    #[serde(default)]
    pub variables: IndexMap<String, String>,
    pub flow: Option<FlowControl>,
    pub body: BodyType,
    pub scripts: ScriptHooks,
    #[serde(default)]
    pub assert: Option<String>,
    #[serde(default)]
    pub tests: Option<String>,
}

impl Default for YaracRequest {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "New Request".to_string(),
            method: "GET".to_string(),
            url: "".to_string(),
            settings: Settings::default(),
            headers: IndexMap::new(),
            auth: None,
            query_params: IndexMap::new(),
            variables: IndexMap::new(),
            flow: None,
            body: BodyType::None,
            scripts: ScriptHooks::default(),
            assert: None,
            tests: None,
        }
    }
}

fn default_max_redirects() -> u32 {
    5
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub inherit_auth: bool,
    pub verify_ssl: bool,
    pub follow_redirects: bool,
    #[serde(default = "default_max_redirects")]
    pub max_redirects: u32,
    pub timeout_ms: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            inherit_auth: true,
            verify_ssl: true,
            follow_redirects: true,
            max_redirects: 5,
            timeout_ms: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    Bearer {
        token: String,
    },
    Basic {
        username: String,
        password: String,
    },
    ApiKey {
        key: String,
        value: String,
        in_header: bool,
    },
    AwsSigV4 {
        access_key: String,
        secret_key: String,
        region: String,
        service: String,
        session_token: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowControl {
    pub once_per_run: bool,
    pub retry_until: Option<String>,
    pub retry_interval_ms: u64,
    pub max_retries: u32,
    pub delay_ms: Option<u64>,
    pub jitter_min_ms: Option<u64>,
    pub jitter_max_ms: Option<u64>,
}

impl Default for FlowControl {
    fn default() -> Self {
        Self {
            once_per_run: false,
            retry_until: None,
            retry_interval_ms: 1000,
            max_retries: 3,
            delay_ms: None,
            jitter_min_ms: None,
            jitter_max_ms: None,
        }
    }
}

impl FlowControl {
    pub fn calculate_pacing_delay(&self) -> Option<std::time::Duration> {
        let mut total_ms = self.delay_ms.unwrap_or(0);
        if let (Some(min), Some(max)) = (self.jitter_min_ms, self.jitter_max_ms) {
            if max >= min {
                let jitter = if max == min {
                    min
                } else {
                    use rand::Rng;
                    rand::thread_rng().gen_range(min..=max)
                };
                total_ms += jitter;
            }
        }
        if total_ms > 0 {
            Some(std::time::Duration::from_millis(total_ms))
        } else {
            None
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum BodyType {
    None,
    Json(String),
    Xml(String),
    Text(String),
    Raw {
        content: String,
        content_type: Option<String>,
    },
    Form(IndexMap<String, String>),
    Multipart(Vec<MultipartPart>),
    Binary {
        file_path: String,
        content_type: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultipartPart {
    pub name: String,
    pub value: Option<String>,
    /// Relative path to workspace file
    pub file_path: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ScriptHooks {
    pub pre_request: Option<String>,
    pub post_response: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub variables: IndexMap<String, VariableValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VariableValue {
    pub value: String,
    pub is_secret: bool,
    pub keyring_ref: Option<String>,
}

impl VariableValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            is_secret: false,
            keyring_ref: None,
        }
    }

    pub fn secret(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            is_secret: true,
            keyring_ref: None,
        }
    }

    pub fn from_keyring(keyring_ref: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            is_secret: true,
            keyring_ref: Some(keyring_ref.into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub name: String,
    pub passed: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub request_name: String,
    pub method: String,
    pub url: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub response_headers: IndexMap<String, String>,
    pub response_body: String,
    pub assertions: Vec<AssertionResult>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

impl ExecutionReport {
    pub fn all_passed(&self) -> bool {
        self.error.is_none() && self.assertions.iter().all(|a| a.passed)
    }

    pub fn exit_code(&self) -> i32 {
        if self.error.is_some() {
            // Check if error is network vs config
            return 3; // Network / execution error
        }
        if self.assertions.iter().any(|a| !a.passed) {
            1 // Assertion failure
        } else {
            0 // Success
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionMeta {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub request_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub collections: Vec<CollectionMeta>,
    #[serde(default)]
    pub active_environment_id: Option<String>,
    #[serde(default)]
    pub open_tabs: Vec<String>,
    #[serde(default)]
    pub active_tab: Option<String>,
}
