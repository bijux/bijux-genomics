use insta::Settings;
use std::fs;

fn snapshot_name(group: &str, name: &str) -> String {
    format!("bijux-dna-api__{group}__{name}")
}

/// Snapshot locks API public surface for v1 cross endpoints.
#[test]
fn public_surface_is_snapshotted() {
    let lib = crate::support::crate_src("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate src: {err}"))
        .join("lib.rs");
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
    settings.set_snapshot_path(
        crate::support::crate_snapshots("bijux-dna-api")
            .unwrap_or_else(|err| panic!("resolve snapshots root: {err}")),
    );
    settings.bind(|| {
        insta::assert_snapshot!(name, bijux_dna_testkit::snapshot_normalize_text(&snapshot));
    });
}
