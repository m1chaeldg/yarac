use yarac_ui::state::app_controller::AppController;
use yarac_ui::state::request_state::RequestState;

#[tokio::test]
async fn test_integration_postman_echo_all_methods() {
    let mut controller = AppController::new();

    // 1. POST https://postman-echo.com/post
    let mut post_req = RequestState::new("Echo POST", "POST", "https://postman-echo.com/post");
    post_req.body_type = "json".into();
    post_req.raw_body = r#"{"name": "yarac", "action": "test_post"}"#.into();
    post_req.add_header("Content-Type", "application/json", true);
    let post_resp = controller.execute_request(&post_req).await
        .expect("POST request to postman-echo.com should succeed");
    eprintln!("POST RESP STATUS: {}", post_resp.status);
    eprintln!("POST RESP BODY: {}", post_resp.body);
    assert_eq!(post_resp.status, 200, "POST response status should be 200");
    assert!(post_resp.body.contains("test_post"), "POST response body should reflect json payload");
    assert!(post_resp.duration_ms > 0, "Duration should be recorded");

    // 2. GET https://postman-echo.com/get?foo1=bar1&foo2=bar2
    let get_req = RequestState::new("Echo GET", "GET", "https://postman-echo.com/get?foo1=bar1&foo2=bar2");
    let get_resp = controller.execute_request(&get_req).await
        .expect("GET request to postman-echo.com should succeed");
    assert_eq!(get_resp.status, 200, "GET response status should be 200");
    assert!(get_resp.body.contains("bar1"), "GET response body should contain foo1=bar1");
    assert!(get_resp.body.contains("bar2"), "GET response body should contain foo2=bar2");

    // 3. DELETE https://postman-echo.com/delete
    let del_req = RequestState::new("Echo DELETE", "DELETE", "https://postman-echo.com/delete");
    let del_resp = controller.execute_request(&del_req).await
        .expect("DELETE request to postman-echo.com should succeed");
    assert_eq!(del_resp.status, 200, "DELETE response status should be 200");

    // 4. PATCH https://postman-echo.com/patch
    let mut patch_req = RequestState::new("Echo PATCH", "PATCH", "https://postman-echo.com/patch");
    patch_req.body_type = "json".into();
    patch_req.raw_body = r#"{"patch": true}"#.into();
    let patch_resp = controller.execute_request(&patch_req).await
        .expect("PATCH request to postman-echo.com should succeed");
    assert_eq!(patch_resp.status, 200, "PATCH response status should be 200");
    assert!(patch_resp.body.contains("patch"), "PATCH response body should echo payload");

    // 5. PUT https://postman-echo.com/put
    let mut put_req = RequestState::new("Echo PUT", "PUT", "https://postman-echo.com/put");
    put_req.body_type = "json".into();
    put_req.raw_body = r#"{"updated": 123}"#.into();
    let put_resp = controller.execute_request(&put_req).await
        .expect("PUT request to postman-echo.com should succeed");
    assert_eq!(put_resp.status, 200, "PUT response status should be 200");
    assert!(put_resp.body.contains("123"), "PUT response body should echo updated value");
}

#[tokio::test]
async fn test_integration_postman_echo_collection_runner() {
    let mut controller = AppController::new();

    let reqs = vec![
        RequestState::new("Echo GET", "GET", "https://postman-echo.com/get?foo1=bar1&foo2=bar2"),
        RequestState::new("Echo POST", "POST", "https://postman-echo.com/post"),
    ];

    let stats = controller.execute_collection_runner(&reqs, 2).await
        .expect("Runner execution should succeed");

    assert_eq!(stats.count, 4, "Total executions across 2 iterations for 2 requests should be 4");
    assert_eq!(stats.failure_count, 0, "No failures expected on postman-echo");
    assert_eq!(stats.success_count, 4, "All 4 requests should pass");
    assert!(stats.min_ms > 0);
    assert!(stats.max_ms >= stats.min_ms);
}
