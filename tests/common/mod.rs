/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 20, 2022
 * License: MIT
 */

use std::path::PathBuf;

use assert_cmd::Command;

pub fn get_tmp_path() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("tests")
}

pub fn vsv() -> Command {
    let mut cmd = Command::cargo_bin("vsv").unwrap();

    cmd.env_clear();

    cmd
}
