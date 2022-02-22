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
        assert_eq!(&header[i].trim_end(), good_item);
    }

    Ok(lines)
}

fn create_service(cfg: &Config, name: &str, pid: Option<&str>) -> Result<()> {
    let svc_dir = cfg.service_path.join(name);
    let supervise_dir = svc_dir.join("supervise");
    let stat_file = supervise_dir.join("stat");

    fs::create_dir(&svc_dir)?;
    fs::create_dir(&supervise_dir)?;
    fs::write(&stat_file, "run\n")?;

    // write pid and proc info if supplied
    if let Some(pid) = pid {
        let proc_pid_dir = cfg.proc_path.join(pid);
        let pid_file = supervise_dir.join("pid");
        let cmd_file = proc_pid_dir.join("cmdline");

        fs::create_dir(&proc_pid_dir)?;
        fs::write(&pid_file, format!("{}\n", pid))?;
        fs::write(&cmd_file, format!("{}-cmd\0", name))?;
    }

    Ok(())
}

fn remove_service(cfg: &Config, name: &str, pid: Option<&str>) -> Result<()> {
    let svc_dir = cfg.service_path.join(name);
    fs::remove_dir_all(&svc_dir)?;

    if let Some(pid) = pid {
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
    for (line_no, have_items) in have.iter().enumerate() {
        let want_items = want[line_no];

        // loop each field in the line
        for (field_no, want_item) in want_items.iter().enumerate() {
            let have_item = &have_items[field_no].trim_end();

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
    let mut cmd = common::vsv()?;
    cmd.env("SVDIR", &cfg.service_path);
    cmd.env("PROC_DIR", &cfg.proc_path);

    // start fresh by removing the service and proc paths
    let _ = fs::remove_dir_all(&tmp_path);

    // vsv should fail when the service dir doesn't exist
    cmd.assert().failure();

    // create test dirs
    for p in [&tmp_path, &cfg.proc_path, &cfg.service_path] {
        fs::create_dir(p)?;
    }

    // test no services
    let want: &[&[&str; 6]] = &[];
    run_command_compare_output(&mut cmd, want)?;

    // test service
    create_service(&cfg, "foo", Some("123"))?;
    let want = &[&["✔", "foo", "run", "true", "123", "foo-cmd"]];
    run_command_compare_output(&mut cmd, want)?;

    // test another service
    create_service(&cfg, "bar", Some("234"))?;
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut cmd, want)?;

    // test service no pid
    create_service(&cfg, "baz", None)?;
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "baz", "run", "true", "---", "---"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut cmd, want)?;

    // test service bad pid
    create_service(&cfg, "bat", Some("uh oh this one won't parse"))?;
    let want = &[
        &["✔", "bar", "run", "true", "234", "bar-cmd"],
        &["✔", "bat", "run", "true", "---", "---"],
        &["✔", "baz", "run", "true", "---", "---"],
        &["✔", "foo", "run", "true", "123", "foo-cmd"],
    ];
    run_command_compare_output(&mut cmd, want)?;

    // remove services
    remove_service(&cfg, "bar", Some("234"))?;
    remove_service(&cfg, "bat", None)?;
    remove_service(&cfg, "baz", None)?;
    let want = &[&["✔", "foo", "run", "true", "123", "foo-cmd"]];
    run_command_compare_output(&mut cmd, want)?;

    Ok(())
}
