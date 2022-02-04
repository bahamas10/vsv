use libc::pid_t;
use std::env;
use std::fs;
use std::path;
use std::time;
use std::ffi::OsString;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use yansi::Paint;
use lazy_static::lazy_static;

const PROC_DIR: &str = "/proc";

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

pub fn format_status_line(
    status_char: Paint<&str>,
    name: Paint<&str>,
    state: Paint<&str>,
    enabled: Paint<&str>,
    pid: Paint<&str>,
    command: Paint<&str>,
    time: Paint<&str>) -> String {

    let status_char_len = 1;
    let name_len = 20;
    let state_len = 7;
    let enabled_len = 9;
    let pid_len = 8;
    let command_len = 17;
    // time is unmodified (it's the end of the output)

    let status_char = trim_long_paint(&status_char, status_char_len, "");
    let name = trim_long_paint(&name, name_len, "...");
    let state = trim_long_paint(&state, state_len, "...");
    let enabled = trim_long_paint(&enabled, enabled_len, "...");
    let pid = trim_long_paint(&pid, pid_len, "...");
    let command = trim_long_paint(&command, command_len, "...");

    format!("  {0:1$} {2:3$} {4:5$} {6:7$} {8:9$} {10:11$} {12}",
        status_char, status_char_len,
        name, name_len,
        state, state_len,
        enabled, enabled_len,
        pid, pid_len,
        command, command_len,
        time)
}

pub fn cmd_from_pid(pid: pid_t) -> Result<String> {
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

pub fn run_program(args: &[&str]) -> Result<String> {
    assert!(!args.is_empty(), "run_program requires at least 1 argument");

    let cmd = &args[0];
    let args = &args[1..];

    let output = Command::new(cmd)
        .args(args)
        .output()?;

    if ! output.status.success() {
        return Err(anyhow!("program '{}' returned non-zero", cmd));
    }

    let stdout = String::from_utf8(output.stdout)?;

    Ok(stdout)
}

pub fn relative_duration(t: time::Duration) -> String {
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

pub fn trim_long_paint(p: &Paint<&str>, limit: usize, suffix: &str) -> Paint<String> {
    let style = p.style();
    let s = p.inner();

    let new_s = trim_long_string(s, limit, suffix);

    style.paint(new_s)
}

pub fn trim_long_string(s: &str, limit: usize, suffix: &str) -> String {
    let suffix_len = suffix.len();

    assert!(limit > suffix_len, "number too small");

    let len = s.len();

    // don't do anything if string is smaller than limit
    if len < limit {
        return s.to_string();
    }

    // make new string (without formatting)
    format!("{}{}",
        s.chars().take(limit - suffix_len).collect::<String>(),
        suffix)
}
