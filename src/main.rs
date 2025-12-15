//! echidnabot CLI and server entry point

use clap::{Parser, Subcommand};
use echidnabot::{Config, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "echidnabot")]
#[command(about = "Proof-aware CI bot for theorem prover repositories")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to configuration file
    #[arg(short, long, default_value = "echidnabot.toml")]
    config: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the webhook server
    Serve {
        /// Host to bind to
        #[arg(short = 'H', long, default_value = "0.0.0.0")]
        host: String,

        /// Port to bind to
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Register a repository for monitoring
    Register {
        /// Repository in format owner/name
        #[arg(short, long)]
        repo: String,

        /// Platform (github, gitlab, bitbucket)
        #[arg(short, long, default_value = "github")]
        platform: String,

        /// Provers to enable (comma-separated)
        #[arg(long, default_value = "metamath")]
        provers: String,
    },

    /// Manually trigger a proof check
    Check {
        /// Repository in format owner/name
        #[arg(short, long)]
        repo: String,

        /// Commit SHA (defaults to HEAD)
        #[arg(short, long)]
        commit: Option<String>,

        /// Specific prover to use
        #[arg(short, long)]
        prover: Option<String>,
    },

    /// Show status of a repository or job
    Status {
        /// Repository in format owner/name, or job ID
        #[arg(short, long)]
        target: String,
    },

    /// Initialize the database
    InitDb,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let _config = Config::load(&cli.config)?;

    match cli.command {
        Commands::Serve { host, port } => {
            tracing::info!("Starting echidnabot server on {}:{}", host, port);
            serve(&host, port).await
        }
        Commands::Register {
            repo,
            platform,
            provers,
        } => {
            tracing::info!(
                "Registering {} on {} with provers: {}",
                repo,
                platform,
                provers
            );
            register(&repo, &platform, &provers).await
        }
        Commands::Check {
            repo,
            commit,
            prover,
        } => {
            tracing::info!("Triggering check for {} at {:?}", repo, commit);
            check(&repo, commit.as_deref(), prover.as_deref()).await
        }
        Commands::Status { target } => {
            tracing::info!("Getting status for {}", target);
            status(&target).await
        }
        Commands::InitDb => {
            tracing::info!("Initializing database");
            init_db().await
        }
    }
}

async fn serve(host: &str, port: u16) -> Result<()> {
    use axum::{routing::get, Router};

    let app = Router::new()
        .route("/health", get(health))
        .route("/", get(root));

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    tracing::info!("Listening on http://{}:{}", host, port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> &'static str {
    "OK"
}

async fn root() -> &'static str {
    "echidnabot - Proof-aware CI bot\n\nEndpoints:\n  GET /health\n  POST /webhooks/github\n  POST /graphql"
}

async fn register(_repo: &str, _platform: &str, _provers: &str) -> Result<()> {
    tracing::warn!("register command not yet implemented");
    Ok(())
}

async fn check(_repo: &str, _commit: Option<&str>, _prover: Option<&str>) -> Result<()> {
    tracing::warn!("check command not yet implemented");
    Ok(())
}

async fn status(_target: &str) -> Result<()> {
    tracing::warn!("status command not yet implemented");
    Ok(())
}

async fn init_db() -> Result<()> {
    tracing::warn!("init-db command not yet implemented");
    Ok(())
}
