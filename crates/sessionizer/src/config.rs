use clap::{Parser, Subcommand};
use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub directories: Vec<crate::directories::Directory>,
    pub sessions: Vec<String>,
    pub env: Vec<String>,
}

impl Config {
    pub fn new() -> Self {
        Self { directories: vec![], sessions: vec![], env: vec![] }
    }

    pub fn save(config: &Config) -> Result<()> {
        let path = Self::home()?;

        let text = serde_yaml::to_string(&config).wrap_err("fail to serialize config")?;

        std::fs::write(path, text).wrap_err("fail to save config")
    }

    pub fn load() -> Result<Config> {
        let path = Self::home()?;

        // Read path and store it on a variable called yaml
        let yaml = std::fs::read_to_string(path)?;
        serde_yaml::from_str(&yaml).wrap_err("fail to deserialize config")
    }

    pub fn home() -> Result<String> {
        Ok(format!("{}/.sessionizer.bak.yaml", std::env::var("HOME")?))
    }
}

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
    let path = Config::home()?;

    if std::path::Path::new(&path).exists() && !force {
        return Err(eyre!("Configuration file already exists. Use --force to override."));
    }

    let config = Config::new();

    Config::save(&config)?;

    println!("Configuration file created.");

    Ok(())
}

pub async fn edit() -> Result<()> {
    let editor = std::env::var("EDITOR")?;
    let path = Config::home()?;

    // Execute the command `editor path`
    match tokio::process::Command::new(&editor)
        .arg(path)
        .spawn()
        .wrap_err("fail to open editor")?
        .wait()
        .await
    {
        Ok(status) => {
            if !status.success() {
                Err(eyre!("Editor exited with status: {}", status))
            } else {
                println!("Configuration file edited.");
                Ok(())
            }
        }
        Err(err) => Err(eyre!("fail to wait for editor: {}", err)),
    }
}
