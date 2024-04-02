use clap::{Parser, Subcommand};
use color_eyre::eyre::{eyre, ContextCompat, OptionExt, Result, WrapErr};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

use crate::config::Config;
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
    }
}

pub async fn history() -> Result<()> {
    let config = Config::load()?;

    println!("{}", config.sessions.join("\n"));

    Ok(())
}

pub async fn set(session: &str) -> Result<()> {
    // Create the session if it doesn't exist.
    if !tmux::has_session(session).await? {
        tmux::new_session(session).await?
    }

    // Attach to the existing session.
    if tmux::is_active()? {
        tmux::switch_client(session).await?;
    } else {
        tmux::attach(session).await?;
    }

    Ok(())
}

pub async fn go(session: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    let session = if session.is_none() {
        if config.sessions.is_empty() {
            println!("No sessions in the history.");
            return Ok(());
        }

        Some(fzf_sessions(config.sessions.clone()).await?)
    } else {
        session
    }
    .unwrap();

    let session = session.trim();

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

pub async fn fzf_sessions(sessions: Vec<String>) -> Result<String> {
    let mut fzf = tokio::process::Command::new("fzf")
        .args([
            "--header",
            "Press CTRL-X to delete a session.",
            "--bind",
            "ctrl-x:execute-silent(sessionizer sessions remove {+})+reload(sessionizer sessions list)"
        ])
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .wrap_err("Failed to spawn fzf")?;

    let mut stdin = fzf.stdin.take().ok_or_eyre("fail to take stdin")?;
    tokio::spawn(async move {
        stdin.write_all(sessions.join("\n").as_bytes()).await.expect("fail to write to stdin");
        drop(stdin);
    });

    // wait for the process to complete
    let fzf = fzf.wait_with_output().await?;

    // Bail if the status of fzf was an error
    if !fzf.status.success() {
        Err(eyre!("fzf error"))
    } else {
        Ok(String::from_utf8(fzf.stdout)?)
    }
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
