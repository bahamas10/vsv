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
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use yansi::Style;

/// Format a status line - made specifically for vsv.
///
/// # Example
/// ```
/// use yansi::Style;
/// let bold_style = Style::default().bold();
/// println!(
///     "{}",
///     format_status_line(
///         ("", &bold_style),
///         ("SERVICE", &bold_style),
///         ("STATE", &bold_style),
///         ("ENABLED", &bold_style),
///         ("PID", &bold_style),
///         ("COMMAND", &bold_style),
///         ("TIME", &bold_style),
///     )
/// );
/// ```
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

/// Get the program name (arg0) for a PID.
///
/// # Example
///
/// ```
/// use std::path::PathBuf;
///
/// let pid = 1;
/// let proc_path = PathBuf::from("/proc");
/// let cmd = cmd_from_pid(pid, &proc_path)?;
/// println!("pid {} program is {}", pid, cmd);
/// ```
pub fn cmd_from_pid(pid: pid_t, proc_path: &PathBuf) -> Result<String> {
    // /<proc_path>/<pid>/cmdline
    let p = proc_path.join(pid.to_string()).join("cmdline");

    let data = fs::read_to_string(&p)
        .with_context(|| format!("failed to read pid file: {:?}", p))?;

    let first = data.split('\0').next();

    match first {
        Some(f) => Ok(f.to_string()),
        None => Err(anyhow!("failed to split cmdline data: {:?}", first)),
    }
}

/// Run a program and get stdout.
///
/// # Example
///
/// ```
/// let cmd = "echo";
/// let args = ["hello", "world"];
/// let out = run_program_get_output(&cmd, &args)?;
/// println!("stdout is '{}'", out);
/// ```
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

/// Run a program and get the exit status.
///
/// # Example
///
/// ```
/// let cmd = "echo";
/// let args = ["hello", "world"];
/// let c = run_program_get_status(&cmd, &args);
/// match c {
///     Ok(status) => println!("exited with code: {}",
///                   status.code().unwrap_or(-1)),
///     Err(err) => eprintln!("program failed: {}", err),
/// };
/// ```
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
/// assert_eq!(relative_duration(&dur), "5 seconds".to_string());
/// ```
pub fn relative_duration(t: &Duration) -> String {
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

        assert_eq!(out, "hello world\n", "stdout is correct");

        Ok(())
    }

    #[test]
    fn test_run_program_get_output_good_program_exit_failure() -> Result<()> {
        let cmd = "false";
        let args: [&str; 0] = [];
        let out = run_program_get_output(&cmd, &args);

        assert!(out.is_err(), "program generates an error");

        Ok(())
    }

    #[test]
    fn test_run_program_get_output_bad_program() -> Result<()> {
        let cmd = "this-command-should-never-exist---seriously";
        let args: [&str; 0] = [];
        let out = run_program_get_output(&cmd, &args);

        assert!(out.is_err(), "program generates an error");

        Ok(())
    }

    #[test]
    fn test_run_program_get_status_good_program_exit_success() -> Result<()> {
        let cmd = "true";
        let args: [&str; 0] = [];
        let c = run_program_get_status(&cmd, &args)?;

        assert_eq!(c.code().unwrap_or(-1), 0, "program exits successfully");

        Ok(())
    }

    #[test]
    fn test_run_program_get_status_good_program_exit_failure() -> Result<()> {
        let cmd = "false";
        let args: [&str; 0] = [];
        let c = run_program_get_status(&cmd, &args)?;

        let code =
            c.code().ok_or_else(|| anyhow!("failed to get exit code"))?;

        assert_ne!(code, 0, "program exit code is not 0");

        Ok(())
    }

    #[test]
    fn test_run_program_get_status_bad_program() -> Result<()> {
        let cmd = "this-command-should-never-exist---seriously";
        let args: [&str; 0] = [];
        let c = run_program_get_status(&cmd, &args);

        assert!(c.is_err(), "program generates an error");

        Ok(())
    }

    #[test]
    fn test_isatty_bad_fd() {
        let b = isatty(-1);

        assert_eq!(b, false, "fd -1 is not a tty");
    }

    #[test]
    fn test_relative_durations() {
        use std::time::Duration;

        let arr = [
            (0, "0 seconds"),
            (3, "3 seconds"),
            (3 * 60, "3 minutes"),
            (3 * 60 * 60, "3 hours"),
            (3 * 60 * 60 * 24, "3 days"),
            (3 * 60 * 60 * 24 * 7, "3 weeks"),
            (3 * 60 * 60 * 24 * 30, "3 months"),
            (3 * 60 * 60 * 24 * 365, "3 years"),
        ];

        for (secs, s) in arr {
            let dur = Duration::new(secs, 0);
            assert_eq!(relative_duration(&dur), s, "duration mismatch");
        }
    }
}
