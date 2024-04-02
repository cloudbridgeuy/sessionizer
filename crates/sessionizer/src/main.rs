use clap::{Parser, Subcommand};
use color_eyre::eyre::bail;

mod config;
mod directories;
mod fzf;
mod sessions;
mod shutdown;
mod tmux;

use crate::config::Config;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Handle the sessionizer configuration
    #[clap(name = "config")]
    Config(crate::config::Cli),
    /// Handle sessionizer folders
    #[clap(name = "directories")]
    Directories(crate::directories::Cli),
    /// Handle tmux sessions created through sessionizer
    #[clap(name = "sessions")]
    Sessions(crate::sessions::Cli),
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = "Handle tmux sessions based on your file system")]
pub struct Cli {
    /// Custom path for the configuration file
    #[clap(short, long, env = "SESSIONIZER_CONFIG", global = true)]
    pub config: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[tokio::main]
async fn main() -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init();

    // Create the shutdown handler
    let shutdown = crate::shutdown::Shutdown::new()?;

    // Run app in separate async task
    tokio::spawn(async {
        if let Err(e) = run().await {
            bail!("Application error: {}", e)
        }

        Ok(())
    });

    shutdown.handle().await;

    Ok(())
}

async fn run() -> color_eyre::eyre::Result<()> {
    log::debug!("Parsing CLI arguments");
    let cli = Cli::parse();

    log::debug!("Loading configuration path");
    let config_path = match cli.config {
        Some(path) => path,
        None => Config::home()?,
    };

    let config: Config = if let Commands::Config(sub) = &cli.command {
        if let crate::config::Commands::Init { force: _ } = sub.command {
            log::debug!("Avoid creating configuration file");
            Config::new(&config_path)
        } else {
            log::debug!("Loading configuration from {}", config_path);
            Config::load(&config_path)?
        }
    } else {
        Config::load(&config_path)?
    };

    log::debug!("Running command");
    let result = match cli.command {
        Commands::Config(cli) => crate::config::run(&config_path, cli).await,
        Commands::Directories(cli) => crate::directories::run(config, cli).await,
        Commands::Sessions(cli) => crate::sessions::run(config, cli).await,
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
