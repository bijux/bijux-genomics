use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn pipelines_tree_matches_architecture_contract() {
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
            "bam/",
            "contract/",
            "cross/",
            "defaults/",
            "fastq/",
            "lib.rs",
            "public_api/",
            "registry/",
            "vcf/",
        ]),
        "src tree must match the documented pipelines layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract")),
        entries([
            "OWNER.toml",
            "capabilities.rs",
            "invariants.rs",
            "mod.rs",
            "profile.rs",
            "projections.rs",
        ]),
        "contract namespace must stay partitioned by concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/defaults")),
        entries([
            "OWNER.toml",
            "ledger.rs",
            "merge.rs",
            "mod.rs",
            "params.rs",
            "serde_codec.rs",
        ]),
        "defaults namespace must keep ledgers, envelopes, and merge logic separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/cross/fastq_to_bam")),
        entries([
            "OWNER.toml",
            "defaults.rs",
            "mod.rs",
            "profiles.rs",
            "required_stages.rs",
        ]),
        "fastq-to-bam cross namespace must keep defaults, profiles, and required stages separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "mod.rs"]),
        "public api namespace must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/registry")),
        entries([
            "OWNER.toml",
            "catalog.rs",
            "mod.rs",
            "pipeline_id.rs",
            "profile_collections.rs",
            "profile_lookup.rs",
        ]),
        "registry namespace must keep identity, collections, and lookups separated"
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
            "guardrails.rs",
            "invariant_fast.rs",
            "schemas/",
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
