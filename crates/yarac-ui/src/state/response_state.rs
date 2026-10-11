use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseState {
    pub status: u16,
    pub status_text: String,
    pub duration_ms: u64,
    pub size_bytes: usize,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub timestamp: String,
    pub assertions: Vec<AssertionResult>,
    pub logs: Vec<String>,
    pub error: Option<String>,
}

impl ResponseState {
    pub fn empty() -> Self {
        Self {
            status: 0,
            status_text: String::new(),
            duration_ms: 0,
            size_bytes: 0,
            headers: HashMap::new(),
            body: String::new(),
            timestamp: String::new(),
            assertions: Vec::new(),
            logs: Vec::new(),
            error: None,
        }
    }
}
