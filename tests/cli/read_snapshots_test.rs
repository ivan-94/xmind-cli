use assert_cmd::Command;

#[test]
fn read_command_json_outputs_match_snapshots() {
    let cases = [
        (
            "inspect_json",
            vec![
                "inspect",
                "tests/fixtures/xmind/multiple-sheets.xmind",
                "--json",
            ],
        ),
        (
            "sheets_json",
            vec!["sheets", "tests/fixtures/xmind/minimal.xmind", "--json"],
        ),
        (
            "tree_json",
            vec![
                "tree",
                "tests/fixtures/xmind/minimal.xmind",
                "--depth",
                "2",
                "--json",
            ],
        ),
        (
            "get_json",
            vec![
                "get",
                "tests/fixtures/xmind/minimal.xmind",
                "--node",
                "id:topic-payment",
                "--json",
            ],
        ),
        (
            "find_json",
            vec![
                "find",
                "tests/fixtures/xmind/minimal.xmind",
                "--title",
                "Payment",
                "--json",
            ],
        ),
    ];

    for (name, args) in cases {
        insta::assert_snapshot!(name, read_output(&args));
    }
}

fn read_output(args: &[&str]) -> String {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(args)
        .output()
        .expect("read command runs");

    assert!(
        output.status.success(),
        "read command failed for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "read command emitted stderr for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("read output is valid UTF-8")
}
