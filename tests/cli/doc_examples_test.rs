use assert_cmd::Command;

struct DocExample {
    command: &'static str,
    args: &'static [&'static str],
}

const DOC_EXAMPLES: &[DocExample] = &[
    DocExample {
        command: "xmind inspect tests/fixtures/xmind/minimal.xmind --json",
        args: &["inspect", "tests/fixtures/xmind/minimal.xmind", "--json"],
    },
    DocExample {
        command: "xmind sheets tests/fixtures/xmind/minimal.xmind --json",
        args: &["sheets", "tests/fixtures/xmind/minimal.xmind", "--json"],
    },
    DocExample {
        command: "xmind tree tests/fixtures/xmind/minimal.xmind --depth 2 --json",
        args: &[
            "tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--depth",
            "2",
            "--json",
        ],
    },
    DocExample {
        command: "xmind get tests/fixtures/xmind/minimal.xmind --node path:/Q2/Payment --json",
        args: &[
            "get",
            "tests/fixtures/xmind/minimal.xmind",
            "--node",
            "path:/Q2/Payment",
            "--json",
        ],
    },
    DocExample {
        command: "xmind find tests/fixtures/xmind/minimal.xmind --title Payment --json",
        args: &[
            "find",
            "tests/fixtures/xmind/minimal.xmind",
            "--title",
            "Payment",
            "--json",
        ],
    },
    DocExample {
        command: "xmind add-tree tests/fixtures/xmind/minimal.xmind --parent path:/Q2 --input docs/examples/simple-tree.yaml --dry-run --json",
        args: &[
            "add-tree",
            "tests/fixtures/xmind/minimal.xmind",
            "--parent",
            "path:/Q2",
            "--input",
            "docs/examples/simple-tree.yaml",
            "--dry-run",
            "--json",
        ],
    },
    DocExample {
        command: "xmind patch tests/fixtures/xmind/minimal.xmind --ops docs/examples/patch-add-tree.yaml --dry-run --json",
        args: &[
            "patch",
            "tests/fixtures/xmind/minimal.xmind",
            "--ops",
            "docs/examples/patch-add-tree.yaml",
            "--dry-run",
            "--json",
        ],
    },
];

#[test]
fn documented_fixture_examples_are_present_and_run() {
    let readme = std::fs::read_to_string("docs/examples/README.md")
        .expect("docs examples README is readable");

    for example in DOC_EXAMPLES {
        assert!(
            readme.contains(example.command),
            "docs/examples/README.md must include runnable example: {}",
            example.command
        );

        Command::cargo_bin("xmind")
            .expect("xmind binary is built for CLI tests")
            .args(example.args)
            .assert()
            .success();
    }
}
