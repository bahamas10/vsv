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

use lazy_static::lazy_static;
use anyhow::{anyhow, Result};

const SERVICE_DIR: &str = "/var/service";
const PROC_DIR: &str = "/proc";

lazy_static! {
    static ref PROC_PATH: path::PathBuf = {
        let procdir = match env::var_os("PROC_DIR") {
            Some(dir) => dir,
            None => OsString::from(PROC_DIR),
        };
        path::PathBuf::from(&procdir)
    };
}

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
        let p = self.path.join("down");

        p.exists()
    }

    pub fn get_pid(&self) -> Result<pid_t> {
        let p = self.path.join("supervise").join("pid");

        let pid: pid_t = fs::read_to_string(p)?.trim().parse()?;

        Ok(pid)
    }

    pub fn get_state(&self) -> ServiceState {
        let p = self.path.join("supervise").join("stat");

        if let Ok(s) = fs::read_to_string(p) {
            return match s.trim() {
                "run" => ServiceState::Run,
                "down" => ServiceState::Down,
                "finish" => ServiceState::Finish,
                _ => ServiceState::Unknown,
            };
        }

        ServiceState::Unknown
    }

    pub fn get_start_time(&self) -> Result<time::SystemTime> {
        let p = self.path.join("supervise").join("pid");

        Ok(fs::metadata(p)?.modified()?)
    }
}

fn cmd_from_pid(pid: pid_t) -> Result<String> {
    // /proc/<pid>/cmdline
    let p = PROC_PATH.join(pid.to_string()).join("cmdline");

    let data = fs::read_to_string(p)?;

    let first = data.split('\0').next();

    match first {
        Some(f) => Ok(f.to_string()),
        None => Err(anyhow!("failed to split cmdline")),
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
    let time = service.get_start_time();

    let mut command = String::from("---");
    if let Ok(p) = pid {
        if let Ok(cmd) = cmd_from_pid(p) {
            command = cmd;
        }
    }

    let mut time_s = String::from("---");
    if let Ok(t) = time {
        if let Ok(t) = t.elapsed() {
            time_s = t.as_secs().to_string();
        }
    }

    let enabled = match wants_down {
        true => "false",
        false => "true"
    };
    let pid_s = match pid {
        Ok(pid) => pid.to_string(),
        Err(_) => String::from("---")
    };

    println!("  {:1} {:15} {:10} {:10} {:10} {:15} {:10}",
        state.get_char(), name, state, enabled, pid_s, command, time_s);

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
    println!("  {:1} {:15} {:10} {:10} {:10} {:15} {:10}",
        "", "SERVICE", "STATE", "ENABLED", "PID", "COMMAND", "TIME");

    // process each service found
    for service in services {
        let _ = process_service(&service);
    }

    println!();
}
