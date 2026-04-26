use std::fs;

#[test]
fn backend_invariants_are_documented() {
    let doc = crate::support::crate_root("bijux-dna-runner")
        .unwrap_or_else(|err| panic!("resolve runner root: {err}"))
        .join("docs")
        .join("EXECUTION_SPEC.md");
    let content =
        fs::read_to_string(&doc).unwrap_or_else(|err| panic!("read EXECUTION_SPEC.md: {err}"));

    for invariant in ["cwd", "env", "mounts", "stdout/stderr", "exit semantics"] {
        assert!(content.contains(invariant), "EXECUTION_SPEC.md missing invariant {invariant}");
    }

    assert!(
        content.contains("tests/boundaries/backend/backend_invariants.rs"),
        "EXECUTION_SPEC.md must reference tests/boundaries/backend/backend_invariants.rs"
    );
}
