/*
 * Integration tests for vsv.
 *
 * Author: Dave Eddy <dave@daveeddy.com>
 * Date: February 19, 2022
 * License: MIT
 */

mod common;

#[test]
fn usage() {
    let assert = common::vsv().arg("-h").assert();

    assert.success().stderr("");
}
