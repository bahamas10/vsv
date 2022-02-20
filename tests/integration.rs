/*
 * Integration tests for vsv.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 19, 2022
 * License: MIT
 */

use std::fs;
use std::str;

use anyhow::Result;

mod common;

fn parse_and_verify_status_output<'a>(s: &'a str) -> Result<Vec<Vec<&'a str>>> {
    let mut lines: Vec<Vec<&str>> = vec![];

    let spl: Vec<&str> = s.lines().collect();
    let len = spl.len();

    for (i, line) in spl.iter().enumerate() {
        // remove first and last line of output (blank lines)
        if i == 0 || i == (len - 1) {
            assert!(line.is_empty(), "first and last status line not empty");
            continue;
        }

        let line = line.trim();
        let items: Vec<&str> = line.split_whitespace().collect();
        lines.push(items);
    }

    assert!(!lines.is_empty(), "status must have at least one line (the header)");

    let header = lines.remove(0);
    let good_header = &["SERVICE", "STATE", "ENABLED", "PID", "COMMAND", "TIME"];

    for (i, s) in good_header.iter().enumerate() {
        assert_eq!(&header[i], s);
    }

    Ok(lines)
}

fn create_service(name: &str, pid: &str) -> Result<()> {
    todo!()
}

#[test]
fn full_synthetic_test() -> Result<()> {
    let tmp_path = common::get_tmp_path();
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
    let mut cmd = common::vsv();
    cmd.env("SVDIR", service_path);
    cmd.env("PROC_PATH", proc_path);

    // test no services
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = str::from_utf8(&output.stdout)?;

    let status = parse_and_verify_status_output(stdout)?;

    assert!(status.is_empty(), "no services");


    Ok(())
}
