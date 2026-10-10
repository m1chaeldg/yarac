mod runner;
mod server;

use clap::{Parser, Subcommand};
use runner::{parse_duration_str, run_suite, RunnerOptions};
use server::YaracLocalServer;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use yarac_core::models::YaracRequest;
use yarac_core::watcher::WorkspaceWatcher;
use yarac_importer::{
    export_requests_to_workspace, BrunoImporter, OpenApiExporter, OpenApiImporter, PostmanImporter,
};
use yarac_parser::parse;

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
        /// Number of concurrent workers (micro-load testing)
        #[arg(short, long, default_value_t = 1)]
        concurrency: usize,
        /// Worker ramp-up time (e.g. "10s", "500ms")
        #[arg(long)]
        ramp_up: Option<String>,
        /// Total duration for load run (e.g. "30s", "1m")
        #[arg(long)]
        duration: Option<String>,

        /// Number of iterations through the collection
        #[arg(short, long, default_value_t = 1)]
        iterations: usize,
        /// Path to iteration data file (.csv or .json)
        #[arg(short, long)]
        data: Option<PathBuf>,
    },
    /// Exports .yarac requests to OpenAPI v3.1 specification
    Export {
        /// Format to export (openapi)
        #[arg(short, long, default_value = "openapi")]
        format: String,
        /// Path to .yarac file or collection directory
        path: PathBuf,
        /// Output file for OpenAPI specification
        #[arg(short, long, default_value = "openapi.json")]
        output: PathBuf,
        /// API Title for specification
        #[arg(long, default_value = "Yarac Exported API")]
        title: String,
        /// API Version
        #[arg(long, default_value = "1.0.0")]
        version: String,
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
    /// Imports external collections (Postman v2.1, Bruno .bru, OpenAPI v3/v3.1) into .yarac format
    Import {
        /// Path to collection or spec file
        file: PathBuf,
        /// Format to import: postman, bruno, openapi
        #[arg(short, long, default_value = "postman")]
        format: String,
        /// Output directory for generated .yarac files
        #[arg(short, long, default_value = "./collections")]
        output: PathBuf,
    },
    /// Starts the local full-stack Yarac daemon and file watcher on 127.0.0.1
    Serve {
        /// Port to bind (default: 9475)
        #[arg(short, long, default_value_t = 9475)]
        port: u16,
        /// Path to workspace directory
        #[arg(short, long, default_value = "./workspace")]
        workspace: PathBuf,
    },
    /// Watches a workspace directory for external changes
    Watch {
        /// Path to workspace directory
        #[arg(default_value = "./workspace")]
        path: PathBuf,
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
            concurrency,
            ramp_up,
            duration,
            iterations,
            data,
        } => {
            let parsed_ramp_up = match ramp_up {
                Some(ref s) => match parse_duration_str(s) {
                    Ok(d) => Some(d),
                    Err(e) => {
                        eprintln!("Error in --ramp-up: {e}");
                        return ExitCode::from(2);
                    }
                },
                None => None,
            };

            let parsed_duration = match duration {
                Some(ref s) => match parse_duration_str(s) {
                    Ok(d) => Some(d),
                    Err(e) => {
                        eprintln!("Error in --duration: {e}");
                        return ExitCode::from(2);
                    }
                },
                None => None,
            };

            let options = RunnerOptions {
                env_path: env,
                reporter,
                concurrency,
                ramp_up: parsed_ramp_up,
                duration: parsed_duration,
                iterations,
                data_path: data,
            };

            let summary = run_suite(&path, options).await;
            ExitCode::from(summary.exit_code as u8)
        }
        Commands::Export {
            format,
            path,
            output,
            title,
            version,
        } => {
            if format.to_lowercase() != "openapi" {
                eprintln!("Unsupported export format '{format}'. Only 'openapi' is supported.");
                return ExitCode::from(2);
            }

            let mut files = Vec::new();
            collect_yarac_files(&path, &mut files);
            if files.is_empty() {
                eprintln!("No .yarac files found at: {}", path.display());
                return ExitCode::from(2);
            }
            files.sort();

            let mut requests = Vec::new();
            for f in &files {
                let content = match std::fs::read_to_string(f) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read '{}': {e}", f.display());
                        return ExitCode::from(2);
                    }
                };
                let cst = match parse(&content) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Syntax error in '{}': {e}", f.display());
                        return ExitCode::from(2);
                    }
                };
                let req = match cst.to_request() {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Config error in '{}': {e}", f.display());
                        return ExitCode::from(2);
                    }
                };
                requests.push(req);
            }

            match OpenApiExporter::export_to_file(&requests, &title, &version, &output) {
                Ok(_) => {
                    println!(
                        "Successfully exported {} requests to OpenAPI v3.1 spec at: {}",
                        requests.len(),
                        output.display()
                    );
                    ExitCode::from(0)
                }
                Err(e) => {
                    eprintln!("Failed to export OpenAPI spec: {e}");
                    ExitCode::from(2)
                }
            }
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
        Commands::Import { file, format, output } => {
            let requests: Vec<YaracRequest> = match format.to_lowercase().as_str() {
                "postman" => {
                    let json_str = match std::fs::read_to_string(&file) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error reading Postman file '{}': {e}", file.display());
                            return ExitCode::from(2);
                        }
                    };
                    match PostmanImporter::import_collection(&json_str) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed to parse Postman collection: {e}");
                            return ExitCode::from(2);
                        }
                    }
                }
                "bruno" => {
                    if file.is_dir() {
                        match BrunoImporter::import_directory(&file) {
                            Ok(r) => r,
                            Err(e) => {
                                eprintln!("Failed to import Bruno directory: {e}");
                                return ExitCode::from(2);
                            }
                        }
                    } else {
                        let content = match std::fs::read_to_string(&file) {
                            Ok(s) => s,
                            Err(e) => {
                                eprintln!("Error reading Bruno file '{}': {e}", file.display());
                                return ExitCode::from(2);
                            }
                        };
                        match BrunoImporter::import_bru(&content) {
                            Ok(r) => vec![r],
                            Err(e) => {
                                eprintln!("Failed to parse Bruno file: {e}");
                                return ExitCode::from(2);
                            }
                        }
                    }
                }
                "openapi" => {
                    let spec_str = match std::fs::read_to_string(&file) {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Error reading OpenAPI file '{}': {e}", file.display());
                            return ExitCode::from(2);
                        }
                    };
                    match OpenApiImporter::import_spec(&spec_str) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!("Failed to parse OpenAPI spec: {e}");
                            return ExitCode::from(2);
                        }
                    }
                }
                other => {
                    eprintln!(
                        "Unsupported import format '{other}'. Choose from: 'postman', 'bruno', 'openapi'."
                    );
                    return ExitCode::from(2);
                }
            };

            match export_requests_to_workspace(&requests, &output) {
                Ok(files) => {
                    println!(
                        "Successfully imported {} requests into {}",
                        files.len(),
                        output.display()
                    );
                    ExitCode::from(0)
                }
                Err(e) => {
                    eprintln!("Failed to write .yarac files: {e}");
                    ExitCode::from(2)
                }
            }
        }
        Commands::Serve { port, workspace } => {
            let server = Arc::new(YaracLocalServer::new(workspace));
            if let Err(e) = server.run(port).await {
                eprintln!("Server error: {e}");
                ExitCode::from(1)
            } else {
                ExitCode::from(0)
            }
        }
        Commands::Watch { path } => {
            println!("👀 Watching workspace at {} for changes...", path.display());
            match WorkspaceWatcher::new(&path) {
                Ok(watcher) => loop {
                    if let Some(event) = watcher.recv_timeout(std::time::Duration::from_millis(500)) {
                        println!("⚡ File changed: [{:?}] {}", event.kind, event.path.display());
                    }
                },
                Err(e) => {
                    eprintln!("Failed to start watcher: {e}");
                    ExitCode::from(2)
                }
            }
        }
    }
}

fn collect_yarac_files(target: &Path, out: &mut Vec<PathBuf>) {
    if target.is_dir() {
        if let Ok(entries) = std::fs::read_dir(target) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_yarac_files(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("yarac") {
                    out.push(path);
                }
            }
        }
    } else if target.exists() {
        out.push(target.to_path_buf());
    }
}
