use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn completion_bash_prints_shell_completion_script() {
    let mut cmd = Command::cargo_bin("xmind").expect("xmind binary is built for CLI tests");

    cmd.args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_xmind"))
        .stdout(predicate::str::contains("complete -F _xmind"));
}
