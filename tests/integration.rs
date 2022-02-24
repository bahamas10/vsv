/*
 * Integration tests for vsv.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 19, 2022
 * License: MIT
 */

use std::fs;
use std::path::PathBuf;
use std::str;

use anyhow::{anyhow, Result};
use assert_cmd::Command;

mod common;

struct Config {
    proc_path: PathBuf,
    service_path: PathBuf,
}

fn vsv(cfg: &Config) -> Result<Command> {
    let mut cmd = common::vsv()?;

    cmd.env("SVDIR", &cfg.service_path);
    cmd.env("PROC_DIR", &cfg.proc_path);

    Ok(cmd)
}

fn get_tmp_path() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tests")
}

fn parse_status_line(line: &str) -> Result<Vec<&str>> {
    let mut vec: Vec<&str> = vec![];
    let mut chars = line.chars().map(|c| c.len_utf8());

    let lengths = [1, 20, 7, 9, 8, 17];

    // skip the first space char
    let mut start = 0;
    start += chars.next().ok_or(anyhow!("first char must be a space"))?;
    assert_eq!(start, 1, "first char should always be 1 byte (space)");

    for num in lengths {
        let mut end = start;

        for _ in 0..num {
            end += chars.next().ok_or(anyhow!("not enough chars in line"))?;
        }

        vec.push(&line[start..end]);

        let space = chars
            .next()
            .ok_or(anyhow!("next field should have a space char"))?;

        assert_eq!(space, 1, "should be space character");

        start = end + space;
    }

    vec.push(&line[start..]);

    Ok(vec)
}

fn parse_status_output(s: &str) -> Result<Vec<Vec<&str>>> {
    let mut lines: Vec<Vec<&str>> = vec![];

    let spl: Vec<&str> = s.lines().collect();
    let len = spl.len();

    for (i, line) in spl.iter().enumerate() {
        // remove first and last line of output (blank lines)
        if i == 0 || i == (len - 1) {
            assert!(line.is_empty(), "first and last line should be empty");
            continue;
        }

        let items = parse_status_line(line)?;
        lines.push(items);
    }

    assert!(
        !lines.is_empty(),
        "status must have at least one line (the header)"
    );

    // check header
    let header = lines.remove(0);
    let good_header =
        &["", "SERVICE", "STATE", "ENABLED", "PID", "COMMAND", "TIME"];

    for (i, good_item) in good_header.iter().enumerate() {
        assert_eq!(&header[i].trim_end(), good_item, "check header field");
    }

    Ok(lines)
}

fn create_service(
    cfg: &Config,
    name: &str,
    state: &str,
    pid: Option<&str>,
    log_pid: Option<&str>,
) -> Result<()> {
    let svc_dir = cfg.service_path.join(name);
    let dirs = [("cmd", &svc_dir, pid), ("log", &svc_dir.join("log"), log_pid)];

    for (s, dir, pid) in dirs {
        let supervise_dir = dir.join("supervise");
        let stat_file = supervise_dir.join("stat");

        fs::create_dir(&dir)?;
        fs::create_dir(&supervise_dir)?;
        fs::write(&stat_file, format!("{}\n", state))?;

        // write pid and proc info if supplied
        if let Some(pid) = pid {
            let proc_pid_dir = cfg.proc_path.join(pid);
            let pid_file = supervise_dir.join("pid");
            let cmd_file = proc_pid_dir.join("cmdline");

            fs::create_dir(&proc_pid_dir)?;
            fs::write(&pid_file, format!("{}\n", pid))?;
            fs::write(&cmd_file, format!("{}-{}\0", name, s))?;
        }
    }

    Ok(())
}

fn remove_service(
    cfg: &Config,
    name: &str,
    pid: Option<&str>,
    log_pid: Option<&str>,
) -> Result<()> {
    let svc_dir = cfg.service_path.join(name);
    fs::remove_dir_all(&svc_dir)?;

    for pid in [pid, log_pid].into_iter().flatten() {
        let proc_pid_dir = cfg.proc_path.join(pid);
        fs::remove_dir_all(&proc_pid_dir)?;
    }

    Ok(())
}

//fn compare_output(have: &Vec<Vec<&str>>, want: &Vec<Vec<&str>>) {
fn compare_output(have: &[Vec<&str>], want: &[&[&str; 6]]) {
    println!("compare_output\nhave = '{:?}'\nwant = '{:?}'", have, want);

    assert_eq!(have.len(), want.len(), "status lines not same length");

    // loop each line of output
    for (i, have_items) in have.iter().enumerate() {
        let line_no = i + 1;
        let want_items = want[i];

        // loop each field in the line
        for (j, want_item) in want_items.iter().enumerate() {
            let field_no = j + 1;
            let have_item = &have_items[j].trim_end();

            println!(
                "line {} field {}: checking '{}' == '{}'",
                line_no, field_no, have_item, want_item
            );

            // compare the fields to each other
            assert_eq!(
                have_item, want_item,
                "line {} field {} incorrect",
                line_no, field_no
            );
        }
    }

    println!("output the same\n");
}

fn run_command_compare_output(
    cmd: &mut Command,
    want: &[&[&str; 6]],
) -> Result<()> {
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = str::from_utf8(&output.stdout)?;
    let status = parse_status_output(stdout)?;

    compare_output(&status, want);

    Ok(())
}

#[test]
fn full_synthetic_test() -> Result<()> {
    let tmp_path = get_tmp_path();

    let cfg = Config {
        proc_path: tmp_path.join("proc"),
        service_path: tmp_path.join("service"),
    };

    // create the vsv command to use for all tests
    let mut status_cmd = vsv(&cfg)?;
    let mut status_cmd_l = vsv(&cfg)?;
    status_cmd_l.arg("status").arg("-l");

    // start fresh by removing the service and proc paths
    let _ = fs::remove_dir_all(&tmp_path);

    // vsv should fail when the service dir doesn't exist
    status_cmd.assert().failure();

    // create test dirs
    for p in [&tmp_path, &cfg.proc_path, &cfg.service_path] {
        fs::create_dir(p)?;
    }

    // test no services
    let want: &[&[&str; 6]] = &[];
    run_command_compare_output(&mut status_cmd, want)?;

    // test service
    create_service(&cfg, "foo", "run", Some("123"), None)?;
    let want = &[&["✔", "foo", "run", "true", "123", "foo-cmd"]];
    run_command_compare_output(&mut status_cmd, want)?;

    // test another service
    create_service(&cfg, "bar", "run", Some("234"), None)?;
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // test service no pid
    create_service(&cfg, "baz", "run", None, None)?;
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "baz", "run", "true", "---", "---"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // test service bad pid
    create_service(
        &cfg,
        "bat",
        "run",
        Some("uh oh this one won't parse"),
        None,
    )?;
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "bat", "run", "true", "---", "---"],
        &["✔", "baz", "run", "true", "---", "---"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // remove services
    remove_service(&cfg, "bar", Some("234"), None)?;
    remove_service(&cfg, "bat", None, None)?;
    remove_service(&cfg, "baz", None, None)?;
    let want = &[&["✔", "foo", "run", "true", "123", "foo-cmd"]];
    run_command_compare_output(&mut status_cmd, want)?;

    // add down service
    create_service(&cfg, "bar", "down", None, None)?;
    let want = &[
        &["X", "bar", "down", "true", "---", "---"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // add unknown state service
    create_service(&cfg, "bat", "something-bad", None, None)?;
    let want = &[
        &["X", "bar", "down", "true", "---", "---"],
        &["?", "bat", "n/a", "true", "---", "---"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // add long service name
    create_service(
        &cfg,
        "some-really-long-service-name",
        "run",
        Some("1"),
        None,
    )?;
    let want = &[
        &["X", "bar", "down", "true", "---", "---"],
        &["?", "bat", "n/a", "true", "---", "---"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
        &["✔", "some-really-long-...", "run", "true", "1", "some-really-lo..."],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // remove services
    remove_service(&cfg, "some-really-long-service-name", Some("1"), None)?;
    remove_service(&cfg, "bar", None, None)?;
    remove_service(&cfg, "bat", None, None)?;
    let want = &[&["✔", "foo", "run", "true", "123", "foo-cmd"]];
    run_command_compare_output(&mut status_cmd, want)?;

    // add some more services
    create_service(&cfg, "bar", "run", Some("234"), None)?;
    create_service(&cfg, "baz", "run", Some("345"), None)?;
    create_service(&cfg, "bat", "run", Some("456"), None)?;

    // test disable
    let mut cmd = vsv(&cfg)?;
    cmd.args(&["disable", "bar", "baz"]).assert().success();

    let want = &[
        &["✔", "bar", "run", "false", "234", "bar-cmd"],
        &["✔", "bat", "run", "true", "456", "bat-cmd"],
        &["✔", "baz", "run", "false", "345", "baz-cmd"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // test enable
    let mut cmd = vsv(&cfg)?;
    cmd.args(&["enable", "foo", "bar"]).assert().success();
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "bat", "run", "true", "456", "bat-cmd"],
        &["✔", "baz", "run", "false", "345", "baz-cmd"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // test bad disable
    let mut cmd = vsv(&cfg)?;
    cmd.args(&["disable", "fake-service", "foo"]).assert().failure();
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "bat", "run", "true", "456", "bat-cmd"],
        &["✔", "baz", "run", "false", "345", "baz-cmd"],
        &["✔", "foo", "run", "false", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // test bad enable
    let mut cmd = vsv(&cfg)?;
    cmd.args(&["enable", "fake-service", "foo"]).assert().failure();
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "bat", "run", "true", "456", "bat-cmd"],
        &["✔", "baz", "run", "false", "345", "baz-cmd"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut status_cmd, want)?;

    // remove all services
    remove_service(&cfg, "foo", Some("123"), None)?;
    remove_service(&cfg, "bar", Some("234"), None)?;
    remove_service(&cfg, "baz", Some("345"), None)?;
    remove_service(&cfg, "bat", Some("456"), None)?;
    let want: &[&[&str; 6]] = &[];
    run_command_compare_output(&mut status_cmd, want)?;

    // create a service with a logger function
    create_service(&cfg, "foo", "run", Some("100"), Some("150"))?;
    let want = &[
        &["✔", "foo", "run", "true", "100", "foo-cmd"],
        &["✔", "- log", "run", "true", "150", "foo-log"],
    ];
    run_command_compare_output(&mut status_cmd_l, want)?;

    Ok(())
}
