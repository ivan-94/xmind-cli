use std::fs;

#[test]
fn cargo_dist_release_contract_is_checked_in() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Cargo.toml is readable");
    let release_workflow = fs::read_to_string(".github/workflows/release.yml")
        .expect("cargo-dist release workflow should be checked in");
    let release_policy = fs::read_to_string("docs/technical/release-policy.md")
        .expect("release policy is readable");
    let installation =
        fs::read_to_string("docs/installation.md").expect("installation doc is readable");
    let changelog = fs::read_to_string("CHANGELOG.md").expect("changelog is readable");

    for expected in [
        r#"repository = "https://github.com/ivan-94/xmind-cli""#,
        "[profile.dist]",
        r#"inherits = "release""#,
        "[workspace.metadata.dist]",
        r#"cargo-dist-version = "0.31.0""#,
        r#"ci = ["github"]"#,
        r#"hosting = ["github"]"#,
        r#"installers = []"#,
        r#"checksum = "sha256""#,
        r#"create-release = true"#,
        r#"pr-run-mode = "plan""#,
        r#"dist = true"#,
        r#"targets = ["x86_64-unknown-linux-gnu"]"#,
    ] {
        assert!(
            cargo_toml.contains(expected),
            "Cargo.toml should include `{expected}`"
        );
    }

    for expected in [
        "on:",
        "push:",
        "tags:",
        "v*",
        "pull_request:",
        "permissions:",
        "contents: write",
        "id-token: write",
        "dist plan",
        "dist build",
        "cargo-dist@v0.31.0",
        "x86_64-unknown-linux-gnu",
        "GitHub Release",
    ] {
        assert!(
            release_workflow.contains(expected),
            "release workflow should include `{expected}`"
        );
    }

    assert!(
        !cargo_toml.contains(r#""homebrew""#) && !release_workflow.contains("publish-jobs"),
        "cargo-dist config should not require Homebrew publication in issue #5"
    );

    for expected in [
        "cargo-dist",
        "GitHub Releases",
        "CHANGELOG.md",
        "SHA256",
        "per-artifact `.sha256` checksum files",
        "issue #5",
    ] {
        assert!(
            release_policy.contains(expected),
            "release policy should document `{expected}`"
        );
    }

    for expected in [
        "cargo dist plan",
        "cargo dist build --artifacts=local",
        "per-artifact `.sha256` checksum files",
    ] {
        assert!(
            installation.contains(expected),
            "installation doc should mention `{expected}`"
        );
    }

    assert!(
        changelog.contains("cargo-dist release workflow"),
        "changelog should mention cargo-dist release workflow"
    );
}
