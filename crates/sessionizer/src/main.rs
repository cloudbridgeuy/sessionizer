use clap::{Parser, Subcommand};

mod config;
mod directories;
mod fzf;
mod sessions;
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

    ctrlc::set_handler(move || {
        log::error!("Ctrl-C received, stopping the program");
        std::process::exit(1);
    })?;

    run().await
}

#[derive(Copy, Clone)]
pub enum ExitStatus {
    /// Program finished successfully.
    Success,
    /// Program finished with an known error.
    Failure,
    /// Program finished with an unknown error.
    Error,
}

impl From<ExitStatus> for i32 {
    fn from(status: ExitStatus) -> i32 {
        match status {
            ExitStatus::Success => 0,
            ExitStatus::Failure => 1,
            ExitStatus::Error => 2,
        }
    }
}

async fn run() -> color_eyre::eyre::Result<()> {
    log::debug!("Parsing CLI arguments");
    let cli = Cli::parse();

    log::debug!("Loading configuration path");

    let (config, config_path) = get_config(&cli)?;

    log::debug!("Running command");
    match cli.command {
        Commands::Config(cli) => crate::config::run(&config_path, cli).await,
        Commands::Directories(cli) => crate::directories::run(config, cli).await,
        Commands::Sessions(cli) => crate::sessions::run(config, cli).await,
    }
}

fn get_config(cli: &Cli) -> color_eyre::eyre::Result<(Config, String)> {
    let config_path = match cli.config.clone() {
        Some(path) => path,
        None => Config::home()?,
    };
    let config = if let Commands::Config(sub) = &cli.command {
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

    Ok((config, config_path))
}
