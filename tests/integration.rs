/*
 * Integration tests for vsv.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 19, 2022
 * License: MIT
 */

use std::fs;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{anyhow, Result};
//use assert_cmd::Command;

mod common;

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

fn create_service(
    name: &str,
    pid: &str,
    proc_path: &Path,
    service_path: &Path,
) -> Result<()> {
    let svc_dir = service_path.join(name);
    let proc_pid_dir = proc_path.join(pid);
    let supervise_dir = svc_dir.join("supervise");

    fs::create_dir(&svc_dir)?;
    fs::create_dir(&proc_pid_dir)?;
    fs::create_dir(&supervise_dir)?;

    let pid_file = supervise_dir.join("pid");
    let stat_file = supervise_dir.join("stat");
    let cmd_file = proc_pid_dir.join("cmdline");

    fs::write(&pid_file, format!("{}\n", pid))?;
    fs::write(&stat_file, "run\n")?;
    fs::write(&cmd_file, format!("{}-cmd\0", name))?;

    Ok(())
}

//fn compare_output(have: &Vec<Vec<&str>>, want: &Vec<Vec<&str>>) {
fn compare_output(have: &[Vec<&str>], want: &[&[&str; 6]]) {
    println!("compare_output\nhave = '{:?}'\nwant = '{:?}'", have, want);

    assert_eq!(have.len(), want.len(), "status lines not same length");

    for (line_no, have_items) in have.iter().enumerate() {
        let want_items = want[line_no];

        for (field_no, want_item) in want_items.iter().enumerate() {
            let have_item = &have_items[field_no].trim_end();
            println!(
                "line {} field {}: checking '{}' == '{}'",
                line_no, field_no, have_item, want_item
            );
            assert_eq!(
                have_item, want_item,
                "line {} field {} incorrect",
                line_no, field_no
            );
        }
    }

    println!("output the same\n");
}

/*
fn run_cmd_get_parsed_output(cmd: &Command) -> Vec<Vec<&
*/

#[test]
fn full_synthetic_test() -> Result<()> {
    let tmp_path = get_tmp_path();
    let proc_path = tmp_path.join("proc");
    let service_path = tmp_path.join("service");

    // initialize directories
    // this can fail - that's ok
    let _ = fs::remove_dir_all(&tmp_path);

    // create test dirs
    for p in [&tmp_path, &proc_path, &service_path] {
        fs::create_dir(p)?;
    }

    // create the vsv command to use for all tests
    let mut cmd = common::vsv()?;
    cmd.env("SVDIR", &service_path);
    cmd.env("PROC_DIR", &proc_path);

    // test no services
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = str::from_utf8(&output.stdout)?;

    let status = parse_status_output(stdout)?;

    assert!(status.is_empty(), "no services");

    // test 1 service
    create_service("foo", "123", &proc_path, &service_path)?;

    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = str::from_utf8(&output.stdout)?;

    let status = parse_status_output(stdout)?;

    let want = &[&["✔", "foo", "run", "true", "123", "foo-cmd"]];
    compare_output(&status, want);

    Ok(())
}
