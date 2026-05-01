use std::fs;
use std::path::PathBuf;

use bijux_dna_core::contract::PipelineDomain;

#[test]
fn public_surface_is_constrained() -> anyhow::Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("lib.rs");
    let source = fs::read_to_string(lib_path)?;
    let mut pub_mods = Vec::new();
    let mut pub_use_lines = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split([';', ' ']).next() {
                pub_mods.push(name.to_string());
            }
        }
        if trimmed.starts_with("pub use ") {
            pub_use_lines.push(trimmed.to_string());
        }
    }

    let allowed_mods = [
        "bench_repository",
        "banks",
        "execution_support",
        "invariants",
        "metrics",
        "observer",
        "params",
        "prelude",
        "pipeline_contract",
        "run",
        "stages",
        "stage_contract",
        "id_catalog",
        "stage_semantics",
        "stage_specs",
        "types",
    ];
    for name in &pub_mods {
        assert!(allowed_mods.contains(&name.as_str()), "unexpected public module: {name}");
    }
    let allowed_substrings = [
        "adapter_bank",
        "banks",
        "contaminant_bank",
        "bench_repository",
        "cluster_otus_artifacts",
        "artifacts::{",
        "comparison_contract",
        "chunking",
        "FastqDomain",
        "execution_support",
        "filter_policy_matrix",
        "integration_matrix",
        "metrics",
        "observer::",
        "polyx_bank",
        "prelude",
        "stages",
        "stage_contract",
        "id_catalog",
        "stage_semantics",
        "stage_specs",
        "pipeline_contract",
        "types",
        "run",
        "params",
        "invariants",
        "observer_contract",
        "qc_contract",
        "stage_tool_governance",
        "RawFailure",
    ];
    for line in &pub_use_lines {
        assert!(
            allowed_substrings.iter().any(|token| line.contains(token)),
            "unexpected public re-export: {line}"
        );
    }
    Ok(())
}

#[test]
fn fastq_domain_adapter_exposes_pipeline_contract() {
    assert_eq!(bijux_dna_domain_fastq::FastqDomain::domain_id(), "fastq");
    assert!(
        !bijux_dna_domain_fastq::FastqDomain::canonical_pipeline().ordered_stage_ids().is_empty(),
        "domain adapter must expose the canonical FASTQ pipeline"
    );
}
