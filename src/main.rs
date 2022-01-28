/*
 * A rust replacement for vsv
 *
 * Original: https://github.com/bahamas10/vsv
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 25, 2022
 * License: MIT
 */

use libc::pid_t;
use std::env;
use std::fmt;
use std::fs;
use std::path;
use std::time;
use std::ffi::OsString;
use std::process::Command;

use anyhow::{anyhow, Result, Context};
use colored::*;
use lazy_static::lazy_static;

mod die;
use die::die;

mod service;
use service::{Service, ServiceState};

const SERVICE_DIR: &str = "/var/service";
const PROC_DIR: &str = "/proc";
const COLORIZE: bool = false;

static IS_VERBOSE: bool = true;
static PSTREE: bool = false;

macro_rules! verbose {
    ($fmt:expr $(, $args:expr )* $(,)? ) => {
        if IS_VERBOSE {
            let s = format!($fmt $(, $args)*);
            eprintln!("{}  {}", ">", s.dimmed());
        }
    };
}


/*
 * Make the proc dir var (overrideable via env vars) accessible everywhere after first access.
 */
lazy_static! {
    static ref PROC_PATH: path::PathBuf = {
        let procdir = match env::var_os("PROC_DIR") {
            Some(dir) => dir,
            None => OsString::from(PROC_DIR),
        };
        path::PathBuf::from(&procdir)
    };
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ServiceState::Run => "run".green(),
            ServiceState::Down => "down".red(),
            ServiceState::Finish => "finish".red(),
            ServiceState::Unknown => "n/a".yellow(),
        };

        s.fmt(f)
    }
}

impl ServiceState {
    pub fn get_char(&self) -> String {
        let s = match self {
            ServiceState::Run => "✔".green(),
            ServiceState::Down => "X".red(),
            ServiceState::Finish => "X".red(),
            ServiceState::Unknown => "?".yellow(),
        };

        s.to_string()
    }
}

fn pstree(pid: pid_t) -> Result<String> {
    let output = Command::new("pstree")
        .args(["-ac", &pid.to_string()])
        .output()?;

    if ! output.status.success() {
        return Err(anyhow!("pstree return non-zero"));
    }

    let stdout = String::from_utf8(output.stdout)?;

    Ok(stdout)
}

fn cmd_from_pid(pid: pid_t) -> Result<String> {
    // /proc/<pid>/cmdline
    let p = PROC_PATH.join(pid.to_string()).join("cmdline");

    let data = fs::read_to_string(&p)
        .with_context(|| format!("failed to read pid file: {:?}", p))?;

    let first = data.split('\0').next();

    match first {
        Some(f) => Ok(f.to_string()),
        None => Err(anyhow!("failed to split cmdline data: {:?}", first)),
    }
}

fn relative_duration(t: time::Duration) -> String {
    let secs = t.as_secs();

    let v = vec![
        (secs / 60 / 60 / 24 / 365, "year"),
        (secs / 60 / 60 / 24 / 30 , "month"),
        (secs / 60 / 60 / 24 / 7  , "week"),
        (secs / 60 / 60 / 24      , "day"),
        (secs / 60 / 60           , "hour"),
        (secs / 60                , "minute"),
        (secs                     , "second"),
    ];

    let mut plural = "";
    for (num, name) in v {
        if num > 1 {
            plural = "s"
        }

        if num > 0 {
            return format!("{} {}{}", num, name, plural);
        }
    }

    String::from("0 seconds")
}

fn process_service(service: &Service) -> Result<()> {
    verbose!("processing {:?}", service);

    // extract service name from path (basename)
    let name = match service.path.file_name() {
        Some(name) => name,
        None => return Err(anyhow!("failed to get name from service")),
    };
    let name = match name.to_str() {
        Some(name) => name,
        None => return Err(anyhow!("failed to parse name from service")),
    };

    let enabled = service.enabled();
    let pid = service.get_pid();
    let state = service.get_state();
    let time = service.get_start_time();

    let mut command = String::from("---");
    if let Ok(p) = pid {
        match cmd_from_pid(p) {
            Ok(cmd) => {
                command = cmd;
            }
            Err(err) => {
                verbose!("failed to get command for pid {}: {:?}", p, err);
            }
        };
    }
    let command = command.green();

    let mut time_s = String::from("---");
    if let Ok(t) = time {
        if let Ok(t) = t.elapsed() {
            time_s = relative_duration(t);
        }
    }
    let time_s = time_s.dimmed();

    let enabled_s = match enabled {
        true => "true".green(),
        false => "false".red(),
    };

    let pid_s = match pid {
        Ok(pid) => pid.to_string(),
        Err(ref err) => {
            verbose!("failed to get pid: {}", err);
            String::from("---")
        }
    }.magenta();

    println!("  {:1} {:15} {:10} {:10} {:10} {:15} {:10}",
        state.get_char(),
        name,
        state,
        enabled_s,
        pid_s,
        command,
        time_s);

    if PSTREE {
        if let Ok(pid) = pid {
            match pstree(pid) {
                Ok(stdout) => println!("\n{}\n", stdout.trim().dimmed()),
                Err(err) => eprintln!("\npstree call failed: {}\n", err.to_string().red()),
            }
        }
    }

    Ok(())
}

fn get_services(path: &path::Path) -> Result<Vec<Service>> {
    // loop services directory and collect service names
    let mut dirs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();

        if ! p.is_dir() {
            continue;
        }

        let service = Service::new(p);

        dirs.push(service);
    }

    dirs.sort();

    Ok(dirs)
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
    let services = match get_services(svdir) {
        Ok(svcs) => svcs,
        Err(err) => die!(1, "failed to list services: {}", err),
    };

    println!();
    verbose!("found {} services in {:?}", services.len(), svdir);
    println!("  {:1} {:15} {:10} {:10} {:10} {:15} {:10}",
        "",
        "SERVICE".bold(),
        "STATE".bold(),
        "ENABLED".bold(),
        "PID".bold(),
        "COMMAND".bold(),
        "TIME".bold());

    // process each service found
    for service in services {
        let _ = process_service(&service);
    }

    println!();
}
