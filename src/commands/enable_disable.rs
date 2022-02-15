/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 15, 2022
 * License: MIT
 */

//! `vsv enable` and `vsv disable`.

use anyhow::{anyhow, Result};

use crate::config::Config;
use crate::runit::RunitService;

enum Mode {
    Enable,
    Disable,
}

/// Handle `vsv enable`.
pub fn do_enable(cfg: &Config, svcs: &[String]) -> Result<()> {
    _do_enable_disable(cfg, svcs, Mode::Enable)
}

/// Handle `vsv enable`.
pub fn do_disable(cfg: &Config, svcs: &[String]) -> Result<()> {
    _do_enable_disable(cfg, svcs, Mode::Disable)
}

/// Handle `vsv enable` and `vsv disable`.
fn _do_enable_disable(cfg: &Config, args: &[String], mode: Mode) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!("at least one (1) service required"));
    }

    let cfg = dbg!(cfg);
    let args = dbg!(args);

    let mut error = anyhow!("failed to modify service(s)");
    let mut had_error = false;

    for name in args {
        let p = cfg.svdir.join(name);
        let svc = RunitService::new(name, &p);
        println!("service = {:?}", svc);

        let ret = match mode {
            Mode::Enable => svc.enable(),
            Mode::Disable => svc.disable(),
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
