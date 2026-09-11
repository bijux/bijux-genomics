use super::input::ensure_repo_relative_file;
use super::input::required_row_value;
use super::observations::LocalSmokeProof;
use crate::readiness::input::read_json;
use crate::readiness::input::required_json_path;
use crate::readiness::input::required_string;
use crate::readiness::paths::path_relative_to_repo;
use crate::readiness::ports::LocalProofKind;
use crate::readiness::proof_fields::collect_local_smoke_fields;
use crate::readiness::ReadinessContext;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::id_catalog::{BAM_GC_BIAS, BAM_INSERT_SIZE};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
pub(in crate::readiness) fn collect_local_smoke_proofs(
    context: &ReadinessContext<'_>,
) -> Result<BTreeMap<String, LocalSmokeProof>> {
    let repo_root = context.repo_root;
    let insert_size_report_path =
        context.native.write(repo_root, LocalProofKind::InsertSizeSmoke)?;
    let insert_size_payload: serde_json::Value = read_json(&insert_size_report_path)?;

    let insert_size_summary_path = repo_root.join(required_json_path(
        &insert_size_payload,
        "insert_size_summary",
        &insert_size_report_path,
    )?);
    let insert_size_stage_metrics_path = repo_root.join(required_json_path(
        &insert_size_payload,
        "stage_metrics",
        &insert_size_report_path,
    )?);
    let insert_size_summary: serde_json::Value = read_json(&insert_size_summary_path)?;
    let insert_size_stage_metrics: serde_json::Value = read_json(&insert_size_stage_metrics_path)?;

    let mut insert_size_artifact_paths =
        vec![path_relative_to_repo(repo_root, &insert_size_report_path)];
    for relative in [
        required_json_path(&insert_size_payload, "insert_size_report", &insert_size_report_path)?,
        required_json_path(
            &insert_size_payload,
            "insert_size_histogram",
            &insert_size_report_path,
        )?,
        required_json_path(&insert_size_payload, "insert_size_summary", &insert_size_report_path)?,
        required_json_path(&insert_size_payload, "stage_metrics", &insert_size_report_path)?,
    ] {
        insert_size_artifact_paths.push(ensure_repo_relative_file(repo_root, &relative)?);
    }

    let mut proofs = BTreeMap::new();
    proofs.insert(
        BAM_INSERT_SIZE.to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &insert_size_report_path),
            sample_id: required_string(
                &insert_size_payload,
                "sample_id",
                &insert_size_report_path,
            )?,
            artifact_paths: insert_size_artifact_paths,
            observed_fields: collect_local_smoke_fields(&[
                insert_size_payload.clone(),
                insert_size_summary,
                insert_size_stage_metrics,
            ]),
        },
    );

    let gc_bias_tsv_path = context.native.write(repo_root, LocalProofKind::GcBiasSummary)?;
    let gc_bias_tsv = fs::read_to_string(&gc_bias_tsv_path)
        .with_context(|| format!("read {}", gc_bias_tsv_path.display()))?;
    let gc_bias_rows = parse_gc_bias_local_smoke_rows(&gc_bias_tsv, &gc_bias_tsv_path)?;
    let first_gc_bias_row = gc_bias_rows
        .first()
        .ok_or_else(|| anyhow!("{} is missing gc-bias rows", gc_bias_tsv_path.display()))?;
    let gc_bias_summary_path = repo_root.join(required_row_value(
        first_gc_bias_row,
        "gc_bias_summary_json",
        &gc_bias_tsv_path,
    )?);
    let gc_bias_stage_metrics_path =
        repo_root.join(required_row_value(first_gc_bias_row, "stage_metrics", &gc_bias_tsv_path)?);
    let gc_bias_summary: serde_json::Value = read_json(&gc_bias_summary_path)?;
    let gc_bias_stage_metrics: serde_json::Value = read_json(&gc_bias_stage_metrics_path)?;

    let mut gc_bias_artifact_paths = Vec::new();
    for relative in [
        required_row_value(first_gc_bias_row, "gc_bias_tsv", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "gc_bias_summary_json", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "gc_bias_metrics", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "gc_bias_plot", &gc_bias_tsv_path)?,
        required_row_value(first_gc_bias_row, "stage_metrics", &gc_bias_tsv_path)?,
    ] {
        gc_bias_artifact_paths.push(ensure_repo_relative_file(repo_root, &relative)?);
    }

    let mut gc_bias_observed_fields = collect_local_smoke_fields(&[
        gc_bias_summary,
        gc_bias_stage_metrics,
        serde_json::json!({
            "gc_bin": gc_bias_rows.iter().map(|row| row.get("gc_bin").cloned().unwrap_or_default()).collect::<Vec<_>>(),
            "normalized_coverage": gc_bias_rows.iter().map(|row| row.get("normalized_coverage").cloned().unwrap_or_default()).collect::<Vec<_>>(),
            "windows": gc_bias_rows.iter().map(|row| row.get("windows").cloned().unwrap_or_default()).collect::<Vec<_>>(),
            "read_starts": gc_bias_rows.iter().map(|row| row.get("read_starts").cloned().unwrap_or_default()).collect::<Vec<_>>(),
        }),
    ]);
    gc_bias_observed_fields.sort();
    gc_bias_observed_fields.dedup();

    proofs.insert(
        BAM_GC_BIAS.to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &gc_bias_tsv_path),
            sample_id: required_row_value(first_gc_bias_row, "sample_id", &gc_bias_tsv_path)?,
            artifact_paths: gc_bias_artifact_paths,
            observed_fields: gc_bias_observed_fields,
        },
    );

    Ok(proofs)
}
pub(in crate::readiness) fn parse_gc_bias_local_smoke_rows(
    body: &str,
    proof_path: &Path,
) -> Result<Vec<BTreeMap<String, String>>> {
    let mut lines = body.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("{} is missing a header row", proof_path.display()))?;
    let columns = header.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut rows = Vec::new();

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        if fields.len() != columns.len() {
            return Err(anyhow!(
                "{} has a malformed gc-bias local-smoke row",
                proof_path.display()
            ));
        }
        let row = columns.iter().cloned().zip(fields).collect::<BTreeMap<_, _>>();
        rows.push(row);
    }

    if !rows.iter().any(|row| row.get("gc_bin").is_some_and(|value| value == "0"))
        || !rows.iter().any(|row| row.get("gc_bin").is_some_and(|value| value == "50"))
        || !rows.iter().any(|row| row.get("gc_bin").is_some_and(|value| value == "100"))
    {
        return Err(anyhow!(
            "{} must keep the governed 0/50/100 gc-bias rows",
            proof_path.display()
        ));
    }

    Ok(rows)
}
