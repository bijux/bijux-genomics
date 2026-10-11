use std::collections::BTreeMap;

use bijux_dna_core::contract::{ReportSeverity, StageReportContract, StageReportKind};
use bijux_dna_core::prelude::invariants::{InvariantResultV1, InvariantStatusV1, StageVerdictV1};
use bijux_dna_stage_contract::{ArtifactRef, StagePlanV1, StageReportPartV1};

pub(super) struct OutputContractEvaluation {
    pub(super) artifacts: Vec<ArtifactRef>,
    pub(super) invariants: Vec<InvariantResultV1>,
    pub(super) materialized_count: usize,
    pub(super) exact_contract: bool,
}

pub(super) fn evaluate_output_contract(
    plan: &StagePlanV1,
    outputs: &[ArtifactRef],
) -> OutputContractEvaluation {
    let mut artifacts = if outputs.is_empty() { plan.io.outputs.clone() } else { outputs.to_vec() };
    artifacts.sort_by(|left, right| {
        left.name.as_str().cmp(right.name.as_str()).then_with(|| left.path.cmp(&right.path))
    });
    let declared = artifact_contracts(&plan.io.outputs);
    let observed = artifact_contracts(&artifacts);
    let exact_contract = plan.io.outputs.len() == artifacts.len()
        && observed.len() == artifacts.len()
        && declared == observed;
    let materialized_count = artifacts.iter().filter(|artifact| artifact.path.exists()).count();
    let all_materialized = !artifacts.is_empty() && materialized_count == artifacts.len();
    let invariants = vec![
        invariant(
            "bam_stage_output_contract_exact",
            if exact_contract { InvariantStatusV1::Pass } else { InvariantStatusV1::Fail },
            if exact_contract {
                "observed BAM outputs exactly match declared names and artifact contracts"
                    .to_string()
            } else {
                "observed BAM outputs differ from declared names or artifact contracts".to_string()
            },
        ),
        invariant(
            "bam_stage_outputs_materialized",
            if all_materialized { InvariantStatusV1::Pass } else { InvariantStatusV1::Fail },
            format!("materialized {materialized_count} of {} BAM stage outputs", artifacts.len()),
        ),
    ];
    OutputContractEvaluation { artifacts, invariants, materialized_count, exact_contract }
}

pub(super) fn stage_verdict(
    plan: &StagePlanV1,
    invariants: &[InvariantResultV1],
    scientific_key_metrics: &serde_json::Value,
    materialized_count: usize,
    artifact_count: usize,
) -> StageVerdictV1 {
    invariants.iter().fold(
        StageVerdictV1 {
            stage_id: plan.stage_id.as_str().to_string(),
            verdict: InvariantStatusV1::Pass,
            reasons: Vec::new(),
            key_metrics: serde_json::json!({
                "artifact_count": artifact_count,
                "materialized_artifact_count": materialized_count,
                "scientific": scientific_key_metrics,
            }),
        },
        |mut verdict, invariant| {
            verdict.verdict = std::cmp::max(verdict.verdict, invariant.status.clone());
            if invariant.status != InvariantStatusV1::Pass {
                verdict.reasons.push(invariant.id.clone());
            }
            verdict
        },
    )
}

pub(super) fn warnings(evaluation: &OutputContractEvaluation) -> Vec<String> {
    let mut warnings = Vec::new();
    if !evaluation.exact_contract {
        warnings.push("observed BAM output authority differs from the stage plan".to_string());
    }
    if evaluation.materialized_count != evaluation.artifacts.len() {
        warnings.push("one or more declared BAM stage outputs are not materialized".to_string());
    }
    warnings
}

pub(super) fn report_part(
    plan: &StagePlanV1,
    evaluation: &OutputContractEvaluation,
    invariants: &[InvariantResultV1],
    verdict: &StageVerdictV1,
) -> StageReportPartV1 {
    StageReportPartV1 {
        name: "bam_stage_outputs".to_string(),
        file_name: "bam-stage-outputs.json".to_string(),
        contract: StageReportContract {
            report_id: "bam.stage_outputs".to_string(),
            kind: StageReportKind::Qc,
            schema_version: "bijux.bam.stage_outputs.report.v1".to_string(),
            required_fields: vec![
                "stage_id".to_string(),
                "tool_id".to_string(),
                "artifacts".to_string(),
                "invariants".to_string(),
                "verdict".to_string(),
            ],
            advisory_fields: Vec::new(),
            severity: ReportSeverity::Warning,
        },
        payload: serde_json::json!({
            "stage_id": plan.stage_id,
            "tool_id": plan.tool_id,
            "artifacts": artifact_facts(&evaluation.artifacts),
            "invariants": invariants,
            "verdict": verdict,
        }),
    }
}

fn artifact_contracts(artifacts: &[ArtifactRef]) -> BTreeMap<String, Option<String>> {
    artifacts
        .iter()
        .map(|artifact| (artifact.name.as_str().to_string(), artifact.artifact_contract_id.clone()))
        .collect()
}

fn artifact_facts(artifacts: &[ArtifactRef]) -> Vec<serde_json::Value> {
    let mut facts = artifacts
        .iter()
        .map(|artifact| {
            serde_json::json!({
                "artifact_id": artifact.name,
                "artifact_contract_id": artifact.artifact_contract_id,
                "materialized": artifact.path.exists(),
                "bytes": artifact.path.metadata().ok().filter(std::fs::Metadata::is_file).map(|metadata| metadata.len()),
                "sha256": artifact.path.is_file().then(|| bijux_dna_infra::hash_file_sha256(&artifact.path).ok()).flatten(),
            })
        })
        .collect::<Vec<_>>();
    facts.sort_by(|left, right| left["artifact_id"].as_str().cmp(&right["artifact_id"].as_str()));
    facts
}

fn invariant(id: &str, status: InvariantStatusV1, message: String) -> InvariantResultV1 {
    InvariantResultV1 { id: id.to_string(), status, message, remediation: None }
}
