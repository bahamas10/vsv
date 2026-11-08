/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

//! Argument parsing logic (via `clap`) for vsv.

use std::path;

use clap::{
    builder::styling::{AnsiColor, Styles},
    ArgAction, Parser, Subcommand,
};

fn help_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default())
        .usage(AnsiColor::Yellow.on_default())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Green.on_default())
}

#[derive(Debug, Parser)]
#[clap(author, version, about, verbatim_doc_comment, long_about = None, styles = help_styles())]
#[clap(before_help = "\x1b[35m __   _______   __\x1b[0m
\x1b[35m \\ \\ / / __\\ \\ / /   \x1b[32mVoid Service Manager\x1b[0m
\x1b[35m  \\ V /\\__ \\\\ V /    \x1b[32mSource: https://github.com/bahamas10/vsv\x1b[0m
\x1b[35m   \\_/ |___/ \\_/     \x1b[32mMIT License\x1b[0m
\x1b[35m   -------------\x1b[0m
    Manage and view runit services
    Made specifically for Void Linux but should work anywhere
    Author: Dave Eddy <dave@daveeddy.com> (bahamas10)")]
#[clap(
    after_help = "Any other subcommand gets passed directly to the 'sv' command, see sv(1) for
the full list of subcommands and information about what each does specifically.
Common subcommands:

    start <service>           Start the service
    stop <service>            Stop the service
    restart <service>         Restart the service
    reload <service>          Reload the service (send SIGHUP)
"
)]
pub struct Args {
    /// Enable or disable color output.
    #[clap(short, long, value_name = "yes|no|auto")]
    pub color: Option<String>,

    /// Directory to look into, defaults to env SVDIR or /var/service if unset.
    #[clap(short, long, value_parser, value_name = "dir")]
    pub dir: Option<path::PathBuf>,

    /// Show log processes, this is a shortcut for `status -l`.
    #[clap(short, long)]
    pub log: bool,

    /// Tree view, this is a shortcut for `status -t`.
    #[clap(short, long)]
    pub tree: bool,

    /// User mode, this is a shortcut for `-d ~/runit/service`.
    #[clap(short, long)]
    pub user: bool,

    /// Increase Verbosity.
    #[clap(short, long, action = ArgAction::Count)]
    pub verbose: u8,

    /// Subcommand.
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show process status.
    Status {
        /// Show associated log processes.
        #[clap(short, long)]
        log: bool,

        /// Tree view (calls pstree(1) on PIDs found).
        #[clap(short, long)]
        tree: bool,

        filter: Vec<String>,
    },

    /// Enable service(s).
    Enable { services: Vec<String> },

    /// Disable service(s).
    Disable { services: Vec<String> },

    /// Pass arguments directly to `sv`.
    #[clap(external_subcommand)]
    External(Vec<String>),
}

pub fn parse() -> Args {
    Args::parse()
}
