use hdrhistogram::Histogram;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use yarac_core::models::*;
use yarac_core::scope::VariableScopeStack;
use yarac_core::scrubber::{ScrubConfig, SecretRegistry};
use yarac_net::YaracHttpClient;
use yarac_parser::parse;
use yarac_scripting::ScriptRunner;

#[derive(Debug, Clone)]
pub struct RunnerOptions {
    pub env_path: Option<PathBuf>,
    pub reporter: String,
    pub concurrency: usize,
    pub ramp_up: Option<Duration>,
    pub duration: Option<Duration>,
    pub iterations: usize,
    pub data_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ProfileStats {
    pub total_requests: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub min_ms: u64,
    pub max_ms: u64,
    pub mean_ms: f64,
    pub p50_ms: u64,
    pub p90_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub rps: f64,
    pub error_rate: f64,
    pub elapsed: Duration,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct RunSummary {
    pub exit_code: i32,
    pub reports: Vec<ExecutionReport>,
    pub stats: Option<ProfileStats>,
}


/// Parses human duration strings like "10s", "500ms", "2m", "1h"
pub fn parse_duration_str(s: &str) -> Result<Duration, String> {
    let s = s.trim().to_lowercase();
    if s.ends_with("ms") {
        let num: u64 = s[..s.len() - 2]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid ms duration '{s}': {e}"))?;
        Ok(Duration::from_millis(num))
    } else if s.ends_with('s') {
        let num: u64 = s[..s.len() - 1]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid seconds duration '{s}': {e}"))?;
        Ok(Duration::from_secs(num))
    } else if s.ends_with('m') {
        let num: u64 = s[..s.len() - 1]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid minutes duration '{s}': {e}"))?;
        Ok(Duration::from_secs(num * 60))
    } else if s.ends_with('h') {
        let num: u64 = s[..s.len() - 1]
            .trim()
            .parse()
            .map_err(|e| format!("Invalid hours duration '{s}': {e}"))?;
        Ok(Duration::from_secs(num * 3600))
    } else {
        // Default to seconds if plain integer
        let num: u64 = s
            .parse()
            .map_err(|_| format!("Invalid duration format '{s}'. Use '500ms', '10s', '1m', etc."))?;
        Ok(Duration::from_secs(num))
    }
}

/// Loads CSV or JSON iteration records
pub fn load_iteration_data(path: &Path) -> Result<Vec<HashMap<String, String>>, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext == "csv" {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(path)
            .map_err(|e| format!("Failed to open CSV file: {e}"))?;

        let headers = rdr
            .headers()
            .map_err(|e| format!("Failed to read CSV headers: {e}"))?
            .clone();

        let mut rows = Vec::new();
        for result in rdr.records() {
            let record = result.map_err(|e| format!("Failed to parse CSV record: {e}"))?;
            let mut map = HashMap::new();
            for (h, val) in headers.iter().zip(record.iter()) {
                map.insert(h.to_string(), val.to_string());
            }
            rows.push(map);
        }
        Ok(rows)
    } else if ext == "json" {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read JSON file: {e}"))?;
        let v: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse JSON file: {e}"))?;

        let arr = v
            .as_array()
            .ok_or_else(|| "JSON data file must contain an array of objects".to_string())?;

        let mut rows = Vec::new();
        for item in arr {
            if let Some(obj) = item.as_object() {
                let mut map = HashMap::new();
                for (k, val) in obj {
                    let s = match val {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    map.insert(k.clone(), s);
                }
                rows.push(map);
            }
        }
        Ok(rows)
    } else {
        Err(format!("Unsupported data file format '{ext}'. Expected .csv or .json"))
    }
}

pub async fn run_suite(target_path: &Path, options: RunnerOptions) -> RunSummary {
    let mut files_to_run = Vec::new();
    if target_path.is_dir() {
        collect_yarac_files(target_path, &mut files_to_run);
        files_to_run.sort();
    } else if target_path.exists() {
        files_to_run.push(target_path.to_path_buf());
    } else {
        eprintln!("Error: Target path does not exist: {}", target_path.display());
        return RunSummary {
            exit_code: 2,
            reports: Vec::new(),
            stats: None,
        };
    }

    if files_to_run.is_empty() {
        eprintln!("No .yarac files found to execute at: {}", target_path.display());
        return RunSummary {
            exit_code: 2,
            reports: Vec::new(),
            stats: None,
        };
    }

    // Load Environment
    let mut base_env = HashMap::new();
    if let Some(ep) = &options.env_path {
        if ep.exists() {
            if let Ok(content) = std::fs::read_to_string(ep) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.starts_with('#') {
                        continue;
                    }
                    if let Some((k, v)) = trimmed.split_once('=') {
                        let clean_k = k.trim().to_string();
                        let clean_v = v.trim().trim_matches('"').trim_matches('\'').to_string();
                        base_env.insert(clean_k, VariableValue::new(clean_v));
                    }
                }
            }
        }
    }

    // Load Data Iteration Records (CSV/JSON)
    let iteration_records = if let Some(dp) = &options.data_path {
        match load_iteration_data(dp) {
            Ok(rows) => {
                if rows.is_empty() {
                    eprintln!("Warning: Data file '{}' has no rows", dp.display());
                    vec![HashMap::new()]
                } else {
                    rows
                }
            }
            Err(e) => {
                eprintln!("Error loading data file: {e}");
                return RunSummary {
                    exit_code: 2,
                    reports: Vec::new(),
                    stats: None,
                };
            }
        }
    } else {
        vec![HashMap::new()]
    };

    let total_iterations = if options.data_path.is_some() {
        iteration_records.len()
    } else {
        options.iterations.max(1)
    };

    let workspace_root = if target_path.is_dir() {
        target_path.to_path_buf()
    } else {
        target_path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };

    let concurrency = options.concurrency.max(1);
    let is_load_mode = concurrency > 1 || options.duration.is_some() || total_iterations > 1;

    let (tx, mut rx) = mpsc::channel(1024);
    let stop_signal = Arc::new(AtomicBool::new(false));
    let start_time = Instant::now();

    // Spawn concurrent worker tasks
    let mut handles = Vec::new();
    for worker_idx in 0..concurrency {
        let files = files_to_run.clone();
        let base_env_clone = base_env.clone();
        let iteration_records_clone = iteration_records.clone();
        let ws_root = workspace_root.clone();
        let tx_clone = tx.clone();
        let stop_clone = stop_signal.clone();
        let ramp_up = options.ramp_up;
        let duration = options.duration;

        let handle = tokio::spawn(async move {
            // Apply worker ramp-up delay
            if let Some(r) = ramp_up {
                if concurrency > 1 && worker_idx > 0 {
                    let delay_per_worker = r / (concurrency as u32);
                    let initial_delay = delay_per_worker * (worker_idx as u32);
                    tokio::time::sleep(initial_delay).await;
                }
            }

            let script_runner = ScriptRunner::default();
            let scrub_config = ScrubConfig::default();
            let mut iter = 0;

            loop {
                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                if let Some(d) = duration {
                    if start_time.elapsed() >= d {
                        stop_clone.store(true, Ordering::Relaxed);
                        break;
                    }
                } else if iter >= total_iterations {
                    break;
                }

                let row_data = &iteration_records_clone[iter % iteration_records_clone.len()];

                // Prepare scope for this iteration
                let mut scope = VariableScopeStack::new();
                for (k, v) in &base_env_clone {
                    scope.environment.insert(k.clone(), v.clone());
                }
                for (k, v) in row_data {
                    scope.environment.insert(k.clone(), VariableValue::new(v.clone()));
                }

                for file in &files {
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }

                    let content = match std::fs::read_to_string(file) {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = tx_clone
                                .send(WorkerEvent::Error(format!(
                                    "Error reading file '{}': {e}",
                                    file.display()
                                )))
                                .await;
                            continue;
                        }
                    };

                    let cst = match parse(&content) {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = tx_clone
                                .send(WorkerEvent::Error(format!(
                                    "Syntax error in '{}': {e}",
                                    file.display()
                                )))
                                .await;
                            continue;
                        }
                    };

                    let mut req = match cst.to_request() {
                        Ok(r) => r,
                        Err(e) => {
                            let _ = tx_clone
                                .send(WorkerEvent::Error(format!(
                                    "Config error in '{}': {e}",
                                    file.display()
                                )))
                                .await;
                            continue;
                        }
                    };

                    // Flow Control: once_per_run check
                    if let Some(flow) = &req.flow {
                        if flow.once_per_run && iter > 0 {
                            continue; // Skip execution on subsequent runs
                        }

                        // Flow Control: request pacing delay & jitter
                        if let Some(pacing) = flow.calculate_pacing_delay() {
                            tokio::time::sleep(pacing).await;
                        }
                    }

                    // Pre-request script
                    let mut secrets = HashMap::new();
                    for (k, var) in &scope.environment {
                        if var.is_secret {
                            secrets.insert(k.clone(), var.value.clone());
                        }
                    }

                    if let Some(pre_script) = req.scripts.pre_request.clone() {
                        let _ = script_runner
                            .run_pre_request(&pre_script, &mut req, &secrets)
                            .await;
                    }

                    let interpolated_url = scope.interpolate(&req.url);

                    let client = match YaracHttpClient::new(
                        req.settings.verify_ssl,
                        req.settings.follow_redirects,
                        req.settings.timeout_ms,
                        None,
                    ) {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = tx_clone
                                .send(WorkerEvent::Error(format!("Client config error: {e}")))
                                .await;
                            continue;
                        }
                    };

                    // Execution with retry_until polling support
                    let max_retries = req.flow.as_ref().map(|f| f.max_retries).unwrap_or(0);
                    let retry_interval = req
                        .flow
                        .as_ref()
                        .map(|f| Duration::from_millis(f.retry_interval_ms))
                        .unwrap_or(Duration::from_millis(1000));
                    let retry_until = req.flow.as_ref().and_then(|f| f.retry_until.clone());

                    let mut retries = 0;
                    loop {
                        match client.execute(&req, &ws_root, &interpolated_url).await {
                            Ok(mut report) => {
                                // Post-response script
                                if let Some(post_script) = &req.scripts.post_response {
                                    if let Ok(mutated) = script_runner
                                        .run_post_response(post_script, &mut report, &secrets)
                                        .await
                                    {
                                        for (k, v) in mutated {
                                            scope.environment.insert(k, VariableValue::new(v));
                                        }
                                    }
                                }

                                // Scrub body
                                let mut registry = SecretRegistry::new();
                                for sec in secrets.values() {
                                    registry.register(sec);
                                }
                                report.response_body =
                                    registry.scrub(&report.response_body, &scrub_config);

                                // Check retry_until condition if configured
                                if let Some(ref pred) = retry_until {
                                    if retries < max_retries {
                                        let matched = script_runner
                                            .evaluate_predicate(
                                                pred,
                                                report.status_code,
                                                &report.response_body,
                                                &report.response_headers,
                                            )
                                            .await
                                            .unwrap_or(false);

                                        if !matched {
                                            retries += 1;
                                            tokio::time::sleep(retry_interval).await;
                                            continue; // re-execute
                                        }
                                    }
                                }

                                let _ = tx_clone.send(WorkerEvent::Report(report)).await;
                                break;
                            }
                            Err(e) => {
                                if retries < max_retries && retry_until.is_some() {
                                    retries += 1;
                                    tokio::time::sleep(retry_interval).await;
                                    continue;
                                }
                                let _ = tx_clone
                                    .send(WorkerEvent::NetError(format!(
                                        "Network error executing '{}': {e}",
                                        file.display()
                                    )))
                                    .await;
                                break;
                            }
                        }
                    }
                }

                iter += 1;
            }
        });
        handles.push(handle);
    }
    drop(tx); // Close root sender so channel drains when workers finish

    // Aggregate results into HDR Histogram & reports
    let mut histogram = Histogram::<u64>::new_with_bounds(1, 3_600_000, 3).unwrap();
    let mut reports = Vec::new();
    let mut highest_exit_code = 0;
    let mut success_count = 0;
    let mut failure_count = 0;

    while let Some(event) = rx.recv().await {
        match event {
            WorkerEvent::Report(rep) => {
                let code = rep.exit_code();
                if code > highest_exit_code {
                    highest_exit_code = code;
                }
                if code == 0 {
                    success_count += 1;
                } else {
                    failure_count += 1;
                }
                let duration_clamped = rep.duration_ms.max(1);
                let _ = histogram.record(duration_clamped);
                reports.push(rep);
            }
            WorkerEvent::NetError(e) => {
                eprintln!("{e}");
                failure_count += 1;
                if highest_exit_code < 3 {
                    highest_exit_code = 3;
                }
            }
            WorkerEvent::Error(e) => {
                eprintln!("{e}");
                if highest_exit_code < 2 {
                    highest_exit_code = 2;
                }
            }
        }
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = start_time.elapsed();
    let total_requests = success_count + failure_count;

    let stats = if total_requests > 0 {
        let rps = total_requests as f64 / elapsed.as_secs_f64().max(0.001);
        let error_rate = (failure_count as f64 / total_requests as f64) * 100.0;
        Some(ProfileStats {
            total_requests,
            success_count,
            failure_count,
            min_ms: histogram.min(),
            max_ms: histogram.max(),
            mean_ms: histogram.mean(),
            p50_ms: histogram.value_at_quantile(0.50),
            p90_ms: histogram.value_at_quantile(0.90),
            p95_ms: histogram.value_at_quantile(0.95),
            p99_ms: histogram.value_at_quantile(0.99),
            rps,
            error_rate,
            elapsed,
        })
    } else {
        None
    };

    // Render output
    render_reports(&reports, &options.reporter, stats.as_ref(), is_load_mode);

    RunSummary {
        exit_code: highest_exit_code,
        reports,
        stats,
    }
}

enum WorkerEvent {
    Report(ExecutionReport),
    NetError(String),
    Error(String),
}

fn collect_yarac_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_yarac_files(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("yarac") {
                out.push(path);
            }
        }
    }
}

pub fn render_reports(
    reports: &[ExecutionReport],
    reporter: &str,
    stats: Option<&ProfileStats>,
    is_load_mode: bool,
) {
    match reporter.to_lowercase().as_str() {
        "junit" => {
            println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
            println!("<testsuites tests=\"{}\" name=\"Yarac Suite\">", reports.len());
            for rep in reports {
                let failures = rep.assertions.iter().filter(|a| !a.passed).count();
                println!(
                    "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" time=\"{:.3}\">",
                    xml_escape(&rep.request_name),
                    rep.assertions.len(),
                    failures,
                    rep.duration_ms as f64 / 1000.0
                );
                for a in &rep.assertions {
                    if a.passed {
                        println!("    <testcase name=\"{}\"/>", xml_escape(&a.name));
                    } else {
                        println!(
                            "    <testcase name=\"{}\"><failure message=\"{}\"/></testcase>",
                            xml_escape(&a.name),
                            xml_escape(a.error_message.as_deref().unwrap_or("Assertion failed"))
                        );
                    }
                }
                println!("  </testsuite>");
            }
            println!("</testsuites>");
        }
        "tap" => {
            let total_assertions: usize = reports.iter().map(|r| r.assertions.len()).sum();
            println!("TAP version 13");
            println!("1..{}", total_assertions.max(1));
            let mut count = 1;
            for rep in reports {
                for a in &rep.assertions {
                    if a.passed {
                        println!("ok {} - {} -> {}", count, rep.request_name, a.name);
                    } else {
                        println!(
                            "not ok {} - {} -> {} # {}",
                            count,
                            rep.request_name,
                            a.name,
                            a.error_message.as_deref().unwrap_or("Failed")
                        );
                    }
                    count += 1;
                }
            }
        }
        _ => {
            if !is_load_mode {
                println!("\n========================================================");
                println!("                YARAC EXECUTION SUMMARY                 ");
                println!("========================================================");
                for rep in reports {
                    let passed = rep.assertions.iter().filter(|a| a.passed).count();
                    let status_indicator = if rep.all_passed() { "PASS" } else { "FAIL" };
                    println!(
                        "[{}] {} {} ({}ms, Status: {}, Assertions: {}/{})",
                        status_indicator,
                        rep.method,
                        rep.request_name,
                        rep.duration_ms,
                        rep.status_code,
                        passed,
                        rep.assertions.len()
                    );
                    for a in &rep.assertions {
                        if a.passed {
                            println!("  ✓ {}", a.name);
                        } else {
                            println!(
                                "  ✗ {} - {}",
                                a.name,
                                a.error_message.as_deref().unwrap_or("Assertion failed")
                            );
                        }
                    }
                }
                println!("========================================================\n");
            }

            // Display Statistical Profiling & Latency Histogram Table and Bar Chart
            if let Some(s) = stats {
                render_profile_chart(s);
            }
        }
    }
}

pub fn render_profile_chart(s: &ProfileStats) {
    println!("\n================================================================================");
    println!("              YARAC STATISTICAL PROFILING & HDR HISTOGRAM REPORT                ");
    println!("================================================================================");
    println!(
        "Total Requests: {:<8} | Success: {:<8} | Failures: {:<8}",
        s.total_requests, s.success_count, s.failure_count
    );
    println!(
        "Throughput:     {:.2} req/s | Error Rate:   {:.2}% | Total Elapsed: {:.2}s",
        s.rps,
        s.error_rate,
        s.elapsed.as_secs_f64()
    );
    println!("--------------------------------------------------------------------------------");
    println!(" PERCENTILE METRICS (ms)");
    println!("   Min    : {:>6} ms", s.min_ms);
    println!("   p50    : {:>6} ms", s.p50_ms);
    println!("   p90    : {:>6} ms", s.p90_ms);
    println!("   p95    : {:>6} ms", s.p95_ms);
    println!("   p99    : {:>6} ms", s.p99_ms);
    println!("   Max    : {:>6} ms", s.max_ms);
    println!("   Mean   : {:>6.2} ms", s.mean_ms);
    println!("--------------------------------------------------------------------------------");
    println!(" LATENCY DISTRIBUTION CHART");

    let percentiles = [
        ("Min", s.min_ms),
        ("p50", s.p50_ms),
        ("p90", s.p90_ms),
        ("p95", s.p95_ms),
        ("p99", s.p99_ms),
        ("Max", s.max_ms),
    ];

    let max_val = s.max_ms.max(1) as f64;
    for (label, val) in percentiles {
        let bar_len = ((val as f64 / max_val) * 40.0).round() as usize;
        let bar = "█".repeat(bar_len.max(1));
        println!("   {:<4} {:>6} ms | {}", label, val, bar);
    }
    println!("================================================================================\n");
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration_str("500ms").unwrap(), Duration::from_millis(500));
        assert_eq!(parse_duration_str("10s").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_duration_str("2m").unwrap(), Duration::from_secs(120));
        assert_eq!(parse_duration_str("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration_str("15").unwrap(), Duration::from_secs(15));
    }

    #[test]
    fn test_profile_stats_calculation() {
        let mut hist = Histogram::<u64>::new_with_bounds(1, 10000, 3).unwrap();
        hist.record(10).unwrap();
        hist.record(20).unwrap();
        hist.record(30).unwrap();
        hist.record(100).unwrap();

        let stats = ProfileStats {
            total_requests: 4,
            success_count: 4,
            failure_count: 0,
            min_ms: hist.min(),
            max_ms: hist.max(),
            mean_ms: hist.mean(),
            p50_ms: hist.value_at_quantile(0.50),
            p90_ms: hist.value_at_quantile(0.90),
            p95_ms: hist.value_at_quantile(0.95),
            p99_ms: hist.value_at_quantile(0.99),
            rps: 40.0,
            error_rate: 0.0,
            elapsed: Duration::from_millis(100),
        };

        assert_eq!(stats.min_ms, 10);
        assert_eq!(stats.max_ms, 100);
        assert_eq!(stats.total_requests, 4);
    }
}
