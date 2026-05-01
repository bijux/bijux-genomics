use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn pipelines_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );

    let src_entries = dir_entries(&root.join("src"));
    for required in [
        "bam/",
        "contract/",
        "cross.rs",
        "defaults/",
        "fastq.rs",
        "lib.rs",
        "public_api/",
        "registry/",
        "vcf/",
    ] {
        assert!(
            src_entries.contains(required),
            "src tree must include required pipelines surface: {required}"
        );
    }

    let contract_entries = dir_entries(&root.join("src/contract"));
    for required in [
        "OWNER.toml",
        "effective_defaults.rs",
        "invariants.rs",
        "mod.rs",
        "pipeline_capabilities.rs",
        "profile.rs",
        "profile_manifest.rs",
        "stable_surface.rs",
        "vocabulary.rs",
        "workflow_template.rs",
    ] {
        assert!(
            contract_entries.contains(required),
            "contract namespace must include required contract surface: {required}"
        );
    }

    let defaults_entries = dir_entries(&root.join("src/defaults"));
    for required in [
        "OWNER.toml",
        "default_params.rs",
        "empty_params.rs",
        "ledger.rs",
        "mod.rs",
        "stable_surface.rs",
    ] {
        assert!(
            defaults_entries.contains(required),
            "defaults namespace must include required defaults surface: {required}"
        );
    }

    let registry_entries = dir_entries(&root.join("src/registry"));
    for required in ["OWNER.toml", "mod.rs", "pipeline_id.rs", "stable_surface.rs"] {
        assert!(
            registry_entries.contains(required),
            "registry namespace must include required identity surface: {required}"
        );
    }

    let public_api_entries = dir_entries(&root.join("src/public_api"));
    assert_eq!(
        public_api_entries,
        entries(["OWNER.toml", "mod.rs", "stable_surface.rs"]),
        "public api namespace must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "guardrails.rs",
            "invariant_fast.rs",
            "snapshots/",
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
