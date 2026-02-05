use std::path::Path;

use anyhow::Result;

#[test]
fn core_scope_only_allows_contracts_and_primitives() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let allow_dirs = ["contract", "plan", "primitives"];
    let allow_files = [
        "lib.rs",
        "boundaries.md",
        "explain.rs",
        "metrics.rs",
        "metrics_registry.rs",
        "run_index.rs",
    ];

    for entry in std::fs::read_dir(&src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            assert!(
                allow_dirs.contains(&name.as_str()),
                "core scope violation: unexpected dir {name}"
            );
            continue;
        }
        if path.is_file() {
            assert!(
                allow_files.contains(&name.as_str()),
                "core scope violation: unexpected file {name}"
            );
        }
    }
    Ok(())
}
