use yarac_ui::state::workspace_state::*;
use yarac_ui::state::request_state::*;
use yarac_ui::state::environment_state::*;

#[test]
fn test_workspace_state_creation_and_collection_tree() {
    let mut ws = WorkspaceState::new("My Workspace");
    assert_eq!(ws.name, "My Workspace");
    assert!(ws.collections.is_empty());

    let col_id = ws.add_collection("Users API");
    assert_eq!(ws.collections.len(), 1);
    assert_eq!(ws.collections[0].name, "Users API");

    let req_id = ws.create_request(&col_id, "Create User", "POST", "https://api.example.com/users");
    assert_eq!(ws.active_request_id, Some(req_id.clone()));

    let req = ws.get_active_request().expect("Active request must exist");
    assert_eq!(req.name, "Create User");
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "https://api.example.com/users");
}

#[test]
fn test_request_state_dirty_tracking_and_dsl_sync() {
    let mut req = RequestState::new("Test Req", "GET", "https://api.test/v1");
    assert!(!req.is_dirty);

    req.set_url("https://api.test/v2");
    assert!(req.is_dirty);
    assert_eq!(req.url, "https://api.test/v2");

    req.add_header("Authorization", "Bearer my-secret-token", true);
    assert_eq!(req.headers.len(), 1);
    assert_eq!(req.headers[0].key, "Authorization");
    assert!(req.headers[0].enabled);

    // Toggle active state with tilde '~' convention
    req.toggle_header(0);
    assert!(!req.headers[0].enabled);

    // Serialization to Yarac DSL format
    let dsl = req.to_dsl();
    assert!(dsl.contains("name = \"Test Req\""));
    assert!(dsl.contains("method = \"GET\""));
    assert!(dsl.contains("url = \"https://api.test/v2\""));
}

#[test]
fn test_environment_state_and_variable_resolution() {
    let mut env = EnvironmentState::new("Production");
    env.set_variable("base_url", "https://prod.api.com", false);
    env.set_variable("auth_token", "secret-token-xyz", true); // secret

    assert_eq!(env.variables.len(), 2);
    
    let vars = env.active_variables_map();
    assert_eq!(vars.get("base_url"), Some(&"https://prod.api.com".to_string()));
    assert_eq!(vars.get("auth_token"), Some(&"secret-token-xyz".to_string()));

    // Variable interpolation
    let raw_url = "{{base_url}}/users";
    let interpolated = env.interpolate(raw_url);
    assert_eq!(interpolated, "https://prod.api.com/users");
}
