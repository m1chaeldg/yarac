use std::path::{Path, PathBuf};
use yarac_core::models::*;
use crate::ImportError;

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
