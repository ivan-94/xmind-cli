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
        "rustup toolchain install stable --profile minimal",
        "cargo +stable install cargo-audit --version 0.22.1 --locked",
        "cargo +stable audit",
        "EmbarkStudios/cargo-deny-action@v2",
        "cargo deny",
    ] {
        assert!(
            workflow.contains(expected),
            "CI workflow should include `{expected}`"
        );
    }
}

#[test]
fn branch_protection_script_documents_required_github_gate() {
    let script = fs::read_to_string("scripts/configure-branch-protection.sh")
        .expect("branch protection script should be committed");
    let quality_gates =
        fs::read_to_string("docs/technical/quality-gates.md").expect("quality gates doc exists");

    for expected in [
        "gh api",
        "/repos/${REPO}/branches/${BRANCH}/protection",
        "\"strict\": true",
        "\"Rust quality gate\"",
        "\"Stable PR E2E subset\"",
        "\"Security\"",
        "\"required_pull_request_reviews\"",
        "\"allow_force_pushes\": false",
        "\"allow_deletions\": false",
        "print-json",
    ] {
        assert!(
            script.contains(expected),
            "branch protection script should include `{expected}`"
        );
    }

    for expected in [
        "scripts/configure-branch-protection.sh apply",
        "Require a pull request before merging",
        "Require branches to be up to date before merging",
        "Rust quality gate",
        "Stable PR E2E subset",
        "Security",
        "disable force pushes and deletions",
    ] {
        assert!(
            quality_gates.contains(expected),
            "quality gates doc should document branch protection `{expected}`"
        );
    }
}
