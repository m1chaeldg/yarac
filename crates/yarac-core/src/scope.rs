use crate::models::VariableValue;
use indexmap::IndexMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeLevel {
    Local,
    Collection,
    Environment,
    Workspace,
    Global,
}

#[derive(Debug, Clone)]
pub struct ScopeResolutionResult {
    pub value: String,
    pub level: ScopeLevel,
    pub is_secret: bool,
    pub keyring_ref: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct VariableScopeStack {
    pub local: IndexMap<String, VariableValue>,
    pub collection: IndexMap<String, VariableValue>,
    pub environment: IndexMap<String, VariableValue>,
    pub workspace: IndexMap<String, VariableValue>,
    pub global: IndexMap<String, VariableValue>,
}

impl VariableScopeStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lookup(&self, key: &str) -> Option<ScopeResolutionResult> {
        // Dynamic variables (evaluated on demand)
        if key.starts_with('$') {
            if let Some(dyn_val) = eval_dynamic_var(key) {
                return Some(ScopeResolutionResult {
                    value: dyn_val,
                    level: ScopeLevel::Local,
                    is_secret: false,
                    keyring_ref: None,
                });
            }
        }

        // Ordered hierarchy: Local -> Collection -> Environment -> Workspace -> Global
        if let Some(var) = self.local.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Local,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.collection.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Collection,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.environment.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Environment,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.workspace.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Workspace,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.global.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Global,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }

        None
    }

    /// Interpolates all `{{variable}}` occurrences without using regex.
    pub fn interpolate(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.char_indices().peekable();

        while let Some((idx, c)) = chars.next() {
            if c == '{' && input[idx..].starts_with("{{") {
                // Find closing '}}'
                if let Some(close_offset) = input[idx + 2..].find("}}") {
                    let var_name = input[idx + 2..idx + 2 + close_offset].trim();
                    if let Some(resolved) = self.lookup(var_name) {
                        result.push_str(&resolved.value);
                    } else {
                        // Keep unchanged if not found
                        result.push_str(&input[idx..idx + 4 + close_offset]);
                    }
                    // Skip past '}}'
                    let target_idx = idx + 4 + close_offset;
                    while let Some(&(next_idx, _)) = chars.peek() {
                        if next_idx < target_idx {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
            result.push(c);
        }

        result
    }
}

fn eval_dynamic_var(key: &str) -> Option<String> {
    match key {
        "$uuid" | "$guid" => Some(Uuid::new_v4().to_string()),
        "$timestamp" => {
            let start = SystemTime::now();
            let since_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
            Some(since_epoch.as_secs().to_string())
        }
        "$timestampMs" => {
            let start = SystemTime::now();
            let since_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
            Some(since_epoch.as_millis().to_string())
        }
        "$randomInt" => {
            // Simple pseudo-random using time ticks without heavy dependencies
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            Some((nanos % 10000).to_string())
        }
        "$randomEmail" => {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            Some(format!("user_{}@example.com", nanos % 10000))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation_hierarchy() {
        let mut stack = VariableScopeStack::new();
        stack.environment.insert("base_url".into(), VariableValue::new("https://api.yarac.dev"));
        stack.local.insert("endpoint".into(), VariableValue::new("users"));

        let template = "{{base_url}}/api/v1/{{endpoint}}";
        let interpolated = stack.interpolate(template);
        assert_eq!(interpolated, "https://api.yarac.dev/api/v1/users");
    }

    #[test]
    fn test_dynamic_uuid() {
        let stack = VariableScopeStack::new();
        let out = stack.interpolate("id={{$uuid}}");
        assert!(out.starts_with("id="));
        assert_eq!(out.len(), 3 + 36);
    }
}
