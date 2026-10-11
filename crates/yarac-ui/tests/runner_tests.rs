use yarac_ui::state::runner_state::*;

#[test]
fn test_runner_state_iteration_and_hdr_histogram_percentiles() {
    let mut runner = RunnerState::new(vec!["req_1".into(), "req_2".into()], 10, 1, 0);
    assert_eq!(runner.iterations, 10);
    assert_eq!(runner.concurrency, 1);
    assert_eq!(runner.request_ids.len(), 2);

    // Record sample responses
    let sample_latencies = vec![10, 12, 15, 18, 20, 25, 30, 45, 100, 250];
    for (i, &lat) in sample_latencies.iter().enumerate() {
        runner.record_result(IterationResult {
            iteration: i + 1,
            request_name: "req_1".into(),
            status_code: 200,
            duration_ms: lat,
            passed: true,
            error: None,
        });
    }

    let stats = runner.compute_stats();
    assert_eq!(stats.count, 10);
    assert_eq!(stats.success_count, 10);
    assert_eq!(stats.failure_count, 0);
    assert_eq!(stats.min_ms, 10);
    assert_eq!(stats.max_ms, 250);
    assert!(stats.p50_ms >= 18 && stats.p50_ms <= 25);
    assert!(stats.p99_ms >= 200);
}
