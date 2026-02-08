use std::fs;
use std::path::PathBuf;

/// Snapshot locks infra public surface to keep it minimal.
#[test]
fn public_surface_is_snapshotted() {
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    let _guard = settings.bind_to_scope();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib = manifest_dir.join("src").join("lib.rs");
    let content = fs::read_to_string(&lib)
        .unwrap_or_else(|err| panic!("read lib.rs at {}: {err}", lib.display()));
    let mut snapshot = String::new();
    for line in content.lines() {
        if line.trim_start().starts_with("pub mod") || line.trim_start().starts_with("pub use") {
            snapshot.push_str(line);
            snapshot.push('\n');
        }
    }
    let name = bijux_dna_testkit::snapshot_name("schemas", "public_surface");
    insta::assert_snapshot!(name, bijux_dna_testkit::snapshot_normalize_text(&snapshot));
}
