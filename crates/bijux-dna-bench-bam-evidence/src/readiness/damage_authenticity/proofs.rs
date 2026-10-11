use super::input::ensure_repo_relative_file;
use super::observations::LocalSmokeProof;
use crate::readiness::input::read_json;
use crate::readiness::input::required_proof_artifacts;
use crate::readiness::input::required_string;
use crate::readiness::paths::path_relative_to_repo;
use crate::readiness::ports::LocalProofKind;
use crate::readiness::proof_fields::collect_local_smoke_fields;
use crate::readiness::ReadinessContext;
use anyhow::Result;
use bijux_dna_core::id_catalog::{BAM_AUTHENTICITY, BAM_DAMAGE};
use std::collections::BTreeMap;
pub(in crate::readiness) fn collect_local_smoke_proofs(
    context: &ReadinessContext<'_>,
) -> Result<BTreeMap<String, LocalSmokeProof>> {
    let repo_root = context.repo_root;
    let damage_report_path = context.native.write(repo_root, LocalProofKind::DamageSmoke)?;
    let damage_payload: serde_json::Value = read_json(&damage_report_path)?;
    let [damage_report_json, damage_terminal_position_metrics, damage_parser_output, damage_stage_metrics] =
        crate::readiness::input::required_json_documents(
            repo_root,
            &damage_payload,
            &damage_report_path,
            ["damage_report", "terminal_position_metrics", "parser_output", "stage_metrics"],
        )?;

    let authenticity_report_path =
        context.native.write(repo_root, LocalProofKind::AuthenticitySmoke)?;
    let authenticity_payload: serde_json::Value = read_json(&authenticity_report_path)?;
    let [authenticity_report_json, authenticity_summary, authenticity_composite, authenticity_stage_metrics] =
        crate::readiness::input::required_json_documents(
            repo_root,
            &authenticity_payload,
            &authenticity_report_path,
            [
                "authenticity_report",
                "authenticity_summary",
                "authenticity_composite",
                "stage_metrics",
            ],
        )?;

    let mut proofs = BTreeMap::new();
    proofs.insert(
        BAM_DAMAGE.to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &damage_report_path),
            sample_id: required_string(&damage_payload, "sample_id", &damage_report_path)?,
            artifact_paths: required_proof_artifacts(
                repo_root,
                &damage_payload,
                &damage_report_path,
                &[
                    "damage_report",
                    "terminal_position_metrics",
                    "parser_output",
                    "advisory_boundary",
                    "udg_regime",
                    "stage_metrics",
                ],
                ensure_repo_relative_file,
            )?,
            observed_fields: collect_local_smoke_fields(&[
                damage_payload,
                damage_report_json,
                damage_terminal_position_metrics,
                damage_parser_output,
                damage_stage_metrics,
            ]),
        },
    );

    proofs.insert(
        BAM_AUTHENTICITY.to_string(),
        LocalSmokeProof {
            proof_path: path_relative_to_repo(repo_root, &authenticity_report_path),
            sample_id: required_string(
                &authenticity_payload,
                "sample_id",
                &authenticity_report_path,
            )?,
            artifact_paths: required_proof_artifacts(
                repo_root,
                &authenticity_payload,
                &authenticity_report_path,
                &[
                    "authenticity_report",
                    "authenticity_summary",
                    "authenticity_composite",
                    "advisory_boundary",
                    "stage_metrics",
                    "damage_unified_metrics",
                    "contamination_summary",
                    "complexity_summary",
                    "coverage_regime",
                    "mapping_summary",
                ],
                ensure_repo_relative_file,
            )?,
            observed_fields: collect_local_smoke_fields(&[
                authenticity_payload,
                authenticity_report_json,
                authenticity_summary,
                authenticity_composite,
                authenticity_stage_metrics,
            ]),
        },
    );

    Ok(proofs)
}
