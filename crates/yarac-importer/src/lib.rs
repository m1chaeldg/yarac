use indexmap::IndexMap;
use serde_json::Value;
use std::path::{Path, PathBuf};
use thiserror::Error;
use yarac_core::models::*;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Bruno parse error: {0}")]
    Bruno(String),
}

// ==============================================================================
// Serialization Helper
// ==============================================================================

/// Serializes a YaracRequest model into valid .yarac format
pub fn serialize_to_yarac(req: &YaracRequest) -> String {
    let mut out = String::new();
    out.push_str(&format!("version = \"{}\"\n", req.version));
    out.push_str(&format!("name = \"{}\"\n", req.name));
    out.push_str(&format!("method = \"{}\"\n", req.method));
    out.push_str(&format!("url = \"{}\"\n\n", req.url));

    out.push_str("[settings]\n");
    out.push_str(&format!("inherit_auth = {}\n", req.settings.inherit_auth));
    out.push_str(&format!("verify_ssl = {}\n", req.settings.verify_ssl));
    out.push_str(&format!("follow_redirects = {}\n", req.settings.follow_redirects));
    out.push_str(&format!("timeout_ms = {}\n\n", req.settings.timeout_ms));

    if !req.headers.is_empty() {
        out.push_str("[headers]\n");
        for (k, v) in &req.headers {
            out.push_str(&format!("{} = \"{}\"\n", k, v));
        }
        out.push('\n');
    }

    if let Some(auth) = &req.auth {
        out.push_str("[auth]\n");
        match auth {
            AuthConfig::Bearer { token } => {
                out.push_str("type = \"bearer\"\n");
                out.push_str(&format!("token = \"{}\"\n\n", token));
            }
            AuthConfig::Basic { username, password } => {
                out.push_str("type = \"basic\"\n");
                out.push_str(&format!("username = \"{}\"\n", username));
                out.push_str(&format!("password = \"{}\"\n\n", password));
            }
            AuthConfig::ApiKey { key, value, in_header } => {
                out.push_str("type = \"api_key\"\n");
                out.push_str(&format!("key = \"{}\"\n", key));
                out.push_str(&format!("value = \"{}\"\n", value));
                out.push_str(&format!("in_header = {}\n\n", in_header));
            }
            AuthConfig::AwsSigV4 { access_key, secret_key, region, service, .. } => {
                out.push_str("type = \"aws_sig_v4\"\n");
                out.push_str(&format!("access_key = \"{}\"\n", access_key));
                out.push_str(&format!("secret_key = \"{}\"\n", secret_key));
                out.push_str(&format!("region = \"{}\"\n", region));
                out.push_str(&format!("service = \"{}\"\n\n", service));
            }
        }
    }

    if !req.query_params.is_empty() {
        out.push_str("[params.query]\n");
        for (k, v) in &req.query_params {
            out.push_str(&format!("{} = \"{}\"\n", k, v));
        }
        out.push('\n');
    }

    if let Some(flow) = &req.flow {
        out.push_str("[flow]\n");
        out.push_str(&format!("once_per_run = {}\n", flow.once_per_run));
        if let Some(r) = &flow.retry_until {
            out.push_str(&format!("retry_until = \"{}\"\n", r));
        }
        out.push_str(&format!("retry_interval_ms = {}\n", flow.retry_interval_ms));
        out.push_str(&format!("max_retries = {}\n", flow.max_retries));
        if let Some(d) = flow.delay_ms {
            out.push_str(&format!("delay_ms = {}\n", d));
        }
        if let Some(jmin) = flow.jitter_min_ms {
            out.push_str(&format!("jitter_min_ms = {}\n", jmin));
        }
        if let Some(jmax) = flow.jitter_max_ms {
            out.push_str(&format!("jitter_max_ms = {}\n", jmax));
        }
        out.push('\n');
    }

    match &req.body {
        BodyType::None => {}
        BodyType::Json(s) => {
            out.push_str("[body.json]\n```json\n");
            out.push_str(s);
            out.push_str("\n```\n\n");
        }
        BodyType::Xml(s) => {
            out.push_str("[body.xml]\n```xml\n");
            out.push_str(s);
            out.push_str("\n```\n\n");
        }
        BodyType::Text(s) => {
            out.push_str("[body.text]\n```text\n");
            out.push_str(s);
            out.push_str("\n```\n\n");
        }
        BodyType::Raw { content, content_type } => {
            let tag = content_type.as_deref().unwrap_or("text");
            out.push_str(&format!("[body.raw]\n```{}\n", tag));
            out.push_str(content);
            out.push_str("\n```\n\n");
        }
        BodyType::Form(form) => {
            out.push_str("[body.form]\n");
            for (k, v) in form {
                out.push_str(&format!("{} = \"{}\"\n", k, v));
            }
            out.push('\n');
        }
        BodyType::Multipart(parts) => {
            out.push_str("[body.multipart]\n");
            for part in parts {
                if let Some(file_path) = &part.file_path {
                    out.push_str(&format!("{} = \"@{}\"\n", part.name, file_path));
                } else if let Some(val) = &part.value {
                    out.push_str(&format!("{} = \"{}\"\n", part.name, val));
                }
            }
            out.push('\n');
        }
        BodyType::Binary { file_path, content_type } => {
            out.push_str("[body.binary]\n");
            out.push_str(&format!("file = \"{}\"\n", file_path));
            if let Some(ct) = content_type {
                out.push_str(&format!("content_type = \"{}\"\n", ct));
            }
            out.push('\n');
        }
    }

    if let Some(pre) = &req.scripts.pre_request {
        out.push_str("[scripts.pre-request]\n```javascript\n");
        out.push_str(pre);
        out.push_str("\n```\n\n");
    }
    if let Some(post) = &req.scripts.post_response {
        out.push_str("[scripts.post-response]\n```javascript\n");
        out.push_str(post);
        out.push_str("\n```\n\n");
    }

    out
}

pub fn export_requests_to_workspace(
    requests: &[YaracRequest],
    target_dir: &Path,
) -> Result<Vec<PathBuf>, ImportError> {
    std::fs::create_dir_all(target_dir)?;
    let mut written_files = Vec::new();

    for (i, req) in requests.iter().enumerate() {
        let sanitized_name = req
            .name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect::<String>();
        let filename = format!("{}_{}.yarac", i + 1, sanitized_name);
        let file_path = target_dir.join(filename);

        let content = serialize_to_yarac(req);
        std::fs::write(&file_path, content)?;
        written_files.push(file_path);
    }

    Ok(written_files)
}

// ==============================================================================
// 1. Postman Importer
// ==============================================================================

pub struct PostmanImporter;

impl PostmanImporter {
    /// Translates a Postman v2.1 collection JSON into native Yarac requests
    pub fn import_collection(json_str: &str) -> Result<Vec<YaracRequest>, ImportError> {
        let v: Value = serde_json::from_str(json_str)?;
        let items = v
            .get("item")
            .and_then(|i| i.as_array())
            .ok_or_else(|| ImportError::InvalidSchema("Missing 'item' array".into()))?;

        let mut requests = Vec::new();
        Self::extract_items(items, &mut requests);
        Ok(requests)
    }

    pub fn export_to_workspace(
        requests: &[YaracRequest],
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>, ImportError> {
        export_requests_to_workspace(requests, target_dir)
    }

    pub fn serialize_to_yarac(req: &YaracRequest) -> String {
        serialize_to_yarac(req)
    }

    fn extract_items(items: &[Value], out: &mut Vec<YaracRequest>) {
        for item in items {
            if let Some(sub_items) = item.get("item").and_then(|i| i.as_array()) {
                Self::extract_items(sub_items, out);
            } else if let Some(req_val) = item.get("request") {
                let name = item
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("Imported Request")
                    .to_string();

                let method = req_val
                    .get("method")
                    .and_then(|m| m.as_str())
                    .unwrap_or("GET")
                    .to_uppercase();

                let mut query_params = IndexMap::new();

                let url = if let Some(url_str) = req_val.get("url").and_then(|u| u.as_str()) {
                    url_str.to_string()
                } else if let Some(url_obj) = req_val.get("url").and_then(|u| u.as_object()) {
                    if let Some(query_arr) = url_obj.get("query").and_then(|q| q.as_array()) {
                        for q in query_arr {
                            if let (Some(k), Some(v)) = (
                                q.get("key").and_then(|k| k.as_str()),
                                q.get("value").and_then(|v| v.as_str()),
                            ) {
                                query_params.insert(k.to_string(), v.to_string());
                            }
                        }
                    }

                    url_obj
                        .get("raw")
                        .and_then(|r| r.as_str())
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                };

                let mut headers = IndexMap::new();
                if let Some(header_arr) = req_val.get("header").and_then(|h| h.as_array()) {
                    for h in header_arr {
                        if let (Some(k), Some(v)) = (
                            h.get("key").and_then(|k| k.as_str()),
                            h.get("value").and_then(|v| v.as_str()),
                        ) {
                            headers.insert(k.to_string(), v.to_string());
                        }
                    }
                }

                let mut auth = None;
                if let Some(auth_obj) = req_val.get("auth").and_then(|a| a.as_object()) {
                    let auth_type = auth_obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    match auth_type {
                        "bearer" => {
                            if let Some(arr) = auth_obj.get("bearer").and_then(|b| b.as_array()) {
                                for entry in arr {
                                    if entry.get("key").and_then(|k| k.as_str()) == Some("token") {
                                        if let Some(tok) = entry.get("value").and_then(|v| v.as_str()) {
                                            auth = Some(AuthConfig::Bearer { token: tok.to_string() });
                                        }
                                    }
                                }
                            }
                        }
                        "basic" => {
                            let mut user = String::new();
                            let mut pass = String::new();
                            if let Some(arr) = auth_obj.get("basic").and_then(|b| b.as_array()) {
                                for entry in arr {
                                    match entry.get("key").and_then(|k| k.as_str()) {
                                        Some("username") => user = entry.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        Some("password") => pass = entry.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        _ => {}
                                    }
                                }
                            }
                            auth = Some(AuthConfig::Basic { username: user, password: pass });
                        }
                        "apikey" => {
                            let mut key = String::new();
                            let mut val = String::new();
                            let mut in_header = true;
                            if let Some(arr) = auth_obj.get("apikey").and_then(|b| b.as_array()) {
                                for entry in arr {
                                    match entry.get("key").and_then(|k| k.as_str()) {
                                        Some("key") => key = entry.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        Some("value") => val = entry.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                        Some("in") => in_header = entry.get("value").and_then(|v| v.as_str()) != Some("query"),
                                        _ => {}
                                    }
                                }
                            }
                            auth = Some(AuthConfig::ApiKey { key, value: val, in_header });
                        }
                        _ => {}
                    }
                }

                let mut body = BodyType::None;
                if let Some(body_obj) = req_val.get("body") {
                    let mode = body_obj.get("mode").and_then(|m| m.as_str()).unwrap_or("");
                    match mode {
                        "raw" => {
                            if let Some(raw_text) = body_obj.get("raw").and_then(|r| r.as_str()) {
                                body = BodyType::Json(raw_text.to_string());
                            }
                        }
                        "urlencoded" => {
                            let mut form = IndexMap::new();
                            if let Some(arr) = body_obj.get("urlencoded").and_then(|u| u.as_array()) {
                                for entry in arr {
                                    if let (Some(k), Some(v)) = (
                                        entry.get("key").and_then(|k| k.as_str()),
                                        entry.get("value").and_then(|v| v.as_str()),
                                    ) {
                                        form.insert(k.to_string(), v.to_string());
                                    }
                                }
                            }
                            body = BodyType::Form(form);
                        }
                        _ => {}
                    }
                }

                out.push(YaracRequest {
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
                    scripts: ScriptHooks::default(),
                    assert: None,
                    tests: None,
                });
            }
        }
    }
}

// ==============================================================================
// 2. Bruno (.bru) Importer
// ==============================================================================

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
        export_requests_to_workspace(requests, target_dir)
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

// ==============================================================================
// 3. OpenAPI v3 / v3.1 Importer
// ==============================================================================

pub struct OpenApiImporter;

impl OpenApiImporter {
    /// Translates an OpenAPI v3 / v3.1 spec (JSON or YAML) into native Yarac requests
    pub fn import_spec(spec_str: &str) -> Result<Vec<YaracRequest>, ImportError> {
        let v: Value = if let Ok(j) = serde_json::from_str(spec_str) {
            j
        } else {
            serde_yaml::from_str(spec_str)?
        };

        let mut requests = Vec::new();
        let servers = v.get("servers").and_then(|s| s.as_array());
        let base_url = servers
            .and_then(|arr| arr.first())
            .and_then(|s| s.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("{{base_url}}")
            .trim_end_matches('/');

        if let Some(paths) = v.get("paths").and_then(|p| p.as_object()) {
            for (path, path_val) in paths {
                if let Some(path_obj) = path_val.as_object() {
                    for method in ["get", "post", "put", "delete", "patch", "options", "head"] {
                        if let Some(op) = path_obj.get(method).and_then(|o| o.as_object()) {
                            let summary = op.get("summary").and_then(|s| s.as_str());
                            let op_id = op.get("operationId").and_then(|s| s.as_str());
                            let name = summary.or(op_id).unwrap_or(path).to_string();

                            let mut query_params = IndexMap::new();
                            let mut headers = IndexMap::new();

                            // Parameters
                            if let Some(params) = op.get("parameters").and_then(|p| p.as_array()) {
                                for p in params {
                                    if let (Some(p_name), Some(p_in)) = (
                                        p.get("name").and_then(|n| n.as_str()),
                                        p.get("in").and_then(|i| i.as_str()),
                                    ) {
                                        let default_val = p.get("example")
                                            .and_then(|e| e.as_str())
                                            .or_else(|| p.get("schema").and_then(|s| s.get("example")).and_then(|e| e.as_str()))
                                            .unwrap_or("")
                                            .to_string();
                                        if p_in == "query" {
                                            query_params.insert(p_name.to_string(), default_val);
                                        } else if p_in == "header" {
                                            headers.insert(p_name.to_string(), default_val);
                                        }
                                    }
                                }
                            }

                            // Request body
                            let mut body = BodyType::None;
                            if let Some(rb) = op.get("requestBody") {
                                if let Some(content_obj) = rb.get("content").and_then(|c| c.as_object()) {
                                    if let Some(json_content) = content_obj.get("application/json") {
                                        if let Some(example) = json_content.get("example") {
                                            body = BodyType::Json(serde_json::to_string_pretty(example).unwrap_or_default());
                                        } else if let Some(examples) = json_content.get("examples").and_then(|e| e.as_object()) {
                                            if let Some((_, ex_val)) = examples.iter().next() {
                                                if let Some(val) = ex_val.get("value") {
                                                    body = BodyType::Json(serde_json::to_string_pretty(val).unwrap_or_default());
                                                }
                                            }
                                        } else if let Some(schema) = json_content.get("schema") {
                                            body = BodyType::Json(serde_json::to_string_pretty(schema).unwrap_or_default());
                                        }
                                    }
                                }
                            }

                            // Auth from security requirement
                            let mut auth = None;
                            if let Some(sec_arr) = op.get("security").or_else(|| v.get("security")).and_then(|s| s.as_array()) {
                                if let Some(first_sec) = sec_arr.first().and_then(|s| s.as_object()) {
                                    if let Some(sec_name) = first_sec.keys().next() {
                                        if sec_name.to_lowercase().contains("bearer") || sec_name.to_lowercase().contains("token") {
                                            auth = Some(AuthConfig::Bearer { token: "{{bearer_token}}".into() });
                                        } else if sec_name.to_lowercase().contains("basic") {
                                            auth = Some(AuthConfig::Basic { username: "{{username}}".into(), password: "{{password}}".into() });
                                        } else if sec_name.to_lowercase().contains("api") || sec_name.to_lowercase().contains("key") {
                                            auth = Some(AuthConfig::ApiKey { key: "X-API-Key".into(), value: "{{api_key}}".into(), in_header: true });
                                        }
                                    }
                                }
                            }

                            let full_url = if path.starts_with('/') {
                                format!("{}{}", base_url, path)
                            } else {
                                format!("{}/{}", base_url, path)
                            };

                            requests.push(YaracRequest {
                                version: "1.0".to_string(),
                                name,
                                method: method.to_uppercase(),
                                url: full_url,
                                settings: Settings::default(),
                                headers,
                                auth,
                                query_params,
                                variables: IndexMap::new(),
                                flow: None,
                                body,
                                scripts: ScriptHooks::default(),
                                assert: None,
                                tests: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(requests)
    }

    pub fn export_to_workspace(
        requests: &[YaracRequest],
        target_dir: &Path,
    ) -> Result<Vec<PathBuf>, ImportError> {
        export_requests_to_workspace(requests, target_dir)
    }
}

// ==============================================================================
// 4. OpenAPI v3.1 Exporter
// ==============================================================================

pub struct OpenApiExporter;

impl OpenApiExporter {
    /// Translates .yarac requests into an OpenAPI v3.1 specification JSON with `x-yarac-*` overlays
    pub fn export_openapi_v3_1(
        requests: &[YaracRequest],
        title: &str,
        version: &str,
    ) -> Result<String, ImportError> {
        let mut paths_map = serde_json::Map::new();

        for req in requests {
            let route_path = Self::extract_route_path(&req.url);
            let path_entry = paths_map
                .entry(route_path.clone())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));

            if let Some(path_obj) = path_entry.as_object_mut() {
                let method_key = req.method.to_lowercase();
                let mut op_obj = serde_json::Map::new();

                op_obj.insert("summary".into(), Value::String(req.name.clone()));
                op_obj.insert(
                    "operationId".into(),
                    Value::String(
                        req.name
                            .to_lowercase()
                            .chars()
                            .map(|c| if c.is_alphanumeric() { c } else { '_' })
                            .collect(),
                    ),
                );

                // Parameters (Query + Headers)
                let mut parameters = Vec::new();
                for (k, v) in &req.query_params {
                    let mut p = serde_json::Map::new();
                    p.insert("name".into(), Value::String(k.clone()));
                    p.insert("in".into(), Value::String("query".into()));
                    let mut schema = serde_json::Map::new();
                    schema.insert("type".into(), Value::String("string".into()));
                    p.insert("schema".into(), Value::Object(schema));
                    p.insert("example".into(), Value::String(v.clone()));
                    parameters.push(Value::Object(p));
                }
                for (k, v) in &req.headers {
                    let mut p = serde_json::Map::new();
                    p.insert("name".into(), Value::String(k.clone()));
                    p.insert("in".into(), Value::String("header".into()));
                    let mut schema = serde_json::Map::new();
                    schema.insert("type".into(), Value::String("string".into()));
                    p.insert("schema".into(), Value::Object(schema));
                    p.insert("example".into(), Value::String(v.clone()));
                    parameters.push(Value::Object(p));
                }
                if !parameters.is_empty() {
                    op_obj.insert("parameters".into(), Value::Array(parameters));
                }

                // Request body
                match &req.body {
                    BodyType::Json(s) => {
                        let parsed: Value = serde_json::from_str(s).unwrap_or_else(|_| Value::String(s.clone()));
                        let mut rb = serde_json::Map::new();
                        let mut content_map = serde_json::Map::new();
                        let mut app_json = serde_json::Map::new();
                        app_json.insert("example".into(), parsed);
                        content_map.insert("application/json".into(), Value::Object(app_json));
                        rb.insert("content".into(), Value::Object(content_map));
                        op_obj.insert("requestBody".into(), Value::Object(rb));
                    }
                    BodyType::Raw { content, content_type } => {
                        let mime = content_type.clone().unwrap_or_else(|| "text/plain".into());
                        let mut rb = serde_json::Map::new();
                        let mut content_map = serde_json::Map::new();
                        let mut mime_map = serde_json::Map::new();
                        mime_map.insert("example".into(), Value::String(content.clone()));
                        content_map.insert(mime, Value::Object(mime_map));
                        rb.insert("content".into(), Value::Object(content_map));
                        op_obj.insert("requestBody".into(), Value::Object(rb));
                    }
                    _ => {}
                }

                // Responses
                let mut responses = serde_json::Map::new();
                let mut resp_200 = serde_json::Map::new();
                resp_200.insert("description".into(), Value::String("Successful response".into()));
                responses.insert("200".into(), Value::Object(resp_200));
                op_obj.insert("responses".into(), Value::Object(responses));

                // x-yarac-* Vendor Extensions
                op_obj.insert("x-yarac-version".into(), Value::String(req.version.clone()));
                op_obj.insert("x-yarac-settings".into(), serde_json::to_value(&req.settings)?);
                if let Some(flow) = &req.flow {
                    op_obj.insert("x-yarac-flow".into(), serde_json::to_value(flow)?);
                }
                if let Some(auth) = &req.auth {
                    op_obj.insert("x-yarac-auth".into(), serde_json::to_value(auth)?);
                }
                op_obj.insert("x-yarac-scripts".into(), serde_json::to_value(&req.scripts)?);

                path_obj.insert(method_key, Value::Object(op_obj));
            }
        }

        let mut root = serde_json::Map::new();
        root.insert("openapi".into(), Value::String("3.1.0".into()));

        let mut info = serde_json::Map::new();
        info.insert("title".into(), Value::String(title.into()));
        info.insert("version".into(), Value::String(version.into()));
        info.insert("description".into(), Value::String("Exported from Yarac".into()));
        root.insert("info".into(), Value::Object(info));

        root.insert("paths".into(), Value::Object(paths_map));

        Ok(serde_json::to_string_pretty(&Value::Object(root))?)
    }

    pub fn export_to_file(
        requests: &[YaracRequest],
        title: &str,
        version: &str,
        file_path: &Path,
    ) -> Result<(), ImportError> {
        let json_str = Self::export_openapi_v3_1(requests, title, version)?;
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(file_path, json_str)?;
        Ok(())
    }

    fn extract_route_path(url: &str) -> String {
        let stripped = if let Some(idx) = url.find("}}") {
            &url[idx + 2..]
        } else if let Some(idx) = url.find("://") {
            let rest = &url[idx + 3..];
            if let Some(slash_idx) = rest.find('/') {
                &rest[slash_idx..]
            } else {
                "/"
            }
        } else {
            url
        };

        let path_only = stripped.split('?').next().unwrap_or(stripped);
        if path_only.starts_with('/') {
            path_only.to_string()
        } else {
            format!("/{}", path_only)
        }
    }
}

// ==============================================================================
// Tests
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_postman_v2_1_with_auth_and_query() {
        let json = r#"{
            "info": { "name": "Test Col" },
            "item": [
                {
                    "name": "Get Users",
                    "request": {
                        "method": "GET",
                        "url": {
                            "raw": "https://api.example.com/users?page=1",
                            "query": [
                                { "key": "page", "value": "1" }
                            ]
                        },
                        "header": [
                            { "key": "Accept", "value": "application/json" }
                        ],
                        "auth": {
                            "type": "bearer",
                            "bearer": [
                                { "key": "token", "value": "my-secret-token" }
                            ]
                        }
                    }
                }
            ]
        }"#;

        let reqs = PostmanImporter::import_collection(json).unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].name, "Get Users");
        assert_eq!(reqs[0].method, "GET");
        assert_eq!(reqs[0].headers.get("Accept").unwrap(), "application/json");
        assert_eq!(reqs[0].query_params.get("page").unwrap(), "1");
        assert_eq!(
            reqs[0].auth,
            Some(AuthConfig::Bearer {
                token: "my-secret-token".into()
            })
        );

        let yarac_dsl = serialize_to_yarac(&reqs[0]);
        assert!(yarac_dsl.contains("version = \"1.0\""));
        assert!(yarac_dsl.contains("name = \"Get Users\""));
        assert!(yarac_dsl.contains("type = \"bearer\""));
        assert!(yarac_dsl.contains("token = \"my-secret-token\""));
    }

    #[test]
    fn test_import_bruno_bru_file() {
        let bru = r#"
meta {
  name: Create Item
  type: http
  seq: 2
}

post {
  url: {{base_url}}/items
  body: json
  auth: bearer
}

headers {
  Content-Type: application/json
  X-Trace-Id: trace-123
}

params:query {
  dry_run: true
}

auth:bearer {
  token: secret-bruno-token
}

body:json {
  {
    "title": "New Item",
    "price": 99.9
  }
}

script:pre-request {
  console.log("Starting bruno request");
}

tests {
  test("Status is 201", function() {
    expect(res.getStatus()).to.equal(201);
  });
}
"#;

        let req = BrunoImporter::import_bru(bru).unwrap();
        assert_eq!(req.name, "Create Item");
        assert_eq!(req.method, "POST");
        assert_eq!(req.url, "{{base_url}}/items");
        assert_eq!(req.headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(req.headers.get("X-Trace-Id").unwrap(), "trace-123");
        assert_eq!(req.query_params.get("dry_run").unwrap(), "true");
        assert_eq!(
            req.auth,
            Some(AuthConfig::Bearer {
                token: "secret-bruno-token".into()
            })
        );
        match req.body {
            BodyType::Json(s) => assert!(s.contains("New Item")),
            _ => panic!("Expected json body"),
        }
        assert!(req.scripts.pre_request.unwrap().contains("Starting bruno request"));
        assert!(req.scripts.post_response.unwrap().contains("Status is 201"));
    }

    #[test]
    fn test_import_openapi_and_export_with_vendor_overlays() {
        let openapi_json = r#"{
            "openapi": "3.1.0",
            "info": {
                "title": "Petstore API",
                "version": "1.0.0"
            },
            "servers": [
                { "url": "https://api.petstore.io/v1" }
            ],
            "paths": {
                "/pets": {
                    "get": {
                        "summary": "List Pets",
                        "parameters": [
                            {
                                "name": "status",
                                "in": "query",
                                "example": "available"
                            }
                        ]
                    },
                    "post": {
                        "summary": "Add Pet",
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "example": { "name": "Buddy" }
                                }
                            }
                        }
                    }
                }
            }
        }"#;

        let reqs = OpenApiImporter::import_spec(openapi_json).unwrap();
        assert_eq!(reqs.len(), 2);

        let get_req = reqs.iter().find(|r| r.method == "GET").unwrap();
        assert_eq!(get_req.name, "List Pets");
        assert_eq!(get_req.url, "https://api.petstore.io/v1/pets");
        assert_eq!(get_req.query_params.get("status").unwrap(), "available");

        let mut post_req = reqs.iter().find(|r| r.method == "POST").unwrap().clone();
        post_req.flow = Some(FlowControl {
            once_per_run: true,
            retry_until: Some("response.status == 201".into()),
            retry_interval_ms: 500,
            max_retries: 5,
            delay_ms: Some(100),
            jitter_min_ms: Some(50),
            jitter_max_ms: Some(150),
        });

        // Now test OpenAPI v3.1 Exporter
        let exported_spec = OpenApiExporter::export_openapi_v3_1(
            &[get_req.clone(), post_req],
            "Exported Petstore",
            "2.0.0",
        )
        .unwrap();

        assert!(exported_spec.contains("\"openapi\": \"3.1.0\""));
        assert!(exported_spec.contains("\"title\": \"Exported Petstore\""));
        assert!(exported_spec.contains("\"x-yarac-flow\""));
        assert!(exported_spec.contains("\"retry_until\": \"response.status == 201\""));
        assert!(exported_spec.contains("\"delay_ms\": 100"));
    }
}
