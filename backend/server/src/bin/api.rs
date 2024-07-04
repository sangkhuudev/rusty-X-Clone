use std::net::SocketAddr;
use clap::{Parser, command, Subcommand};
use dotenvy::dotenv;
use tracing::{debug, info};
use uchat_crypto::new_rng;
use uchat_query::AsyncConnectionPool;
use uchat_server::{cli::{gen_keys, load_keys}, logging::Verbosity, router::new_router, AppState};
use anyhow::{Result, Context};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // api -d <URL>
    // api --database-url <URL>
    #[clap(
        short,
        long,
        default_value = "postgres://test@localhost/test",
        env = "API_DATABASE_URL"
    )]
    database_url: String,
    #[clap(
        short,
        long,
        default_value = "localhost:8080",
        env = "API_BIND"
    )]
    bind: SocketAddr,
    #[clap(flatten)]
    verbosity: Verbosity,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// generate a session signing key
    GenKey,
}



async fn run() -> Result<()> {
    // color_eyre::install()?;
    let use_dotenv = dotenv();
    let args = Cli::parse();

    if let Ok(path) = use_dotenv {
        debug!(target: "uchat_server", dot_env_found = true, path = %path.to_string_lossy())
    } else {
        debug!(target: "uchat_server", dot_env_found = false);
    }

    if let Some(command) = args.command {
        match command {
            Command::GenKey => {
                let mut rng = new_rng();
                info!(target: "uchat_server", "Generating private key...");
                let (key, _) = gen_keys(&mut rng)?;
                let path ="private_key.base64";
                std::fs::write(path, key.as_str())?;
                info!(target: "uchat_server", path = path, "Private key has been saved to disk");
                info!(target: "uchat_server", "Set API_PRIVATE_KEY environment variable with the content of the key to use it");
                return Ok(());
            }
        }
    }

    debug!(target: "uchat_server", "loading signing keys");
    let signing_keys = load_keys()?;
    info!(target: "uchat_server", database_url = %args.database_url, "connecting to database");
    let db_pool = AsyncConnectionPool::new(&args.database_url)
        .await
        .with_context(|| "Check database url")
        .with_context(|| "Ensure databasae access rights")
        .with_context(|| "Make sure database exists")?;

    let state = AppState {
        db_pool,
        signing_keys,
        rng: new_rng()
    };

    info!(target: "uchat_server", bind_addr = %args.bind);
    let router = new_router(state).await;
    let listener = tokio::net::TcpListener::bind(&args.bind)
        .await
        // .wrap_error_with(|| "Failed to initialize server")
        .with_context(|| "Check bind address")
        .with_context(|| "Check if another service using this port")?;
    // let server = listener.serve(router.await.into_make_service());

    // if let Err(e) = server.await {
    //     error!(target: "uchat_server", server_error = %e);
    // }
    axum::serve(listener, router).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}