use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
    pub enabled: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestState {
    pub id: String,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Vec<KeyValueEntry>,
    pub params: Vec<KeyValueEntry>,
    pub auth_type: String, // "none", "bearer", "basic", "sigv4"
    pub bearer_token: Option<String>,
    pub body_type: String, // "none", "json", "raw", "form_data"
    pub raw_body: String,
    pub pre_script: String,
    pub test_script: String,
    pub is_dirty: bool,
}

impl RequestState {
    pub fn new(name: &str, method: &str, url: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            method: method.to_string(),
            url: url.to_string(),
            headers: Vec::new(),
            params: Vec::new(),
            auth_type: "none".to_string(),
            bearer_token: None,
            body_type: "none".to_string(),
            raw_body: String::new(),
            pre_script: String::new(),
            test_script: String::new(),
            is_dirty: false,
        }
    }

    pub fn set_url(&mut self, url: &str) {
        if self.url != url {
            self.url = url.to_string();
            self.is_dirty = true;
        }
    }

    pub fn add_header(&mut self, key: &str, value: &str, enabled: bool) {
        self.headers.push(KeyValueEntry {
            key: key.to_string(),
            value: value.to_string(),
            enabled,
            description: None,
        });
        self.is_dirty = true;
    }

    pub fn toggle_header(&mut self, index: usize) {
        if let Some(h) = self.headers.get_mut(index) {
            h.enabled = !h.enabled;
            self.is_dirty = true;
        }
    }

    pub fn to_dsl(&self) -> String {
        let mut dsl = format!(
            "version = \"1.0\"\nname = \"{}\"\nmethod = \"{}\"\nurl = \"{}\"\n",
            self.name, self.method, self.url
        );

        if !self.headers.is_empty() {
            dsl.push_str("\n[headers]\n");
            for h in &self.headers {
                let key = if h.enabled { h.key.clone() } else { format!("{}~", h.key) };
                dsl.push_str(&format!("\"{}\" = \"{}\"\n", key, h.value));
            }
        }

        if !self.raw_body.is_empty() {
            dsl.push_str("\n[body]\n");
            dsl.push_str(&format!("type = \"{}\"\n", self.body_type));
            dsl.push_str("content = '''\n");
            dsl.push_str(&self.raw_body);
            dsl.push_str("\n'''\n");
        }

        dsl
    }
}
