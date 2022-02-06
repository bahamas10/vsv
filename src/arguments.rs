use std::path;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Enable or disable color output
    #[clap(short, long, value_name = "yes|no|auto")]
    pub color: Option<String>,

    /// Directory to look into, defaults to env SVDIR or /var/service if unset
    #[clap(short, long, parse(from_os_str), value_name = "dir")]
    pub dir: Option<path::PathBuf>,

    /// Show log processes, this is a shortcut for 'status -l'
    #[clap(short, long)]
    pub log: bool,

    /// Tree view, this is a shortcut for 'status -t'
    #[clap(short, long)]
    pub tree: bool,

    /// User mode, this is a shortcut for '-d ~/runit/service'
    #[clap(short, long)]
    pub user: bool,

    /// Increase Verbosity
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show process status
    Status {
        /// Tree view
        #[clap(short, long)]
        tree: bool,
    },
}

pub fn parse() -> Args {
    Args::parse()
}
