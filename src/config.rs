/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

//! Config context variable and various constants for vsv.
//!
//! The main idea here is that after CLI arguments are parsed by `clap` the args
//! object will be given to the config constructor via `::from_args(&args)` and
//! from that + ENV variables a config object will be created.

use std::env;
use std::ffi::OsString;
use std::path;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;

use crate::arguments::{Args, Commands};
use crate::config;
use crate::utils;

// default values
/// Default Runit service dir.
pub const DEFAULT_SVDIR: &str = "/var/service";

/// Default Linux proc fs dir.
pub const DEFAULT_PROC_DIR: &str = "/proc";

/// Default `sv` command name.
pub const DEFAULT_SV_PROG: &str = "sv";

/// Default `pstree` command name.
pub const DEFAULT_PSTREE_PROG: &str = "pstree";

/// Default Runit user dir (relative to user's home directory).
pub const DEFAULT_USER_DIR: &str = "runit/service";

// env var name
/// Env var name for `NO_COLOR`
pub const ENV_NO_COLOR: &str = "NO_COLOR";

/// Env var name for `SVDIR`
pub const ENV_SVDIR: &str = "SVDIR";

/// Env var name for `PROC_DIR`
pub const ENV_PROC_DIR: &str = "PROC_DIR";

/// Env var name for `SV_PROG`
pub const ENV_SV_PROG: &str = "SV_PROG";

/// Env var name for `PSTREE_PROG`
pub const ENV_PSTREE_PROG: &str = "PSTREE_PROG";

lazy_static! {
    pub static ref SV_PROG: String = {
        env::var(config::ENV_SV_PROG)
            .unwrap_or_else(|_| DEFAULT_SV_PROG.to_string())
    };
    pub static ref PSTREE_PROG: String = {
        env::var(config::ENV_PSTREE_PROG)
            .unwrap_or_else(|_| DEFAULT_PSTREE_PROG.to_string())
    };
}

/// Configuration options derived from the environment and CLI arguments.
///
/// This struct holds all configuration data for the invocation of `vsv` derived
/// from both env variables and CLI arguments.  This object can be passed around
/// and thought of as a "context" variable.
#[derive(Debug)]
pub struct Config {
    // env vars only
    pub proc_path: path::PathBuf,

    // env vars or CLI options
    pub colorize: bool,
    pub svdir: path::PathBuf,

    // CLI options only
    pub tree: bool,
    pub log: bool,
    pub verbose: usize,
    filter: Option<String>,
}

impl Config {
    /// Create a `Config` struct from a clap `Args` struct.
    pub fn from_args(args: &Args) -> Result<Self> {
        let mut tree = args.tree;
        let mut log = args.log;
        let mut filter = None;

        let proc_path: path::PathBuf = env::var_os(config::ENV_PROC_DIR)
            .unwrap_or_else(|| OsString::from(DEFAULT_PROC_DIR))
            .into();

        let colorize = should_colorize_output(&args.color)?;
        let svdir = get_svdir(&args.dir, args.user)?;
        let verbose = args.verbose;

        if let Some(Commands::Status {
            tree: _tree,
            log: _log,
            filter: _filter,
        }) = &args.command
        {
            if *_tree {
                tree = true;
            }
            if *_log {
                log = true;
            }
            if !_filter.is_empty() {
                filter = Some(_filter[0].to_owned());
            }
        };

        let o = Self { proc_path, colorize, svdir, tree, log, verbose, filter };

        Ok(o)
    }

    /// Get a copy of the optional `filter` option.
    pub fn get_filter(&self) -> Option<String> {
        self.filter.as_ref().map(|filter| filter.to_owned())
    }
}

/// Check if the output should be colorized.
///
/// Coloring output goes in order from highest priority to lowest priority
/// -highest priority (first in this list) wins:
///
/// 1. CLI option (`-c`) given.
/// 2. env `NO_COLOR` given.
/// 3. stdout is a tty.
fn should_colorize_output(color_arg: &Option<String>) -> Result<bool> {
    // check CLI option first
    if let Some(s) = color_arg {
        match s.as_str() {
            "yes" | "on" => return Ok(true),
            "no" | "off" => return Ok(false),
            "auto" => (), // fall through
            _ => return Err(anyhow!("unknown color option: '{}'", s)),
        }
    }

    // check env var next
    if env::var_os(config::ENV_NO_COLOR).is_some() {
        return Ok(false);
    }

    // lastly check if stdout is a tty
    let isatty = utils::isatty(1);

    Ok(isatty)
}

/// Determine the `SVDIR` the user wants.
///
/// Check svdir in this order:
///
/// 1. CLI option (`-d`) given
/// 2. CLI option (`-u`) given
/// 3. env `SVDIR` given
/// 4. use `DEFAULT_SVDIR` (`"/var/service"`)
fn get_svdir(
    dir_arg: &Option<path::PathBuf>,
    user_arg: bool,
) -> Result<path::PathBuf> {
    // `-d <dir>`
    if let Some(dir) = dir_arg {
        return Ok(dir.to_path_buf());
    }

    // `-u`
    if user_arg {
        let home_dir = dirs::home_dir().ok_or_else(|| {
            anyhow!("failed to determine users home directory")
        })?;
        let buf = home_dir.join(DEFAULT_USER_DIR);
        return Ok(buf);
    }

    // env or default
    let svdir = env::var_os(config::ENV_SVDIR)
        .unwrap_or_else(|| OsString::from(config::DEFAULT_SVDIR));
    let buf = path::PathBuf::from(&svdir);
    Ok(buf)
}
