pub mod commands;
pub mod templates;
pub mod frameworks;
pub mod project;
pub mod server;

use crate::commands::init_command::handle_init_command;
use clap::{Parser, Subcommand};
use tracing::error;
use tracing_subscriber::EnvFilter;
use crate::commands::dev_command::handle_dev_command;

#[derive(Parser, Debug)]
#[command(name = "nucleus")]
#[command(about = "Nucleus - A blazingly fast, buzzword ridden, simple Content Management System")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize a new nucleus project
    Init { name: String },
    Dev
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("{:?}", e);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        _ => {
            tracing_subscriber::fmt()
                .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
                .without_time()
                .with_target(false)
                .init();
        }
    }

    match cli {
        Cli { command } => match command {
            Commands::Init { name } => handle_init_command(name),
            Commands::Dev => handle_dev_command().await,
        },
    }?;

    Ok(())
}
