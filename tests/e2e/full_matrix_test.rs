const COMMAND_MATRIX: &[&str] = &[
    "inspect",
    "sheets",
    "tree",
    "get",
    "find",
    "validate",
    "diff",
    "add",
    "set",
    "delete",
    "move",
    "copy",
    "add-tree",
    "patch",
    "import",
    "export",
    "backup",
    "restore",
    "completion",
];

#[test]
#[ignore = "full E2E matrix expansion is intentionally filterable from the default PR subset"]
fn full_matrix_command_inventory_is_filterable_for_nightly_or_release_jobs() {
    assert_eq!(COMMAND_MATRIX.len(), 19);
    assert!(COMMAND_MATRIX.contains(&"patch"));
    assert!(COMMAND_MATRIX.contains(&"completion"));
}
