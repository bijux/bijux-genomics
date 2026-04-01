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
            "effective_defaults.rs",
            "invariants.rs",
            "mod.rs",
            "pipeline_capabilities.rs",
            "profile.rs",
            "profile_manifest.rs",
            "projections/",
            "stable_surface.rs",
            "vocabulary.rs",
        ]),
        "contract namespace must stay partitioned by concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/defaults")),
        entries([
            "OWNER.toml",
            "default_params.rs",
            "empty_params.rs",
            "ledger.rs",
            "merge/",
            "mod.rs",
            "serde_codec/",
            "stable_surface.rs",
        ]),
        "defaults namespace must keep ledgers, envelopes, and merge logic separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/projections")),
        entries([
            "contract_projection.rs",
            "defaults_ledger.rs",
            "manifest_projection.rs",
            "mod.rs",
        ]),
        "contract projection namespace must separate manifests, defaults ledgers, and pipeline contract projections"
    );

    assert_eq!(
        dir_entries(&root.join("src/defaults/merge")),
        entries(["mod.rs", "override_application.rs", "validation.rs"]),
        "defaults merge namespace must separate orchestration, override application, and validation"
    );

    assert_eq!(
        dir_entries(&root.join("src/defaults/serde_codec")),
        entries(["deserialize.rs", "mod.rs", "serialize.rs"]),
        "defaults serde namespace must separate serialization from deserialization"
    );

    assert_eq!(
        dir_entries(&root.join("src/cross/fastq_to_bam")),
        entries([
            "OWNER.toml",
            "merged_defaults.rs",
            "mod.rs",
            "profiles/",
            "required_stages.rs",
            "source_profiles.rs",
            "stable_surface.rs",
        ]),
        "fastq-to-bam cross namespace must keep source profiles, merged defaults, profiles, and required stages separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/cross/fastq_to_bam/profiles")),
        entries(["ancient_dna_profile.rs", "default_profile.rs", "mod.rs"]),
        "fastq-to-bam profile namespace must separate modern and ancient-dna profile families"
    );

    assert_eq!(
        dir_entries(&root.join("src/registry/profile_lookup")),
        entries(["cross.rs", "lookup_entry.rs", "mod.rs", "vcf.rs"]),
        "registry profile lookup namespace must separate domain dispatch from concrete families"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "mod.rs", "stable_surface.rs"]),
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
            "profile_lookup/",
            "stable_surface.rs",
        ]),
        "registry namespace must keep identity, collections, and lookups separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/registry/catalog")),
        entries(["OWNER.toml", "mod.rs", "pipeline_registry.rs", "queries/"]),
        "registry catalog namespace must keep assembly and query behavior separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/registry/catalog/queries")),
        entries([
            "mod.rs",
            "profiles_by_domain.rs",
            "profiles_by_stability.rs"
        ]),
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
            "adna/",
            "analysis_params.rs",
            "analysis_tools.rs",
            "defaults_assembly.rs",
            "mod.rs",
            "parameter_defaults.rs",
            "preprocess_params.rs",
            "preprocess_tools.rs",
            "rationales.rs",
            "reference_adna/",
            "stage_order.rs",
            "tool_defaults.rs",
        ]),
        "fastq defaults namespace must keep preprocess vs analysis defaults and rationale assembly separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/defaults/adna")),
        entries([
            "mod.rs",
            "parameter_overrides.rs",
            "rationale_overrides.rs",
            "tool_overrides.rs"
        ]),
        "adna fastq defaults namespace must separate tool, parameter, and rationale overrides"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/defaults/reference_adna")),
        entries(["mod.rs", "parameter_overrides.rs", "rationale_overrides.rs", "tool_overrides.rs"]),
        "reference-grade fastq defaults namespace must separate tool, parameter, and rationale overrides"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/profiles")),
        entries([
            "OWNER.toml",
            "adna_profile.rs",
            "default_profile.rs",
            "minimal_profile.rs",
            "mod.rs",
            "profile_by_id.rs",
            "profile_contracts/",
            "profile_ids.rs",
            "reference_adna_profile.rs",
            "stable_surface.rs"
        ]),
        "fastq profiles namespace must keep baseline and ancient-dna families separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/profiles/profile_contracts")),
        entries(["library_model.rs", "mod.rs", "pipeline_capabilities.rs"]),
        "fastq profile contract namespace must separate library models from pipeline capabilities"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/invariants")),
        entries([
            "OWNER.toml",
            "mod.rs",
            "preset_rules/",
            "stage_parameter_access.rs",
            "stage_scope.rs",
            "stage_requirements/",
            "validation_report_contracts.rs",
            "violation_builder.rs"
        ]),
        "fastq invariants namespace must keep report contracts, stage params, and rule families separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/invariants/stage_requirements")),
        entries([
            "mod.rs",
            "paired_library_rules.rs",
            "required_artifacts.rs",
            "required_params.rs",
            "required_stages.rs",
        ]),
        "fastq invariant requirement namespace must separate stage, param, artifact, and paired-library rules"
    );

    assert_eq!(
        dir_entries(&root.join("src/fastq/invariants/preset_rules")),
        entries(["adna_rules.rs", "mod.rs", "reference_adna_rules.rs"]),
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
