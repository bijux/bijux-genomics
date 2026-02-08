use std::fs;
use std::path::PathBuf;

#[test]
fn backend_invariants_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
