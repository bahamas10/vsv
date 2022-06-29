/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

//! `vsv <anything>`.

use std::env;

use anyhow::{bail, ensure, Context, Result};
use clap::crate_name;
use yansi::Color;

use crate::utils;
use crate::{config, config::Config};

/// Handle `vsv <any-non-matching-command>`.
pub fn do_external(cfg: &Config) -> Result<()> {
    assert!(!cfg.operands.is_empty());

    let sv = cfg.sv_prog.to_owned();

    ensure!(
        cfg.operands.len() >= 2,
        "argument expected for '{} {}'",
        sv,
        cfg.operands[0]
    );

    // format arguments
    let args_s = cfg.operands.join(" ");

    // set SVDIR env to match what user wanted
    env::set_var(config::ENV_SVDIR, &cfg.svdir);

    println!(
        "[{}] {}",
        crate_name!(),
        Color::Cyan.paint(format!(
            "Running {} command ({}={:?} {} {})",
            sv,
            config::ENV_SVDIR,
            &cfg.svdir,
            sv,
            &args_s
        ))
    );

    // run the actual program
    let status = utils::run_program_get_status(&sv, &cfg.operands)
        .with_context(|| format!("failed to execute {}", sv))?;

    // check the process status
    let code = status.code().unwrap_or(-1);
    let color = match code {
        0 => Color::Green,
        _ => Color::Red,
    };

    // print exit code
    println!(
        "[{}] {}",
        crate_name!(),
        color.paint(format!("[{} {}] exit code {}", sv, &args_s, code))
    );

    match code {
        0 => Ok(()),
        _ => bail!("call to {} failed", sv),
    }
}
