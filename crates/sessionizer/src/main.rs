use clap::{Parser, Subcommand};
use color_eyre::eyre::bail;

mod config;
mod directories;
mod fzf;
mod sessions;
mod shutdown;
mod tmux;

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
    let result = match Cli::parse().command {
        Commands::Config(cli) => crate::config::run(cli).await,
        Commands::Directories(cli) => crate::directories::run(cli).await,
        Commands::Sessions(cli) => crate::sessions::run(cli).await,
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
