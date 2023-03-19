use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct FreqDirsConfig {
    #[clap(short, long)]
    pub state_dir: PathBuf,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// List saved directories
    Query {
        /// Show only entries that are subpaths of the provided path (optional)
        #[clap(short, long)]
        path: Option<String>,

        /// Working directory. If provided, child paths will be printed in relative format
        #[clap(long)]
        working_dir: Option<String>,
    },

    /// Save directory
    Save { path: String },

    /// Increment count for a directory, but only if it has been explicitly saved previously
    Update { path: String },
}

pub fn parse_args() -> FreqDirsConfig {
    FreqDirsConfig::parse()
}
