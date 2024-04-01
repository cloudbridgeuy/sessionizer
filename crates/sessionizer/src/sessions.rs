use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Sessions {
    pub current: String,
    pub history: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List the previously visited sessions.
    #[clap(name = "history")]
    History,
    /// Go to a session.
    #[clap(name = "go")]
    Go {
        /// Tmux Session
        session: String,
    },
    /// Add a new session.
    #[clap(name = "add")]
    Add {
        /// Tmux Session
        session: String,
    },
    /// Remove a running session.
    #[clap(name = "remove")]
    Remove {
        /// Tmux Session
        session: String,
    },
    /// Go or show the next session.
    #[clap(name = "next")]
    Next {
        /// Show the next session but don't transition to it.
        #[clap(short, long)]
        show: bool,
    },
    /// Go or show the previous session.
    #[clap(name = "previous")]
    Previous {
        /// Show the previous session but don't transition to it.
        #[clap(short, long)]
        show: bool,
    },
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
        Commands::History => history().await,
        Commands::Go { session } => go(session).await,
        Commands::Add { session } => add(session).await,
        Commands::Remove { session } => remove(session).await,
        Commands::Next { show } => next(show).await,
        Commands::Previous { show } => previous(show).await,
    }
}

pub async fn history() -> Result<()> {
    todo!()
}

pub async fn go(session: String) -> Result<()> {
    todo!()
}

pub async fn add(session: String) -> Result<()> {
    todo!()
}

pub async fn remove(session: String) -> Result<()> {
    todo!()
}

pub async fn next(show: bool) -> Result<()> {
    todo!()
}

pub async fn previous(show: bool) -> Result<()> {
    todo!()
}
