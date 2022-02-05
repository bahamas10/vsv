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

use yansi::{Color, Style, Paint};
use anyhow::{Context, Result};
use rayon::prelude::*;

mod die;
mod runit;
mod utils;
mod service;

use die::die;
use service::Service;

const SERVICE_DIR: &str = "/var/service";

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
    let bold_style = Style::default().bold();

    println!();
    println!("found {} services in {:?}", services.len(), svdir); // verbose
    println!("{}", utils::format_status_line(
        ("", &bold_style),
        ("SERVICE", &bold_style),
        ("STATE", &bold_style),
        ("ENABLED", &bold_style),
        ("PID", &bold_style),
        ("COMMAND", &bold_style),
        ("TIME", &bold_style),
    ));

    // print each service found
    for service in services {
        println!("{}", service);
    }

    println!();

    Ok(())
}

fn main() {
    let isatty = utils::isatty(1);
    let no_color_env = env::var_os("NO_COLOR").is_some();
    let want_color = None;

    let colorize = if let Some(want_color) = want_color {
        // -c takes precedence
        want_color
    } else if no_color_env {
        // env variable is next line
        false
    } else {
        // finally, set color to be dependent on stdout being a TTY
        isatty
    };

    if colorize {
        Paint::enable();
    } else {
        Paint::disable();
    }

    // figure out subcommand to run
    let ret = do_status();

    if let Err(err) = ret {
        die!(1, "{}: {:?}",
            Color::Red.paint("error"),
            err);
    }
}
