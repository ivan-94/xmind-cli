use assert_cmd::Command;

const SUBCOMMANDS: &[&str] = &[
    "inspect", "sheets", "tree", "find", "get", "add", "add-tree", "set", "delete", "move", "copy",
    "patch", "diff", "validate", "export", "import", "backup", "restore",
];

#[test]
fn top_level_help_matches_snapshot() {
    insta::assert_snapshot!("top_level_help", help_output(&["--help"]));
}

#[test]
fn subcommand_help_matches_snapshots() {
    for subcommand in SUBCOMMANDS {
        insta::assert_snapshot!(
            format!("subcommand_{subcommand}_help"),
            help_output(&[subcommand, "--help"])
        );
    }
}

fn help_output(args: &[&str]) -> String {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(args)
        .output()
        .expect("help command runs");

    assert!(
        output.status.success(),
        "help command failed for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("help output is valid UTF-8")
}
