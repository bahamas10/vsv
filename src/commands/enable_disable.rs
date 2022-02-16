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

    let mut error = anyhow!("failed to modify service(s)");
    let mut had_error = false;

    for name in &cfg.operands {
        let p = cfg.svdir.join(name);
        let svc = RunitService::new(name, &p);
        println!("service = {:?}", svc);

        let ret = match cfg.mode {
            config::Mode::Enable => svc.enable(),
            config::Mode::Disable => svc.disable(),
            _ => unreachable!(),
        };

        if let Err(err) = ret {
            had_error = true;
            error = error.context(err);
        }
    }

    if had_error {
        Err(error)
    } else {
        Ok(())
    }
}
