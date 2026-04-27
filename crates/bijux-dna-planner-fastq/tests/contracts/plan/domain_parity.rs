use anyhow::Result;
use bijux_dna_domain_fastq::FASTQ_STAGE_ID_CATALOG;
use bijux_dna_domain_fastq::{default_amplicon_preprocess_stage_order, FastqPipelineMode};
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
        return Err(anyhow::anyhow!("domain/fastq/index.yaml missing or empty stage_ids"));
    }
    Ok(out)
}

#[test]
fn fastq_domain_yaml_matches_rust_stage_catalog() -> Result<()> {
    let yaml = domain_fastq_stage_ids()?;
    let rust = FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage_id| (*stage_id).to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(yaml, rust, "domain fastq stage IDs drifted from Rust catalog");
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
        "fastq.infer_asvs",
        "fastq.cluster_otus",
        "fastq.normalize_abundance",
    ] {
        assert!(stages.contains(required), "planner registry missing stage {required}");
    }
}

#[test]
fn fastq_planner_registry_exposes_stage_semantics() {
    let stages = bijux_dna_planner_fastq::stage_api::fastq::registry();
    assert!(
        stages.iter().all(|stage| stage.version().0 > 0),
        "registered stages must expose nonzero contract versions"
    );
    let mut read_count_mutating = stages
        .iter()
        .filter(|stage| stage.affects_read_counts())
        .map(|stage| stage.id().to_string())
        .collect::<BTreeSet<_>>();
    for required in [
        "fastq.trim_reads",
        "fastq.filter_reads",
        "fastq.merge_pairs",
        "fastq.remove_duplicates",
        "fastq.extract_umis",
    ] {
        assert!(
            read_count_mutating.remove(required),
            "planner registry must mark {required} as read-count affecting"
        );
    }
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
            spec.ordered_stage_ids().iter().any(|stage| stage == required),
            "amplicon mode missing stage {required}"
        );
    }
    assert!(
        !spec.ordered_stage_ids().iter().any(|stage| stage == "fastq.infer_asvs"),
        "default amplicon mode must not schedule optional infer_asvs branches by default"
    );

    let expected = default_amplicon_preprocess_stage_order()
        .into_iter()
        .filter(|stage| stage.as_str() != "fastq.screen_taxonomy")
        .map(|stage| stage.to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        spec.ordered_stage_ids(),
        expected,
        "planner default amplicon order drifted from domain canonical order"
    );
}

#[test]
fn single_end_default_pipeline_uses_contract_essentials_only() {
    let spec = default_pipeline_spec(DefaultPipelineOptions::default());
    let ordered = spec.ordered_stage_ids();

    assert!(
        !ordered.iter().any(|stage| stage == "fastq.merge_pairs"),
        "single-end default pipeline must not schedule paired merge"
    );
    assert!(
        !ordered.iter().any(|stage| stage == "fastq.extract_umis"),
        "single-end default pipeline must not schedule paired UMI extraction"
    );

    let expected = vec![
        "fastq.validate_reads",
        "fastq.profile_read_lengths",
        "fastq.detect_adapters",
        "fastq.trim_polyg_tails",
        "fastq.trim_reads",
        "fastq.filter_reads",
        "fastq.profile_reads",
        "fastq.profile_overrepresented_sequences",
        "fastq.report_qc",
    ];
    assert_eq!(
        ordered, expected,
        "single-end default pipeline must come from FASTQ pipeline-contract essentials"
    );
    assert!(
        spec.declares_graph_topology(),
        "default preprocess pipeline should declare explicit graph topology"
    );
    assert!(
        spec.edges
            .iter()
            .any(|edge| edge.from == "fastq.filter_reads" && edge.to == "fastq.profile_reads"),
        "default preprocess graph must branch filtered reads into profile_reads"
    );
    assert!(
        spec.edges.iter().any(|edge| {
            edge.from == "fastq.filter_reads"
                && edge.to == "fastq.profile_overrepresented_sequences"
        }),
        "default preprocess graph must branch filtered reads into overrepresented-sequence profiling"
    );
    let report_qc_inputs = spec
        .edges
        .iter()
        .filter(|edge| edge.to == "fastq.report_qc")
        .map(|edge| edge.from.clone())
        .collect::<BTreeSet<_>>();
    for contributor in [
        "fastq.validate_reads",
        "fastq.profile_read_lengths",
        "fastq.detect_adapters",
        "fastq.trim_reads",
        "fastq.profile_reads",
        "fastq.profile_overrepresented_sequences",
    ] {
        assert!(
            report_qc_inputs.contains(contributor),
            "default preprocess graph must join {contributor} into report_qc"
        );
    }
}

#[test]
fn single_end_default_pipeline_can_enable_error_correction() {
    let spec = default_pipeline_spec(DefaultPipelineOptions {
        paired: false,
        enable_merge: false,
        enable_correct: true,
        enable_qc_post: true,
        enable_screen: false,
        mode: FastqPipelineMode::Shotgun,
    });
    let ordered = spec.ordered_stage_ids();

    assert!(
        ordered.iter().any(|stage| stage == "fastq.correct_errors"),
        "single-end default pipeline must admit fastq.correct_errors when error correction is enabled"
    );
    assert!(
        !ordered.iter().any(|stage| stage == "fastq.merge_pairs"),
        "single-end default pipeline must still exclude paired-only merge"
    );
    assert!(
        !ordered.iter().any(|stage| stage == "fastq.extract_umis"),
        "single-end default pipeline must still exclude paired-only UMI extraction"
    );
    assert!(
        spec.edges
            .iter()
            .any(|edge| edge.from == "fastq.filter_reads" && edge.to == "fastq.correct_errors"),
        "single-end default graph must route filtered reads into error correction"
    );
    assert!(
        spec.edges
            .iter()
            .any(|edge| edge.from == "fastq.correct_errors" && edge.to == "fastq.profile_reads"),
        "single-end default graph must continue from error correction into downstream profiling"
    );
}
