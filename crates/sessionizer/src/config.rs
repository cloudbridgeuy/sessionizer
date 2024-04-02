use clap::{Parser, Subcommand};
use color_eyre::eyre::{format_err, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub directories: Vec<String>,
    pub sessions: crate::sessions::Sessions,
    pub env: Vec<String>,
}

impl Config {
    pub fn save(config: &Config) -> Result<()> {
        let path = Self::home()?;

        let text = match serde_yaml::to_string(&config) {
            Ok(text) => Ok(text),
            Err(err) => Err(format_err!(err)),
        }?;

        match std::fs::write(path, text) {
            Ok(_) => Ok(()),
            Err(err) => Err(format_err!(err)),
        }
    }

    pub fn load() -> Result<Config> {
        let path = Self::home()?;

        // Read path and store it on a variable called yaml
        let yaml = std::fs::read_to_string(path)?;
        match serde_yaml::from_str(&yaml) {
            Ok(config) => Ok(config),
            Err(err) => Err(format_err!(err)),
        }
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
        return Err(format_err!("Configuration file already exists. Use --force to override."));
    }

    let config = Config {
        directories: vec![],
        sessions: crate::sessions::Sessions { current: String::new(), history: vec![] },
        env: vec![],
    };

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
        .expect(format!("Expect the {} to start", &editor).as_str())
        .wait()
        .await
    {
        Ok(_) => {
            println!("Configuration file edited.");
            Ok(())
        }
        Err(err) => Err(format_err!(err)),
    }
}
