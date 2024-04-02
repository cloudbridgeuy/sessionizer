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
    #[clap(name = "history", alias = "list")]
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
    #[clap(name = "remove", alias = "rm")]
    Remove {
        /// Tmux Session
        session: String,
    },
    /// Go or show the next session.
    #[clap(name = "next", alias = "n")]
    Next {
        /// Show the next session but don't transition to it.
        #[clap(short, long)]
        show: bool,
    },
    /// Go or show the previous session.
    #[clap(name = "prev", alias = "p")]
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
    let config = crate::config::Config::load()?;

    for (index, session) in config.sessions.history.iter().enumerate() {
        println!("{}: {}", index + 1, session);
    }

    Ok(())
}

pub async fn go(session: String) -> Result<()> {
    todo!()
}

pub async fn add(session: String) -> Result<()> {
    let mut config = crate::config::Config::load()?;

    if config.sessions.history.contains(&session) {
        println!("Session already exists in the history.");
        return Ok(());
    }

    config.sessions.history.push(session.clone());

    crate::config::Config::save(&config)?;

    println!("Session {} added to the history.", &session);

    Ok(())
}

pub async fn remove(session: String) -> Result<()> {
    let mut config = crate::config::Config::load()?;

    // Remove the `session` from the `history`.
    config.sessions.history.retain(|s| s != &session);

    crate::config::Config::save(&config)?;

    println!("Session {} removed from the history.", &session);

    Ok(())
}

pub async fn next(show: bool) -> Result<()> {
    let mut config = crate::config::Config::load()?;

    // Check if the `history` has zero entries.
    if config.sessions.history.is_empty() {
        println!("No more sessions in the history.");
        return Ok(());
    }

    // Check if there's only one `session` in the `history`.
    if config.sessions.history.len() == 1 {
        println!("Only one session in the history.");
        return Ok(());
    }

    // Pop the latest `session` of the history.
    let session = config.sessions.history.pop();

    // Move the popped `session` to the first element of the history.
    if let Some(session) = session {
        if show {
            println!("Next session: {}", session);
            return Ok(());
        }
        config.sessions.history.insert(0, session);
    } else {
        println!("No more sessions in the history.");
    }

    crate::config::Config::save(&config)?;

    Ok(())
}

pub async fn previous(show: bool) -> Result<()> {
    let mut config = crate::config::Config::load()?;

    // Check if the `history` has zero entries.
    if config.sessions.history.is_empty() {
        println!("No more sessions in the history.");
        return Ok(());
    }

    // Check if there's only one `session` in the `history`.
    if config.sessions.history.len() == 1 {
        println!("Only one session in the history.");
        return Ok(());
    }

    // Pop the first element from the `history`.
    let session = config.sessions.history.remove(0);

    // Append the `session` to the last of the `history` vector.
    if show {
        println!("Previous session: {}", session);
        return Ok(());
    }

    config.sessions.history.push(session.clone());
    crate::config::Config::save(&config)?;

    Ok(())
}
