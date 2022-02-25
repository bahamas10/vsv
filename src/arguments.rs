/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

//! Argument parsing logic (via `clap`) for vsv.

use std::path;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about, verbatim_doc_comment, long_about = None)]
///  __   _______   __
///  \ \ / / __\ \ / /   Void Service Manager
///   \ V /\__ \\ V /    Source: https://github.com/bahamas10/vsv
///    \_/ |___/ \_/     MIT License
///    -------------
///     Manage and view runit services
///     Made specifically for Void Linux but should work anywhere
///     Author: Dave Eddy <dave@daveeddy.com> (bahamas10)
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
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show process status
    Status {
        /// Show associated log processes
        #[clap(short, long)]
        log: bool,

        /// Tree view (calls pstree(1) on PIDs found)
        #[clap(short, long)]
        tree: bool,

        filter: Vec<String>,
    },

    /// Enable service(s).
    Enable { services: Vec<String> },

    /// Disable service(s).
    Disable { services: Vec<String> },

    #[clap(external_subcommand)]
    External(Vec<String>),
}

pub fn parse() -> Args {
    Args::parse()
}
