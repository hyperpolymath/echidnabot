//! echidnabot CLI and server entry point

use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use echidnabot::{
    adapters::Platform,
    api::{
        graphql::{create_schema, AppContext, EchidnabotSchema},
        webhooks,
    },
    config::Config,
    dispatcher::{EchidnaClient, ProverKind},
    scheduler::{JobScheduler, ProofJob},
    store::{models::Repository, SqliteStore, Store},
    Error, Result,
};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

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

        /// Webhook secret for signature verification
        #[arg(long)]
        webhook_secret: Option<String>,
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

    /// List registered repositories
    List {
        /// Filter by platform
        #[arg(short, long)]
        platform: Option<String>,
    },

    /// Initialize the database
    InitDb,
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub store: Arc<dyn Store>,
    pub scheduler: Arc<JobScheduler>,
    pub echidna: Arc<EchidnaClient>,
    pub schema: EchidnabotSchema,
    webhook_state: webhooks::WebhookState,
}

impl AsRef<webhooks::WebhookState> for AppState {
    fn as_ref(&self) -> &webhooks::WebhookState {
        &self.webhook_state
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = Arc::new(Config::load(&cli.config)?);

    match cli.command {
        Commands::Serve { host, port } => {
            tracing::info!("Starting echidnabot server on {}:{}", host, port);
            serve(config, &host, port).await
        }
        Commands::Register {
            repo,
            platform,
            provers,
            webhook_secret,
        } => {
            tracing::info!(
                "Registering {} on {} with provers: {}",
                repo,
                platform,
                provers
            );
            register(config, &repo, &platform, &provers, webhook_secret.as_deref()).await
        }
        Commands::Check {
            repo,
            commit,
            prover,
        } => {
            tracing::info!("Triggering check for {} at {:?}", repo, commit);
            check(config, &repo, commit.as_deref(), prover.as_deref()).await
        }
        Commands::Status { target } => {
            tracing::info!("Getting status for {}", target);
            status(config, &target).await
        }
        Commands::List { platform } => {
            list_repos(config, platform.as_deref()).await
        }
        Commands::InitDb => {
            tracing::info!("Initializing database");
            init_db(config).await
        }
    }
}

async fn create_app_state(config: Arc<Config>) -> Result<AppState> {
    // Initialize store
    let store: Arc<dyn Store> = Arc::new(SqliteStore::new(&config.database.url).await?);

    // Initialize scheduler
    let scheduler = Arc::new(JobScheduler::new(
        config.scheduler.max_concurrent,
        config.scheduler.queue_size,
    ));

    // Initialize ECHIDNA client
    let echidna = Arc::new(EchidnaClient::new(&config.echidna));

    // Create GraphQL schema with context
    let ctx = AppContext {
        store: store.clone(),
        scheduler: scheduler.clone(),
        echidna: echidna.clone(),
        config: config.clone(),
    };
    let schema = create_schema(ctx);

    // Create webhook state
    let webhook_state = webhooks::WebhookState {
        config: config.clone(),
        store: store.clone(),
        scheduler: scheduler.clone(),
    };

    Ok(AppState {
        config,
        store,
        scheduler,
        echidna,
        schema,
        webhook_state,
    })
}

async fn serve(config: Arc<Config>, host: &str, port: u16) -> Result<()> {
    let state = create_app_state(config.clone()).await?;

    // Check ECHIDNA connectivity
    match state.echidna.health_check().await {
        Ok(true) => tracing::info!("ECHIDNA Core is reachable"),
        Ok(false) => tracing::warn!("ECHIDNA Core is not responding"),
        Err(e) => tracing::warn!("Could not reach ECHIDNA Core: {}", e),
    }

    let app = Router::new()
        // Health and info
        .route("/health", get(health))
        .route("/", get(root))
        // GraphQL
        .route("/graphql", post(graphql_handler))
        .route("/graphql/playground", get(graphql_playground))
        // Webhooks
        .route("/webhooks/github", post(webhooks::handle_github_webhook::<AppState>))
        .route("/webhooks/gitlab", post(webhooks::handle_gitlab_webhook::<AppState>))
        .route(
            "/webhooks/bitbucket",
            post(webhooks::handle_bitbucket_webhook::<AppState>),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port)).await?;
    tracing::info!("Listening on http://{}:{}", host, port);
    tracing::info!("GraphQL playground: http://{}:{}/graphql/playground", host, port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> &'static str {
    // Check database connectivity
    match state.store.health_check().await {
        Ok(true) => "OK",
        _ => "DEGRADED",
    }
}

async fn root() -> &'static str {
    r#"echidnabot - Proof-aware CI bot for theorem prover repositories

Endpoints:
  GET  /health              - Health check
  GET  /graphql/playground  - GraphQL playground
  POST /graphql             - GraphQL API
  POST /webhooks/github     - GitHub webhook receiver
  POST /webhooks/gitlab     - GitLab webhook receiver
  POST /webhooks/bitbucket  - Bitbucket webhook receiver

Documentation: https://github.com/hyperpolymath/echidnabot
"#
}

async fn graphql_handler(
    State(state): State<AppState>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    state.schema.execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>echidnabot GraphQL Playground</title>
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/css/index.css" />
    <script src="https://cdn.jsdelivr.net/npm/graphql-playground-react/build/static/js/middleware.js"></script>
</head>
<body>
    <div id="root"></div>
    <script>
        window.addEventListener('load', function() {
            GraphQLPlayground.init(document.getElementById('root'), {
                endpoint: '/graphql'
            })
        })
    </script>
</body>
</html>"#,
    )
}

async fn register(
    config: Arc<Config>,
    repo: &str,
    platform: &str,
    provers: &str,
    webhook_secret: Option<&str>,
) -> Result<()> {
    let store = SqliteStore::new(&config.database.url).await?;

    // Parse repository
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::Config(
            "Repository must be in format owner/name".to_string(),
        ));
    }
    let (owner, name) = (parts[0], parts[1]);

    // Parse platform
    let platform = match platform.to_lowercase().as_str() {
        "github" => Platform::GitHub,
        "gitlab" => Platform::GitLab,
        "bitbucket" => Platform::Bitbucket,
        "codeberg" => Platform::Codeberg,
        _ => return Err(Error::Config(format!("Unknown platform: {}", platform))),
    };

    // Parse provers
    let enabled_provers: Vec<ProverKind> = provers
        .split(',')
        .filter_map(|p| parse_prover(p.trim()))
        .collect();

    if enabled_provers.is_empty() {
        return Err(Error::Config("No valid provers specified".to_string()));
    }

    // Check if already registered
    if let Some(existing) = store
        .get_repository_by_name(platform, owner, name)
        .await?
    {
        println!("Repository already registered with ID: {}", existing.id);
        return Ok(());
    }

    // Create repository record
    let mut repo = Repository::new(platform, owner.to_string(), name.to_string());
    repo.enabled_provers = enabled_provers.clone();
    repo.webhook_secret = webhook_secret.map(|s| s.to_string());

    store.create_repository(&repo).await?;

    println!("Registered repository: {}/{}", owner, name);
    println!("  ID: {}", repo.id);
    println!("  Platform: {:?}", platform);
    println!(
        "  Provers: {}",
        enabled_provers
            .iter()
            .map(|p| p.display_name())
            .collect::<Vec<_>>()
            .join(", ")
    );

    if webhook_secret.is_some() {
        println!("  Webhook secret: (configured)");
    }

    println!("\nNext steps:");
    println!("  1. Configure a webhook in your repository settings");
    println!("  2. Point it to: https://your-server/webhooks/{}", platform_slug(platform));
    println!("  3. Set content type to application/json");
    if webhook_secret.is_some() {
        println!("  4. Use the same secret you provided");
    }

    Ok(())
}

async fn check(
    config: Arc<Config>,
    repo: &str,
    commit: Option<&str>,
    prover: Option<&str>,
) -> Result<()> {
    let store = SqliteStore::new(&config.database.url).await?;
    let scheduler = JobScheduler::new(
        config.scheduler.max_concurrent,
        config.scheduler.queue_size,
    );

    // Parse repository
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        return Err(Error::Config(
            "Repository must be in format owner/name".to_string(),
        ));
    }
    let (owner, name) = (parts[0], parts[1]);

    // Find repository - try all platforms
    let mut found_repo = None;
    for platform in [
        Platform::GitHub,
        Platform::GitLab,
        Platform::Bitbucket,
        Platform::Codeberg,
    ] {
        if let Some(r) = store.get_repository_by_name(platform, owner, name).await? {
            found_repo = Some(r);
            break;
        }
    }

    let repository = found_repo.ok_or_else(|| {
        Error::RepoNotFound(format!(
            "{}/{}. Register it first with 'echidnabot register'",
            owner, name
        ))
    })?;

    // Determine commit
    let commit_sha = commit.unwrap_or("HEAD").to_string();

    // Determine prover(s)
    let provers: Vec<ProverKind> = match prover {
        Some(p) => {
            vec![parse_prover(p).ok_or_else(|| Error::InvalidProver(p.to_string()))?]
        }
        None => repository.enabled_provers.clone(),
    };

    println!("Triggering proof check:");
    println!("  Repository: {}/{}", owner, name);
    println!("  Commit: {}", commit_sha);
    println!(
        "  Provers: {}",
        provers
            .iter()
            .map(|p| p.display_name())
            .collect::<Vec<_>>()
            .join(", ")
    );

    // Create jobs for each prover
    for prover_kind in provers {
        let job = ProofJob::new(
            repository.id,
            commit_sha.clone(),
            prover_kind,
            vec![], // Will detect files during processing
        )
        .with_priority(echidnabot::scheduler::JobPriority::Critical);

        match scheduler.enqueue(job).await? {
            Some(job_id) => {
                println!("  Created job {} for {}", job_id, prover_kind.display_name());

                // Also persist to database
                if let Some(queued_job) = scheduler.get_job(job_id).await {
                    store.create_job(&queued_job.into()).await?;
                }
            }
            None => {
                println!(
                    "  Skipped {} (duplicate job exists)",
                    prover_kind.display_name()
                );
            }
        }
    }

    println!("\nJobs queued. Run 'echidnabot serve' to process them.");

    Ok(())
}

async fn status(config: Arc<Config>, target: &str) -> Result<()> {
    let store = SqliteStore::new(&config.database.url).await?;

    // Try to parse as UUID (job ID)
    if let Ok(job_id) = Uuid::parse_str(target) {
        let job = store
            .get_job(echidnabot::scheduler::JobId(job_id))
            .await?
            .ok_or_else(|| Error::JobNotFound(job_id))?;

        println!("Job: {}", job.id);
        println!("  Status: {:?}", job.status);
        println!("  Prover: {:?}", job.prover);
        println!("  Commit: {}", job.commit_sha);
        println!("  Queued: {}", job.queued_at);
        if let Some(started) = job.started_at {
            println!("  Started: {}", started);
        }
        if let Some(completed) = job.completed_at {
            println!("  Completed: {}", completed);
        }
        if let Some(error) = &job.error_message {
            println!("  Error: {}", error);
        }

        // Check for result
        if let Some(result) = store
            .get_result_for_job(echidnabot::scheduler::JobId(job_id))
            .await?
        {
            println!("  Result: {}", if result.success { "PASS" } else { "FAIL" });
            println!("  Duration: {}ms", result.duration_ms);
            if !result.message.is_empty() {
                println!("  Message: {}", result.message);
            }
        }
    } else {
        // Parse as repository
        let parts: Vec<&str> = target.split('/').collect();
        if parts.len() != 2 {
            return Err(Error::Config(
                "Target must be a job ID (UUID) or repository (owner/name)".to_string(),
            ));
        }
        let (owner, name) = (parts[0], parts[1]);

        // Find repository
        let mut found_repo = None;
        for platform in [
            Platform::GitHub,
            Platform::GitLab,
            Platform::Bitbucket,
            Platform::Codeberg,
        ] {
            if let Some(r) = store.get_repository_by_name(platform, owner, name).await? {
                found_repo = Some(r);
                break;
            }
        }

        let repository = found_repo.ok_or_else(|| {
            Error::RepoNotFound(format!("{}/{}", owner, name))
        })?;

        println!("Repository: {}/{}", owner, name);
        println!("  ID: {}", repository.id);
        println!("  Platform: {:?}", repository.platform);
        println!("  Enabled: {}", repository.enabled);
        println!(
            "  Provers: {}",
            repository
                .enabled_provers
                .iter()
                .map(|p| p.display_name())
                .collect::<Vec<_>>()
                .join(", ")
        );
        if let Some(commit) = &repository.last_checked_commit {
            println!("  Last checked: {}", commit);
        }

        // Show recent jobs
        let jobs = store.list_jobs_for_repo(repository.id, 5).await?;
        if !jobs.is_empty() {
            println!("\nRecent jobs:");
            for job in jobs {
                println!(
                    "  {} {:?} {} ({:?})",
                    &job.id.to_string()[..8],
                    job.prover,
                    &job.commit_sha[..8.min(job.commit_sha.len())],
                    job.status
                );
            }
        }
    }

    Ok(())
}

async fn list_repos(config: Arc<Config>, platform: Option<&str>) -> Result<()> {
    let store = SqliteStore::new(&config.database.url).await?;

    let platform_filter = platform.map(|p| match p.to_lowercase().as_str() {
        "github" => Platform::GitHub,
        "gitlab" => Platform::GitLab,
        "bitbucket" => Platform::Bitbucket,
        "codeberg" => Platform::Codeberg,
        _ => Platform::GitHub,
    });

    let repos = store.list_repositories(platform_filter).await?;

    if repos.is_empty() {
        println!("No repositories registered.");
        println!("Use 'echidnabot register -r owner/name' to register one.");
        return Ok(());
    }

    println!("Registered repositories:\n");
    for repo in repos {
        let status = if repo.enabled { "enabled" } else { "disabled" };
        println!(
            "  {:?} {}/{} [{}]",
            repo.platform, repo.owner, repo.name, status
        );
        println!(
            "    Provers: {}",
            repo.enabled_provers
                .iter()
                .map(|p| p.display_name())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    Ok(())
}

async fn init_db(config: Arc<Config>) -> Result<()> {
    println!("Initializing database at: {}", config.database.url);

    // Creating the store runs migrations automatically
    let store = SqliteStore::new(&config.database.url).await?;

    // Verify it works
    match store.health_check().await {
        Ok(true) => {
            println!("Database initialized successfully!");
            println!("\nTables created:");
            println!("  - repositories");
            println!("  - proof_jobs");
            println!("  - proof_results");
        }
        Ok(false) => {
            println!("Warning: Database may not be fully functional");
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(())
}

fn parse_prover(s: &str) -> Option<ProverKind> {
    match s.to_lowercase().as_str() {
        "agda" => Some(ProverKind::Agda),
        "coq" => Some(ProverKind::Coq),
        "lean" => Some(ProverKind::Lean),
        "isabelle" => Some(ProverKind::Isabelle),
        "z3" => Some(ProverKind::Z3),
        "cvc5" => Some(ProverKind::Cvc5),
        "metamath" | "mm" => Some(ProverKind::Metamath),
        "hol-light" | "hollight" => Some(ProverKind::HolLight),
        "mizar" => Some(ProverKind::Mizar),
        "pvs" => Some(ProverKind::Pvs),
        "acl2" => Some(ProverKind::Acl2),
        "hol4" => Some(ProverKind::Hol4),
        _ => None,
    }
}

fn platform_slug(platform: Platform) -> &'static str {
    match platform {
        Platform::GitHub => "github",
        Platform::GitLab => "gitlab",
        Platform::Bitbucket => "bitbucket",
        Platform::Codeberg => "codeberg",
    }
}
