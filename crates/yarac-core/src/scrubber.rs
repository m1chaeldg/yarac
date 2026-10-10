use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrubConfig {
    pub scrub_tainted_secrets: bool,
}

impl Default for ScrubConfig {
    fn default() -> Self {
        Self {
            scrub_tainted_secrets: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SecretRegistry {
    /// Secrets >= 6 characters eligible for substring scrubbing
    long_secrets: HashSet<String>,
    /// Secrets < 6 characters requiring exact token boundary or header-specific matching
    short_secrets: HashSet<String>,
}

impl SecretRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, secret: &str) {
        let trimmed = secret.trim();
        if trimmed.is_empty() {
            return;
        }
        if trimmed.len() >= 6 {
            self.long_secrets.insert(trimmed.to_string());
        } else {
            self.short_secrets.insert(trimmed.to_string());
        }
    }

    /// Redacts tainted secrets according to AGENTS.md rules:
    /// - automated substring scrubbing for secrets >= 6 characters
    /// - exact token boundaries for secrets < 6 characters
    pub fn scrub(&self, input: &str, config: &ScrubConfig) -> String {
        if !config.scrub_tainted_secrets {
            return input.to_string();
        }

        let mut output = input.to_string();

        // 1. Substring scrubbing for long secrets (>= 6 chars)
        for secret in &self.long_secrets {
            if output.contains(secret) {
                output = output.replace(secret, "[REDACTED]");
            }
        }

        // 2. Token-boundary matching for short secrets (< 6 chars)
        for secret in &self.short_secrets {
            output = scrub_token_boundary(&output, secret);
        }

        output
    }
}

fn scrub_token_boundary(input: &str, secret: &str) -> String {
    if !input.contains(secret) {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut i = 0;
    let bytes = input.as_bytes();
    let sec_bytes = secret.as_bytes();
    let sec_len = sec_bytes.len();

    while i < input.len() {
        if i + sec_len <= input.len() && &bytes[i..i + sec_len] == sec_bytes {
            // Check preceding boundary
            let prev_boundary = if i == 0 {
                true
            } else {
                let prev_char = input[..i].chars().next_back().unwrap();
                is_boundary_char(prev_char)
            };

            // Check following boundary
            let next_boundary = if i + sec_len == input.len() {
                true
            } else {
                let next_char = input[i + sec_len..].chars().next().unwrap();
                is_boundary_char(next_char)
            };

            if prev_boundary && next_boundary {
                result.push_str("[REDACTED]");
                i += sec_len;
                continue;
            }
        }

        // Push current character
        if let Some(c) = input[i..].chars().next() {
            result.push(c);
            i += c.len_utf8();
        } else {
            break;
        }
    }

    result
}

fn is_boundary_char(c: char) -> bool {
    c.is_whitespace() || matches!(c, '"' | '\'' | '`' | ':' | '=' | ',' | ';' | '(' | ')' | '[' | ']' | '{' | '}' | '/' | '\\')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_long_secret_substring_scrubbing() {
        let mut registry = SecretRegistry::new();
        registry.register("super_secret_token_123");

        let input = "Authorization: Bearer super_secret_token_123 in payload: {token: 'super_secret_token_123'}";
        let scrubbed = registry.scrub(input, &ScrubConfig::default());
        assert_eq!(
            scrubbed,
            "Authorization: Bearer [REDACTED] in payload: {token: '[REDACTED]'}"
        );
    }

    #[test]
    fn test_short_secret_token_boundary() {
        let mut registry = SecretRegistry::new();
        registry.register("key"); // 3 characters

        // Should NOT scrub "keyboard" or "monkey"
        let input = "keyboard monkey key \"key\" key:val";
        let scrubbed = registry.scrub(input, &ScrubConfig::default());
        assert_eq!(
            scrubbed,
            "keyboard monkey [REDACTED] \"[REDACTED]\" [REDACTED]:val"
        );
    }

    #[test]
    fn test_disabled_scrubbing() {
        let mut registry = SecretRegistry::new();
        registry.register("secret_password");

        let input = "password is secret_password";
        let config = ScrubConfig {
            scrub_tainted_secrets: false,
        };
        assert_eq!(registry.scrub(input, &config), input);
    }
}
