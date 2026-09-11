use super::declarations::LOCAL_PROOF_KIND_READY_PLAN;
use super::declarations::LOCAL_PROOF_KIND_SMOKE;
use super::declarations::SEX_REQUIRED_LOCAL_PROOF_OUTPUT_IDS;
use super::input::ensure_repo_relative_file;
use super::observations::LocalProof;
use crate::readiness::input::read_json;
use crate::readiness::input::required_command_shell;
use crate::readiness::input::required_json_path;
use crate::readiness::input::required_nested_string;
use crate::readiness::input::required_proof_artifacts;
use crate::readiness::input::required_string;
use crate::readiness::paths::path_relative_to_repo;
use crate::readiness::plan_artifacts::collect_plan_artifact_paths;
use crate::readiness::plan_artifacts::collect_plan_output_ids;
use crate::readiness::ports::LocalProofKind;
use crate::readiness::proof_fields::collect_json_fields;
use crate::readiness::ReadinessContext;
use anyhow::Result;
#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::BAM_HAPLOGROUPS;
use bijux_dna_core::id_catalog::{BAM_CONTAMINATION, BAM_SEX};
use std::collections::BTreeMap;
pub(in crate::readiness) fn collect_local_proofs(
    context: &ReadinessContext<'_>,
) -> Result<BTreeMap<String, LocalProof>> {
    let repo_root = context.repo_root;
    let contamination_plan_path =
        context.native.write(repo_root, LocalProofKind::ContaminationPlan)?;
    let contamination_plan: serde_json::Value = read_json(&contamination_plan_path)?;

    let sex_report_path = context.native.write(repo_root, LocalProofKind::SexSmoke)?;
    let sex_payload: serde_json::Value = read_json(&sex_report_path)?;
    let sex_report_json_path =
        repo_root.join(required_json_path(&sex_payload, "sex_report", &sex_report_path)?);
    let sex_summary_path =
        repo_root.join(required_json_path(&sex_payload, "sex_summary", &sex_report_path)?);
    let sex_stage_metrics_path =
        repo_root.join(required_json_path(&sex_payload, "stage_metrics", &sex_report_path)?);
    let sex_report_json: serde_json::Value = read_json(&sex_report_json_path)?;
    let sex_summary_json: serde_json::Value = read_json(&sex_summary_path)?;
    let sex_stage_metrics_json: serde_json::Value = read_json(&sex_stage_metrics_path)?;

    let mut proofs = BTreeMap::new();
    proofs.insert(
        BAM_CONTAMINATION.to_string(),
        plan_proof(repo_root, &contamination_plan_path, &contamination_plan)?,
    );
    proofs.insert(
        BAM_SEX.to_string(),
        LocalProof {
            proof_kind: LOCAL_PROOF_KIND_SMOKE.to_string(),
            proof_path: path_relative_to_repo(repo_root, &sex_report_path),
            tool_id: required_string(&sex_payload, "method", &sex_report_path)?,
            sample_id: required_string(&sex_payload, "sample_id", &sex_report_path)?,
            artifact_paths: required_proof_artifacts(
                repo_root,
                &sex_payload,
                &sex_report_path,
                &["sex_report", "sex_summary", "stage_metrics"],
                ensure_repo_relative_file,
            )?,
            observed_fields: collect_json_fields(&[
                sex_payload.clone(),
                sex_report_json,
                sex_summary_json,
                sex_stage_metrics_json,
            ]),
            declared_output_ids: SEX_REQUIRED_LOCAL_PROOF_OUTPUT_IDS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
            command_shell: None,
        },
    );

    #[cfg(feature = "bam_downstream")]
    {
        let haplogroups_plan_path =
            context.native.write(repo_root, LocalProofKind::HaplogroupsPlan)?;
        let haplogroups_plan: serde_json::Value = read_json(&haplogroups_plan_path)?;
        proofs.insert(
            BAM_HAPLOGROUPS.to_string(),
            plan_proof(repo_root, &haplogroups_plan_path, &haplogroups_plan)?,
        );
    }

    Ok(proofs)
}

fn plan_proof(
    repo_root: &std::path::Path,
    path: &std::path::Path,
    payload: &serde_json::Value,
) -> Result<LocalProof> {
    Ok(LocalProof {
        proof_kind: LOCAL_PROOF_KIND_READY_PLAN.to_string(),
        proof_path: path_relative_to_repo(repo_root, path),
        tool_id: required_string(payload, "tool_id", path)?,
        sample_id: required_nested_string(payload, &["params", "sample_id"], path)?,
        artifact_paths: collect_plan_artifact_paths(repo_root, payload, path)?,
        observed_fields: collect_json_fields(std::slice::from_ref(payload)),
        declared_output_ids: collect_plan_output_ids(payload, path)?,
        command_shell: Some(required_command_shell(payload, path)?),
    })
}
