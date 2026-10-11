#[cfg(feature = "bam_downstream")]
use super::declarations::KINSHIP_REQUIRED_OUTPUT_IDS;
#[cfg(feature = "bam_downstream")]
use super::declarations::LOCAL_PROOF_KIND_SMOKE;
#[cfg(feature = "bam_downstream")]
use super::declarations::LOCAL_PROOF_SAMPLE_ID;
#[cfg(feature = "bam_downstream")]
use super::input::ensure_repo_relative_file;
use super::observations::LocalProof;
#[cfg(feature = "bam_downstream")]
use crate::readiness::input::read_json;
#[cfg(feature = "bam_downstream")]
use crate::readiness::input::required_json_path;
#[cfg(feature = "bam_downstream")]
use crate::readiness::input::required_string;
#[cfg(feature = "bam_downstream")]
use crate::readiness::paths::path_relative_to_repo;
#[cfg(feature = "bam_downstream")]
use crate::readiness::ports::LocalProofKind;
#[cfg(feature = "bam_downstream")]
use crate::readiness::proof_fields::collect_json_fields;
use crate::readiness::ReadinessContext;
#[cfg(feature = "bam_downstream")]
use anyhow::anyhow;
use anyhow::Result;
#[cfg(feature = "bam_downstream")]
use bijux_dna_core::id_catalog::BAM_KINSHIP;
use std::collections::BTreeMap;
#[cfg(feature = "bam_downstream")]
use std::path::Path;
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) fn collect_local_proofs(
    context: &ReadinessContext<'_>,
) -> Result<BTreeMap<String, LocalProof>> {
    let repo_root = context.repo_root;
    let report_path = context.native.write(repo_root, LocalProofKind::KinshipSmoke)?;
    let payload: serde_json::Value = read_json(&report_path)?;
    let cases = payload
        .get("cases")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing array `cases`", report_path.display()))?;
    if cases.is_empty() {
        return Err(anyhow!("{} must keep at least one kinship smoke case", report_path.display()));
    }

    let mut artifact_paths = vec![path_relative_to_repo(repo_root, &report_path)];
    let mut observed_values = vec![payload.clone()];
    let tool_id = collect_kinship_proof_tool_id(cases, &report_path)?;

    for case in cases {
        let kinship_report_path =
            repo_root.join(required_json_path(case, "kinship_report", &report_path)?);
        let kinship_summary_path =
            repo_root.join(required_json_path(case, "kinship_summary", &report_path)?);
        let stage_metrics_path =
            repo_root.join(required_json_path(case, "stage_metrics", &report_path)?);

        let kinship_report_json: serde_json::Value = read_json(&kinship_report_path)?;
        let kinship_summary_json: serde_json::Value = read_json(&kinship_summary_path)?;
        let stage_metrics_json: serde_json::Value = read_json(&stage_metrics_path)?;

        artifact_paths.push(ensure_repo_relative_file(
            repo_root,
            &required_json_path(case, "kinship_report", &report_path)?,
        )?);
        artifact_paths.push(ensure_repo_relative_file(
            repo_root,
            &required_json_path(case, "kinship_summary", &report_path)?,
        )?);
        artifact_paths.push(ensure_repo_relative_file(
            repo_root,
            &required_json_path(case, "kinship_segments", &report_path)?,
        )?);
        artifact_paths.push(ensure_repo_relative_file(
            repo_root,
            &required_json_path(case, "stage_metrics", &report_path)?,
        )?);

        observed_values.push(case.clone());
        observed_values.push(kinship_report_json);
        observed_values.push(kinship_summary_json);
        observed_values.push(stage_metrics_json);
    }

    artifact_paths.sort();
    artifact_paths.dedup();

    let mut proofs = BTreeMap::new();
    proofs.insert(
        BAM_KINSHIP.to_string(),
        LocalProof {
            proof_kind: LOCAL_PROOF_KIND_SMOKE.to_string(),
            proof_path: path_relative_to_repo(repo_root, &report_path),
            tool_id,
            sample_id: LOCAL_PROOF_SAMPLE_ID.to_string(),
            artifact_paths,
            observed_fields: collect_json_fields(&observed_values),
            declared_output_ids: KINSHIP_REQUIRED_OUTPUT_IDS
                .iter()
                .map(|value| (*value).to_string())
                .collect(),
        },
    );
    Ok(proofs)
}
#[cfg(feature = "bam_downstream")]
pub(in crate::readiness) fn collect_kinship_proof_tool_id(
    cases: &[serde_json::Value],
    path: &Path,
) -> Result<String> {
    let mut methods = cases
        .iter()
        .map(|case| required_string(case, "method", path))
        .collect::<Result<Vec<_>>>()?;
    methods.sort();
    methods.dedup();
    if methods.len() != 1 {
        return Err(anyhow!(
            "{} must keep a single kinship local-smoke method, found [{}]",
            path.display(),
            methods.join(", ")
        ));
    }
    Ok(methods.remove(0))
}
