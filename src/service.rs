use libc::pid_t;
use std::fmt;
use std::time;

use anyhow::{anyhow, Result};
use yansi::{Style, Color};

use crate::runit::{RunitService, RunitServiceState};

use crate::utils;

pub enum ServiceState {
    Run,
    Down,
    Finish,
    Unknown
}

impl ServiceState {
    pub fn get_style(&self) -> Style {
        let style = Style::default();

        let color = match self {
            ServiceState::Run => Color::Green,
            ServiceState::Down => Color::Red,
            ServiceState::Finish => Color::Yellow,
            ServiceState::Unknown => Color::Yellow,
        };

        style.fg(color)
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

    fn format_name(&self) -> String {
        self.name.to_string()
    }

    fn format_status_char(&self) -> String {
        self.state.get_char()
    }

    fn format_state(&self) -> String {
        self.state.to_string()
    }

    fn format_enabled(&self) -> String {
        self.enabled.to_string()
    }

    fn format_pid(&self) -> String {
        match self.pid {
            Some(pid) => pid.to_string(),
            None => String::from("---"),
        }
    }

    fn format_command(&self) -> String {
        match &self.command {
            Some(cmd) => cmd.clone(),
            None => String::from("---"),
        }
    }

    fn format_time(&self) -> String {
        match self.start_time {
            Some(time) => {
                match time.elapsed() {
                    Ok(t) => utils::relative_duration(t),
                    Err(err) => err.to_string(),
                }
            },
            None => String::from("---"),
        }
    }
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let state_style = self.state.get_style();

        let status_char = (self.format_status_char(), &state_style);

        let name = (self.format_name(), &Style::default());

        let state = (self.format_state(), &state_style);

        let enabled = match self.enabled {
            true => Style::default().fg(Color::Green),
            false => Style::default().fg(Color::Red),
        };
        let enabled = (self.format_enabled(), &enabled);

        let pid = (self.format_pid(), &Style::default().fg(Color::Magenta));

        let command = (self.format_command(), &Style::default().fg(Color::Green));

        let time = (self.format_time(), &Style::default().dimmed());

        let mut base = utils::format_status_line(
            status_char,
            name,
            state,
            enabled,
            pid,
            command,
            time);

        // add pstree if applicable
        if let Some(tree) = &self.pstree {
            let tree_s = match tree {
                Ok(stdout) => Style::default().dimmed().paint(stdout.trim().to_string()),
                Err(err) => Color::Red.paint(format!("pstree call failed: {}", err)),
            };
            base = format!("{}\n\n{}\n", base, tree_s);
        }

        base.fmt(f)
    }
}

fn get_pstree(pid: pid_t) -> Result<String> {
    utils::run_program(&["pstree", "-ac", &pid.to_string()])
}
