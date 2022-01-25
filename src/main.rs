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
use std::fmt;
use std::fs;
use std::path;
use std::ffi::OsString;

use anyhow::{anyhow, Result};

macro_rules! die {
    () => {
        std::process::exit(1);
    };

    ( $code:expr $(,)? ) => {
        std::process::exit($code);
    };

    ( $code:expr, $fmt:expr $( , $args:expr )* $(,)? ) => {{
        eprintln!($fmt $( , $args )*);
        std::process::exit($code);
    }};
}

const DEFAULT_DIR: &str = "/var/service";

enum ServiceState {
    Run,
    Down,
    Finish,
    Unknown
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ServiceState::Run => "run",
            ServiceState::Down => "down",
            ServiceState::Finish => "finish",
            ServiceState::Unknown => "---",
        };
        s.fmt(f)
    }
}

impl ServiceState {
    fn get_char(&self) -> String {
        let s = match self {
            ServiceState::Run => "✔",
            ServiceState::Down => "X",
            ServiceState::Finish => "X",
            ServiceState::Unknown => "?",
        };

        String::from(s)
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Service {
    path: path::PathBuf,
}

impl Service {
    pub fn new(path: path::PathBuf) -> Self {
        Self {
            path
        }
    }

    pub fn wants_down(&self) -> bool {
        let path = self.path.join("down");

        path.exists()
    }

    pub fn get_pid(&self) -> Option<u32> {
        let path = self.path.join("supervise").join("pid");

        if let Ok(data) = fs::read_to_string(path) {
            if let Ok(pid) = data.trim().parse::<u32>() {
                return Some(pid);
            }
        }

        None
    }

    pub fn get_state(&self) -> ServiceState {
        let path = self.path.join("supervise").join("stat");

        if let Ok(s) = fs::read_to_string(path) {
            return match s.trim() {
                "run" => ServiceState::Run,
                "down" => ServiceState::Down,
                "finish" => ServiceState::Finish,
                _ => ServiceState::Unknown,
            };
        }

        ServiceState::Unknown
    }
}

fn process_service(service: &Service) -> Result<()> {
    // extract service name from path (basename)
    let name = match service.path.file_name() {
        Some(name) => name,
        None => return Err(anyhow!("failed to get name from service")),
    };
    let name = match name.to_str() {
        Some(name) => name,
        None => return Err(anyhow!("failed to parse name from service")),
    };

    let wants_down = service.wants_down();
    let pid = service.get_pid();
    let state = service.get_state();

    let down = match wants_down {
        true => "down",
        false => "---"
    };
    let pid_s = match pid {
        Some(pid) => pid.to_string(),
        None => String::from("---")
    };

    println!("  {:1} {:10} {:10} {:10} {:10}",
        state.get_char(), name, state, pid_s, down);

    Ok(())
}

fn get_services() -> Result<Vec<Service>> {
    // loop services directory and collect service names
    let mut dirs = Vec::new();

    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();

        if ! path.is_dir() {
            continue;
        }

        let service = Service::new(path);

        dirs.push(service);
    }

    dirs.sort();

    Ok(dirs)
}

fn main() {
    // cd into SVDIR or the default dir
    let svdir = match env::var_os("SVDIR") {
        Some(dir) => dir,
        None => OsString::from(DEFAULT_DIR),
    };
    let svdir = path::Path::new(&svdir);
    if let Err(err) = env::set_current_dir(&svdir) {
        die!(1, "failed to chdir to SVDIR {:?}: {}", svdir, err);
    }

    // find all services
    let services = match get_services() {
        Ok(svcs) => svcs,
        Err(err) => die!(1, "failed to list services: {}", err),
    };

    println!("  {:1} {:10} {:10} {:10} {:10}",
        "", "NAME", "STATE", "PID", "DOWN");

    // process each service found
    for service in services {
        let _ = process_service(&service);
    }
}
