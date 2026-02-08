use std::fs;
use std::path::PathBuf;

fn snapshot_name(bucket: &str, test_name: &str) -> String {
    format!("{}__{}__{}", env!("CARGO_PKG_NAME"), bucket, test_name)
}

/// Snapshot locks CLI public surface to prevent accidental exports.
#[test]
fn cli_public_surface_is_snapshotted() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib = manifest_dir.join("src").join("lib.rs");
    let content = fs::read_to_string(lib).expect("read lib.rs");
    let mut snapshot = String::new();
    for line in content.lines() {
        if line.trim_start().starts_with("pub mod") {
            snapshot.push_str(line);
            snapshot.push('\n');
        }
    }
    let name = snapshot_name("schemas", "public_surface");
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(manifest_dir.join("tests").join("snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(name, snapshot);
    });
}
