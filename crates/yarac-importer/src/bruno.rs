use indexmap::IndexMap;
use std::path::{Path, PathBuf};
use yarac_core::models::*;
use crate::{serializer, ImportError};

pub struct BrunoImporter;

impl BrunoImporter {
    /// Translates a single Bruno `.bru` file content into a YaracRequest
    pub fn import_bru(content: &str) -> Result<YaracRequest, ImportError> {
        let blocks = Self::parse_blocks(content);

        let mut name = "Bruno Request".to_string();
        let mut method = "GET".to_string();
        let mut url = String::new();
        let mut headers = IndexMap::new();
        let mut query_params = IndexMap::new();
        let mut auth = None;
        let mut body = BodyType::None;
        let mut scripts = ScriptHooks::default();

        // 1. Meta block
        if let Some(meta_content) = blocks.get("meta") {
            for line in meta_content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once(':') {
                    if k.trim() == "name" {
                        name = v.trim().to_string();
                    }
                }
            }
        }

        // 2. HTTP method blocks: get, post, put, delete, patch, options, head
        for (m, block_content) in &blocks {
            let upper = m.to_uppercase();
            if ["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS", "HEAD"].contains(&upper.as_str()) {
                method = upper;
                for line in block_content.lines() {
                    let trimmed = line.trim();
                    if let Some((k, v)) = trimmed.split_once(':') {
                        if k.trim() == "url" {
                            url = v.trim().to_string();
                        }
                    }
                }
            }
        }

        // 3. Headers block
        if let Some(h_content) = blocks.get("headers") {
            for line in h_content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once(':') {
                    headers.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }

        // 4. Params blocks
        if let Some(q_content) = blocks.get("params:query") {
            for line in q_content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("//") {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once(':') {
                    query_params.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }

        // 5. Auth blocks
        if let Some(auth_content) = blocks.get("auth:bearer") {
            for line in auth_content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once(':') {
                    if k.trim() == "token" {
                        auth = Some(AuthConfig::Bearer { token: v.trim().to_string() });
                    }
                }
            }
        } else if let Some(auth_content) = blocks.get("auth:basic") {
            let mut username = String::new();
            let mut password = String::new();
            for line in auth_content.lines() {
                let trimmed = line.trim();
                if let Some((k, v)) = trimmed.split_once(':') {
                    match k.trim() {
                        "username" => username = v.trim().to_string(),
                        "password" => password = v.trim().to_string(),
                        _ => {}
                    }
                }
            }
            auth = Some(AuthConfig::Basic { username, password });
        }

        // 6. Body blocks
        if let Some(json_content) = blocks.get("body:json") {
            body = BodyType::Json(json_content.trim().to_string());
        } else if let Some(text_content) = blocks.get("body:text") {
            body = BodyType::Raw {
                content: text_content.trim().to_string(),
                content_type: Some("text/plain".into()),
            };
        }

        // 7. Scripts
        if let Some(pre) = blocks.get("script:pre-request") {
            scripts.pre_request = Some(pre.trim().to_string());
        }
        if let Some(tests) = blocks.get("tests") {
            scripts.post_response = Some(tests.trim().to_string());
        }

        Ok(YaracRequest {
            version: "1.0".to_string(),
            name,
            method,
            url,
            settings: Settings::default(),
            headers,
            auth,
            query_params,
            variables: IndexMap::new(),
            flow: None,
            body,
            scripts,
            assert: None,
            tests: None,
        })
    }

    /// Recursively parses a folder containing `.bru` files into requests
    pub fn import_directory(dir: &Path) -> Result<Vec<YaracRequest>, ImportError> {
        let mut requests = Vec::new();
        Self::collect_bru_files(dir, &mut requests)?;
        Ok(requests)
    }

    fn collect_bru_files(dir: &Path, out: &mut Vec<YaracRequest>) -> Result<(), ImportError> {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    Self::collect_bru_files(&p, out)?;
                } else if p.extension().and_then(|s| s.to_str()) == Some("bru") {
                    let content = std::fs::read_to_string(&p)?;
                    if let Ok(req) = Self::import_bru(&content) {
                        out.push(req);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn export_to_workspace(
        requests: &[YaracRequest],
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>, ImportError> {
        serializer::export_requests_to_workspace(requests, target_dir)
    }

    /// Parses blocks structured as `identifier { ... }` with nested brace tracking
    fn parse_blocks(content: &str) -> IndexMap<String, String> {
        let mut blocks = IndexMap::new();
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            // Skip whitespace
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }
            if i >= len {
                break;
            }

            // Find block name
            let name_start = i;
            while i < len && chars[i] != '{' && !chars[i].is_whitespace() {
                i += 1;
            }
            let block_name = content[name_start..i].trim().to_string();

            // Find opening brace '{'
            while i < len && chars[i] != '{' {
                i += 1;
            }
            if i >= len {
                break;
            }
            i += 1; // skip '{'

            let body_start = i;
            let mut brace_depth = 1;
            while i < len && brace_depth > 0 {
                if chars[i] == '{' {
                    brace_depth += 1;
                } else if chars[i] == '}' {
                    brace_depth -= 1;
                }
                if brace_depth == 0 {
                    break;
                }
                i += 1;
            }

            let block_body = if i <= len {
                content[body_start..i].to_string()
            } else {
                String::new()
            };

            if brace_depth == 0 {
                i += 1; // skip closing '}'
            }

            if !block_name.is_empty() {
                blocks.insert(block_name, block_body);
            }
        }

        blocks
    }
}
