/*
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 20, 2022
 * License: MIT
 */

use anyhow::Result;
use assert_cmd::Command;

pub fn vsv() -> Result<Command> {
    let mut cmd = Command::cargo_bin("vsv")?;

    cmd.env_clear();

    Ok(cmd)
}
