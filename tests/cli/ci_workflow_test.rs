use std::fs;

#[test]
fn ci_workflow_defines_the_pr_merge_gate_contract() {
    let workflow = fs::read_to_string(".github/workflows/ci.yml")
        .expect("CI workflow should be committed at .github/workflows/ci.yml");

    for expected in [
        "pull_request:",
        "push:",
        "branches:",
        "master",
        "Install pinned Rust toolchain from rust-toolchain.toml",
        "Swatinem/rust-cache@v2",
        "save-if:",
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "cargo test --workspace --all-features",
        "stable-pr-e2e:",
        "Stable PR E2E subset placeholder",
        "Blocked on GitHub issue #12",
    ] {
        assert!(
            workflow.contains(expected),
            "CI workflow should include `{expected}`"
        );
    }
}
