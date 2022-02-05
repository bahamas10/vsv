use libc::{pid_t, c_int};
use std::env;
use std::fs;
use std::path;
use std::time;
use std::ffi::OsString;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use yansi::Style;
use lazy_static::lazy_static;

/*
 * Make the proc dir var (overrideable via env vars) accessible everywhere after first access.
 */
lazy_static! {
    static ref PROC_PATH: path::PathBuf = {
        let proc_default = "/proc";
        let proc_dir = match env::var_os("PROC_DIR") {
            Some(dir) => dir,
            None => OsString::from(proc_default),
        };

        path::PathBuf::from(&proc_dir)
    };
}

pub fn format_status_line<T: AsRef<str>>(
    status_char: (T, &Style),
    name: (T, &Style),
    state: (T, &Style),
    enabled: (T, &Style),
    pid: (T, &Style),
    command: (T, &Style),
    time: (T, &Style)) -> String {

    let status_char_len = 1;
    let name_len = 20;
    let state_len = 7;
    let enabled_len = 9;
    let pid_len = 8;
    let command_len = 17;
    let time_len = 100;

    let status_char_s = trim_long_string(status_char.0.as_ref(), status_char_len, "");
    let name_s = trim_long_string(name.0.as_ref(), name_len, "...");
    let state_s = trim_long_string(state.0.as_ref(), state_len, "...");
    let enabled_s = trim_long_string(enabled.0.as_ref(), enabled_len, "...");
    let pid_s = trim_long_string(pid.0.as_ref(), pid_len, "...");
    let command_s = trim_long_string(pid.0.as_ref(), command_len, "...");
    let time_s = trim_long_string(time.0.as_ref(), time_len, "...");

    format!("  {0:1$} {2:3$} {4:5$} {6:7$} {8:9$} {10:11$} {12}",
        status_char.1.paint(status_char_s), status_char_len,
        name.1.paint(name_s), name_len,
        state.1.paint(state_s), state_len,
        enabled.1.paint(enabled_s), enabled_len,
        pid.1.paint(pid_s), pid_len,
        command.1.paint(command_s), command_len,
        time.1.paint(time_s))
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

pub fn isatty(fd: c_int) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}
