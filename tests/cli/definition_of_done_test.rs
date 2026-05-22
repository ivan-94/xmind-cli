use std::fs;

use assert_cmd::Command;

#[test]
fn documented_command_references_exist_in_cli_help() {
    let top_help = help_output(&["--help"]);
    let mut documented_commands = fs::read_dir("docs/reference/commands")
        .expect("command reference directory is readable")
        .map(|entry| {
            entry
                .expect("command reference entry is readable")
                .path()
                .file_stem()
                .and_then(|stem| stem.to_str())
                .expect("command reference filename is UTF-8")
                .to_owned()
        })
        .collect::<Vec<_>>();
    documented_commands.sort();

    for command in documented_commands {
        assert!(
            top_help.contains(&command),
            "top-level help should include documented command `{command}`"
        );
        help_output(&[&command, "--help"]);
    }
}

#[test]
fn documented_global_and_shared_options_are_exposed_in_help() {
    let top_help = help_output(&["--help"]);
    for option in [
        "--json",
        "--format",
        "--fields",
        "--quiet",
        "--no-color",
        "--sheet",
        "--sheet-id",
        "--sheet-index",
    ] {
        assert!(
            top_help.contains(option),
            "top-level help should expose global option `{option}`"
        );
    }

    for (command, options) in [
        ("get", vec!["--node"]),
        (
            "add",
            vec![
                "--parent",
                "--dry-run",
                "--apply",
                "--backup",
                "--create-missing-path",
                "--position",
            ],
        ),
        (
            "move",
            vec!["--node", "--to", "--position", "--dry-run", "--apply"],
        ),
        ("export", vec!["--output", "--overwrite"]),
        ("find", vec!["--limit", "--offset"]),
        ("tree", vec!["--depth"]),
    ] {
        let command_help = help_output(&[command, "--help"]);
        for option in options {
            assert!(
                command_help.contains(option),
                "`xmind {command} --help` should expose shared documented option `{option}`"
            );
        }
    }
}

#[test]
fn mutating_command_help_requires_dry_run_or_apply() {
    for command in [
        "add", "add-tree", "set", "delete", "move", "copy", "patch", "import", "restore",
    ] {
        let command_help = help_output(&[command, "--help"]);
        assert!(
            command_help.contains("<--dry-run|--apply>"),
            "`xmind {command} --help` should require exactly one of --dry-run or --apply"
        );
    }
}

#[test]
fn documented_error_codes_are_represented_in_tests() {
    let error_docs =
        fs::read_to_string("docs/reference/errors.md").expect("errors doc is readable");
    let test_sources = collect_rust_sources("src") + &collect_rust_sources("tests/cli");

    for line in error_docs.lines() {
        let Some(code) = line
            .strip_prefix("| `")
            .and_then(|rest| rest.split('`').next())
        else {
            continue;
        };
        if code == "Code" {
            continue;
        }
        assert!(
            test_sources.contains(code),
            "documented error code `{code}` should be represented in Rust tests"
        );
    }
}

fn collect_rust_sources(root: &str) -> String {
    let mut combined = String::new();
    let mut stack = vec![std::path::PathBuf::from(root)];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path).expect("source directory is readable") {
                stack.push(entry.expect("source entry is readable").path());
            }
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            combined.push_str(&fs::read_to_string(&path).expect("source file is readable"));
            combined.push('\n');
        }
    }
    combined
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
