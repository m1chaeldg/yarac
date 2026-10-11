use hdrhistogram::Histogram;
use yarac_core::models::*;
use yarac_importer::{BrunoImporter, OpenApiExporter, OpenApiImporter};
use yarac_parser::{parse, serialize};
use yarac_scripting::ScriptRunner;


#[test]
fn test_flow_control_parsing_and_pacing() {
    let dsl = r#"version = "1.0"
name = "Flow Test"
method = "GET"
url = "http://localhost/poll"

[flow]
once_per_run = true
retry_until = "response.status == 200 && response.body.status == 'ready'"
retry_interval_ms = 500
max_retries = 5
delay_ms = 100
jitter_min_ms = 20
jitter_max_ms = 80
"#;

    let cst = parse(dsl).expect("Parse flow DSL");
    assert_eq!(serialize(&cst), dsl);

    let req = cst.to_request().expect("To request");
    let flow = req.flow.expect("Flow exists");
    assert!(flow.once_per_run);
    assert_eq!(
        flow.retry_until.as_deref(),
        Some("response.status == 200 && response.body.status == 'ready'")
    );
    assert_eq!(flow.retry_interval_ms, 500);
    assert_eq!(flow.max_retries, 5);
    assert_eq!(flow.delay_ms, Some(100));
    assert_eq!(flow.jitter_min_ms, Some(20));
    assert_eq!(flow.jitter_max_ms, Some(80));

    // Pacing duration calculation test
    for _ in 0..10 {
        let delay = flow.calculate_pacing_delay().unwrap();
        let ms = delay.as_millis();
        assert!(ms >= 120 && ms <= 180, "Delay {ms}ms must be within 100 + [20, 80]");
    }
}

#[tokio::test]
async fn test_flow_retry_until_predicate_resolution() {
    let runner = ScriptRunner::default();
    let headers = indexmap::IndexMap::new();

    let predicate = "response.status == 200 && response.body.state == 'completed'";

    // Pending state
    let ready1 = runner
        .evaluate_predicate(
            predicate,
            200,
            r#"{"state": "processing"}"#,
            &headers,
        )
        .await
        .unwrap();
    assert!(!ready1);

    // Completed state
    let ready2 = runner
        .evaluate_predicate(
            predicate,
            200,
            r#"{"state": "completed"}"#,
            &headers,
        )
        .await
        .unwrap();
    assert!(ready2);
}

#[test]
fn test_openapi_3_1_export_and_import_cycle() {
    let mut req = YaracRequest {
        version: "1.0".into(),
        name: "Create Order".into(),
        method: "POST".into(),
        url: "{{base_url}}/api/v1/orders".into(),
        settings: Settings::default(),
        headers: indexmap::IndexMap::new(),
        auth: Some(AuthConfig::Bearer {
            token: "{{api_key}}".into(),
        }),
        query_params: indexmap::IndexMap::new(),
        flow: Some(FlowControl {
            once_per_run: false,
            retry_until: Some("response.status == 201".into()),
            retry_interval_ms: 1000,
            max_retries: 3,
            delay_ms: Some(150),
            jitter_min_ms: Some(10),
            jitter_max_ms: Some(50),
        }),
        body: BodyType::Json(r#"{"item_id": 42, "qty": 1}"#.into()),
        scripts: ScriptHooks {
            pre_request: Some("console.log('Order starting');".into()),
            post_response: Some("expect(yarac.response.status).to.equal(201);".into()),
        },
        variables: indexmap::IndexMap::new(),
        assert: None,
        tests: None,
    };
    req.headers.insert("Content-Type".into(), "application/json".into());
    req.query_params.insert("fast_track".into(), "true".into());

    let exported_openapi = OpenApiExporter::export_openapi_v3_1(
        &[req.clone()],
        "Order Service API",
        "1.1.0",
    )
    .expect("Export OpenAPI 3.1");

    assert!(exported_openapi.contains("\"openapi\": \"3.1.0\""));
    assert!(exported_openapi.contains("\"title\": \"Order Service API\""));
    assert!(exported_openapi.contains("\"x-yarac-flow\""));
    assert!(exported_openapi.contains("\"x-yarac-scripts\""));
    assert!(exported_openapi.contains("\"retry_until\": \"response.status == 201\""));
    assert!(exported_openapi.contains("\"fast_track\""));

    // Now re-import using OpenApiImporter
    let imported_reqs =
        OpenApiImporter::import_spec(&exported_openapi).expect("Re-import OpenAPI spec");
    assert_eq!(imported_reqs.len(), 1);
    assert_eq!(imported_reqs[0].name, "Create Order");
    assert_eq!(imported_reqs[0].method, "POST");
    assert!(imported_reqs[0].url.contains("/api/v1/orders"));
}

#[test]
fn test_bruno_importer_comprehensive() {
    let bru_content = r#"
meta {
  name: Check Health
  type: http
  seq: 1
}

get {
  url: {{base_url}}/health
  body: none
  auth: none
}

headers {
  Accept: application/json
  X-Custom-Trace: bru-trace-42
}

params:query {
  verbose: true
}

tests {
  test("Health is ok", function() {
    expect(res.getStatus()).to.equal(200);
  });
}
"#;

    let req = BrunoImporter::import_bru(bru_content).expect("Parse Bruno");
    assert_eq!(req.name, "Check Health");
    assert_eq!(req.method, "GET");
    assert_eq!(req.url, "{{base_url}}/health");
    assert_eq!(req.headers.get("Accept").unwrap(), "application/json");
    assert_eq!(req.headers.get("X-Custom-Trace").unwrap(), "bru-trace-42");
    assert_eq!(req.query_params.get("verbose").unwrap(), "true");
    assert!(req.scripts.post_response.unwrap().contains("Health is ok"));
}

#[test]
fn test_hdr_histogram_percentile_profiling() {
    let mut hist = Histogram::<u64>::new_with_bounds(1, 3_600_000, 3).unwrap();

    // Populate simulated latencies (ms)
    for _ in 0..500 {
        hist.record(25).unwrap(); // 50%
    }
    for _ in 0..400 {
        hist.record(50).unwrap(); // up to 90%
    }
    for _ in 0..50 {
        hist.record(80).unwrap(); // up to 95%
    }
    for _ in 0..40 {
        hist.record(120).unwrap(); // up to 99%
    }
    for _ in 0..10 {
        hist.record(250).unwrap(); // max
    }

    assert_eq!(hist.len(), 1000);
    assert_eq!(hist.min(), 25);
    assert_eq!(hist.max(), 250);
    assert!(hist.value_at_quantile(0.50) <= 25);
    assert!(hist.value_at_quantile(0.90) <= 50);
    assert!(hist.value_at_quantile(0.95) <= 80);
    assert!(hist.value_at_quantile(0.99) <= 120);
}

#[test]
fn test_nested_collection_manifest_and_models() {
    let manifest = WorkspaceManifest {
        id: "ws_test".to_string(),
        name: "Test Workspace".to_string(),
        collections: vec![
            CollectionMeta {
                id: "col_root".to_string(),
                name: "Billing".to_string(),
                path: Some("Billing".to_string()),
                parent_id: None,
                request_files: vec![],
            },
            CollectionMeta {
                id: "col_sub".to_string(),
                name: "Invoices".to_string(),
                path: Some("Billing/Invoices".to_string()),
                parent_id: Some("col_root".to_string()),
                request_files: vec![],
            },
        ],
        active_environment_id: None,
        open_tabs: vec!["req_invoice".to_string()],
        active_tab: Some("req_invoice".to_string()),
    };

    let serialized = serde_json::to_string(&manifest).expect("Serialize manifest");
    let deserialized: WorkspaceManifest = serde_json::from_str(&serialized).expect("Deserialize manifest");

    assert_eq!(deserialized.collections.len(), 2);
    assert_eq!(deserialized.collections[1].path.as_deref(), Some("Billing/Invoices"));
    assert_eq!(deserialized.collections[1].parent_id.as_deref(), Some("col_root"));
}
