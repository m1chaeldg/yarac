use crate::models::{AuthConfig, BodyType, VariableValue, YaracRequest};
use indexmap::IndexMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeLevel {
    Local,
    Collection,
    Environment,
    Workspace,
    Global,
}

#[derive(Debug, Clone)]
pub struct ScopeResolutionResult {
    pub value: String,
    pub level: ScopeLevel,
    pub is_secret: bool,
    pub keyring_ref: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct VariableScopeStack {
    pub local: IndexMap<String, VariableValue>,
    pub collection: IndexMap<String, VariableValue>,
    pub environment: IndexMap<String, VariableValue>,
    pub workspace: IndexMap<String, VariableValue>,
    pub global: IndexMap<String, VariableValue>,
}

impl VariableScopeStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lookup(&self, key: &str) -> Option<ScopeResolutionResult> {
        if key.ends_with('~') || key.starts_with('~') {
            return None;
        }

        // Dynamic variables (evaluated on demand)
        if key.starts_with('$') {
            if let Some(dyn_val) = eval_dynamic_var(key) {
                return Some(ScopeResolutionResult {
                    value: dyn_val,
                    level: ScopeLevel::Local,
                    is_secret: false,
                    keyring_ref: None,
                });
            }
        }

        // Ordered hierarchy: Local -> Collection -> Environment -> Workspace -> Global
        if let Some(var) = self.local.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Local,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.collection.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Collection,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.environment.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Environment,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.workspace.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Workspace,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }
        if let Some(var) = self.global.get(key) {
            return Some(ScopeResolutionResult {
                value: var.value.clone(),
                level: ScopeLevel::Global,
                is_secret: var.is_secret,
                keyring_ref: var.keyring_ref.clone(),
            });
        }

        None
    }

    /// Interpolates all `{{variable}}` occurrences without using regex.
    pub fn interpolate(&self, input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.char_indices().peekable();

        while let Some((idx, c)) = chars.next() {
            if c == '{' && input[idx..].starts_with("{{") {
                // Find closing '}}'
                if let Some(close_offset) = input[idx + 2..].find("}}") {
                    let var_name = input[idx + 2..idx + 2 + close_offset].trim();
                    if let Some(resolved) = self.lookup(var_name) {
                        result.push_str(&resolved.value);
                    } else {
                        // Keep unchanged if not found
                        result.push_str(&input[idx..idx + 4 + close_offset]);
                    }
                    // Skip past '}}'
                    let target_idx = idx + 4 + close_offset;
                    while let Some(&(next_idx, _)) = chars.peek() {
                        if next_idx < target_idx {
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
            result.push(c);
        }

        result
    }

    /// Recursively interpolates all fields of a YaracRequest (url, headers, query params, auth, body).
    pub fn interpolate_request(&self, req: &mut YaracRequest) -> String {
        let effective_stack = if !req.variables.is_empty() {
            let mut s = self.clone();
            for (k, v) in &req.variables {
                s.local.insert(k.clone(), VariableValue::new(v.clone()));
            }
            s
        } else {
            self.clone()
        };

        let interpolated_url = effective_stack.interpolate(&req.url);

        // Interpolate headers
        for (_, v) in req.headers.iter_mut() {
            *v = effective_stack.interpolate(v);
        }

        // Interpolate query parameters
        for (_, v) in req.query_params.iter_mut() {
            *v = effective_stack.interpolate(v);
        }

        // Interpolate auth
        if let Some(auth) = &mut req.auth {
            match auth {
                AuthConfig::Bearer { token } => {
                    *token = effective_stack.interpolate(token);
                }
                AuthConfig::Basic { username, password } => {
                    *username = effective_stack.interpolate(username);
                    *password = effective_stack.interpolate(password);
                }
                AuthConfig::ApiKey { key, value, .. } => {
                    *key = effective_stack.interpolate(key);
                    *value = effective_stack.interpolate(value);
                }
                AuthConfig::AwsSigV4 {
                    access_key,
                    secret_key,
                    region,
                    service,
                    session_token,
                } => {
                    *access_key = effective_stack.interpolate(access_key);
                    *secret_key = effective_stack.interpolate(secret_key);
                    *region = effective_stack.interpolate(region);
                    *service = effective_stack.interpolate(service);
                    if let Some(tok) = session_token {
                        *tok = effective_stack.interpolate(tok);
                    }
                }
            }
        }

        // Interpolate body
        match &mut req.body {
            BodyType::Json(s) => {
                *s = effective_stack.interpolate(s);
            }
            BodyType::Raw { content, .. } => {
                *content = effective_stack.interpolate(content);
            }
            BodyType::Form(map) => {
                for (_, v) in map.iter_mut() {
                    *v = effective_stack.interpolate(v);
                }
            }
            BodyType::Multipart(parts) => {
                for part in parts.iter_mut() {
                    if let Some(val) = &mut part.value {
                        *val = effective_stack.interpolate(val);
                    }
                }
            }
            BodyType::None => {}
        }

        interpolated_url
    }
}

fn eval_dynamic_var(key: &str) -> Option<String> {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let nanos = since_epoch.subsec_nanos() as u64;
    let secs = since_epoch.as_secs();

    const FIRST_NAMES: &[&str] = &[
        "James", "Mary", "Robert", "Patricia", "John", "Jennifer", "Michael", "Linda",
        "David", "Elizabeth", "William", "Barbara", "Richard", "Susan", "Joseph", "Jessica",
        "Thomas", "Sarah", "Charles", "Karen", "Daniel", "Nancy", "Matthew", "Lisa",
    ];

    const LAST_NAMES: &[&str] = &[
        "Smith", "Johnson", "Williams", "Brown", "Jones", "Garcia", "Miller", "Davis",
        "Rodriguez", "Martinez", "Hernandez", "Lopez", "Gonzalez", "Wilson", "Anderson",
        "Thomas", "Taylor", "Moore", "Jackson", "Martin", "Lee", "Perez", "Thompson",
    ];

    const CITIES: &[&str] = &[
        "New York", "London", "Tokyo", "Paris", "Berlin", "Sydney", "Toronto", "Singapore",
        "San Francisco", "Amsterdam", "Dublin", "Seoul", "Austin", "Zurich", "Madrid",
    ];

    const COUNTRIES: &[&str] = &[
        "United States", "United Kingdom", "Canada", "Germany", "France", "Japan", "Australia",
        "Netherlands", "Sweden", "Switzerland", "Ireland", "Singapore", "New Zealand",
    ];

    const COUNTRY_CODES: &[&str] = &[
        "US", "GB", "CA", "DE", "FR", "JP", "AU", "NL", "SE", "CH", "IE", "SG", "NZ",
    ];

    const STREET_NAMES: &[&str] = &[
        "Main St", "High St", "Maple Ave", "Oak St", "Park Blvd", "Broadway", "Market St",
        "Cedar Lane", "Pine St", "Elm St", "Washington Ave", "Lakeview Dr",
    ];

    const WORDS: &[&str] = &[
        "agile", "robust", "scalable", "ephemeral", "secure", "distributed", "protocol",
        "payload", "stream", "pipeline", "cluster", "gateway", "latency", "throughput",
        "matrix", "vector", "socket", "endpoint", "cache", "token", "session",
    ];

    const COMPANIES: &[&str] = &[
        "Acme Corp", "Globex", "Initech", "Umbrella", "Cyberdyne", "Soylent", "Massive Dynamic",
        "Hooli", "Pied Piper", "Stark Industries", "Wayne Enterprises",
    ];

    const JOB_TITLES: &[&str] = &[
        "Software Engineer", "DevOps Specialist", "Product Manager", "QA Architect",
        "Systems Analyst", "Site Reliability Engineer", "Security Researcher", "Data Scientist",
    ];

    let fn_idx = (nanos as usize) % FIRST_NAMES.len();
    let ln_idx = ((nanos / 7) as usize) % LAST_NAMES.len();
    let city_idx = ((nanos / 13) as usize) % CITIES.len();
    let country_idx = ((nanos / 17) as usize) % COUNTRIES.len();
    let street_idx = ((nanos / 23) as usize) % STREET_NAMES.len();
    let word_idx = ((nanos / 29) as usize) % WORDS.len();
    let company_idx = ((nanos / 31) as usize) % COMPANIES.len();
    let job_idx = ((nanos / 37) as usize) % JOB_TITLES.len();

    match key {
        "$uuid" | "$guid" => Some(Uuid::new_v4().to_string()),
        "$timestamp" => Some(secs.to_string()),
        "$timestampMs" => Some(since_epoch.as_millis().to_string()),
        "$isoTimestamp" => {
            // ISO 8601 UTC formatted string
            let total_secs = secs;
            let s = total_secs % 60;
            let m = (total_secs / 60) % 60;
            let h = (total_secs / 3600) % 24;
            let days_since_epoch = total_secs / 86400;

            // Approximate year/month/day calculation
            let mut days = days_since_epoch;
            let mut year = 1970;
            loop {
                let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
                let days_in_year = if leap { 366 } else { 365 };
                if days >= days_in_year {
                    days -= days_in_year;
                    year += 1;
                } else {
                    break;
                }
            }
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let month_days = [
                31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
            ];
            let mut month = 1;
            for &md in &month_days {
                if days >= md {
                    days -= md;
                    month += 1;
                } else {
                    break;
                }
            }
            let day = days + 1;
            Some(format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                year, month, day, h, m, s, since_epoch.subsec_millis()
            ))
        }
        "$randomInt" => Some(((nanos % 9000) + 1000).to_string()),
        "$randomFloat" => {
            let whole = (nanos % 900) + 10;
            let frac = (nanos / 3) % 100;
            Some(format!("{}.{:02}", whole, frac))
        }
        "$randomFirstName" => Some(FIRST_NAMES[fn_idx].to_string()),
        "$randomLastName" => Some(LAST_NAMES[ln_idx].to_string()),
        "$randomFullName" => Some(format!("{} {}", FIRST_NAMES[fn_idx], LAST_NAMES[ln_idx])),
        "$randomUserName" => Some(format!(
            "{}_{}",
            FIRST_NAMES[fn_idx].to_lowercase(),
            (nanos % 900) + 100
        )),
        "$randomEmail" => Some(format!(
            "{}.{}{}@example.com",
            FIRST_NAMES[fn_idx].to_lowercase(),
            LAST_NAMES[ln_idx].to_lowercase(),
            (nanos % 90) + 10
        )),
        "$randomPassword" => Some(format!(
            "P@ssw0rd_{:x}_{:x}",
            nanos % 0xffff,
            (secs / 7) % 0xffff
        )),
        "$randomBoolean" => Some(if nanos % 2 == 0 { "true".to_string() } else { "false".to_string() }),
        "$randomCity" => Some(CITIES[city_idx].to_string()),
        "$randomCountry" => Some(COUNTRIES[country_idx].to_string()),
        "$randomCountryCode" => Some(COUNTRY_CODES[country_idx % COUNTRY_CODES.len()].to_string()),
        "$randomStreetAddress" => Some(format!("{} {}", (nanos % 9000) + 100, STREET_NAMES[street_idx])),
        "$randomStreetName" => Some(STREET_NAMES[street_idx].to_string()),
        "$randomPhoneNumber" => Some(format!(
            "+1-555-{:03}-{:04}",
            (nanos % 900) + 100,
            (nanos / 1000) % 9000 + 1000
        )),
        "$randomWord" => Some(WORDS[word_idx].to_string()),
        "$randomWords" => {
            let w1 = WORDS[word_idx];
            let w2 = WORDS[(word_idx + 1) % WORDS.len()];
            let w3 = WORDS[(word_idx + 2) % WORDS.len()];
            Some(format!("{} {} {}", w1, w2, w3))
        }
        "$randomAlphaNumeric" => {
            let mut s = String::with_capacity(16);
            for i in 0..16 {
                let shift = (nanos.wrapping_mul(i + 1)) % 36;
                let c = if shift < 10 {
                    (b'0' + shift as u8) as char
                } else {
                    (b'a' + (shift - 10) as u8) as char
                };
                s.push(c);
            }
            Some(s)
        }
        "$randomHexColor" => Some(format!("#{:06x}", nanos % 0xffffff)),
        "$randomIP" => Some(format!(
            "192.168.{}.{}",
            (nanos % 254) + 1,
            ((nanos / 256) % 254) + 1
        )),
        "$randomIPv6" => Some(format!(
            "2001:db8:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}",
            nanos % 0xffff,
            (nanos / 2) % 0xffff,
            (nanos / 4) % 0xffff,
            (nanos / 8) % 0xffff,
            (nanos / 16) % 0xffff,
            (nanos / 32) % 0xffff
        )),
        "$randomUrl" => Some(format!(
            "https://{}.example.com/api/v1",
            WORDS[word_idx]
        )),
        "$randomPrice" => Some(format!("{}.{:02}", (nanos % 500) + 5, (nanos / 7) % 100)),
        "$randomCompany" => Some(COMPANIES[company_idx].to_string()),
        "$randomJobTitle" => Some(JOB_TITLES[job_idx].to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolation_hierarchy() {
        let mut stack = VariableScopeStack::new();
        stack.environment.insert("base_url".into(), VariableValue::new("https://api.yarac.dev"));
        stack.local.insert("endpoint".into(), VariableValue::new("users"));

        let template = "{{base_url}}/api/v1/{{endpoint}}";
        let interpolated = stack.interpolate(template);
        assert_eq!(interpolated, "https://api.yarac.dev/api/v1/users");
    }

    #[test]
    fn test_dynamic_uuid() {
        let stack = VariableScopeStack::new();
        let out = stack.interpolate("id={{$uuid}}");
        assert!(out.starts_with("id="));
        assert_eq!(out.len(), 3 + 36);
    }

    #[test]
    fn test_interpolate_request_complete() {
        let mut stack = VariableScopeStack::new();
        stack.environment.insert("host".into(), VariableValue::new("127.0.0.1:9475"));
        stack.environment.insert("token".into(), VariableValue::new("secret_bearer_123"));
        stack.environment.insert("user_id".into(), VariableValue::new("42"));

        let mut req = YaracRequest {
            url: "http://{{host}}/api/users/{{user_id}}".into(),
            headers: [("Authorization".into(), "Bearer {{token}}".into())].into_iter().collect(),
            auth: Some(AuthConfig::Bearer { token: "{{token}}".into() }),
            query_params: [("q".into(), "{{user_id}}".into())].into_iter().collect(),
            body: BodyType::Json(r#"{"id": "{{user_id}}"}"#.into()),
            ..Default::default()
        };

        let url = stack.interpolate_request(&mut req);
        assert_eq!(url, "http://127.0.0.1:9475/api/users/42");
        assert_eq!(req.headers.get("Authorization").unwrap(), "Bearer secret_bearer_123");
        assert_eq!(req.query_params.get("q").unwrap(), "42");
        if let Some(AuthConfig::Bearer { token }) = req.auth {
            assert_eq!(token, "secret_bearer_123");
        } else {
            panic!("Auth not bearer");
        }
        if let BodyType::Json(ref s) = req.body {
            assert_eq!(s, r#"{"id": "42"}"#);
        } else {
            panic!("Body not json");
        }
    }
}
