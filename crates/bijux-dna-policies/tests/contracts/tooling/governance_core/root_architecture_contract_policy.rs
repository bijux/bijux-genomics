#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| bijux_dna_policies::policy_panic!("resolve repository root"))
        .to_path_buf()
}

#[test]
fn policy__contracts__root_architecture_contract_policy__root_contract_declares_reviewable_authority(
) {
    let root = repo_root();
    let path = root.join("docs/10-architecture/ARCHITECTURE_CONTRACT.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("read {}: {err}", path.display()));
    let required = [
        "Owner: Architecture",
        "Scope: Repository root architecture map and boundary authority",
        "Contract version: v1",
        "## Allowed inputs",
        "## Forbidden dependencies",
        "## Forbidden effects",
        "## Validation commands",
        "docs/10-architecture/BOUNDARY_MAP.md",
        "TARGET_ARCHITECTURE.md",
        "docs/10-architecture/CONTRACT_SPINE.md",
        "cargo test -p bijux-dna-policies --test boundaries",
        "cargo test -p bijux-dna-policies --test contracts",
    ];

    let missing =
        required.into_iter().filter(|needle| !content.contains(needle)).collect::<Vec<_>>();

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "root architecture contract is missing required authority fields:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__contracts__root_architecture_contract_policy__target_architecture_assigns_product_owners(
) {
    let root = repo_root();
    let path = root.join("docs/10-architecture/TARGET_ARCHITECTURE.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("read {}: {err}", path.display()));
    let required = [
        "Status: Normative",
        "## Product Boundary",
        "## Exclusive Product Owners",
        "## Generic Stage Lifecycle",
        "## Non-Waiting Control Flow",
        "## Identity and Publication",
        "## Licensed External Artifacts",
        "## Repository Publication Surfaces",
        "`bijux-dna-bench-model`",
        "`bijux-dna-bench`",
        "`bijux-dna-runtime`",
        "`bijux-dna-runner`",
        "`bijux-dna-containers`",
        "`bijux-dna-containers-qa`",
        "`bijux-dna-stage-contract`",
        "`bijux-dna-science`",
        "`bijux-dna-policies`",
        "`bijux-dna`",
        "`bijux-dna-dev`",
        "`ArtifactContract`",
        "`StageContract`",
        "`ToolBinding`",
        "`ParameterProjection`",
        "`ResultNormalizer`",
        "`DownstreamProbe`",
    ];
    let missing =
        required.into_iter().filter(|needle| !content.contains(needle)).collect::<Vec<_>>();

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "target architecture is missing required product boundaries:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__contracts__root_architecture_contract_policy__data_portfolio_separates_sources_from_evidence(
) {
    let root = repo_root();
    let path = root.join("docs/20-science/DATA_PORTFOLIO_AND_TRUTH_CONTRACT.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("read {}: {err}", path.display()));
    let required = [
        "Status: Normative",
        "## Ownership",
        "## Evidence Classes",
        "## Required Identity Closure",
        "## Scientific Fitness",
        "## Truth and Claim Rules",
        "## Materialization and Admission",
        "## Publication and Replay",
        "`bijux-dna-bench-model`",
        "`bijux-dna-bench`",
        "`bijux-dna-science`",
        "`bijux-dna-policies`",
        "source_defined",
        "curated_truth",
        "masked_truth",
        "orthogonal_expectation",
        "observational_performance_only",
        "tool agreement is not truth",
        "identity-addressed STUDY",
        "benchmarks/contracts/portfolio.toml",
        "benchmarks/contracts/data-fitness.toml",
    ];
    let missing =
        required.into_iter().filter(|needle| !content.contains(needle)).collect::<Vec<_>>();

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "data portfolio and truth contract is missing required evidence boundaries:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__contracts__root_architecture_contract_policy__suite_run_manifest_has_one_model_authority(
) {
    let root = repo_root();
    let owner_path = root.join("crates/bijux-dna-bench-model/src/model/suite/manifest.rs");
    let consumer_path = root
        .join("crates/bijux-dna/src/commands/benchmark/suite/suite_sections/spec_and_loading.rs");
    let owner = std::fs::read_to_string(&owner_path).unwrap_or_else(|err| {
        bijux_dna_policies::policy_panic!("read {}: {err}", owner_path.display())
    });
    let consumer = std::fs::read_to_string(&consumer_path).unwrap_or_else(|err| {
        bijux_dna_policies::policy_panic!("read {}: {err}", consumer_path.display())
    });

    let authority_declaration = ["pub struct ", "SuiteRunManifest"].concat();
    bijux_dna_policies::policy_assert!(
        owner.contains(&authority_declaration),
        "bijux-dna-bench-model must own the persisted suite run manifest"
    );
    let mut authorities = walkdir::WalkDir::new(root.join("crates"))
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("rs"))
        .filter_map(|entry| {
            std::fs::read_to_string(entry.path())
                .ok()
                .filter(|source| source.contains(&authority_declaration))
                .and_then(|_| entry.path().strip_prefix(&root).ok().map(Path::to_path_buf))
        })
        .collect::<Vec<_>>();
    authorities.sort();
    bijux_dna_policies::policy_assert!(
        authorities == [PathBuf::from("crates/bijux-dna-bench-model/src/model/suite/manifest.rs")],
        "persisted suite run manifest authorities drifted: {authorities:?}"
    );
    bijux_dna_policies::policy_assert!(
        consumer.contains("use bijux_dna_api::v1::api::bench::{")
            && consumer.contains("SuiteRunManifest")
            && consumer.contains("ToolInvocationRow"),
        "the thin bijux-dna CLI must consume owner types through the documented API adapter"
    );
}
