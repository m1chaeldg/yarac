use indexmap::IndexMap;
use serde_json::Value;
use std::path::{Path, PathBuf};
use yarac_core::models::*;
use crate::{serializer, ImportError};

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
        serializer::export_requests_to_workspace(requests, target_dir)
    }

    pub fn serialize_to_yarac(req: &YaracRequest) -> String {
        serializer::serialize_to_yarac(req)
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
