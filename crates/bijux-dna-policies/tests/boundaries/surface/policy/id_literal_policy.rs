#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

const CAMPAIGN_IDENTITY_PATHS: &[&str] = &[
    "/crates/bijux-dna-bench-campaign/src/catalog.rs",
    "/crates/bijux-dna-bench-campaign/src/catalog/contracts.rs",
    "/crates/bijux-dna-bench-campaign/src/planning/manifest.rs",
    "/crates/bijux-dna-bench-campaign/src/cache/layout.rs",
    "/crates/bijux-dna-bench-campaign/src/campaign/service.rs",
    "/crates/bijux-dna-bench-campaign/src/campaign/samples.rs",
    "/crates/bijux-dna-bench-campaign/src/qc/prepare.rs",
    "/crates/bijux-dna-bench-campaign/src/qc/model.rs",
    "/crates/bijux-dna-bench-campaign/src/qc/contracts.rs",
];

// Pure campaign projections consume persisted stage identities owned by the domain catalog.
const BAM_CAMPAIGN_STAGE_IDENTITY_PATHS: &[&str] = &[
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/alignment.rs",
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/ancient_dna.rs",
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/contamination.rs",
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/filtering.rs",
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/identity.rs",
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/metrics.rs",
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/resources.rs",
    "/crates/bijux-dna-bench-model/src/policy/bam_campaign/validation.rs",
];

fn is_allowed_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    if path_str.contains("/crates/bijux-dna-stages-") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-domain-") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-pipelines/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-analyze/") {
        return true;
    }
    // Campaign registries and parameter projections consume persisted stage identities.
    // Keep the allowance at the owning records and projections, not the whole crate.

    if CAMPAIGN_IDENTITY_PATHS.iter().any(|suffix| path_str.ends_with(suffix)) {
        return true;
    }
    if BAM_CAMPAIGN_STAGE_IDENTITY_PATHS.iter().any(|suffix| path_str.ends_with(suffix)) {
        return true;
    }
    // Canonical parser family catalogs and the BAM genotyping wire producer own these identities.
    if path_str.contains("/crates/bijux-dna-bench-parser-proofs/src/catalog/")
        || path_str.ends_with("/crates/bijux-dna-bench-bam-proofs/src/parser_probes/genotyping.rs")
    {
        return true;
    }
    // Inventory domains own their persisted stage-prefix admission.
    if path_str.ends_with("/crates/bijux-dna-bench-stage/src/repository/domain.rs") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-bench/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-bench-model/src/model/normalized_result/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-api/")
        && !path_str.contains("/crates/bijux-dna-api/src/v1/")
    {
        return true;
    }
    // Compatibility domain modules retain the existing explicit catalog allowance.
    if path_str.ends_with("/crates/bijux-dna-api/src/v1/bam/domain.rs")
        || path_str.ends_with("/crates/bijux-dna-api/src/v1/fastq/domain.rs")
    {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-planner-fastq/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-planner-bam/") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-planner-vcf/") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-stage-contract/src/executor_registry.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-stage-contract/src/tool_binding/catalog.rs")
        || path_str.contains("/crates/bijux-dna-stage-contract/src/tool_binding/catalog/")
    {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-core/src/metrics/registry.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-core/src/ids.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-core/src/id_catalog.rs") {
        return true;
    }
    if path_str.contains("/crates/bijux-dna-core/src/id_catalog/stage/") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-runtime/src/manifests.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-runtime/src/manifests/classification.rs") {
        return true;
    }
    if path_str.ends_with("/crates/bijux-dna-stage-contract/src/stage_plugin.rs")
        || path_str.ends_with("/crates/bijux-dna-stage-contract/src/plan_run/mod.rs")
        || path_str.ends_with(
            "/crates/bijux-dna-dev/src/commands/ops/tooling/operator_workflow_maturity.rs",
        )
        || path_str.ends_with(
            "/crates/bijux-dna-dev/src/commands/ops/tooling/scientific_caveat_propagation.rs",
        )
        || path_str
            .ends_with("/crates/bijux-dna-dev/src/commands/ops/tooling/reference_external_data.rs")
        || path_str.ends_with("/crates/bijux-dna-bench-model/src/contract/records.rs")
    {
        return true;
    }
    if path_str.contains("/tests/") {
        return true;
    }
    false
}

fn canonical_stage_ids(root: &Path, prefixes: &[&str]) -> Vec<String> {
    let mut ids = Vec::new();
    let stage_catalog = root.join("crates/bijux-dna-core/src/id_catalog/stage");
    for entry in WalkDir::new(stage_catalog).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|suffix| suffix.to_str()) != Some("rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read stage catalog");
        ids.extend(
            content
                .split('"')
                .skip(1)
                .step_by(2)
                .filter(|literal| prefixes.iter().any(|prefix| literal.starts_with(prefix)))
                .map(str::to_owned),
        );
    }
    ids.sort_unstable();
    ids.dedup();
    ids
}

fn raw_stage_ids(content: &str, canonical_ids: &[String]) -> Vec<String> {
    canonical_ids
        .iter()
        .filter(|stage_id| content.contains(&format!("\"{stage_id}\"")))
        .cloned()
        .collect()
}

#[test]
fn policy__boundaries__id_literal_policy__raw_id_catalog_are_confined_to_registries() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let canonical_ids = canonical_stage_ids(&root, &["fastq.", "bam.", "cross.", "core."]);
    let api_ids = canonical_stage_ids(&root, &["fastq.", "bam.", "vcf.", "cross.", "core."]);
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if is_allowed_path(entry.path()) {
            continue;
        }
        let content = bijux_dna_testkit::workspace_support::strip_rust_test_modules(
            &std::fs::read_to_string(entry.path()).expect("read source"),
        );
        let ids = if entry.path().to_string_lossy().contains("/crates/bijux-dna-api/src/v1/") {
            &api_ids
        } else {
            &canonical_ids
        };
        for stage_id in raw_stage_ids(&content, ids) {
            offenders.push(format!(
                "{} contains raw stage id literal {stage_id:?}",
                entry.path().display()
            ));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "raw stage ids must be confined to registries/stage crates:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__id_literal_policy__file_extensions_are_not_stage_ids() {
    let ids = vec!["fastq.validate_reads".to_owned()];
    assert!(raw_stage_ids("let suffix = \"fastq.gz\";", &ids).is_empty());
    assert_eq!(raw_stage_ids("let stage = \"fastq.validate_reads\";", &ids), ids);
    assert!(
        raw_stage_ids("/// Stability: stable v1 adapter over submission ports.", &ids).is_empty()
    );
    assert!(!is_allowed_path(Path::new("/repo/crates/bijux-dna-api/src/v1/run/hpc_submission.rs")));
    assert!(is_allowed_path(Path::new("/repo/crates/bijux-dna-api/src/v1/fastq/domain.rs")));
    assert!(is_allowed_path(Path::new("/repo/crates/bijux-dna-api/src/v1/bam/domain.rs")));
    assert!(is_allowed_path(Path::new(
        "/repo/crates/bijux-dna-bench-model/src/policy/bam_campaign/alignment.rs"
    )));
    for path in [
        "/repo/crates/bijux-dna-bench-model/src/policy/bam_campaign/alignment.rs.disabled",
        "/repo/crates/bijux-dna-bench-model/src/policy/bam_campaign_neighbor/alignment.rs",
        "/repo/crates/bijux-dna-bench-model/src/policy/unrelated.rs",
    ] {
        assert!(!is_allowed_path(Path::new(path)));
    }
}
