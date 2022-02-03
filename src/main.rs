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

use colored::*;
use rayon::prelude::*;

mod die;
use die::die;

mod runit;
//use runit::{RunitService, RunitServiceState};

mod utils;

mod service;
use service::Service;

const SERVICE_DIR: &str = "/var/service";
const COLORIZE: bool = false;

static IS_VERBOSE: bool = true;
static PSTREE: bool = true;

macro_rules! verbose {
    ($fmt:expr $(, $args:expr )* $(,)? ) => {
        if IS_VERBOSE {
            let s = format!($fmt $(, $args)*);
            eprintln!("{}  {}", ">", s.dimmed());
        }
    };
}

fn print_service(service: &Service) {
    println!("{}", service);
}

fn main() {
    colored::control::set_override(COLORIZE);
    colored::control::unset_override();

    let svdir = match env::var_os("SVDIR") {
        Some(dir) => dir,
        None => OsString::from(SERVICE_DIR),
    };
    let svdir = path::Path::new(&svdir);

    // find all services
    let services = match runit::get_services(svdir) {
        Ok(svcs) => svcs,
        Err(err) => die!(1, "failed to list services: {}", err),
    };

    // process each service found (just gather data here, can be done in parallel)
    let objects: Vec<Service> = services
        .par_iter()
        .filter_map(|service| {
            match Service::from_runit_service(service, PSTREE) {
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
    verbose!("found {} services in {:?}", objects.len(), svdir);
    println!("  {:1} {:15} {:10} {:10} {:10} {:15} {:10}",
        "",
        "SERVICE".bold(),
        "STATE".bold(),
        "ENABLED".bold(),
        "PID".bold(),
        "COMMAND".bold(),
        "TIME".bold());

    // print each service found
    for object in objects {
        print_service(&object);
    }

    println!();
}
