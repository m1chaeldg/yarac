use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    pub iteration: usize,
    pub request_name: String,
    pub status_code: u16,
    pub duration_ms: u64,
    pub passed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerStats {
    pub count: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub min_ms: u64,
    pub max_ms: u64,
    pub mean_ms: f64,
    pub p50_ms: u64,
    pub p90_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerState {
    pub request_ids: Vec<String>,
    pub iterations: usize,
    pub concurrency: usize,
    pub delay_ms: u64,
    pub results: Vec<IterationResult>,
    pub is_running: bool,
}

impl RunnerState {
    pub fn new(request_ids: Vec<String>, iterations: usize, concurrency: usize, delay_ms: u64) -> Self {
        Self {
            request_ids,
            iterations,
            concurrency,
            delay_ms,
            results: Vec::new(),
            is_running: false,
        }
    }

    pub fn record_result(&mut self, res: IterationResult) {
        self.results.push(res);
    }

    pub fn compute_stats(&self) -> RunnerStats {
        let count = self.results.len();
        if count == 0 {
            return RunnerStats {
                count: 0,
                success_count: 0,
                failure_count: 0,
                min_ms: 0,
                max_ms: 0,
                mean_ms: 0.0,
                p50_ms: 0,
                p90_ms: 0,
                p95_ms: 0,
                p99_ms: 0,
                error_rate: 0.0,
            };
        }

        let mut latencies: Vec<u64> = self.results.iter().map(|r| r.duration_ms).collect();
        latencies.sort_unstable();

        let success_count = self.results.iter().filter(|r| r.passed).count();
        let failure_count = count - success_count;
        let sum: u64 = latencies.iter().sum();
        let mean_ms = sum as f64 / count as f64;

        let p50_ms = latencies[((count as f64 * 0.50).floor() as usize).min(count - 1)];
        let p90_ms = latencies[((count as f64 * 0.90).floor() as usize).min(count - 1)];
        let p95_ms = latencies[((count as f64 * 0.95).floor() as usize).min(count - 1)];
        let p99_ms = latencies[((count as f64 * 0.99).floor() as usize).min(count - 1)];

        RunnerStats {
            count,
            success_count,
            failure_count,
            min_ms: latencies[0],
            max_ms: latencies[count - 1],
            mean_ms,
            p50_ms,
            p90_ms,
            p95_ms,
            p99_ms,
            error_rate: (failure_count as f64 / count as f64) * 100.0,
        }
    }
}
