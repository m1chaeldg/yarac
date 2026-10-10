use indexmap::IndexMap;
use serde_json::Value;
use std::path::Path;
use thiserror::Error;
use yarac_core::models::*;

#[derive(Error, Debug)]
pub enum ImportError {
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid Postman Collection schema: {0}")]
    InvalidSchema(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

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

    /// Exports imported requests as individual `.yarac` files in the specified directory
    pub fn export_to_workspace(
        requests: &[YaracRequest],
        target_dir: &Path,
    ) -> Result<Vec<std::path::PathBuf>, ImportError> {
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

            let content = Self::serialize_to_yarac(req);
            std::fs::write(&file_path, content)?;
            written_files.push(file_path);
        }

        Ok(written_files)
    }

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

        match &req.body {
            BodyType::None => {}
            BodyType::Json(s) => {
                out.push_str("[body.json]\n```json\n");
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
            BodyType::Multipart(_) => {}
        }

        out
    }

    fn extract_items(items: &[Value], out: &mut Vec<YaracRequest>) {
        for item in items {
            if let Some(sub_items) = item.get("item").and_then(|i| i.as_array()) {
                // Folder / sub-collection
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

                // Auth extraction
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
                    flow: None,
                    body,
                    scripts: ScriptHooks::default(),
                });
            }
        }
    }
}

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

        let yarac_dsl = PostmanImporter::serialize_to_yarac(&reqs[0]);
        assert!(yarac_dsl.contains("version = \"1.0\""));
        assert!(yarac_dsl.contains("name = \"Get Users\""));
        assert!(yarac_dsl.contains("type = \"bearer\""));
        assert!(yarac_dsl.contains("token = \"my-secret-token\""));
    }
}
