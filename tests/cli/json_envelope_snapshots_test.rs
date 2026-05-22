use assert_cmd::Command;

#[test]
fn success_json_envelope_matches_snapshot() {
    insta::assert_snapshot!(
        "success_json_envelope",
        command_output(&["sheets", "tests/fixtures/xmind/minimal.xmind", "--json"])
    );
}

#[test]
fn failure_json_envelope_matches_snapshot() {
    insta::assert_snapshot!(
        "failure_json_envelope",
        command_output_allowing_failure(&[
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "id:missing-topic",
            "--json",
        ])
    );
}

fn command_output(args: &[&str]) -> String {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(args)
        .output()
        .expect("command runs");

    assert!(
        output.status.success(),
        "command failed for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "command emitted stderr for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("output is valid UTF-8")
}

fn command_output_allowing_failure(args: &[&str]) -> String {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(args)
        .output()
        .expect("command runs");

    assert!(
        !output.status.success(),
        "command unexpectedly succeeded for args {args:?}"
    );
    assert!(
        output.stderr.is_empty(),
        "command emitted stderr for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("output is valid UTF-8")
}
