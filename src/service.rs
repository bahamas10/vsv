/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 26, 2022
 * License: MIT
 */

//! Generic service related structs and enums.

use libc::pid_t;
use std::fmt;
use std::path::Path;
use std::time;

use anyhow::Result;
use yansi::{Color, Style};

use crate::runit::{RunitService, RunitServiceState};
use crate::utils;

/// Possible states for a service.
pub enum ServiceState {
    Run,
    Down,
    Finish,
    Unknown,
}

impl ServiceState {
    /// Get a suitable `yansi::Style` for the state.
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

    /// Get a suitable char for the state (as a `String`).
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

/**
 * A struct suitable for describing an abstract service.
 *
 * This struct itself doesn't do much - it just stores information about a
 * service and knows how to format it to look pretty.
 */
pub struct Service {
    name: String,
    state: ServiceState,
    enabled: bool,
    command: Option<String>,
    pid: Option<pid_t>,
    start_time: Result<time::SystemTime>,
    pstree: Option<Result<String>>,
}

impl Service {
    /// Create a new service from a `RunitService`.
    pub fn from_runit_service(
        service: &RunitService,
        want_pstree: bool,
        proc_path: &Path,
        pstree_prog: &str,
    ) -> (Self, Vec<String>) {
        let mut messages: Vec<String> = vec![];
        let name = service.name.to_string();
        let enabled = service.enabled();
        let pid = service.get_pid();
        let state = service.get_state();
        let start_time = service.get_start_time();

        let mut command = None;
        if let Ok(p) = pid {
            match utils::cmd_from_pid(p, proc_path) {
                Ok(cmd) => {
                    command = Some(cmd);
                }
                Err(err) => {
                    messages.push(format!(
                        "{:?}: failed to get command for pid {}: {:?}",
                        service.path, p, err
                    ));
                }
            };
        }

        let pid = match pid {
            Ok(pid) => Some(pid),
            Err(ref err) => {
                messages.push(format!(
                    "{:?}: failed to get pid: {}",
                    service.path, err
                ));
                None
            }
        };

        // optionally get pstree.  None if the user wants it, Some if the user
        // wants it regardless of execution success.
        let pstree = if want_pstree {
            pid.map(|pid| get_pstree(pid, pstree_prog))
        } else {
            None
        };

        let state = match state {
            RunitServiceState::Run => ServiceState::Run,
            RunitServiceState::Down => ServiceState::Down,
            RunitServiceState::Finish => ServiceState::Finish,
            RunitServiceState::Unknown => ServiceState::Unknown,
        };

        let svc =
            Self { name, state, enabled, command, pid, start_time, pstree };

        (svc, messages)
    }

    /// Format the service name as a string.
    fn format_name(&self) -> (String, Style) {
        (self.name.to_string(), Style::default())
    }

    /// Format the service char as a string.
    fn format_status_char(&self) -> (String, Style) {
        (self.state.get_char(), self.state.get_style())
    }

    /// Format the service state as a string.
    fn format_state(&self) -> (String, Style) {
        (self.state.to_string(), self.state.get_style())
    }

    /// Format the service enabled status as a string.
    fn format_enabled(&self) -> (String, Style) {
        let style = match self.enabled {
            true => Style::default().fg(Color::Green),
            false => Style::default().fg(Color::Red),
        };

        let s = self.enabled.to_string();

        (s, style)
    }

    /// Format the service pid as a string.
    fn format_pid(&self) -> (String, Style) {
        let style = Style::default().fg(Color::Magenta);

        let s = match self.pid {
            Some(pid) => pid.to_string(),
            None => String::from("---"),
        };

        (s, style)
    }

    /// Format the service command a string.
    fn format_command(&self) -> (String, Style) {
        let style = Style::default().fg(Color::Green);

        let s = match &self.command {
            Some(cmd) => cmd.clone(),
            None => String::from("---"),
        };

        (s, style)
    }

    /// Format the service time as a string.
    fn format_time(&self) -> (String, Style) {
        let style = Style::default();

        let time = match &self.start_time {
            Ok(time) => time,
            Err(err) => return (err.to_string(), style.fg(Color::Red)),
        };

        let t = match time.elapsed() {
            Ok(t) => t,
            Err(err) => return (err.to_string(), style.fg(Color::Red)),
        };

        let s = utils::relative_duration(&t);
        let style = match t.as_secs() {
            t if t < 5 => style.fg(Color::Red),
            t if t < 30 => style.fg(Color::Yellow),
            _ => style.dimmed(),
        };

        (s, style)
    }

    /// Format the service `pstree` output as a string.
    pub fn format_pstree(&self) -> (String, Style) {
        let style = Style::default();

        let tree = match &self.pstree {
            Some(tree) => tree,
            None => return ("".into(), style),
        };

        let (tree_s, style) = match tree {
            Ok(stdout) => (stdout.trim().into(), style.dimmed()),
            Err(err) => {
                (format!("pstree call failed: {}", err), style.fg(Color::Red))
            }
        };

        (format!("\n{}\n", tree_s), style)
    }
}

impl fmt::Display for Service {
    /// Format the service as a string suitable for output by `vsv`.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let base = utils::format_status_line(
            self.format_status_char(),
            self.format_name(),
            self.format_state(),
            self.format_enabled(),
            self.format_pid(),
            self.format_command(),
            self.format_time(),
        );

        base.fmt(f)
    }
}

/// Get the `pstree` for a given pid.
fn get_pstree(pid: pid_t, pstree_prog: &str) -> Result<String> {
    let cmd = pstree_prog.to_string();
    let args = ["-ac".to_string(), pid.to_string()];
    utils::run_program_get_output(&cmd, &args)
}
