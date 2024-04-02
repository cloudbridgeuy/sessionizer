use clap::{Parser, Subcommand};
use color_eyre::eyre::bail;
use color_eyre::eyre::{ContextCompat, Result};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::fzf;
use crate::tmux;

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
        session: Option<String>,
    },
    /// Create and go to a new session.
    #[clap(name = "new")]
    New {
        /// Tmux Session
        session: Option<String>,
    },
    /// Add a new session.
    #[clap(name = "add")]
    Add {
        /// Tmux Session
        session: String,
        /// Set the new session and transition to it.
        #[clap(short, long)]
        set: bool,
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
    /// Sync the sessionizer sessions to and from `tmux`.
    ///
    /// NOTE: To TMUX by default.
    Sync {
        /// Reverse the action to synchronize the sessions from `tmux`
        #[clap(short, long)]
        reverse: bool,
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
        Commands::Add { session, set } => add(session, set).await,
        Commands::Remove { session } => remove(session).await,
        Commands::Next { show } => next(show).await,
        Commands::Previous { show } => previous(show).await,
        Commands::Sync { reverse } => sync(reverse).await,
        Commands::New { session } => new(session).await,
    }
}

pub async fn history() -> Result<()> {
    let config = Config::load()?;

    println!("{}", config.sessions.join("\n"));

    Ok(())
}

pub async fn set(session: &str) -> Result<()> {
    // Create the session if it doesn't exist.
    log::debug!("Checking if tmux has session: {}", session);
    if !tmux::has_session(session).await? {
        log::debug!("Creating session: {}", session);
        tmux::new_session(session).await?
    }

    // Attach to the existing session.
    log::debug!("Checking if tmux is running");
    if tmux::is_active()? {
        log::debug!("Changing client to point to session: {}", session);
        tmux::switch_client(session).await?;
    } else {
        log::debug!("Attaching tmux to session: {}", session);
        tmux::attach(session).await?;
    }

    Ok(())
}

pub async fn new(session: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    let session = if session.is_none() {
        if config.sessions.is_empty() {
            println!("No sessions in the history.");
            return Ok(());
        }

        Some(fzf::directories().await?)
    } else {
        session
    }
    .unwrap();

    let session = session.trim();

    // Check that the `session` points to an existing directory.
    if !std::path::Path::new(&session).exists() {
        bail!("the session does not exists as a directory in the fs");
    }

    set(session).await?;

    config.sessions.retain(|s| s != session);
    config.sessions.push(String::from(session));
    Config::save(&config)?;

    Ok(())
}

pub async fn go(session: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    let session = if session.is_none() {
        if config.sessions.is_empty() {
            println!("No sessions in the history.");
            return Ok(());
        }

        Some(fzf::sessions(config.sessions.clone()).await?)
    } else {
        session
    }
    .unwrap();

    let session = session.trim();

    // Check that the `session` points to an existing directory.
    if !std::path::Path::new(&session).exists() {
        bail!("the session does not exists as a directory in the fs");
    }

    if !config.sessions.iter().any(|s| s == session) {
        println!("Session not found in the history.");
        return Ok(());
    }

    set(session).await?;

    config.sessions.retain(|s| s != session);
    config.sessions.push(String::from(session));
    Config::save(&config)?;

    Ok(())
}

pub async fn add(session: String, set: bool) -> Result<()> {
    let mut config = Config::load()?;

    if config.sessions.contains(&session) {
        println!("Session already exists in the history.");
        return Ok(());
    }

    tmux::new_session(&session).await?;
    config.sessions.push(session.clone());

    Config::save(&config)?;

    if set {
        crate::sessions::set(&session).await?;
    }

    println!("Session {} added to the history.", &session);

    Ok(())
}

pub async fn remove(session: String) -> Result<()> {
    let mut config = Config::load()?;

    // If we are currently on the session to be removed then bail
    if tmux::current_session().await? == session {
        println!("Cannot remove the current session.");
        return Ok(());
    }

    // Remove the `session` from the `history`.
    config.sessions.retain(|s| s != &session);

    Config::save(&config)?;

    Ok(())
}

pub async fn sync(reverse: bool) -> Result<()> {
    match reverse {
        true => sync_from_tmux().await,
        false => sync_to_tmux().await,
    }
}

pub async fn sync_to_tmux() -> Result<()> {
    let config = Config::load()?;
    let sessions = tmux::ls().await?;

    for session in sessions {
        if !config.sessions.contains(&session) {
            tmux::kill_session(&session).await?;
        }
    }

    for session in &config.sessions {
        if tmux::has_session(session).await? {
            continue;
        }

        tmux::new_session(session).await?;
    }

    Ok(())
}

pub async fn sync_from_tmux() -> Result<()> {
    let mut config = Config::load()?;

    let tmux_sessions = tmux::ls().await?;

    println!("tmux sessions: {:?}", tmux_sessions);

    config.sessions = tmux_sessions;

    // Get a copy of the last element of the config.sessions array
    let last = config.sessions.last().cloned().wrap_err("fail to get the last session")?;

    set(&last).await?;

    Config::save(&config)?;

    Ok(())
}

pub async fn next(show: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Check if the `history` has zero entries.
    if config.sessions.is_empty() {
        println!("No more sessions in the history.");
        return Ok(());
    }

    // Check if there's only one `session` in the `history`.
    if config.sessions.len() == 1 {
        println!("Only one session in the history.");
        return Ok(());
    }

    // Pop the latest `session` of the history.
    let session = config.sessions.pop();

    // Move the popped `session` to the first element of the history.
    if let Some(session) = session {
        if show {
            println!("Next session: {}", session);
            return Ok(());
        }
        set(&session).await?;

        config.sessions.insert(0, session);
    } else {
        println!("No more sessions in the history.");
    }

    Config::save(&config)?;

    Ok(())
}

pub async fn previous(show: bool) -> Result<()> {
    let mut config = Config::load()?;

    // Check if the `history` has zero entries.
    if config.sessions.is_empty() {
        println!("No more sessions in the history.");
        return Ok(());
    }

    // Check if there's only one `session` in the `history`.
    if config.sessions.len() == 1 {
        println!("Only one session in the history.");
        return Ok(());
    }

    // Pop the first element from the `history`.
    let session = config.sessions.remove(0);

    // Append the `session` to the last of the `history` vector.
    if show {
        println!("Previous session: {}", session);
        return Ok(());
    }

    set(&session).await?;
    config.sessions.push(session.clone());
    Config::save(&config)?;

    Ok(())
}
