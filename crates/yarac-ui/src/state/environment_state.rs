use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub name: String,
    pub variables: Vec<EnvironmentVariable>,
}

impl EnvironmentState {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            variables: Vec::new(),
        }
    }

    pub fn set_variable(&mut self, key: &str, value: &str, is_secret: bool) {
        if let Some(v) = self.variables.iter_mut().find(|v| v.key == key) {
            v.value = value.to_string();
            v.is_secret = is_secret;
        } else {
            self.variables.push(EnvironmentVariable {
                key: key.to_string(),
                value: value.to_string(),
                is_secret,
                enabled: true,
            });
        }
    }

    pub fn active_variables_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for v in &self.variables {
            if v.enabled && !v.key.ends_with('~') && !v.key.trim().is_empty() {
                map.insert(v.key.clone(), v.value.clone());
            }
        }
        map
    }

    pub fn interpolate(&self, template: &str) -> String {
        let map = self.active_variables_map();
        let mut result = template.to_string();
        for (k, v) in map {
            let pattern = format!("{{{{{}}}}}", k);
            result = result.replace(&pattern, &v);
        }
        result
    }
}
