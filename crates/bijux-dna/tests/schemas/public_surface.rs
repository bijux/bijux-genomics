use std::fs;

fn snapshot_name(bucket: &str, test_name: &str) -> String {
    format!("{}__{}__{}", env!("CARGO_PKG_NAME"), bucket, test_name)
}

/// Snapshot locks CLI public surface to prevent accidental exports.
#[test]
fn cli_public_surface_is_snapshotted() {
    let crate_root = crate::support::crate_root("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let lib = crate_root.join("src").join("lib.rs");
    let content = fs::read_to_string(&lib)
        .unwrap_or_else(|err| panic!("read lib.rs at {}: {err}", lib.display()));
    let mut snapshot = String::new();
    for line in content.lines() {
        if line.trim_start().starts_with("pub mod") {
            snapshot.push_str(line);
            snapshot.push('\n');
        }
    }
    let name = snapshot_name("schemas", "public_surface");
    let mut settings = insta::Settings::new();
    settings.set_snapshot_path(crate_root.join("tests").join("snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(name, snapshot);
    });
}
