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
    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("pub mod") {
            snapshot.push_str(line);
            snapshot.push('\n');
            continue;
        }
        if trimmed.starts_with("pub use") {
            let mut block = line.to_string();
            while !block.trim_end().ends_with(';') {
                if let Some(next) = lines.next() {
                    block.push(' ');
                    block.push_str(next.trim());
                } else {
                    break;
                }
            }
            snapshot.push_str(&block);
            snapshot.push('\n');
        }
    }
    let name = bijux_dna_testkit::snapshot_name("schemas", "public_surface");
    insta::assert_snapshot!(name, bijux_dna_testkit::snapshot_normalize_text(&snapshot));
}
