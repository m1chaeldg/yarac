use indexmap::IndexMap;
use serde_json::Value;
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

                let url = if let Some(url_str) = req_val.get("url").and_then(|u| u.as_str()) {
                    url_str.to_string()
                } else if let Some(url_raw) = req_val.get("url").and_then(|u| u.get("raw")).and_then(|r| r.as_str()) {
                    url_raw.to_string()
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

                let mut body = BodyType::None;
                if let Some(body_obj) = req_val.get("body") {
                    let mode = body_obj.get("mode").and_then(|m| m.as_str()).unwrap_or("");
                    if mode == "raw" {
                        if let Some(raw_text) = body_obj.get("raw").and_then(|r| r.as_str()) {
                            body = BodyType::Json(raw_text.to_string());
                        }
                    }
                }

                out.push(YaracRequest {
                    version: "1.0".to_string(),
                    name,
                    method,
                    url,
                    settings: Settings::default(),
                    headers,
                    auth: None,
                    query_params: IndexMap::new(),
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
    fn test_import_postman_v2_1() {
        let json = r#"{
            "info": { "name": "Test Col" },
            "item": [
                {
                    "name": "Get Users",
                    "request": {
                        "method": "GET",
                        "url": "https://api.example.com/users",
                        "header": [
                            { "key": "Accept", "value": "application/json" }
                        ]
                    }
                }
            ]
        }"#;

        let reqs = PostmanImporter::import_collection(json).unwrap();
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].name, "Get Users");
        assert_eq!(reqs[0].method, "GET");
        assert_eq!(reqs[0].url, "https://api.example.com/users");
        assert_eq!(reqs[0].headers.get("Accept").unwrap(), "application/json");
    }
}
