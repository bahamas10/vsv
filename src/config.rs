/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

/*!
 * Config context variable and various constants for vsv.
 *
 * The main idea here is that after CLI arguments are parsed by `clap` the args
 * object will be given to the config constructor via `::from_args(&args)` and
 * from that + ENV variables a config object will be created.
 */

use std::env;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};

use crate::arguments::{Args, Commands};
use crate::config;
use crate::utils;

// default values
pub const DEFAULT_SVDIR: &str = "/var/service";
pub const DEFAULT_PROC_DIR: &str = "/proc";
pub const DEFAULT_SV_PROG: &str = "sv";
pub const DEFAULT_PSTREE_PROG: &str = "pstree";
pub const DEFAULT_USER_DIR: &str = "runit/service";

// env var name
pub const ENV_NO_COLOR: &str = "NO_COLOR";
pub const ENV_SVDIR: &str = "SVDIR";
pub const ENV_PROC_DIR: &str = "PROC_DIR";
pub const ENV_SV_PROG: &str = "SV_PROG";
pub const ENV_PSTREE_PROG: &str = "PSTREE_PROG";

/// vsv execution modes (subcommands).
#[derive(Debug)]
pub enum ProgramMode {
    Status,
    Enable,
    Disable,
    External,
}

impl fmt::Display for ProgramMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ProgramMode::Status => "status",
            ProgramMode::Enable => "enable",
            ProgramMode::Disable => "disable",
            ProgramMode::External => "<external>",
        };

        s.fmt(f)
    }
}

/**
 * Configuration options derived from the environment and CLI arguments.
 *
 * This struct holds all configuration data for the invocation of `vsv` derived
 * from both env variables and CLI arguments.  This object can be passed around
 * and thought of as a "context" variable.
 */
#[derive(Debug)]
pub struct Config {
    // env vars only
    pub proc_path: PathBuf,
    pub sv_prog: String,
    pub pstree_prog: String,

    // env vars or CLI options
    pub colorize: bool,
    pub svdir: PathBuf,

    // CLI options only
    pub tree: bool,
    pub log: bool,
    pub verbose: usize,
    pub operands: Vec<String>,
    pub mode: ProgramMode,
}

impl Config {
    /// Create a `Config` struct from a clap `Args` struct.
    pub fn from_args(args: &Args) -> Result<Self> {
        let mut tree = args.tree;
        let mut log = args.log;

        let proc_path: PathBuf = env::var_os(config::ENV_PROC_DIR)
            .unwrap_or_else(|| OsString::from(DEFAULT_PROC_DIR))
            .into();
        let sv_prog = env::var(config::ENV_SV_PROG)
            .unwrap_or_else(|_| DEFAULT_SV_PROG.to_string());
        let pstree_prog = env::var(config::ENV_PSTREE_PROG)
            .unwrap_or_else(|_| DEFAULT_PSTREE_PROG.to_string());

        let colorize = should_colorize_output(&args.color)?;
        let svdir = get_svdir(&args.dir, args.user)?;
        let verbose = args.verbose;

        // let arguments after `vsv status` work as well.
        if let Some(Commands::Status { tree: _tree, log: _log, filter: _ }) =
            &args.command
        {
            if *_tree {
                tree = true;
            }
            if *_log {
                log = true;
            }
        };

        // figure out subcommand to run
        let (mode, operands) = match &args.command {
            // `vsv` (no subcommand)
            None => {
                let v: Vec<String> = vec![];
                (ProgramMode::Status, v)
            }
            // `vsv status`
            Some(Commands::Status { tree: _, log: _, filter: operands }) => {
                (ProgramMode::Status, operands.to_vec())
            }
            // `vsv enable ...`
            Some(Commands::Enable { services }) => {
                (ProgramMode::Enable, services.to_vec())
            }
            // `vsv disable ...`
            Some(Commands::Disable { services }) => {
                (ProgramMode::Disable, services.to_vec())
            }
            // `vsv <anything> ...`
            Some(Commands::External(args)) => {
                // -t or -l will put the program into status mode
                let mode = if tree || log {
                    ProgramMode::Status
                } else {
                    ProgramMode::External
                };
                (mode, args.to_vec())
            }
        };

        let o = Self {
            proc_path,
            sv_prog,
            pstree_prog,
            colorize,
            svdir,
            tree,
            log,
            verbose,
            operands,
            mode,
        };

        Ok(o)
    }
}

/**
 * Check if the output should be colorized.
 *
 * Coloring output goes in order from highest priority to lowest priority
 * -highest priority (first in this list) wins:
 *
 * 1. CLI option (`-c`) given.
 * 2. env `NO_COLOR` given.
 * 3. stdout is a tty.
 */
fn should_colorize_output(color_arg: &Option<String>) -> Result<bool> {
    // check CLI option first
    if let Some(s) = color_arg {
        match s.as_str() {
            "yes" | "on" | "always" => return Ok(true),
            "no" | "off" | "never" => return Ok(false),
            "auto" => (), // fall through
            _ => bail!("unknown color option: '{}'", s),
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

/**
 * Determine the `SVDIR` the user wants.
 *
 * Check svdir in this order:
 *
 * 1. CLI option (`-d`) given
 * 2. CLI option (`-u`) given
 * 3. env `SVDIR` given
 * 4. use `DEFAULT_SVDIR` (`"/var/service"`)
 */
fn get_svdir(dir_arg: &Option<PathBuf>, user_arg: bool) -> Result<PathBuf> {
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
    let buf = PathBuf::from(&svdir);

    Ok(buf)
}
