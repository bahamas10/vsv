/*
 * Integration tests for vsv.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 19, 2022
 * License: MIT
 */

use anyhow::Result;

mod common;

#[test]
fn usage() -> Result<()> {
    let assert = common::vsv()?.arg("-h").assert();

    assert.success().stderr("");

    Ok(())
}

#[test]
fn external_success() -> Result<()> {
    let mut cmd = common::vsv()?;

    cmd.env("SV_PROG", "true");

    let assert = cmd.args(&["external", "cmd"]).assert();

    assert.success();

    Ok(())
}

#[test]
fn external_failure() -> Result<()> {
    let mut cmd = common::vsv()?;

    cmd.env("SV_PROG", "false");

    let assert = cmd.args(&["external", "cmd"]).assert();

    assert.failure();

    Ok(())
}
