use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn policies_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries([
            "BOUNDARY.md",
            "Cargo.toml",
            "PUBLIC_API.md",
            "README.md",
            "docs/",
            "src/",
            "tests/",
        ]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "assertions/",
            "checks/",
            "guardrails/",
            "lib.rs",
            "policy_diagnostics/",
            "public_api/",
            "source_scan/",
        ]),
        "src tree must match the documented policy engine layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/assertions")),
        entries([
            "comparison_assertions.rs",
            "condition_assertions.rs",
            "mod.rs",
            "panic_assertion.rs",
        ]),
        "assertions tree must keep exported macro families partitioned by behavior"
    );

    assert_eq!(
        dir_entries(&root.join("src/checks")),
        entries([
            "OWNER.toml",
            "directory_layout.rs",
            "failure_policy.rs",
            "mod.rs",
            "module_files.rs",
            "public_surface.rs",
            "stable_surface.rs",
            "stage_id_literals.rs",
        ]),
        "checks tree must stay partitioned by rule family"
    );

    assert_eq!(
        dir_entries(&root.join("src/guardrails")),
        entries([
            "OWNER.toml",
            "baseline.rs",
            "configuration.rs",
            "mod.rs",
            "presets.rs",
            "runner.rs",
            "source_inventory.rs",
            "stable_surface.rs",
        ]),
        "guardrails tree must keep configuration, presets, and runner wiring separate"
    );

    assert_eq!(
        dir_entries(&root.join("src/policy_diagnostics")),
        entries([
            "OWNER.toml",
            "contracts.rs",
            "mod.rs",
            "render.rs",
            "stable_surface.rs",
        ]),
        "policy diagnostics tree must keep contracts and rendering separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "mod.rs", "stable_surface.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/source_scan")),
        entries([
            "OWNER.toml",
            "mod.rs",
            "rust_sources.rs",
            "stable_surface.rs"
        ]),
        "source scan tree must stay focused on deterministic Rust source discovery"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "README.md",
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "schemas/",
            "snapshots/",
            "support/",
        ]),
        "test tree must stay organized by enduring intent"
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
