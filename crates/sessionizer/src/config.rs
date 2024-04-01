use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List the available sessions
    #[clap(name = "init")]
    Init {
        /// Force the override of the current configuration file.
        #[clap(short, long)]
        force: bool,
    },
    /// Reads a session or a session message
    #[clap(name = "edit")]
    Edit,
}

#[derive(Debug, Parser)]
#[command(name = "config")]
#[command(about = "Manage the sessionizer configuration")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init { force } => init(force).await,
        Commands::Edit => edit().await,
    }
}

pub async fn init(force: bool) -> Result<()> {
    todo!()
}

pub async fn edit() -> Result<()> {
    todo!()
}
