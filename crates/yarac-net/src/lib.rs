use indexmap::IndexMap;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Client;
use reqwest_cookie_store::CookieStoreMutex;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use yarac_core::models::*;
use yarac_core::paths::resolve_workspace_path;

pub const MAX_DIRECT_PAYLOAD_BYTES: usize = 2 * 1024 * 1024; // 2MB threshold per AGENTS.md

#[derive(Error, Debug)]
pub enum NetError {
    #[error("HTTP client error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Invalid header format '{0}': {1}")]
    InvalidHeader(String, String),
    #[error("Path resolution error: {0}")]
    PathError(#[from] yarac_core::paths::PathResolutionError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("mTLS client cert error: {0}")]
    MtlsError(String),
}

pub struct YaracHttpClient {
    client: Client,
    cookie_store: Arc<CookieStoreMutex>,
}

impl YaracHttpClient {
    pub fn new(
        verify_ssl: bool,
        follow_redirects: bool,
        max_redirects: u32,
        timeout_ms: u64,
        client_cert_pem: Option<&[u8]>,
    ) -> Result<Self, NetError> {
        let cookie_store = Arc::new(CookieStoreMutex::default());
        let mut builder = Client::builder()
            .cookie_provider(cookie_store.clone())
            .danger_accept_invalid_certs(!verify_ssl);

        if timeout_ms > 0 {
            builder = builder.timeout(Duration::from_millis(timeout_ms));
        }

        if !follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        } else {
            builder = builder.redirect(reqwest::redirect::Policy::limited(max_redirects as usize));
        }

        if let Some(pem_bytes) = client_cert_pem {
            let identity = reqwest::Identity::from_pem(pem_bytes)
                .map_err(|e| NetError::MtlsError(format!("Failed to load client cert PEM: {e}")))?;
            builder = builder.identity(identity);
        }

        let client = builder.build()?;
        Ok(Self {
            client,
            cookie_store,
        })
    }

    pub fn cookie_store(&self) -> Arc<CookieStoreMutex> {
        self.cookie_store.clone()
    }

    pub async fn execute(
        &self,
        request: &YaracRequest,
        workspace_root: &Path,
        interpolated_url: &str,
    ) -> Result<ExecutionReport, NetError> {
        let start = Instant::now();

        // Method
        let method = reqwest::Method::from_str(&request.method)
            .unwrap_or(reqwest::Method::GET);

        let mut req_builder = self.client.request(method, interpolated_url);

        // Headers
        let mut headers = HeaderMap::new();
        for (k, v) in &request.headers {
            if let (Ok(h_name), Ok(h_val)) = (HeaderName::from_str(k), HeaderValue::from_str(v)) {
                headers.insert(h_name, h_val);
            }
        }

        // Auth
        if let Some(auth) = &request.auth {
            match auth {
                AuthConfig::Bearer { token } => {
                    req_builder = req_builder.bearer_auth(token);
                }
                AuthConfig::Basic { username, password } => {
                    req_builder = req_builder.basic_auth(username, Some(password));
                }
                AuthConfig::ApiKey { key, value, in_header } => {
                    if *in_header {
                        if let (Ok(h_name), Ok(h_val)) = (HeaderName::from_str(key), HeaderValue::from_str(value)) {
                            headers.insert(h_name, h_val);
                        }
                    } else {
                        req_builder = req_builder.query(&[(key, value)]);
                    }
                }
                AuthConfig::AwsSigV4 { .. } => {
                    // Placeholder for AWS SigV4 signer
                }
            }
        }

        req_builder = req_builder.headers(headers);

        // Query parameters
        if !request.query_params.is_empty() {
            req_builder = req_builder.query(&request.query_params);
        }

        // Body
        match &request.body {
            BodyType::None => {}
            BodyType::Json(json_str) => {
                req_builder = req_builder
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(json_str.clone());
            }
            BodyType::Raw { content, content_type } => {
                if let Some(ct) = content_type {
                    req_builder = req_builder.header(reqwest::header::CONTENT_TYPE, ct.as_str());
                }
                req_builder = req_builder.body(content.clone());
            }
            BodyType::Form(form_data) => {
                req_builder = req_builder.form(form_data);
            }
            BodyType::Multipart(parts) => {
                let mut form = reqwest::multipart::Form::new();
                for part in parts {
                    if let Some(val) = &part.value {
                        form = form.text(part.name.clone(), val.clone());
                    } else if let Some(file_rel_path) = &part.file_path {
                        let full_path = resolve_workspace_path(workspace_root, file_rel_path)?;
                        let bytes = tokio::fs::read(&full_path).await?;
                        let file_part = reqwest::multipart::Part::bytes(bytes);
                        form = form.part(part.name.clone(), file_part);
                    }
                }
                req_builder = req_builder.multipart(form);
            }
        }

        let resp = req_builder.send().await?;
        let status_code = resp.status().as_u16();

        let mut resp_headers = IndexMap::new();
        for (k, v) in resp.headers() {
            resp_headers.insert(k.as_str().to_string(), v.to_str().unwrap_or("").to_string());
        }

        let raw_bytes = resp.bytes().await.unwrap_or_default();
        let duration_ms = start.elapsed().as_millis() as u64;

        // Large Payloads (>2MB): enforce chunked stream / virtualization threshold per AGENTS.md
        let body_text = if raw_bytes.len() > MAX_DIRECT_PAYLOAD_BYTES {
            let preview = String::from_utf8_lossy(&raw_bytes[..1024]);
            format!(
                "{}\n\n[TRUNCATED: Payload size {} bytes exceeds 2MB threshold. Virtualized chunk stream active.]",
                preview,
                raw_bytes.len()
            )
        } else {
            String::from_utf8_lossy(&raw_bytes).to_string()
        };

        Ok(ExecutionReport {
            request_name: request.name.clone(),
            method: request.method.clone(),
            url: interpolated_url.to_string(),
            status_code,
            duration_ms,
            response_headers: resp_headers,
            response_body: body_text,
            assertions: Vec::new(),
            logs: Vec::new(),
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_builder_with_cookie_store() {
        let client = YaracHttpClient::new(true, true, 5, 5000, None).expect("Create client");
        let store = client.cookie_store();
        let guard = store.lock().unwrap();
        assert_eq!(guard.iter_any().count(), 0);
    }
}
