#[cfg(feature = "bam_downstream")]
use super::declarations::LOCAL_PROOF_KIND_READY_PLAN;
use super::declarations::LOCAL_PROOF_KIND_SMOKE;
use super::declarations::RECALIBRATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS;
use super::input::ensure_repo_relative_file;
use super::observations::LocalProof;
use crate::readiness::input::read_json;
#[cfg(feature = "bam_downstream")]
use crate::readiness::input::required_command_shell;
use crate::readiness::input::required_json_path;
#[cfg(feature = "bam_downstream")]
use crate::readiness::input::required_nested_string;
use crate::readiness::input::required_proof_artifacts;
use crate::readiness::input::required_string;
use crate::readiness::paths::path_relative_to_repo;
#[cfg(feature = "bam_downstream")]
use crate::readiness::plan_artifacts::collect_plan_artifact_paths;
#[cfg(feature = "bam_downstream")]
use crate::readiness::plan_artifacts::collect_plan_output_ids;
use crate::readiness::ports::LocalProofKind;
use crate::readiness::proof_fields::collect_json_fields;
use crate::readiness::ReadinessContext;
use anyhow::Result;
#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::BAM_GENOTYPING;
use bijux_dna_core::id_catalog::BAM_RECALIBRATION;
use std::collections::BTreeMap;
pub(in crate::readiness) fn collect_local_proofs(
    context: &ReadinessContext<'_>,
) -> Result<BTreeMap<String, LocalProof>> {
    let repo_root = context.repo_root;
    let recalibration_report_path =
        context.native.write(repo_root, LocalProofKind::RecalibrationSmoke)?;
    let recalibration_payload: serde_json::Value = read_json(&recalibration_report_path)?;
    let recalibration_summary_path = repo_root.join(required_json_path(
        &recalibration_payload,
        "recalibration_summary",
        &recalibration_report_path,
    )?);
    let recalibration_stage_metrics_path = repo_root.join(required_json_path(
        &recalibration_payload,
        "stage_metrics",
        &recalibration_report_path,
    )?);
    let recalibration_summary_json: serde_json::Value = read_json(&recalibration_summary_path)?;
    let recalibration_stage_metrics_json: serde_json::Value =
        read_json(&recalibration_stage_metrics_path)?;

    let mut proofs = BTreeMap::new();
    proofs.insert(
        BAM_RECALIBRATION.to_string(),
        LocalProof {
            proof_kind: LOCAL_PROOF_KIND_SMOKE.to_string(),
            proof_path: path_relative_to_repo(repo_root, &recalibration_report_path),
            tool_id: "gatk".to_string(),
            sample_id: required_string(
                &recalibration_payload,
                "sample_id",
                &recalibration_report_path,
            )?,
            artifact_paths: required_proof_artifacts(
                repo_root,
                &recalibration_payload,
                &recalibration_report_path,
                &[
                    "recalibrated_bam",
                    "recalibration_report",
                    "recalibration_summary",
                    "stage_metrics",
                ],
                ensure_repo_relative_file,
            )?,
            observed_fields: collect_json_fields(&[
                recalibration_payload.clone(),
                recalibration_summary_json,
                recalibration_stage_metrics_json,
            ]),
            declared_output_ids: RECALIBRATION_REQUIRED_LOCAL_PROOF_OUTPUT_IDS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            command_shell: None,
        },
    );

    #[cfg(feature = "bam_downstream")]
    {
        let genotyping_plan_path =
            context.native.write(repo_root, LocalProofKind::GenotypingPlan)?;
        let genotyping_plan: serde_json::Value = read_json(&genotyping_plan_path)?;
        proofs.insert(
            BAM_GENOTYPING.to_string(),
            LocalProof {
                proof_kind: LOCAL_PROOF_KIND_READY_PLAN.to_string(),
                proof_path: path_relative_to_repo(repo_root, &genotyping_plan_path),
                tool_id: required_string(&genotyping_plan, "tool_id", &genotyping_plan_path)?,
                sample_id: required_nested_string(
                    &genotyping_plan,
                    &["params", "sample_id"],
                    &genotyping_plan_path,
                )?,
                artifact_paths: collect_plan_artifact_paths(
                    repo_root,
                    &genotyping_plan,
                    &genotyping_plan_path,
                )?,
                observed_fields: collect_json_fields(std::slice::from_ref(&genotyping_plan)),
                declared_output_ids: collect_plan_output_ids(
                    &genotyping_plan,
                    &genotyping_plan_path,
                )?,
                command_shell: Some(required_command_shell(
                    &genotyping_plan,
                    &genotyping_plan_path,
                )?),
            },
        );
    }

    Ok(proofs)
}
