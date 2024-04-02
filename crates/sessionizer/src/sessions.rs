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

pub async fn run(config: Config, cli: Cli) -> Result<()> {
    match cli.command {
        Commands::History => history(config).await,
        Commands::Go { session } => go(config, session).await,
        Commands::Add { session, set } => add(config, session, set).await,
        Commands::Remove { session } => remove(config, session).await,
        Commands::Next { show } => next(config, show).await,
        Commands::Previous { show } => previous(config, show).await,
        Commands::Sync { reverse } => sync(config, reverse).await,
        Commands::New { session } => new(config, session).await,
    }
}

pub async fn history(config: Config) -> Result<()> {
    println!("{}", config.sessions.join("\n"));

    Ok(())
}

pub async fn new(mut config: Config, session: Option<String>) -> Result<()> {
    let session = if session.is_none() {
        if config.sessions.is_empty() {
            println!("No sessions in the history.");
            return Ok(());
        }

        Some(fzf::directories(&config).await?)
    } else {
        session
    }
    .unwrap();

    let session = session.trim();

    // Check that the `session` points to an existing directory.
    if !std::path::Path::new(&session).exists() {
        bail!("the session does not exists as a directory in the fs");
    }

    tmux::set(session).await?;

    config.sessions.retain(|s| s != session);
    config.sessions.push(String::from(session));
    config.save()?;

    Ok(())
}

pub async fn go(mut config: Config, session: Option<String>) -> Result<()> {
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

    tmux::set(session).await?;

    config.sessions.retain(|s| s != session);
    config.sessions.push(String::from(session));
    config.save()?;

    Ok(())
}

pub async fn add(mut config: Config, session: String, set: bool) -> Result<()> {
    if config.sessions.contains(&session) {
        println!("Session already exists in the history.");
        return Ok(());
    }

    tmux::new_session(&session).await?;
    config.sessions.push(session.clone());

    config.save()?;

    if set {
        tmux::set(&session).await?;
    }

    println!("Session {} added to the history.", &session);

    Ok(())
}

pub async fn remove(mut config: Config, session: String) -> Result<()> {
    // If we are currently on the session to be removed then bail
    if tmux::current_session().await? == session {
        println!("Cannot remove the current session.");
        return Ok(());
    }

    // Remove the `session` from the `history`.
    config.sessions.retain(|s| s != &session);

    config.save()?;

    Ok(())
}

pub async fn sync(config: Config, reverse: bool) -> Result<()> {
    match reverse {
        true => sync_from_tmux(config).await,
        false => sync_to_tmux(config).await,
    }
}

pub async fn sync_to_tmux(config: Config) -> Result<()> {
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

pub async fn sync_from_tmux(mut config: Config) -> Result<()> {
    let tmux_sessions = tmux::ls().await?;

    config.sessions = tmux_sessions;

    // Get a copy of the last element of the config.sessions array
    let last = config.sessions.last().cloned().wrap_err("fail to get the last session")?;

    tmux::set(&last).await?;

    config.save()?;

    Ok(())
}

pub async fn next(mut config: Config, show: bool) -> Result<()> {
    if config.sessions.is_empty() {
        println!("No more sessions in the history.");
        return Ok(());
    }

    if config.sessions.len() == 1 {
        println!("Only one session in the history.");
        return Ok(());
    }

    // Pop the first `session` inside `config.sessions`
    let session = config.sessions.remove(0);

    if show {
        println!("Next session: {}", session);
        return Ok(());
    }

    config.sessions.push(session.clone());
    tmux::set(&session).await?;

    config.save()?;

    Ok(())
}

pub async fn previous(mut config: Config, show: bool) -> Result<()> {
    if config.sessions.is_empty() {
        println!("No more sessions in the history.");
        return Ok(());
    }

    if config.sessions.len() == 1 {
        println!("Only one session in the history.");
        return Ok(());
    }

    let current = config.sessions.pop().wrap_err("fail to get the current session")?;
    let prev = config.sessions.last().cloned().wrap_err("fail to get the previous session")?;

    if show {
        println!("Previous session: {}", prev);
        return Ok(());
    }

    tmux::set(&prev).await?;
    // Append the `current` session to the beginning of the `config.sessions` vector.
    config.sessions.insert(0, current);

    config.save()?;

    Ok(())
}
