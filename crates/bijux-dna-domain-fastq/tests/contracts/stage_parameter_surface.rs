use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn stage_manifest(stage_name: &str) -> Result<Value> {
    let path = workspace_root()?.join(format!("domain/fastq/stages/{stage_name}.yaml"));
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

fn stage_parameter_names(stage_name: &str) -> Result<Vec<String>> {
    let manifest = stage_manifest(stage_name)?;
    let Some(parameters) = manifest.get("parameters") else {
        return Ok(Vec::new());
    };
    parameters
        .as_array()
        .with_context(|| format!("parameters must be a sequence in {stage_name}.yaml"))?
        .iter()
        .map(|entry| {
            entry
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .with_context(|| format!("parameter name missing in {stage_name}.yaml"))
        })
        .collect()
}

fn stage_metric_names(stage_name: &str) -> Result<Vec<String>> {
    let manifest = stage_manifest(stage_name)?;
    let Some(metrics) = manifest.get("metrics") else {
        return Ok(Vec::new());
    };
    metrics
        .as_array()
        .with_context(|| format!("metrics must be a sequence in {stage_name}.yaml"))?
        .iter()
        .map(|entry| {
            entry
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .with_context(|| format!("metric name missing in {stage_name}.yaml"))
        })
        .collect()
}

#[test]
fn trim_reads_manifest_exposes_stage_level_cleanup_policy_surface() -> Result<()> {
    assert_eq!(
        stage_parameter_names("trim_reads")?,
        vec![
            "threads",
            "min_length",
            "quality_cutoff",
            "adapter_policy",
            "polyx_policy",
            "n_policy",
            "contaminant_policy",
        ],
        "fastq.trim_reads manifest must surface the same cleanup policy dimensions used by the typed trim contract"
    );
    Ok(())
}

#[test]
fn correct_errors_manifest_exposes_typed_error_correction_controls() -> Result<()> {
    assert_eq!(
        stage_parameter_names("correct_errors")?,
        vec![
            "threads",
            "quality_encoding",
            "kmer_size",
            "musket_kmer_budget",
            "genome_size",
            "max_memory_gb",
            "trusted_kmer_artifact",
            "conservative_mode",
        ],
        "fastq.correct_errors manifest must expose the typed correction controls carried through governed planning"
    );
    Ok(())
}

#[test]
fn cleanup_stage_manifests_keep_distinct_parameter_surfaces() -> Result<()> {
    assert_eq!(
        stage_parameter_names("trim_terminal_damage")?,
        vec![
            "damage_mode",
            "execution_policy",
            "trim_5p_bases",
            "trim_3p_bases",
        ],
        "fastq.trim_terminal_damage must expose its damage-policy selector and trim controls together"
    );
    assert_eq!(
        stage_parameter_names("trim_polyg_tails")?,
        vec!["threads", "trim_polyg", "min_polyg_run"],
        "fastq.trim_polyg_tails must keep its polyG-specific parameter surface"
    );
    assert_eq!(
        stage_parameter_names("remove_duplicates")?,
        vec!["threads", "dedup_mode", "keep_order"],
        "fastq.remove_duplicates must expose the governed duplicate semantics used by benchmark cohorts and stage plans"
    );
    assert_eq!(
        stage_parameter_names("merge_pairs")?,
        vec!["threads", "merge_overlap", "min_len", "unmerged_read_policy"],
        "fastq.merge_pairs must expose worker threads together with overlap and unmerged-read policy controls"
    );
    Ok(())
}

#[test]
fn remove_duplicates_manifest_publishes_governed_dedup_metric_surface() -> Result<()> {
    let metrics = stage_metric_names("remove_duplicates")?;
    for metric in [
        "threads",
        "dedup_mode",
        "keep_order",
        "reads_in",
        "reads_out",
        "duplicates_removed",
        "dedup_rate",
        "duplicate_class_count",
    ] {
        assert!(
            metrics.iter().any(|candidate| candidate == metric),
            "fastq.remove_duplicates manifest must publish governed duplicate metric `{metric}`",
        );
    }
    Ok(())
}

#[test]
fn filter_reads_manifest_exposes_only_supported_governed_controls() -> Result<()> {
    assert_eq!(
        stage_parameter_names("filter_reads")?,
        vec![
            "threads",
            "max_n",
            "max_n_fraction",
            "max_n_count",
            "low_complexity_threshold",
            "entropy_threshold",
            "kmer_ref",
            "polyx_policy",
        ],
        "fastq.filter_reads must expose only the governed filter controls that stage planning and benchmarking actually honor"
    );
    Ok(())
}

#[test]
fn report_qc_manifest_avoids_unmapped_runtime_knobs() -> Result<()> {
    assert_eq!(
        stage_parameter_names("report_qc")?,
        vec!["aggregation_engine", "aggregation_scope"],
        "fastq.report_qc must expose the governed aggregation surface used to plan and compare multiqc runs"
    );
    Ok(())
}

#[test]
fn report_qc_manifest_publishes_governed_qc_metric_surface() -> Result<()> {
    let metrics = stage_metric_names("report_qc")?;
    for metric in [
        "aggregation_engine",
        "aggregation_scope",
        "governed_qc_input_count",
        "governed_qc_contributor_stage_ids",
        "governed_qc_contributor_tool_ids",
        "governed_qc_lineage_hash",
        "multiqc_sample_count",
        "multiqc_module_count",
        "adapter_content_max",
        "duplication_rate",
        "overrepresented_sequence_count",
    ] {
        assert!(
            metrics.iter().any(|candidate| candidate == metric),
            "fastq.report_qc manifest must publish governed QC metric `{metric}`",
        );
    }
    Ok(())
}

#[test]
fn amplicon_stage_manifests_expose_governed_ecology_controls() -> Result<()> {
    assert_eq!(
        stage_parameter_names("normalize_primers")?,
        vec![
            "primer_set_id",
            "orientation_policy",
            "max_mismatch_rate",
            "min_overlap_bp",
            "strict_5p_anchor",
            "allow_iupac_codes",
        ],
        "fastq.normalize_primers must expose the governed primer-orientation and mismatch controls carried through planner and runtime contracts"
    );
    assert_eq!(
        stage_parameter_names("infer_asvs")?,
        vec!["denoising_method", "pooling_mode", "chimera_policy", "threads",],
        "fastq.infer_asvs must keep its denoising policy surface explicit in the stage manifest"
    );
    assert_eq!(
        stage_parameter_names("normalize_abundance")?,
        vec!["method"],
        "fastq.normalize_abundance must expose its normalization-method selector without hidden secondary knobs"
    );
    Ok(())
}

#[test]
fn validate_reads_manifest_avoids_unmapped_quality_cutoff() -> Result<()> {
    assert_eq!(
        stage_parameter_names("validate_reads")?,
        vec!["threads", "validation_mode", "pair_sync_policy"],
        "fastq.validate_reads must expose its governed validation controls while keeping q_cutoff out of the public manifest until backend-native mapping exists"
    );
    Ok(())
}
