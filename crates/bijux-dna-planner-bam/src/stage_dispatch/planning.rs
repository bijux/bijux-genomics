//! Stage planning with explicit physical input observation and unchanged logical paths.
use crate::api::StagePlanRequest;
use anyhow::Result;
use bijux_dna_stage_contract::{PlanDecisionReason, PlanReasonKind, StagePlanV1};
use serde_json::Value;
use std::path::Path;

/// Resolve relative file observations under the supplied root without changing command paths.
///
/// # Errors
/// Returns an error when stage policy, required assets, or the selected adapter rejects the request.
#[allow(clippy::needless_pass_by_value)]
pub fn plan_stage_at(repo_root: &Path, request: StagePlanRequest<'_>) -> Result<StagePlanV1> {
    let stage = bijux_dna_domain_bam::BamStage::try_from(request.stage_id)?;
    crate::tool_policy::enforce(stage, &request.tool.tool_id.0, request.params, request.reference)?;
    let mut plan = super::plan(repo_root, stage, &request)?;
    let mut details = serde_json::Map::new();
    details.insert("defaults_diff".to_string(), serde_json::json!({}));
    if let Some(Ok(hash)) = bijux_dna_domain_bam::stage_contract_hash(request.stage_id) {
        details.insert("contract_hash".to_string(), Value::String(hash));
    }
    let input_bytes = request
        .bam
        .and_then(|path| std::fs::metadata(repo_root.join(path)).ok())
        .map_or(0, |metadata| metadata.len());
    details.insert(
        "resource_plan".to_string(),
        serde_json::to_value(bijux_dna_domain_bam::estimate_bam_stage_resources(
            request.stage_id,
            input_bytes,
        ))
        .unwrap_or_else(|error| serde_json::json!({ "serialization_error": error.to_string() })),
    );
    plan.reason = PlanDecisionReason::new(
        PlanReasonKind::Default,
        format!("tool {} selected by planner", plan.tool_id.0),
    );
    plan.reason.details = Value::Object(details);
    Ok(plan)
}
