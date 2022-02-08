use libc::{pid_t, c_int};
use std::env;
use std::fs;
use std::path;
use std::time;
use std::ffi::OsString;
use std::process::{Command, ExitStatus};

use anyhow::{anyhow, Context, Result};
use yansi::Style;
use lazy_static::lazy_static;

use crate::config;

/*
 * Make the proc dir var (overrideable via env vars) accessible everywhere after first access.
 */
lazy_static! {
    static ref PROC_PATH: path::PathBuf = {
        let proc_dir = match env::var_os(config::ENV_PROC_DIR) {
            Some(dir) => dir,
            None => OsString::from(config::DEFAULT_PROC_DIR),
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

    // ( data + style to print, max width, suffix )
    let data = [
        (status_char, 1,  ""   ),
        (name,        20, "..."),
        (state,       7,  "..."),
        (enabled,     9,  "..."),
        (pid,         8,  "..."),
        (command,     17, "..."),
        (time,        99, "..."),
    ];

    let mut line = String::from(" ");

    for (o, max, suffix) in data {
        let (text, style) = o;

        let text = trim_long_string(text.as_ref(), max, suffix);

        let column = format!(" {0:1$}", style.paint(text), max);
        line.push_str(&column);
    }

    line
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

pub fn run_program_get_output<T: AsRef<str>>(cmd: T, args: &[T]) -> Result<String> {
    let output = make_command(cmd, args).output()?;

    if ! output.status.success() {
        return Err(anyhow!("program '{:?}' returned non-zero", args[0].as_ref()));
    }

    let stdout = String::from_utf8(output.stdout)?;

    Ok(stdout)
}

pub fn run_program_get_status<T: AsRef<str>>(cmd: T, args: &[T]) -> Result<ExitStatus> {
    let p = make_command(cmd, args).status()?;

    Ok(p)
}

fn make_command<T: AsRef<str>>(cmd: T, args: &[T]) -> Command {
    let mut c = Command::new(cmd.as_ref());

    for arg in args {
        c.arg(arg.as_ref());
    }

    c
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
