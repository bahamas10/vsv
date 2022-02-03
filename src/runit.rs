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

use anyhow::Result;

pub enum RunitServiceState {
    Run,
    Down,
    Finish,
    Unknown
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RunitService {
    pub path: path::PathBuf,
}

impl RunitService {
    pub fn new(path: path::PathBuf) -> Self {
        Self {
            path
        }
    }

    pub fn enabled(&self) -> bool {
        let p = self.path.join("down");

        ! p.exists()
    }

    pub fn get_pid(&self) -> Result<pid_t> {
        let p = self.path.join("supervise").join("pid");

        let pid: pid_t = fs::read_to_string(p)?.trim().parse()?;

        Ok(pid)
    }

    pub fn get_state(&self) -> RunitServiceState {
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
        let p = self.path.join("supervise").join("stat");

        Ok(fs::metadata(p)?.modified()?)
    }
}

pub fn get_services(path: &path::Path) -> Result<Vec<RunitService>> {
    // loop services directory and collect service names
    let mut dirs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();

        if ! p.is_dir() {
            continue;
        }

        let service = RunitService::new(p);

        dirs.push(service);
    }

    dirs.sort();

    Ok(dirs)
}
