/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 20, 2022
 * License: MIT
 */

use anyhow::Result;
use assert_cmd::Command;

pub fn vsv() -> Result<Command> {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("vsv");

    cmd.env_clear();

    Ok(cmd)
}
