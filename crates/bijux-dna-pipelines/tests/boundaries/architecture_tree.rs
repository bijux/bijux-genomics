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
            "profiles/",
            "required_stages.rs",
        ]),
        "fastq-to-bam cross namespace must keep defaults, profiles, and required stages separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/cross/fastq_to_bam/profiles")),
        entries(["ancient_dna_profile.rs", "default_profile.rs", "mod.rs"]),
        "fastq-to-bam profile namespace must separate modern and ancient-dna profile families"
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
            "catalog/",
            "families/",
            "mod.rs",
            "pipeline_id.rs",
            "profile_lookup.rs",
        ]),
        "registry namespace must keep identity, collections, and lookups separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/registry/catalog")),
        entries(["OWNER.toml", "mod.rs", "queries/"]),
        "registry catalog namespace must keep assembly and query behavior separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/registry/catalog/queries")),
        entries(["domain_queries.rs", "mod.rs", "stability_filter.rs"]),
        "registry catalog query namespace must separate stability and domain filters"
    );

    assert_eq!(
        dir_entries(&root.join("src/registry/families")),
        entries([
            "OWNER.toml",
            "bam.rs",
            "cross.rs",
            "fastq.rs",
            "mod.rs",
            "vcf.rs"
        ]),
        "registry family namespace must stay partitioned by domain"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq")),
        entries(["defaults/", "invariants/", "mod.rs", "profiles/"]),
        "fastq namespace must stay partitioned by defaults, profiles, and invariants"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/defaults")),
        entries([
            "OWNER.toml",
            "adna.rs",
            "analysis_params.rs",
            "analysis_tools.rs",
            "mod.rs",
            "param_defaults.rs",
            "preprocess_params.rs",
            "preprocess_tools.rs",
            "rationales.rs",
            "reference_adna.rs",
            "stage_order.rs",
            "tooling.rs",
        ]),
        "fastq defaults namespace must keep preprocess vs analysis defaults and rationale assembly separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/profiles")),
        entries([
            "OWNER.toml",
            "ancient_dna_profiles.rs",
            "baseline_profiles.rs",
            "catalog.rs",
            "contract_templates.rs",
            "mod.rs"
        ]),
        "fastq profiles namespace must keep baseline and ancient-dna families separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/invariants")),
        entries([
            "OWNER.toml",
            "mod.rs",
            "preset_rules/",
            "report.rs",
            "required_rules.rs",
            "stage_params.rs"
        ]),
        "fastq invariants namespace must keep report contracts, stage params, and rule families separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/invariants/preset_rules")),
        entries(["ancient_dna_rules.rs", "mod.rs", "reference_adna_rules.rs"]),
        "fastq preset invariant namespace must separate ancient-dna and reference-grade rules"
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
