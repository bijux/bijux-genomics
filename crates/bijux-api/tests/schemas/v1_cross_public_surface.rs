use std::fs;
use std::path::PathBuf;

/// Snapshot locks API public surface for v1 cross endpoints.
#[test]
fn public_surface_is_snapshotted() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib = manifest_dir.join("src").join("lib.rs");
    let content = fs::read_to_string(lib).expect("read lib.rs");
    let mut snapshot = String::new();
    for line in content.lines() {
        if line.trim_start().starts_with("pub mod v1") {
            snapshot.push_str(line);
            snapshot.push('\n');
        }
    }
    let name = bijux_testkit::snapshot_name("schemas", "public_surface");
    insta::assert_snapshot!(name, snapshot);
}
