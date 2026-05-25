use assert_cmd::Command;

const SUBCOMMANDS: &[&str] = &[
    "inspect",
    "sheets",
    "tree",
    "find",
    "get",
    "add",
    "add-tree",
    "set",
    "delete",
    "move",
    "copy",
    "patch",
    "diff",
    "validate",
    "export",
    "import",
    "backup",
    "restore",
    "completion",
];

#[test]
fn top_level_help_matches_snapshot() {
    insta::assert_snapshot!("top_level_help", help_output(&["--help"]));
}

#[test]
fn top_level_help_has_command_descriptions_and_examples() {
    let help = help_output(&["--help"]);

    assert!(
        help.contains("Examples:"),
        "top-level help should include representative examples"
    );
    assert!(
        help.lines()
            .any(|line| line.trim_start().starts_with("inspect  ")
                && line.contains("Summarize workbook metadata")),
        "top-level help should describe the inspect command"
    );
}

#[test]
fn empty_invocation_prints_top_level_help() {
    assert_eq!(help_output(&[]), help_output(&["--help"]));
}

#[test]
fn json_without_subcommand_prints_top_level_help() {
    assert_eq!(help_output(&["--json"]), help_output(&["--help"]));
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

#[test]
fn subcommand_help_has_purpose_and_examples() {
    for subcommand in SUBCOMMANDS {
        let help = help_output(&[subcommand, "--help"]);

        assert!(
            help.contains("Examples:"),
            "{subcommand} help should include at least one copyable example"
        );
        assert!(
            help.lines().next().is_some_and(|line| line.ends_with('.')),
            "{subcommand} help should start with a sentence describing its purpose"
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
