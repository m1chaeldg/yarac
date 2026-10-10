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
