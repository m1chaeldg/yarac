use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use yarac_core::models::*;
use yarac_core::scope::VariableScopeStack;
use yarac_core::scrubber::{ScrubConfig, SecretRegistry};
use yarac_net::YaracHttpClient;
use yarac_parser::parse;
use yarac_scripting::ScriptRunner;

#[derive(Parser)]
#[command(
    name = "yarac-cli",
    author = "Yarac Team",
    version = "0.1.0",
    about = "Fast, low-memory, local-first API testing client"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Executes a .yarac request or collection headlessly
    Run {
        /// Path to .yarac file or collection directory
        path: PathBuf,
        /// Path to environment file (.env.yarac)
        #[arg(short, long)]
        env: Option<PathBuf>,
        /// Output format (cli, junit, tap)
        #[arg(short, long, default_value = "cli")]
        reporter: String,
    },
    /// Runs the Model Context Protocol (MCP) server
    Mcp {
        /// Transport type (stdio or sse)
        #[arg(short, long, default_value = "stdio")]
        transport: String,
        /// Port for HTTP/SSE transport (binds strictly to 127.0.0.1)
        #[arg(short, long, default_value_t = 9472)]
        port: u16,
        /// Bearer auth token for HTTP/SSE transport
        #[arg(long, default_value = "yarac-local-secret-token")]
        token: String,
    },
    /// Imports external collections (Postman v2.1) into .yarac format
    Import {
        /// Path to Postman collection JSON
        file: PathBuf,
        /// Output directory for generated .yarac files
        #[arg(short, long, default_value = "./collections")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            path,
            env,
            reporter,
        } => {
            let code = run_suite(&path, env.as_deref(), &reporter).await;
            ExitCode::from(code as u8)
        }
        Commands::Mcp {
            transport,
            port,
            token,
        } => {
            let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            if transport == "stdio" {
                if let Err(e) = yarac_mcp::YaracMcpServer::run_stdio(&workspace_root).await {
                    eprintln!("MCP stdio error: {e}");
                    return ExitCode::from(2);
                }
                ExitCode::from(0)
            } else {
                if let Err(e) = yarac_mcp::YaracMcpServer::run_http_sse(port, &token, workspace_root).await {
                    eprintln!("MCP HTTP/SSE error: {e}");
                    return ExitCode::from(2);
                }
                ExitCode::from(0)
            }
        }
        Commands::Import { file, output } => {
            let json_str = match std::fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file '{}': {e}", file.display());
                    return ExitCode::from(2);
                }
            };

            let reqs = match yarac_importer::PostmanImporter::import_collection(&json_str) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to parse Postman collection: {e}");
                    return ExitCode::from(2);
                }
            };

            match yarac_importer::PostmanImporter::export_to_workspace(&reqs, &output) {
                Ok(files) => {
                    println!("Successfully imported {} requests into {}", files.len(), output.display());
                    ExitCode::from(0)
                }
                Err(e) => {
                    eprintln!("Failed to write .yarac files: {e}");
                    ExitCode::from(2)
                }
            }
        }
    }
}

async fn run_suite(target_path: &Path, env_path: Option<&Path>, reporter: &str) -> i32 {
    let mut files_to_run = Vec::new();
    if target_path.is_dir() {
        collect_yarac_files(target_path, &mut files_to_run);
        files_to_run.sort();
    } else if target_path.exists() {
        files_to_run.push(target_path.to_path_buf());
    } else {
        eprintln!("Error: Target path does not exist: {}", target_path.display());
        return 2;
    }

    if files_to_run.is_empty() {
        eprintln!("No .yarac files found to execute at: {}", target_path.display());
        return 2;
    }

    // Load Environment
    let mut scope = VariableScopeStack::new();
    if let Some(ep) = env_path {
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
                        scope.environment.insert(clean_k, VariableValue::new(clean_v));
                    }
                }
            }
        }
    }

    let mut reports = Vec::new();
    let mut highest_exit_code = 0;

    let workspace_root = if target_path.is_dir() {
        target_path
    } else {
        target_path.parent().unwrap_or(Path::new("."))
    };

    let script_runner = ScriptRunner::default();
    let scrub_config = ScrubConfig::default();

    for file in &files_to_run {
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading file '{}': {e}", file.display());
                return 2;
            }
        };

        let cst = match parse(&content) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Syntax error in '{}': {e}", file.display());
                return 2;
            }
        };

        let mut req = match cst.to_request() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Configuration error in '{}': {e}", file.display());
                return 2;
            }
        };

        let mut secrets = HashMap::new();
        for (k, var) in &scope.environment {
            if var.is_secret {
                secrets.insert(k.clone(), var.value.clone());
            }
        }

        // Run pre-request script
        if let Some(pre_script) = req.scripts.pre_request.clone() {
            if let Err(e) = script_runner.run_pre_request(&pre_script, &mut req, &secrets).await {
                eprintln!("Pre-request script error: {e}");
            }
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
                eprintln!("Client config error: {e}");
                return 2;
            }
        };

        match client.execute(&req, workspace_root, &interpolated_url).await {
            Ok(mut report) => {
                // Run post-response assertions script
                if let Some(post_script) = &req.scripts.post_response {
                    if let Ok(mutated_env) = script_runner
                        .run_post_response(post_script, &mut report, &secrets)
                        .await
                    {
                        for (k, v) in mutated_env {
                            scope.environment.insert(k, VariableValue::new(v));
                        }
                    }
                }

                // Scrub response body
                let mut registry = SecretRegistry::new();
                for sec in secrets.values() {
                    registry.register(sec);
                }
                report.response_body = registry.scrub(&report.response_body, &scrub_config);

                let code = report.exit_code();
                if code > highest_exit_code {
                    highest_exit_code = code;
                }
                reports.push(report);
            }
            Err(e) => {
                eprintln!("Network error executing '{}': {e}", file.display());
                highest_exit_code = 3;
            }
        }
    }

    render_reports(&reports, reporter);

    highest_exit_code
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

fn render_reports(reports: &[ExecutionReport], reporter: &str) {
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
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
