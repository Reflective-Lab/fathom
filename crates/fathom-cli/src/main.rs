use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "fathom", about = "Convergence-driven analysis of financial filings")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Materialise a HuggingFace dataset slice into the local Iceberg lakehouse.
    Ingest,
    /// Run a formation against the lakehouse for a given company.
    Analyse {
        /// SEC CIK (Central Index Key), e.g. 0000320193 for Apple.
        cik: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Ingest => {
            tracing::info!("ingest: not yet implemented");
        }
        Command::Analyse { cik } => {
            tracing::info!(%cik, "analyse: not yet implemented");
        }
    }
    Ok(())
}
