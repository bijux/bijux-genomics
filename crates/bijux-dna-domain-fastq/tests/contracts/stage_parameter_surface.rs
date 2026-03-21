use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yaml::Value;

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
    serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn stage_parameter_names(stage_name: &str) -> Result<Vec<String>> {
    let manifest = stage_manifest(stage_name)?;
    let Some(parameters) = manifest.get("parameters") else {
        return Ok(Vec::new());
    };
    parameters
        .as_sequence()
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

#[test]
fn trim_reads_manifest_exposes_stage_level_cleanup_policy_surface() -> Result<()> {
    assert_eq!(
        stage_parameter_names("trim_reads")?,
        vec![
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
        vec!["threads"],
        "fastq.correct_errors manifest must avoid stage-level overrides that the current adapter does not map into backend execution"
    );
    Ok(())
}

#[test]
fn cleanup_stage_manifests_keep_distinct_parameter_surfaces() -> Result<()> {
    assert_eq!(
        stage_parameter_names("trim_terminal_damage")?,
        vec!["damage_mode", "trim_5p_bases", "trim_3p_bases"],
        "fastq.trim_terminal_damage must keep its damage-specific parameter surface"
    );
    assert_eq!(
        stage_parameter_names("trim_polyg_tails")?,
        vec!["trim_polyg", "min_polyg_run"],
        "fastq.trim_polyg_tails must keep its polyG-specific parameter surface"
    );
    assert_eq!(
        stage_parameter_names("remove_duplicates")?,
        vec!["dedup_mode", "keep_order"],
        "fastq.remove_duplicates must expose the governed duplicate semantics used by benchmark cohorts and stage plans"
    );
    Ok(())
}

#[test]
fn report_qc_manifest_avoids_unmapped_runtime_knobs() -> Result<()> {
    assert_eq!(
        stage_parameter_names("report_qc")?,
        Vec::<String>::new(),
        "fastq.report_qc must not expose stage parameters that the governed multiqc execution path cannot honor"
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
