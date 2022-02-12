/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 26, 2022
 * License: MIT
 */

//! Runit service related structs and enums.

use libc::pid_t;
use std::fs;
use std::path;
use std::time;
use path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// Possible states for a runit service.
pub enum RunitServiceState {
    Run,
    Down,
    Finish,
    Unknown,
}

/// A runit service.
///
/// This struct doesn't create a new service for runit, does it actually every
/// modify anything on the user's underlying filesystem.  Instead, this struct
/// allows for read-only access into the state of an existing runit service.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RunitService {
    pub path: PathBuf,
    pub name: String,
}

impl RunitService {
    /// Create a new runit service object from a given path and name.
    pub fn new(path: &Path, name: &str) -> Self {
        let path = path.to_path_buf();
        let name = name.to_string();
        Self { path, name }
    }

    /// Check if a service is enabled.
    pub fn enabled(&self) -> bool {
        // "/<svdir>/<service>/down"
        let p = self.path.join("down");

        !p.exists()
    }

    /// Get the service PID if possible.
    pub fn get_pid(&self) -> Result<pid_t> {
        // "/<svdir>/<service>/supervise/pid"
        let p = self.path.join("supervise").join("pid");

        let pid: pid_t = fs::read_to_string(p)?.trim().parse()?;

        Ok(pid)
    }

    /// Get the service state.
    pub fn get_state(&self) -> RunitServiceState {
        // "/<svdir>/<service>/supervise/stat"
        let p = self.path.join("supervise").join("stat");

        let s =
            fs::read_to_string(p).unwrap_or_else(|_| String::from("unknown"));

        match s.trim() {
            "run" => RunitServiceState::Run,
            "down" => RunitServiceState::Down,
            "finish" => RunitServiceState::Finish,
            _ => RunitServiceState::Unknown,
        }
    }

    /// Get the service uptime.
    pub fn get_start_time(&self) -> Result<time::SystemTime> {
        // "/<svdir>/<service>/supervise/stat"
        let p = self.path.join("supervise").join("stat");

        Ok(fs::metadata(p)?.modified()?)
    }
}

/// List the services in a given runit service directory.
///
/// This function optionally allows you to specify the `log` boolean.  If set,
/// this will return the correponding log service for each base-level service
/// found.
///
/// You may also specify an optional filter to only allow services that contain
/// a given string.
pub fn get_services(
    path: &Path,
    log: bool,
    filter: Option<String>,
) -> Result<Vec<RunitService>> {
    // loop services directory and collect service names
    let mut dirs = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();

        if !p.is_dir() {
            continue;
        }

        let name = p
            .file_name()
            .ok_or_else(|| anyhow!("{:?}: failed to get name from service", p))?
            .to_str()
            .ok_or_else(|| {
                anyhow!("{:?}: failed to parse name from service", p)
            })?
            .to_string();

        if let Some(ref filter) = filter {
            if !name.contains(filter) {
                continue;
            }
        }

        let service = RunitService::new(&p, &name);
        dirs.push(service);

        if log {
            let p = entry.path().join("log");
            let name = "- log";
            let service = RunitService::new(&p, name);
            dirs.push(service);
        }
    }

    dirs.sort();

    Ok(dirs)
}
