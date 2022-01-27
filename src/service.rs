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

pub enum ServiceState {
    Run,
    Down,
    Finish,
    Unknown
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Service {
    pub path: path::PathBuf,
}

impl Service {
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
