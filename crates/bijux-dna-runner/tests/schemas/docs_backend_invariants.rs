use std::fs;

#[test]
fn backend_invariants_are_documented() {
    let doc = crate::support::crate_root("bijux-dna-runner")
        .unwrap_or_else(|err| panic!("resolve runner root: {err}"))
        .join("docs")
        .join("BACKENDS.md");
    let content = fs::read_to_string(&doc).unwrap_or_else(|err| panic!("read BACKENDS.md: {err}"));

    for invariant in ["cwd", "env", "mounts", "stdout/stderr", "exit semantics"] {
        assert!(
            content.contains(invariant),
            "BACKENDS.md missing invariant {invariant}"
        );
    }

    assert!(
        content.contains("tests/backend_invariants.rs"),
        "BACKENDS.md must reference tests/backend_invariants.rs"
    );
}
