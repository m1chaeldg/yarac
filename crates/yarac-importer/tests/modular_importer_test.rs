use yarac_core::models::AuthConfig;
use yarac_importer::{
    serialize_to_yarac, BrunoImporter, OpenApiExporter, OpenApiImporter, PostmanImporter,
};

#[test]
fn test_postman_modular_import() {
    let json = r#"{
        "info": { "name": "Modular Test Col" },
        "item": [
            {
                "name": "Get Status",
                "request": {
                    "method": "GET",
                    "url": {
                        "raw": "https://api.example.com/status?version=2",
                        "query": [
                            { "key": "version", "value": "2" }
                        ]
                    },
                    "header": [
                        { "key": "Accept", "value": "application/json" }
                    ],
                    "auth": {
                        "type": "bearer",
                        "bearer": [
                            { "key": "token", "value": "modular-secret" }
                        ]
                    }
                }
            }
        ]
    }"#;

    let reqs = PostmanImporter::import_collection(json).expect("Failed to import Postman collection");
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].name, "Get Status");
    assert_eq!(reqs[0].method, "GET");
    assert_eq!(reqs[0].query_params.get("version").unwrap(), "2");
    assert_eq!(
        reqs[0].auth,
        Some(AuthConfig::Bearer {
            token: "modular-secret".into()
        })
    );

    let serialized = serialize_to_yarac(&reqs[0]);
    assert!(serialized.contains("version = \"1.0\""));
    assert!(serialized.contains("name = \"Get Status\""));
    assert!(serialized.contains("type = \"bearer\""));
}

#[test]
fn test_bruno_modular_import() {
    let bru = r#"
meta {
  name: Query Users
  type: http
  seq: 1
}

get {
  url: https://api.yarac.dev/users
}

headers {
  X-Client: YaracTest
}

params:query {
  limit: 50
}

auth:bearer {
  token: bru-token-xyz
}

tests {
  test("Status 200", function() {
    expect(res.getStatus()).toBe(200);
  });
}
"#;

    let req = BrunoImporter::import_bru(bru).expect("Failed to import Bruno .bru");
    assert_eq!(req.name, "Query Users");
    assert_eq!(req.method, "GET");
    assert_eq!(req.headers.get("X-Client").unwrap(), "YaracTest");
    assert_eq!(req.query_params.get("limit").unwrap(), "50");
    assert_eq!(
        req.auth,
        Some(AuthConfig::Bearer {
            token: "bru-token-xyz".into()
        })
    );
    assert!(req.scripts.post_response.unwrap().contains("Status 200"));
}

#[test]
fn test_openapi_modular_roundtrip() {
    let spec = r#"{
        "openapi": "3.1.0",
        "info": {
            "title": "Modular Store",
            "version": "1.0.0"
        },
        "paths": {
            "/items": {
                "get": {
                    "summary": "List Items",
                    "parameters": [
                        {
                            "name": "category",
                            "in": "query",
                            "example": "books"
                        }
                    ]
                }
            }
        }
    }"#;

    let reqs = OpenApiImporter::import_spec(spec).expect("Failed to import OpenAPI spec");
    assert_eq!(reqs.len(), 1);
    assert_eq!(reqs[0].name, "List Items");
    assert_eq!(reqs[0].query_params.get("category").unwrap(), "books");

    let exported = OpenApiExporter::export_openapi_v3_1(&reqs, "Exported Modular Store", "1.0.0")
        .expect("Failed to export OpenAPI v3.1");
    assert!(exported.contains("\"openapi\": \"3.1.0\""));
    assert!(exported.contains("Modular Store") || exported.contains("Exported Modular Store"));
    assert!(exported.contains("/items"));
}
