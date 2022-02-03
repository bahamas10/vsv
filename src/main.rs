/*
 * A rust replacement for vsv
 *
 * Original: https://github.com/bahamas10/vsv
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

use std::env;
use std::path;
use std::ffi::OsString;

use anyhow::{Context, Result};
use colored::*;
use rayon::prelude::*;

mod die;
mod runit;
mod utils;
mod service;

use die::die;
use service::Service;

const SERVICE_DIR: &str = "/var/service";
const COLORIZE: bool = false;

/*
macro_rules! verbose {
    ($fmt:expr $(, $args:expr )* $(,)? ) => {
        if want_verbose {
            let s = format!($fmt $(, $args)*);
            eprintln!("{}  {}", ">", s.dimmed());
        }
    };
}
*/

fn do_status() -> Result<()> {
    // get SVDIR from env or use default
    let svdir = env::var_os("SVDIR")
        .unwrap_or_else(|| OsString::from(SERVICE_DIR) );
    let svdir = path::Path::new(&svdir);

    // check if user wants pstree
    let want_pstree = env::var_os("PSTREE").is_some();

    // find all services
    let services = runit::get_services(svdir).context("failed to list services")?;

    // process each service found (just gather data here, can be done in parallel)
    let services: Vec<Service> = services
        .par_iter()
        .filter_map(|service| {
            match Service::from_runit_service(service, want_pstree) {
                Ok(svc) => Some(svc),
                Err(err) => {
                    eprintln!("failed to process service {:?}: {}", service, err);
                    None
                }
            }
        })
        .collect();

    // print gathared data
    println!();
    println!("found {} services in {:?}", services.len(), svdir); // verbose
    println!("  {:1} {:20} {:7} {:9} {:8} {:17} {}",
        "",
        "SERVICE".bold(),
        "STATE".bold(),
        "ENABLED".bold(),
        "PID".bold(),
        "COMMAND".bold(),
        "TIME".bold());

    // print each service found
    for service in services {
        println!("{}", service);
    }

    println!();

    Ok(())
}

fn main() {
    // color output
    colored::control::set_override(COLORIZE);
    colored::control::unset_override();

    // figure out subcommand to run
    let ret = do_status();

    if let Err(err) = ret {
        die!(1, "{}: {:?}", "error".red(), err);
    }
}
