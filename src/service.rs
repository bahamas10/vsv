use libc::pid_t;
use std::fmt;
use std::time;

use anyhow::{anyhow, Result};
use colored::*;

use crate::runit::{RunitService, RunitServiceState};

use crate::utils;

pub enum ServiceState {
    Run,
    Down,
    Finish,
    Unknown
}

impl ServiceState {
    pub fn get_color(&self) -> Color {
        match self {
            ServiceState::Run => Color::Green,
            ServiceState::Down => Color::Red,
            ServiceState::Finish => Color::Yellow,
            ServiceState::Unknown => Color::Yellow,
        }
    }

    pub fn get_char(&self) -> String {
        let s = match self {
            ServiceState::Run => "✔",
            ServiceState::Down => "X",
            ServiceState::Finish => "X",
            ServiceState::Unknown => "?",
        };

        s.to_string()
    }
}

impl fmt::Display for ServiceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ServiceState::Run => "run",
            ServiceState::Down => "down",
            ServiceState::Finish => "finish",
            ServiceState::Unknown => "n/a",
        };

        s.fmt(f)
    }
}

pub struct Service {
    pub name: String,
    pub state: ServiceState,
    pub enabled: bool,
    pub command: Option<String>,
    pub pid: Option<pid_t>,
    pub start_time: Option<time::SystemTime>,
    pub pstree: Option<Result<String>>,
}

impl Service {
    pub fn from_runit_service(service: &RunitService, want_pstree: bool) -> Result<Self> {
        let name = service.path
            .file_name()
            .ok_or_else(|| anyhow!("{:?}: failed to get name from service", service.path))?
            .to_str()
            .ok_or_else(|| anyhow!("{:?}: failed to parse name from service", service.path))?
            .to_string();

        let enabled = service.enabled();
        let pid = service.get_pid();
        let state = service.get_state();
        let start_time = service.get_start_time().ok();

        let mut command = None;
        if let Ok(p) = pid {
            match utils::cmd_from_pid(p) {
                Ok(cmd) => {
                    command = Some(cmd);
                }
                Err(err) => {
                    println!("{:?}: failed to get command for pid {}: {:?}", service.path, p, err); // fix this
                }
            };
        }

        let pid = match pid {
            Ok(pid) => Some(pid),
            Err(ref err) => {
                println!("{:?}: failed to get pid: {}", service.path, err); // fix this
                None
            }
        };

        let mut pstree = None;
        if want_pstree {
            if let Some(pid) = pid {
                pstree = Some(get_pstree(pid))
            }
        }

        let state = match state {
            RunitServiceState::Run => ServiceState::Run,
            RunitServiceState::Down => ServiceState::Down,
            RunitServiceState::Finish => ServiceState::Finish,
            RunitServiceState::Unknown => ServiceState::Unknown,
        };

        Ok(Self {
            name,
            state,
            enabled,
            command,
            pid,
            start_time,
            pstree,
        })
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state_color = self.state.get_color();

        let status_char = self.state.get_char().color(state_color);
        let state = self.state.to_string().color(state_color);

        let name = utils::trim_long_string(&self.name, 20, "...");

        let command = match &self.command {
            Some(cmd) => cmd,
            None => "---",
        };
        let command = utils::trim_long_string(command, 17, "...").green();

        let time = match self.start_time {
            Some(time) => {
                match time.elapsed() {
                    Ok(t) => utils::relative_duration(t),
                    Err(err) => err.to_string(),
                }
            },
            None => String::from("---"),
        }.dimmed();

        let enabled = match self.enabled {
            true => "true".green(),
            false => "false".red(),
        };

        let pid = match self.pid {
            Some(pid) => pid.to_string(),
            None => String::from("---"),
        }.magenta();

        let mut base = format!("  {:1} {:20} {:7} {:9} {:8} {:17} {}",
            status_char,
            name,
            state,
            enabled,
            pid,
            command,
            time);

        if let Some(tree) = &self.pstree {
            let tree_s = match tree {
                Ok(stdout) => format!("{}", stdout.trim().dimmed()),
                Err(err) => format!("pstree call failed: {}", err.to_string().red()),
            };
            base = format!("{}\n\n{}\n", base, tree_s);
        }

        base.fmt(f)
    }
}

fn get_pstree(pid: pid_t) -> Result<String> {
    utils::run_program(&["pstree", "-ac", &pid.to_string()])
}
