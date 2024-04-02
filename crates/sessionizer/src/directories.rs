use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Directory {
    pub id: String,
    pub path: String,
    pub mindepth: usize,
    pub maxdepth: usize,
    pub grep: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add a new directory to be tracked by sessionizer
    #[clap(name = "add")]
    Add {
        /// Directory to add
        path: String,
        /// Minimum directory depth to scan from the given directory.
        #[clap(short, long, default_value = "1")]
        mindepth: Option<usize>,
        /// Maximum directory depth to scan from the given directory.
        #[clap(short = 'M', long, default_value = "1")]
        maxdepth: Option<usize>,
        /// Grep a specific type of directory
        #[clap(short, long, default_value = ".*")]
        grep: Option<String>,
    },
    /// Remove a directory to be tracked by sessionizer.
    #[clap(name = "remove")]
    Remove {
        /// Directory unique identifier to add
        id: String,
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

pub async fn run(config: Config, cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add { path, mindepth, maxdepth, grep } => {
            add(config, path, mindepth, maxdepth, grep).await
        }
        Commands::Remove { id } => remove(config, id).await,
        Commands::List => list(config).await,
        Commands::Evaluate => evaluate_cmd(config).await,
    }
}

pub async fn add(
    mut config: Config,
    path: String,
    mindepth: Option<usize>,
    maxdepth: Option<usize>,
    grep: Option<String>,
) -> Result<()> {
    let directory = Directory {
        id: uuid::Uuid::new_v4().to_string(),
        path,
        mindepth: mindepth.unwrap_or(1),
        maxdepth: maxdepth.unwrap_or(1),
        grep,
    };

    config.directories.push(directory);

    config.save()?;

    Ok(())
}

pub async fn remove(mut config: Config, id: String) -> Result<()> {
    config.directories.retain(|d| d.id != id);

    config.save()?;

    Ok(())
}

pub async fn list(config: Config) -> Result<()> {
    println!("{:#?}", config.directories);

    Ok(())
}

pub fn evaluate(config: &Config) -> Result<Vec<String>> {
    let mut directories = Vec::new();

    for directory in config.directories.iter() {
        // Create a regular expression from `directory.grep`.
        let grep = Regex::new(directory.grep.as_ref().unwrap())?;

        log::debug!("Scanning directory: {:?}", directory);
        for entry in WalkDir::new(&directory.path)
            .min_depth(directory.mindepth)
            .max_depth(directory.maxdepth)
            .into_iter()
            .filter_map(|e| {
                // Values myst be `ok` and match `grep` regex
                e.ok().filter(|e| grep.is_match(&e.path().to_string_lossy()))
            })
        {
            if entry.path().is_dir() {
                directories.push(entry.path().to_string_lossy().to_string());
            }
        }
    }

    let mut directories = directories
        .into_iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    directories.sort();

    Ok(directories)
}

pub async fn evaluate_cmd(config: Config) -> Result<()> {
    println!("{}", evaluate(&config)?.join("\n"));

    Ok(())
}
