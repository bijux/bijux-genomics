use insta::Settings;
use std::fs;
use std::path::PathBuf;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-api__{group}__{name}")
}

/// Snapshot locks API public surface for v1 cross endpoints.
#[test]
fn public_surface_is_snapshotted() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib = manifest_dir.join("src").join("lib.rs");
    let content = fs::read_to_string(&lib)
        .unwrap_or_else(|err| panic!("read lib.rs at {}: {err}", lib.display()));
    let mut snapshot = String::new();
    for line in content.lines() {
        if line.trim_start().starts_with("pub mod v1") {
            snapshot.push_str(line);
            snapshot.push('\n');
        }
    }
    let name = snapshot_name("schemas", "public_surface");
    let mut settings = Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots"));
    settings.bind(|| {
        insta::assert_snapshot!(name, bijux_testkit::snapshot_normalize_text(&snapshot));
    });
}
