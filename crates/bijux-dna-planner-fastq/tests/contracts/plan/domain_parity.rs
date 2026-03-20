use anyhow::Result;
use bijux_dna_domain_fastq::{canonical_amplicon_stage_order, FastqPipelineMode};
use bijux_dna_domain_fastq::FASTQ_STAGE_ID_CATALOG;
use bijux_dna_planner_fastq::{default_pipeline_spec, DefaultPipelineOptions};
use std::collections::BTreeSet;

fn domain_fastq_stage_ids() -> Result<BTreeSet<String>> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("workspace root"))?;
    let path = root.join("domain/fastq/index.yaml");
    let raw = std::fs::read_to_string(path)?;
    let mut in_stage_ids = false;
    let mut out = BTreeSet::new();
    for line in raw.lines() {
        if line.trim() == "stage_ids:" {
            in_stage_ids = true;
            continue;
        }
        if in_stage_ids {
            if !line.starts_with("  - ") {
                break;
            }
            out.insert(line.trim_start_matches("  - ").trim().to_string());
        }
    }
    if out.is_empty() {
        return Err(anyhow::anyhow!(
            "domain/fastq/index.yaml missing or empty stage_ids"
        ));
    }
    Ok(out)
}

#[test]
fn fastq_domain_yaml_matches_rust_stage_catalog() -> Result<()> {
    let yaml = domain_fastq_stage_ids()?;
    let rust = FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|s| s.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        yaml, rust,
        "domain fastq stage IDs drifted from Rust catalog"
    );
    Ok(())
}

#[test]
fn fastq_planner_registry_covers_new_amplicon_stages() {
    let stages = bijux_dna_planner_fastq::stage_api::fastq::registry()
        .into_iter()
        .map(|s| s.id().to_string())
        .collect::<BTreeSet<_>>();
    assert!(
        !stages.contains("fastq.preprocess"),
        "planner registry must not expose pseudo-stage fastq.preprocess"
    );
    for required in [
        "fastq.trim_terminal_damage",
        "fastq.normalize_primers",
        "fastq.remove_chimeras",
        "fastq.cluster_otus",
        "fastq.normalize_abundance",
    ] {
        assert!(
            stages.contains(required),
            "planner registry missing stage {required}"
        );
    }
    assert!(
        !stages.contains("fastq.infer_asvs"),
        "planner registry must not expose declared-only ASV inference",
    );
}

#[test]
fn amplicon_mode_pipeline_emits_amplicon_stages() {
    let spec = default_pipeline_spec(DefaultPipelineOptions {
        paired: false,
        enable_merge: false,
        enable_correct: false,
        enable_qc_post: true,
        enable_screen: false,
        mode: FastqPipelineMode::Amplicon,
    });
    for required in [
        "fastq.trim_terminal_damage",
        "fastq.normalize_primers",
        "fastq.remove_chimeras",
        "fastq.cluster_otus",
        "fastq.normalize_abundance",
    ] {
        assert!(
            spec.stages.iter().any(|stage| stage == required),
            "amplicon mode missing stage {required}"
        );
    }
    assert!(
        !spec.stages.iter().any(|stage| stage == "fastq.infer_asvs"),
        "default amplicon mode must not schedule planned ASV inference by default"
    );

    let expected = canonical_amplicon_stage_order()
        .into_iter()
        .filter(|stage| stage.as_str() != "fastq.screen_taxonomy")
        .map(|stage| stage.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        spec.stages, expected,
        "planner default amplicon order drifted from domain canonical order"
    );
}

#[test]
fn single_end_default_pipeline_uses_contract_essentials_only() {
    let spec = default_pipeline_spec(DefaultPipelineOptions::default());

    assert!(
        !spec.stages.iter().any(|stage| stage == "fastq.merge_pairs"),
        "single-end default pipeline must not schedule paired merge"
    );
    assert!(
        !spec.stages.iter().any(|stage| stage == "fastq.correct_errors"),
        "single-end default pipeline must not schedule paired correction"
    );
    assert!(
        !spec.stages.iter().any(|stage| stage == "fastq.extract_umis"),
        "single-end default pipeline must not schedule paired UMI extraction"
    );

    let expected = vec![
        "fastq.validate_reads",
        "fastq.profile_read_lengths",
        "fastq.detect_adapters",
        "fastq.trim_polyg_tails",
        "fastq.trim_terminal_damage",
        "fastq.trim_reads",
        "fastq.filter_reads",
        "fastq.profile_reads",
        "fastq.profile_overrepresented_sequences",
        "fastq.report_qc",
    ];
    assert_eq!(
        spec.stages, expected,
        "single-end default pipeline must come from FASTQ pipeline-contract essentials"
    );
    assert!(
        spec.declares_graph_topology(),
        "default preprocess pipeline should declare explicit graph topology"
    );
}
