/*
 * Various util functions.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: January 26, 2022
 * License: MIT
 */

//! Contains various util functions for vsv.

use libc::{c_int, pid_t};
use std::fs;
use std::process::{Command, ExitStatus};
use std::time;

use anyhow::{anyhow, Context, Result};
use yansi::Style;

use crate::config;

pub fn format_status_line<T: AsRef<str>>(
    status_char: (T, &Style),
    name: (T, &Style),
    state: (T, &Style),
    enabled: (T, &Style),
    pid: (T, &Style),
    command: (T, &Style),
    time: (T, &Style),
) -> String {
    // ( data + style to print, max width, suffix )
    let data = [
        (status_char, 1, ""),
        (name, 20, "..."),
        (state, 7, "..."),
        (enabled, 9, "..."),
        (pid, 8, "..."),
        (command, 17, "..."),
        (time, 99, "..."),
    ];

    let mut line = String::new();

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
    let p = config::PROC_PATH.join(pid.to_string()).join("cmdline");

    let data = fs::read_to_string(&p)
        .with_context(|| format!("failed to read pid file: {:?}", p))?;

    let first = data.split('\0').next();

    match first {
        Some(f) => Ok(f.to_string()),
        None => Err(anyhow!("failed to split cmdline data: {:?}", first)),
    }
}

pub fn run_program_get_output<T1, T2>(cmd: &T1, args: &[T2]) -> Result<String>
where
    T1: AsRef<str>,
    T2: AsRef<str>,
{
    let output = make_command(cmd, args).output()?;

    if !output.status.success() {
        return Err(anyhow!("program '{}' returned non-zero", cmd.as_ref()));
    }

    let stdout = String::from_utf8(output.stdout)?;

    Ok(stdout)
}

pub fn run_program_get_status<T1, T2>(
    cmd: &T1,
    args: &[T2],
) -> Result<ExitStatus>
where
    T1: AsRef<str>,
    T2: AsRef<str>,
{
    let p = make_command(cmd, args).status()?;

    Ok(p)
}

/// Create a `std::process::Command` from a given command name and argument
/// slice.
///
/// # Example
///
/// ```
/// let cmd = "echo";
/// let args = ["hello", "world"];
/// let c = make_command(&cmd, &args);
/// ```
fn make_command<T1, T2>(cmd: &T1, args: &[T2]) -> Command
where
    T1: AsRef<str>,
    T2: AsRef<str>,
{
    let mut c = Command::new(cmd.as_ref());

    for arg in args {
        c.arg(arg.as_ref());
    }

    c
}

/// Convert a duration to a human-readable string like "5 minutes", "2 hours",
/// etc.
///
/// # Example
///
/// Duration for 5 seconds ago:
///
/// ```
/// use std::time::Duration;
/// let dur = Duration::new(5, 0);
/// assert_eq!(relative_duration(dur), "5 seconds".to_string());
/// ```
pub fn relative_duration(t: time::Duration) -> String {
    let secs = t.as_secs();

    let v = [
        (secs / 60 / 60 / 24 / 365, "year"),
        (secs / 60 / 60 / 24 / 30, "month"),
        (secs / 60 / 60 / 24 / 7, "week"),
        (secs / 60 / 60 / 24, "day"),
        (secs / 60 / 60, "hour"),
        (secs / 60, "minute"),
        (secs, "second"),
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

/// Trim a string to be (at most) a certain number of characters with an
/// optional suffix.
///
/// # Examples
///
/// Trim the string `"hello world"` to be (at most) 8 characters and add
/// `"..."`:
///
/// ```
/// let s = trim_long_string("hello world", 8, "...");
/// assert_eq!(s, "hello...");
/// ```
///
/// The suffix will only be added if the original string needed to be trimmed:
///
/// ```
/// let s = trim_long_string("hello world", 100, "...");
/// assert_eq!(s, "hello world");
/// ```
pub fn trim_long_string(s: &str, limit: usize, suffix: &str) -> String {
    let suffix_len = suffix.len();

    assert!(limit > suffix_len, "number too small");

    let len = s.len();

    // don't do anything if string is smaller than limit
    if len < limit {
        return s.to_string();
    }

    // make new string (without formatting)
    format!(
        "{}{}",
        s.chars().take(limit - suffix_len).collect::<String>(),
        suffix
    )
}

/// Check if the given file descriptor (by number) is a tty.
///
/// # Example
///
/// Print "hello world" if stdout is a tty:
///
/// ```
/// if isatty(1) {
///     println!("hello world");
/// }
/// ```
pub fn isatty(fd: c_int) -> bool {
    unsafe { libc::isatty(fd) != 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_program_get_output_good_program_exit_success() -> Result<()> {
        let cmd = "echo";
        let args = ["hello", "world"];
        let out = run_program_get_output(&cmd, &args)?;
        assert_eq!(out, "hello world\n");

        Ok(())
    }

    #[test]
    fn test_run_program_get_output_good_program_exit_failure() -> Result<()> {
        let cmd = "false";
        let args: [&str; 0] = [];
        let out = run_program_get_output(&cmd, &args);
        assert!(out.is_err());

        Ok(())
    }

    #[test]
    fn test_run_program_get_output_bad_program() -> Result<()> {
        let cmd = "this-command-should-never-exist---seriously";
        let args: [&str; 0] = [];
        let out = run_program_get_output(&cmd, &args);
        assert!(out.is_err());

        Ok(())
    }

    #[test]
    fn test_isatty_bad_fd() {
        let b = isatty(-1);
        assert_eq!(b, false);
    }

    #[test]
    fn test_relative_durations() {
        use std::time::Duration;

        let arr = [
            (5, "5 seconds"),
            (5 * 60, "5 minutes"),
            (5 * 60 * 60, "5 hours"),
            (5 * 60 * 60 * 24, "5 days"),
            (5 * 60 * 60 * 24 * 30, "5 months"),
            (5 * 60 * 60 * 24 * 365, "5 years"),
        ];

        for (secs, s) in arr {
            let dur = Duration::new(secs, 0);
            assert_eq!(relative_duration(dur), s);
        }
    }
}
