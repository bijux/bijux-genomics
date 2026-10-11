use super::input::ensure_repo_relative_file;
use super::observations::LocalSmokeProof;
use crate::readiness::input::read_json;
use crate::readiness::input::required_json_path;
use crate::readiness::input::required_proof_artifacts;
use crate::readiness::input::required_string;
use crate::readiness::paths::path_relative_to_repo;
use crate::readiness::ports::LocalProofKind;
use crate::readiness::proof_fields::collect_local_smoke_fields;
use crate::readiness::ReadinessContext;
use anyhow::Result;
use bijux_dna_core::id_catalog::{BAM_ENDOGENOUS_CONTENT, BAM_OVERLAP_CORRECTION};
use std::collections::BTreeMap;
pub(in crate::readiness) fn collect_local_smoke_proofs(
    context: &ReadinessContext<'_>,
) -> Result<BTreeMap<String, LocalSmokeProof>> {
    let repo_root = context.repo_root;
    let endogenous_report_path =
        context.native.write(repo_root, LocalProofKind::EndogenousContentSmoke)?;
    let endogenous_payload: serde_json::Value = read_json(&endogenous_report_path)?;
    let endogenous_report_json_path = repo_root.join(required_json_path(
        &endogenous_payload,
        "endogenous_report",
        &endogenous_report_path,
    )?);
    let endogenous_summary_path = repo_root.join(required_json_path(
        &endogenous_payload,
        "endogenous_summary",
        &endogenous_report_path,
    )?);
    let endogenous_stage_metrics_path = repo_root.join(required_json_path(
        &endogenous_payload,
        "stage_metrics",
        &endogenous_report_path,
    )?);
    let endogenous_report_json: serde_json::Value = read_json(&endogenous_report_json_path)?;
    let endogenous_summary: serde_json::Value = read_json(&endogenous_summary_path)?;
    let endogenous_stage_metrics: serde_json::Value = read_json(&endogenous_stage_metrics_path)?;

    let mut proofs = BTreeMap::new();
    proofs.insert(
        BAM_ENDOGENOUS_CONTENT.to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &endogenous_report_path),
            sample_id: required_string(&endogenous_payload, "sample_id", &endogenous_report_path)?,
            artifact_paths: required_proof_artifacts(
                repo_root,
                &endogenous_payload,
                &endogenous_report_path,
                &["endogenous_report", "endogenous_summary", "stage_metrics"],
                ensure_repo_relative_file,
            )?,
            observed_fields: collect_local_smoke_fields(&[
                endogenous_payload.clone(),
                endogenous_report_json,
                endogenous_summary,
                endogenous_stage_metrics,
            ]),
        },
    );

    let overlap_report_path =
        context.native.write(repo_root, LocalProofKind::OverlapCorrectionSmoke)?;
    let overlap_payload: serde_json::Value = read_json(&overlap_report_path)?;
    let overlap_summary_path = repo_root.join(required_json_path(
        &overlap_payload,
        "overlap_correction_summary",
        &overlap_report_path,
    )?);
    let overlap_stage_metrics_path = repo_root.join(required_json_path(
        &overlap_payload,
        "stage_metrics",
        &overlap_report_path,
    )?);
    let overlap_summary: serde_json::Value = read_json(&overlap_summary_path)?;
    let overlap_stage_metrics: serde_json::Value = read_json(&overlap_stage_metrics_path)?;
    let overlap_corrected_bam =
        required_json_path(&overlap_payload, "overlap_corrected_bam", &overlap_report_path)?;
    let overlap_corrected_bam_index = ensure_repo_relative_file(
        repo_root,
        "artifacts/runs/bench/local-smoke/bam.overlap_correction/overlap_corrected.bam.bai",
    )?;
    let mut overlap_artifact_paths = vec![path_relative_to_repo(repo_root, &overlap_report_path)];
    for relative in [
        overlap_corrected_bam,
        overlap_corrected_bam_index,
        required_json_path(&overlap_payload, "overlap_correction_summary", &overlap_report_path)?,
        required_json_path(&overlap_payload, "flagstat_before", &overlap_report_path)?,
        required_json_path(&overlap_payload, "flagstat_after", &overlap_report_path)?,
        required_json_path(&overlap_payload, "idxstats_before", &overlap_report_path)?,
        required_json_path(&overlap_payload, "idxstats_after", &overlap_report_path)?,
        required_json_path(&overlap_payload, "stage_metrics", &overlap_report_path)?,
    ] {
        overlap_artifact_paths.push(ensure_repo_relative_file(repo_root, &relative)?);
    }

    proofs.insert(
        BAM_OVERLAP_CORRECTION.to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &overlap_report_path),
            sample_id: required_string(&overlap_payload, "sample_id", &overlap_report_path)?,
            artifact_paths: overlap_artifact_paths,
            observed_fields: collect_local_smoke_fields(&[
                overlap_payload,
                overlap_summary,
                overlap_stage_metrics,
            ]),
        },
    );

    Ok(proofs)
}
