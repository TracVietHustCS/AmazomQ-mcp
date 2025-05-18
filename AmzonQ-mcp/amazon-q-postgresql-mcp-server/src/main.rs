mod operator;

use anyhow::{Context, Result};
use std::env;
use std::path::Path;
use std::fs;
use rmcp::ServiceExt;
use rmcp::transport::io::stdio;
use tracing_subscriber::{self, fmt, prelude::*};
use tracing_subscriber::fmt::format::FmtSpan;
use std::io::IsTerminal;

use operator::PostgresOperator;

#[tokio::main]
async fn main() -> Result<()> {
    // Get connection string from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Please provide a PostgreSQL connection string as a command-line argument");
        std::process::exit(1);
    }
    let connection_string = args[1].clone();

    // Setup logging
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        fs::create_dir_all(log_dir).context("Failed to create logs directory")?;
    }
    
    let log_path = log_dir.join("postgresql_server.log");
    let log_file = fs::File::create(&log_path).context("Failed to create log file")?;
    
    // Set up logging to both stderr and file
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(std::io::stderr().is_terminal())
        .with_span_events(FmtSpan::CLOSE);
    
    let file_layer = fmt::layer()
        .with_writer(log_file)
        .with_ansi(false)
        .with_span_events(FmtSpan::CLOSE);
    
    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();

    tracing::info!(
        "Starting PostgreSQL server... Logs will be saved to {}",
        log_path.display()
    );
    tracing::info!("Using connection string: {}", connection_string);

    // Create an instance of PostgreSQL operator
    let service = PostgresOperator::new(connection_string)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Error: {}", e);
        })?;

    service.waiting().await?;

    Ok(())
}