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
        "Run documentation examples",
        "cargo test --test doc_examples_test --all-features documented_bash_e2e_examples_are_extracted_and_run",
        "stable-pr-e2e:",
        "Stable PR E2E subset",
        "cargo test --test e2e_pr_subset --all-features",
    ] {
        assert!(
            workflow.contains(expected),
            "CI workflow should include `{expected}`"
        );
    }
}
