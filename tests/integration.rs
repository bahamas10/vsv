use assert_cmd::Command;

static ENV_VARS: &'static [&str] =
    &["NO_COLOR", "PROC_DIR", "PSTREE_PROG", "RAYON_NUM_THREADS", "SV_PROG"];

fn vsv() -> Command {
    let mut cmd = Command::cargo_bin("vsv").unwrap();

    for env_var in ENV_VARS {
        cmd.env_remove(env_var);
    }

    cmd
}

#[test]
fn usage() {
    let assert = vsv().arg("-h").assert();
    assert.success().stderr("");
}
