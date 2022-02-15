/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

//! A rust port of `vsv`
//!
//! Original: <https://github.com/bahamas10/vsv>

use anyhow::Result;
use yansi::{Color, Paint};

mod arguments;
mod commands;
mod config;
mod die;
mod runit;
mod service;
mod utils;

use arguments::Commands;
use config::Config;
use die::die;

/// Main wrapped to return a result.
fn do_main() -> Result<()> {
    // disable color until we absolutely know we want it
    Paint::disable();

    // parse CLI options + env vars
    let args = arguments::parse();
    let cfg = Config::from_args(&args)?;

    // toggle color if the user wants it or the env dictates
    if cfg.colorize {
        Paint::enable();
    }

    // figure out subcommand to run
    match &args.command {
        None | Some(Commands::Status { .. }) => {
            commands::status::do_status(&cfg)
        }
        Some(Commands::Enable { services }) => {
            commands::enable_disable::do_enable(&cfg, services)
        }
        Some(Commands::Disable { services }) => {
            commands::enable_disable::do_disable(&cfg, services)
        }
        Some(Commands::External(args)) => {
            commands::external::do_external(&cfg, args)
        }
    }
}

fn main() {
    let ret = do_main();

    if let Err(err) = ret {
        die!(1, "{}: {:?}", Color::Red.paint("error"), err);
    }
}
