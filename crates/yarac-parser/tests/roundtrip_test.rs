use yarac_parser::{parse, serialize};

#[test]
fn test_roundtrip_spec_file() {
    let spec_content = include_str!("../../../yarac-spec/specs/file_format.md");
    let cst = parse(spec_content).expect("Failed to parse spec file");
    let serialized = serialize(&cst);
    assert_eq!(serialized, spec_content, "Round-trip invariant failed on spec file");

    let req = cst.to_request().expect("Failed to map CST to model");
    assert_eq!(req.version, "1.0");
    assert_eq!(req.name, "Create User Account");
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "{{base_url}}/api/v1/users");
    assert_eq!(req.settings.inherit_auth, true);
    assert_eq!(req.headers.get("Content-Type").unwrap(), "application/json");
    assert!(req.scripts.pre_request.is_some());
    assert!(req.scripts.post_response.is_some());
}

#[test]
fn test_roundtrip_crlf_and_lf() {
    let lf_input = "version = \"1.0\"\nname = \"Test\"\n\n[headers]\nX-Key = \"Val\"\n";
    let cst_lf = parse(lf_input).expect("Parse LF");
    assert_eq!(serialize(&cst_lf), lf_input);

    let crlf_input = "version = \"1.0\"\r\nname = \"Test\"\r\n\r\n[headers]\r\nX-Key = \"Val\"\r\n";
    let cst_crlf = parse(crlf_input).expect("Parse CRLF");
    assert_eq!(serialize(&cst_crlf), crlf_input);
}

#[test]
fn test_conflict_marker_roundtrip() {
    let conflict_input = "version = \"1.0\"\n<<<<<<< HEAD\nname = \"Local Branch\"\n=======\nname = \"Remote Branch\"\n>>>>>>> main\n\n[headers]\nX-Key = \"Val\"\n";
    let cst = parse(conflict_input).expect("Parse conflict block");
    assert!(cst.has_conflicts(), "Should detect git conflict markers");
    assert_eq!(serialize(&cst), conflict_input, "Roundtrip must preserve conflict markers");
}

#[test]
fn test_roundtrip_flow_with_pacing_and_jitter() {
    let flow_input = "version = \"1.0\"\nname = \"Pacing Test\"\nmethod = \"GET\"\nurl = \"http://api.test/items\"\n\n[flow]\nonce_per_run = true\nretry_until = \"response.status == 200\"\nretry_interval_ms = 500\nmax_retries = 3\ndelay_ms = 250\njitter_min_ms = 50\njitter_max_ms = 150\n";
    let cst = parse(flow_input).expect("Parse flow");
    assert_eq!(serialize(&cst), flow_input, "Round-trip must match flow input exactly");

    let req = cst.to_request().expect("To request");
    let flow = req.flow.expect("Flow present");
    assert_eq!(flow.once_per_run, true);
    assert_eq!(flow.retry_until.as_deref(), Some("response.status == 200"));
    assert_eq!(flow.retry_interval_ms, 500);
    assert_eq!(flow.max_retries, 3);
    assert_eq!(flow.delay_ms, Some(250));
    assert_eq!(flow.jitter_min_ms, Some(50));
    assert_eq!(flow.jitter_max_ms, Some(150));
}

#[test]
fn test_roundtrip_variables_assert_tests_settings() {
    let input = "version = \"1.0\"\nname = \"Complete Request\"\nmethod = \"POST\"\nurl = \"https://api.example.com/items\"\n\n[settings]\ninherit_auth = true\nverify_ssl = true\nfollow_redirects = true\nmax_redirects = 10\ntimeout_ms = 0\n\n[variables]\nitem_id = \"item_999\"\napi_mode = \"strict\"\n\n[assert]\n```javascript\nresponse.status === 200\nresponse.json().success === true\n```\n\n[tests]\n```javascript\ndescribe(\"Item Verification\", () => {\n  it(\"matches expectation\", () => {\n    expect(response.status).toBe(200);\n  });\n});\n```\n";
    let cst = parse(input).expect("Parse full request");
    assert_eq!(serialize(&cst), input, "Roundtrip invariant must hold for variables, assert, tests, settings");

    let req = cst.to_request().expect("To request");
    assert_eq!(req.settings.max_redirects, 10);
    assert_eq!(req.settings.timeout_ms, 0);
    assert_eq!(req.variables.get("item_id").unwrap(), "item_999");
    assert_eq!(req.variables.get("api_mode").unwrap(), "strict");
    assert!(req.assert.unwrap().contains("response.status === 200"));
    assert!(req.tests.unwrap().contains("Item Verification"));
}

