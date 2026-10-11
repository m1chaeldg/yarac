use indexmap::IndexMap;
use serde_json::Value;
use std::path::{Path, PathBuf};
use yarac_core::models::*;
use crate::{serializer, ImportError};

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
                                            .unwrap_or("{{default}}")
                                            .to_string();

                                        match p_in {
                                            "query" => {
                                                query_params.insert(p_name.to_string(), default_val);
                                            }
                                            "header" => {
                                                headers.insert(p_name.to_string(), default_val);
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }

                            // Request body
                            let mut body = BodyType::None;
                            if let Some(rb) = op.get("requestBody").and_then(|r| r.as_object()) {
                                if let Some(content_map) = rb.get("content").and_then(|c| c.as_object()) {
                                    if let Some(app_json) = content_map.get("application/json") {
                                        let ex_str = app_json.get("example")
                                            .map(|ex| serde_json::to_string_pretty(ex).unwrap_or_default())
                                            .unwrap_or_else(|| "{}".to_string());
                                        body = BodyType::Json(ex_str);
                                    } else if let Some(first_entry) = content_map.iter().next() {
                                        let mime = first_entry.0;
                                        let ex_str = first_entry.1.get("example")
                                            .and_then(|ex| ex.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        body = BodyType::Raw {
                                            content: ex_str,
                                            content_type: Some(mime.clone()),
                                        };
                                    }
                                }
                            }

                            // x-yarac-* Vendor Extensions
                            let mut settings = Settings::default();
                            if let Some(s) = op.get("x-yarac-settings") {
                                if let Ok(parsed_settings) = serde_json::from_value(s.clone()) {
                                    settings = parsed_settings;
                                }
                            }

                            let mut flow = None;
                            if let Some(f) = op.get("x-yarac-flow") {
                                if let Ok(parsed_flow) = serde_json::from_value(f.clone()) {
                                    flow = Some(parsed_flow);
                                }
                            }

                            let mut auth = None;
                            if let Some(a) = op.get("x-yarac-auth") {
                                if let Ok(parsed_auth) = serde_json::from_value(a.clone()) {
                                    auth = Some(parsed_auth);
                                }
                            }

                            let mut scripts = ScriptHooks::default();
                            if let Some(sc) = op.get("x-yarac-scripts") {
                                if let Ok(parsed_scripts) = serde_json::from_value(sc.clone()) {
                                    scripts = parsed_scripts;
                                }
                            }

                            let full_url = format!("{}{}", base_url, path);

                            requests.push(YaracRequest {
                                version: "1.0".to_string(),
                                name,
                                method: method.to_uppercase(),
                                url: full_url,
                                settings,
                                headers,
                                auth,
                                query_params,
                                variables: IndexMap::new(),
                                flow,
                                body,
                                scripts,
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
        serializer::export_requests_to_workspace(requests, target_dir)
    }
}

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
