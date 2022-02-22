/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 15, 2022
 * License: MIT
 */

//! `vsv enable` and `vsv disable`.

use anyhow::{anyhow, Result};

use crate::config;
use crate::config::Config;
use crate::runit::RunitService;

/// Handle `vsv enable`.
pub fn do_enable(cfg: &Config) -> Result<()> {
    _do_enable_disable(cfg)
}

/// Handle `vsv enable`.
pub fn do_disable(cfg: &Config) -> Result<()> {
    _do_enable_disable(cfg)
}

/// Handle `vsv enable` and `vsv disable`.
fn _do_enable_disable(cfg: &Config) -> Result<()> {
    if cfg.operands.is_empty() {
        return Err(anyhow!("at least one (1) service required"));
    }

    let mut had_error = false;

    for name in &cfg.operands {
        let p = cfg.svdir.join(name);
        let svc = RunitService::new(name, &p);
        print!("{} service {}... ", cfg.mode, name);

        if !svc.valid() {
            println!("failed! service not valid");
            had_error = true;
            continue;
        }

        let ret = match cfg.mode {
            config::ProgramMode::Enable => svc.enable(),
            config::ProgramMode::Disable => svc.disable(),
            _ => unreachable!(),
        };

        match ret {
            Err(err) => {
                had_error = true;
                println!("failed! {}", err);
            }
            Ok(()) => println!("done."),
        };
    }

    if had_error {
        Err(anyhow!("failed to modify service(s)."))
    } else {
        Ok(())
    }
}
