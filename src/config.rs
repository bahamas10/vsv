use std::path;
use std::env;
use std::ffi::OsString;

use anyhow::{anyhow, Result};
use lazy_static::lazy_static;

use crate::arguments::{Args, Commands};
use crate::config;
use crate::utils;

// default values
pub const DEFAULT_SVDIR: &str = "/var/service";
pub const DEFAULT_PROC_DIR: &str = "/proc";
pub const DEFAULT_SV_PROG: &str = "sv";
pub const DEFAULT_PSTREE_PROG: &str = "pstree";

// env variables used by this program
pub const ENV_NO_COLOR: &str = "NO_COLOR";
pub const ENV_SVDIR: &str = "SVDIR";
pub const ENV_PROC_DIR: &str = "PROC_DIR";
pub const ENV_SV_PROG: &str = "SV_PROG";
pub const ENV_PSTREE_PROG: &str = "PSTREE_PROG";

lazy_static! {
    pub static ref PROC_PATH: path::PathBuf = {
        let d = env::var_os(config::ENV_PROC_DIR)
            .unwrap_or_else(|| OsString::from(DEFAULT_PROC_DIR));

        path::PathBuf::from(&d)
    };

    pub static ref SV_PROG: String = {
        env::var(config::ENV_SV_PROG)
            .unwrap_or_else(|_| DEFAULT_SV_PROG.to_string())
    };

    pub static ref PSTREE_PROG: String = {
        env::var(config::ENV_PSTREE_PROG)
            .unwrap_or_else(|_| DEFAULT_PSTREE_PROG.to_string())
    };
}

#[derive(Debug)]
pub struct Config {
    pub colorize: bool,
    pub svdir: path::PathBuf,
    pub tree: bool,
    pub log: bool,
    pub verbose: usize,
    filter: Option<String>,
}

impl Config {
    pub fn from_args(args: &Args) -> Result<Self> {
        let mut tree = args.tree;
        let mut log = args.log;
        let mut filter = None;
        let verbose = args.verbose;
        let colorize = should_colorize_output(&args.color)?;
        let svdir = get_svdir(&args.dir);

        if let Some(Commands::Status {
            tree: _tree,
            log: _log,
            filter: _filter,
        }) = &args.command {
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

        let o = Self {
            colorize,
            svdir,
            tree,
            log,
            verbose,
            filter,
        };

        Ok(o)
    }

    pub fn get_filter(&self) -> Option<String> {
        self.filter.as_ref().map(|filter| filter.to_owned())
    }
}

/*
 * Coloring output goes in order from highest priority to lowest priority - highest priority
 * (first in this list) wins:
 *
 * 1. CLI option (`-c`) given
 * 2. env NO_COLOR given
 * 3. stdout is a tty
 */
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

/* Check svdir in this order:
 *
 * 1. CLI option (`-d`) given
 * 2. env SVDIR given
 * 3. use DEFAULT_SVDIR
 */
fn get_svdir(dir_arg: &Option<path::PathBuf>) -> path::PathBuf {
    match dir_arg {
        Some(dir) => dir.to_path_buf(),
        None => {
            let svdir = env::var_os(config::ENV_SVDIR)
                .unwrap_or_else(|| OsString::from(config::DEFAULT_SVDIR) );
            path::Path::new(&svdir).to_path_buf()
        }
    }
}
