pub mod bruno;
pub mod openapi;
pub mod postman;
pub mod serializer;

use thiserror::Error;

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

pub use bruno::BrunoImporter;
pub use openapi::{OpenApiExporter, OpenApiImporter};
pub use postman::PostmanImporter;
pub use serializer::{export_requests_to_workspace, serialize_to_yarac};

#[cfg(test)]
mod tests {
    use super::*;
    use yarac_core::models::*;

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
