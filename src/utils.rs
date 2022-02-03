use libc::pid_t;
use std::env;
use std::fs;
use std::path;
use std::time;
use std::ffi::OsString;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
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
