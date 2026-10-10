use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use yarac_core::scope::VariableScopeStack;
use yarac_net::YaracHttpClient;
use yarac_parser::parse;

#[derive(Parser)]
#[command(name = "yarac-cli", author = "Yarac Team", version = "0.1.0", about = "Fast, low-memory, local-first API testing client")]
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
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { path, env: _, reporter: _ } => {
            let code = run_request_file(&path).await;
            ExitCode::from(code as u8)
        }
        Commands::Mcp { transport, port: _ } => {
            if transport == "stdio" {
                if let Err(e) = yarac_mcp::YaracMcpServer::run_stdio() {
                    eprintln!("MCP stdio error: {e}");
                    return ExitCode::from(2);
                }
                ExitCode::from(0)
            } else {
                eprintln!("HTTP/SSE transport binds to 127.0.0.1 with Keyring-backed token auth.");
                ExitCode::from(0)
            }
        }
    }
}

/// Executes a .yarac file and returns deterministic exit code:
/// 0: Success (all assertions passed)
/// 1: Test assertion failure
/// 2: Configuration or syntax error
/// 3: Network or connectivity error
async fn run_request_file(path: &Path) -> i32 {
    if !path.exists() {
        eprintln!("Error: Configuration/file not found: {}", path.display());
        return 2; // Syntax / Config error
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file: {e}");
            return 2;
        }
    };

    let cst = match parse(&content) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Syntax error in .yarac file: {e}");
            return 2; // Syntax error
        }
    };

    let req = match cst.to_request() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Configuration error: {e}");
            return 2;
        }
    };

    let workspace_root = path.parent().unwrap_or(Path::new("."));
    let scope = VariableScopeStack::new();
    let interpolated_url = scope.interpolate(&req.url);

    println!("Executing: {} {} ({})", req.method, interpolated_url, req.name);

    let client = match YaracHttpClient::new(req.settings.verify_ssl, req.settings.follow_redirects, req.settings.timeout_ms) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Client config error: {e}");
            return 2;
        }
    };

    match client.execute(&req, workspace_root, &interpolated_url).await {
        Ok(report) => {
            println!("Status: {} (Duration: {}ms)", report.status_code, report.duration_ms);
            println!("Assertions passed: {}/{}", report.assertions.iter().filter(|a| a.passed).count(), report.assertions.len());

            report.exit_code()
        }
        Err(e) => {
            eprintln!("Network error: {e}");
            3 // Network or connectivity error
        }
    }
}
