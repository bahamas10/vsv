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

fn do_status() -> Result<()> {
    // get SVDIR from env or use default
    let svdir = env::var_os("SVDIR")
        .unwrap_or_else(|| OsString::from(SERVICE_DIR) );
    let svdir = path::Path::new(&svdir);

    // check env
    let want_pstree = env::var_os("PSTREE").is_some();
    let want_verbose = env::var_os("VERBOSE").is_some();

    // find all services
    let services = runit::get_services(svdir)
        .with_context(|| format!("failed to list services in {:?}", svdir))?;

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
    if want_verbose {
        println!(">  {}", Style::default().dimmed().paint(
                format!("found {} services in {:?}", services.len(), svdir)));
    }
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
        if want_verbose {
            for message in service.messages {
                eprintln!(">  {}", Style::default().dimmed().paint(message));
            }
        }
    }

    println!();

    Ok(())
}

fn main() {
    let want_color = None;

    let colorize = if let Some(want_color) = want_color {
        // -c takes precedence
        want_color
    } else {
        utils::should_colorize_output()
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
