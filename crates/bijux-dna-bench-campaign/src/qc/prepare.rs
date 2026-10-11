use super::metadata::{report_qc_artifact_name, report_qc_contributor_id};
use super::resolve::{
    report_qc_contributor_artifact_path, report_qc_required_contributor_artifacts,
};
use super::upstream::ensure_report_qc_upstream_stage_outputs;
use super::{StageSamplePreparation, REPORT_QC_INPUTS_SCHEMA_VERSION};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_corpus::normalized::NormalizedFastqSample as CorpusNormalizedSample;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
pub(crate) fn prepare_report_qc_sample(
    context: &super::PreparationContext<'_>,
    out_root: &Path,
    sample: &CorpusNormalizedSample,
) -> Result<StageSamplePreparation> {
    let workspace_config = context.workspace_config;
    let corpus_id = context.corpus_id;
    let dry_run = context.dry_run;
    let artifacts =
        report_qc_required_contributor_artifacts(workspace_config, corpus_id, &sample.sample_id)?;
    let missing_stage_ids = artifacts
        .iter()
        .filter(|row| !row.path.exists())
        .map(|row| row.contract.stage_id)
        .collect::<BTreeSet<_>>();
    for stage_id in missing_stage_ids {
        ensure_report_qc_upstream_stage_outputs(context, sample, stage_id)?;
    }

    if !dry_run {
        let unresolved = artifacts
            .iter()
            .filter(|row| !row.path.exists())
            .map(|row| row.path.display().to_string())
            .collect::<Vec<_>>();
        if !unresolved.is_empty() {
            return Err(anyhow!(
                "report-qc governed input resolution failed for {}: missing {}",
                sample.sample_id,
                unresolved.join(", ")
            ));
        }
    }

    let raw_fastqc_dir = report_qc_contributor_artifact_path(
        workspace_config,
        corpus_id,
        &sample.sample_id,
        "fastq.detect_adapters",
        "fastqc",
        "fastqc",
    )?;
    let governed_manifest = report_qc_manifest_path(out_root, &sample.sample_id);
    if let Some(parent) = governed_manifest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = serde_json::json!({
        "schema_version": REPORT_QC_INPUTS_SCHEMA_VERSION,
        "qc_inputs": artifacts
            .iter()
            .map(|row| serde_json::json!({
                "name": report_qc_artifact_name(row),
                "path": row.path.display().to_string(),
                "role": row.contract.artifact_role,
                "optional": false,
            }))
            .collect::<Vec<_>>(),
        "contributors": artifacts
            .iter()
            .map(|row| serde_json::json!({
                "contributor_id": report_qc_contributor_id(row),
                "stage_id": row.contract.stage_id,
                "tool_id": row.tool_id,
                "artifact_id": row.contract.artifact_id,
                "artifact_role": row.contract.artifact_role,
                "path": row.path.display().to_string(),
            }))
            .collect::<Vec<_>>(),
        "raw_fastqc_dir": raw_fastqc_dir.display().to_string(),
    });
    fs::write(&governed_manifest, format!("{}\n", serde_json::to_string_pretty(&payload)?))
        .with_context(|| format!("write {}", governed_manifest.display()))?;

    let extra_stage_args = vec![
        "--aggregation-engine".to_string(),
        "multiqc".to_string(),
        "--aggregation-scope".to_string(),
        "governed_qc_artifacts".to_string(),
        "--governed-qc-manifest".to_string(),
        governed_manifest.display().to_string(),
    ];

    let mut run_extra_fields = BTreeMap::new();
    run_extra_fields.insert(
        "governed_qc_manifest".to_string(),
        serde_json::Value::String(governed_manifest.display().to_string()),
    );
    run_extra_fields.insert(
        "governed_qc_input_count".to_string(),
        serde_json::Value::Number(serde_json::Number::from(artifacts.len() as u64)),
    );

    Ok(StageSamplePreparation { extra_stage_args, run_extra_fields })
}

pub(crate) fn report_qc_manifest_path(out_root: &Path, sample_id: &str) -> PathBuf {
    out_root
        .join("bench")
        .join("report_qc")
        .join(sample_id)
        .join("governed_qc_inputs_manifest.json")
}
