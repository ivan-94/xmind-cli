use assert_cmd::Command;

#[test]
fn mutation_dry_run_json_outputs_match_snapshots() {
    let cases = [
        (
            "add_dry_run_json",
            vec![
                "add",
                "tests/fixtures/xmind/minimal.xmind",
                "--parent",
                "path:/Q2",
                "--title",
                "Refund",
                "--dry-run",
                "--json",
            ],
        ),
        (
            "set_title_dry_run_json",
            vec![
                "set",
                "tests/fixtures/xmind/minimal.xmind",
                "--node",
                "id:topic-payment",
                "--title",
                "Checkout",
                "--dry-run",
                "--json",
            ],
        ),
        (
            "delete_dry_run_json",
            vec![
                "delete",
                "tests/fixtures/xmind/minimal.xmind",
                "--node",
                "path:/Q2/Payment",
                "--dry-run",
                "--json",
            ],
        ),
        (
            "move_dry_run_json",
            vec![
                "move",
                "tests/fixtures/xmind/duplicate-titles.xmind",
                "--node",
                "id:topic-payment-q1",
                "--to",
                "root",
                "--dry-run",
                "--json",
            ],
        ),
        (
            "copy_dry_run_json",
            vec![
                "copy",
                "tests/fixtures/xmind/duplicate-titles.xmind",
                "--node",
                "id:topic-q1",
                "--to",
                "id:topic-q2",
                "--title",
                "Q1 Copy",
                "--dry-run",
                "--json",
            ],
        ),
        (
            "patch_add_tree_dry_run_json",
            vec![
                "patch",
                "tests/fixtures/xmind/minimal.xmind",
                "--ops",
                "docs/examples/patch-add-tree.yaml",
                "--dry-run",
                "--json",
            ],
        ),
        (
            "patch_working_copy_dry_run_json",
            vec![
                "patch",
                "tests/fixtures/xmind/minimal.xmind",
                "--ops",
                "tests/fixtures/patch/working-copy.yaml",
                "--dry-run",
                "--json",
            ],
        ),
    ];

    for (name, args) in cases {
        insta::assert_snapshot!(name, mutation_output(&args));
    }
}

fn mutation_output(args: &[&str]) -> String {
    let output = Command::cargo_bin("xmind")
        .expect("xmind binary is built for CLI tests")
        .args(args)
        .output()
        .expect("mutation command runs");

    assert!(
        output.status.success(),
        "mutation command failed for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "mutation command emitted stderr for args {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).expect("mutation output is valid UTF-8")
}
