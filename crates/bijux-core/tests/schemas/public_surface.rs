use std::fs;
use std::path::PathBuf;

use insta::Settings;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-core__{group}__{name}")
}

/// Snapshot locks core public surface exports.
#[test]
fn public_surface_is_snapshotted() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let lib = manifest_dir.join("src").join("lib.rs");
    let content = fs::read_to_string(lib).expect("read lib.rs");
    let mut snapshot = String::new();
    for line in content.lines() {
        if line.trim_start().starts_with("pub mod") || line.trim_start().starts_with("pub use") {
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
