use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add a new directory to be tracked by sessionizer
    #[clap(name = "add")]
    Add {
        /// Directory to add
        directory: String,
        /// Minimum directory depth to scan from the given directory.
        #[clap(short, long, default_value = "1")]
        mindepth: Option<usize>,
        /// Maximum directory depth to scan from the given directory.
        #[clap(short, long, default_value = "1")]
        maxdepth: Option<usize>,
        /// Grep a specific type of directory
        #[clap(short, long)]
        grep: Option<String>,
    },
    /// Remove a directory to be tracked by sessionizer.
    #[clap(name = "remove")]
    Remove {
        /// Directory to add
        directory: String,
    },
    /// List the directoried added.
    #[clap(name = "list")]
    List,
    /// List all the directories available
    #[clap(name = "evaluate")]
    Evaluate,
}

#[derive(Debug, Parser)]
#[command(name = "directories")]
#[command(about = "Manage the sessionizer directories")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add { directory, mindepth, maxdepth, grep } => {
            add(directory, mindepth, maxdepth, grep).await
        }
        Commands::Remove { directory } => remove(directory).await,
        Commands::List => list().await,
        Commands::Evaluate => evaluate().await,
    }
}

pub async fn add(
    directory: String,
    mindepth: Option<usize>,
    maxdepth: Option<usize>,
    grep: Option<String>,
) -> Result<()> {
    todo!()
}

pub async fn remove(directory: String) -> Result<()> {
    todo!()
}

pub async fn list() -> Result<()> {
    todo!()
}

pub async fn evaluate() -> Result<()> {
    todo!()
}
