use clap::{Parser, Subcommand};
use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub directories: Vec<crate::directories::Directory>,
    pub sessions: Vec<String>,
    pub env: Vec<String>,
    #[serde(skip)]
    path: String,
}

impl Config {
    pub fn new(path: &str) -> Self {
        Self { directories: vec![], sessions: vec![], env: vec![], path: path.to_string() }
    }

    pub fn save(&self) -> Result<()> {
        let text = serde_yaml::to_string(&self).wrap_err("fail to serialize config")?;

        std::fs::write(&self.path, text).wrap_err("fail to save config")?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self> {
        if !std::path::Path::new(path).exists() {
            return Err(eyre!("Configuration file does not exist."));
        }
        let yaml = std::fs::read_to_string(path)?;
        let mut config: Self =
            serde_yaml::from_str(&yaml).wrap_err("fail to deserialize config")?;
        log::debug!("config = {:#?}", config);
        config.path = path.to_string();
        Ok(config)
    }

    pub fn home() -> Result<String> {
        Ok(format!("{}/.sessionizer.yaml", std::env::var("HOME")?))
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
    /// Prints the sessionizer configuration to stdout
    #[clap(name = "print")]
    Print,
}

#[derive(Debug, Parser)]
#[command(name = "config")]
#[command(about = "Manage the sessionizer configuration")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub async fn run(path: &str, cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Init { force } => init(path, force).await,
        Commands::Edit => edit().await,
        Commands::Print => print(path).await,
    }
}

pub async fn print(path: &str) -> Result<()> {
    println!("{:#?}", Config::load(path));

    Ok(())
}

pub async fn init(path: &str, force: bool) -> Result<()> {
    if std::path::Path::new(&path).exists() && !force {
        return Err(eyre!("Configuration file already exists. Use --force to override."));
    }

    let config = Config::new(path);

    config.save()?;

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
