use std::path;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// kill yourself homo
    #[clap(short, long)]
    pub color: Option<String>,

    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    pub dir: Option<path::PathBuf>,

    #[clap(short, long)]
    pub log: bool,

    #[clap(short, long)]
    pub tree: bool,

    #[clap(short, long)]
    pub user: bool,

    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// does testing things
    Test {
        /// lists test values
        #[clap(short, long)]
        list: bool,
    },
}

pub fn parse() -> Args {
    Args::parse()
}
