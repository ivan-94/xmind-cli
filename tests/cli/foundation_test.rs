use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn top_level_help_exposes_cli_name_and_initial_commands() {
    let mut cmd = Command::cargo_bin("xmind").expect("xmind binary is built for CLI tests");

    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("xmind"))
        .stdout(predicate::str::contains("inspect"))
        .stdout(predicate::str::contains("tree"))
        .stdout(predicate::str::contains("patch"));
}
