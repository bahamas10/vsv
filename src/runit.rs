/*
 * Runit service related structs and enums.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 26, 2022
 * License: MIT
 */

use libc::pid_t;
use std::fs;
use std::path;
use std::time;

use anyhow::{anyhow, Result};

pub enum RunitServiceState {
    Run,
    Down,
    Finish,
    Unknown
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RunitService {
    pub path: path::PathBuf,
    pub name: String,
}

impl RunitService {
    pub fn new(path: path::PathBuf, name: &str) -> Self {
        let name = name.to_string();
        Self {
            path,
            name,
        }
    }

    pub fn enabled(&self) -> bool {
        // "/<svdir>/<service>/down"
        let p = self.path.join("down");

        ! p.exists()
    }

    pub fn get_pid(&self) -> Result<pid_t> {
        // "/<svdir>/<service>/supervise/pid"
        let p = self.path.join("supervise").join("pid");

        let pid: pid_t = fs::read_to_string(p)?.trim().parse()?;

        Ok(pid)
    }

    pub fn get_state(&self) -> RunitServiceState {
        // "/<svdir>/<service>/supervise/stat"
        let p = self.path.join("supervise").join("stat");

        let s = fs::read_to_string(p).unwrap_or_else(|_| String::from("unknown"));

        match s.trim() {
            "run" => RunitServiceState::Run,
            "down" => RunitServiceState::Down,
            "finish" => RunitServiceState::Finish,
            _ => RunitServiceState::Unknown,
        }
    }

    pub fn get_start_time(&self) -> Result<time::SystemTime> {
        // "/<svdir>/<service>/supervise/stat"
        let p = self.path.join("supervise").join("stat");

        Ok(fs::metadata(p)?.modified()?)
    }
}

pub fn get_services(path: &path::Path, log: bool, filter: Option<String>) -> Result<Vec<RunitService>> {
    // loop services directory and collect service names
    let mut dirs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();

        if ! p.is_dir() {
            continue;
        }

        let name = p
            .file_name()
            .ok_or_else(|| anyhow!("{:?}: failed to get name from service", p))?
            .to_str()
            .ok_or_else(|| anyhow!("{:?}: failed to parse name from service", p))?
            .to_string();

        if let Some(ref filter) = filter {
            if !name.contains(filter) {
                continue;
            }
        }

        let service = RunitService::new(p, &name);
        dirs.push(service);

        if log {
            let p = entry.path().join("log");
            let name = "- log";
            let service = RunitService::new(p, name);
            dirs.push(service);
        }
    }

    dirs.sort();

    Ok(dirs)
}
