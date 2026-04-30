use anyhow::Result;

use super::request::refusal;
use super::{
    atomic_write_json, VcfPipelineRequest, VcfPipelineResult, VcfPreflightResult, VcfRefusalCode,
    VcfStageOutputs,
};

pub(super) fn verify_contract_surface(result: &VcfPipelineResult) -> Result<()> {
    fn expected_stage_artifacts(stage_id: &str) -> &'static [&'static str] {
        match stage_id {
            "vcf.call" | "vcf.call_gl" | "vcf.call_diploid" | "vcf.call_pseudohaploid" => {
                &["call_metrics.json", "call_metrics.tsv", "call_manifest.json"]
            }
            "vcf.filter" => &[
                "filtered.vcf.gz",
                "filtered.vcf.gz.tbi",
                "filter_breakdown.json",
                "filter_breakdown.tsv",
                "filter_explain.json",
            ],
            "vcf.stats" => &["bcftools_stats.txt", "stats.json"],
            "vcf.damage_filter" => &[
                "damage_filter_summary.json",
                "damage_filter_counts.json",
                "warnings.json",
                "damage_genotype_manifest.json",
            ],
            "vcf.postprocess" => &[
                "postprocess.vcf.gz",
                "postprocess.vcf.gz.tbi",
                "validate_outputs.json",
                "final_manifest.json",
            ],
            _ => &[],
        }
    }

    for stage in &result.stages {
        if !stage.artifact_dir.starts_with(result.artifact_root.join("")) {
            return Err(refusal(
                VcfRefusalCode::ContractViolation,
                format!("artifact root violation for {}", stage.stage_id),
            ));
        }
        for required in
            ["stage_manifest.json", "stdout.log", "stderr.log", "command.txt", "env.txt"]
        {
            let p = stage.artifact_dir.join(required);
            if !p.exists() {
                return Err(refusal(
                    VcfRefusalCode::ContractViolation,
                    format!("missing stage sidecar {}", p.display()),
                ));
            }
        }
        for required in expected_stage_artifacts(&stage.stage_id) {
            let p = stage.artifact_dir.join(required);
            if !p.exists() {
                return Err(refusal(
                    VcfRefusalCode::ContractViolation,
                    format!("stage {} missing required artifact {}", stage.stage_id, p.display()),
                ));
            }
        }
    }
    if !result.report_path.exists() {
        return Err(refusal(VcfRefusalCode::ContractViolation, "missing report.json"));
    }
    Ok(())
}

fn read_json(path: &std::path::Path) -> Option<serde_json::Value> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<serde_json::Value>(&raw).ok()
}

pub(super) fn maybe_bam_handoff(
    request: &VcfPipelineRequest,
    artifact_root: &std::path::Path,
) -> Result<Option<serde_json::Value>> {
    let candidates = [
        request.run_root.join("artifacts").join("bam").join("bam_metrics.json"),
        request.run_root.join("artifacts").join("bam").join("metrics.json"),
        request.run_root.join("artifacts").join("bam").join("summary.json"),
    ];
    let bam_metrics = candidates.iter().find_map(|p| read_json(p));
    let Some(bam_metrics) = bam_metrics else {
        return Ok(None);
    };
    let handoff = serde_json::json!({
        "schema_version": "bijux.vcf.handoff.bam_to_vcf.v1",
        "source_domain": "bam",
        "target_domain": "vcf",
        "signals": {
            "damage_ct_ga_rate": bam_metrics.get("damage_ct_ga_rate").cloned().unwrap_or(serde_json::Value::Null),
            "mean_depth": bam_metrics.get("mean_depth").cloned().unwrap_or_else(|| bam_metrics.get("depth_mean").cloned().unwrap_or(serde_json::Value::Null)),
            "udg_treated": bam_metrics.get("udg_treated").cloned().unwrap_or(serde_json::Value::Null),
        },
        "decision_inputs": {
            "damage_filter_considered": true,
            "regime_detection_considered": true,
        },
    });
    let out = artifact_root.join("handoff.bam_to_vcf.json");
    atomic_write_json(&out, &handoff)?;
    Ok(Some(handoff))
}

pub(super) fn write_runtime_explain(
    preflight: &VcfPreflightResult,
    artifact_root: &std::path::Path,
    stages: &[VcfStageOutputs],
    handoff: Option<serde_json::Value>,
) -> Result<()> {
    let preflight_details = serde_json::json!({
        "input_regime": preflight.regime.regime,
        "lowcov_likelihood_hint": preflight.regime.lowcov_likelihood_hint,
        "pseudohaploid_hint": preflight.regime.pseudohaploid_hint,
    });

    let mut backend = serde_json::Value::Null;
    let mut chunk_strategy = serde_json::Value::Null;
    let mut panel_lock = serde_json::Value::Null;
    let mut map_lock = serde_json::Value::Null;
    for stage in stages {
        let imputation_manifest = stage.artifact_dir.join("imputation_manifest.json");
        if backend.is_null() && imputation_manifest.exists() {
            if let Some(json) = read_json(&imputation_manifest) {
                backend = json.get("backend").cloned().unwrap_or(serde_json::Value::Null);
                chunk_strategy = json.get("chunk_plan").cloned().unwrap_or(serde_json::Value::Null);
                panel_lock = serde_json::json!({
                    "panel_id": json.get("panel_id").cloned().unwrap_or(serde_json::Value::Null),
                    "panel_checksums": json.get("panel_checksums").cloned().unwrap_or(serde_json::json!([])),
                });
                map_lock = json.get("map").cloned().unwrap_or(serde_json::Value::Null);
            }
        }
        let panel_manifest = stage.artifact_dir.join("panel_manifest.json");
        if panel_lock.is_null() && panel_manifest.exists() {
            if let Some(json) = read_json(&panel_manifest) {
                panel_lock = serde_json::json!({
                    "panel_id": json.get("panel").and_then(|p| p.get("id")).cloned().unwrap_or(serde_json::Value::Null),
                    "panel_version": json.get("panel").and_then(|p| p.get("version")).cloned().unwrap_or(serde_json::Value::Null),
                });
                map_lock = serde_json::json!({
                    "map_id": json.get("map").and_then(|m| m.get("id")).cloned().unwrap_or(serde_json::Value::Null),
                    "map_version": json.get("map").and_then(|m| m.get("version")).cloned().unwrap_or(serde_json::Value::Null),
                });
            }
        }
    }

    let explain = serde_json::json!({
        "schema_version": "bijux.vcf.runtime_explain.v1",
        "chosen_regime": preflight.regime.regime,
        "chosen_backend": backend,
        "panel_lock_id": panel_lock.get("panel_id").cloned().unwrap_or(serde_json::Value::Null),
        "chunk_plan": chunk_strategy,
        "regime_decision": preflight_details,
        "backend_selection": {
            "selected_backend": backend,
            "reason": "selected from executed stage manifests and input regime constraints",
        },
        "panel_map_lock_ids": {
            "panel_lock": panel_lock,
            "map_lock": map_lock,
        },
        "chunk_strategy": chunk_strategy,
        "handoff": handoff.unwrap_or(serde_json::json!({})),
    });
    atomic_write_json(&artifact_root.join("explain.json"), &explain)?;
    Ok(())
}

pub(super) fn build_vcf_provenance_line(stages: &[VcfStageOutputs]) -> String {
    let mut tools = Vec::<String>::new();
    let mut digests = Vec::<String>::new();
    let mut panel_lock = String::from("none");
    let mut reference_lock = String::from("none");
    for stage in stages {
        let tool_invocation = stage.artifact_dir.join("tool_invocation.json");
        if let Some(json) = read_json(&tool_invocation) {
            if let Some(tool) = json.get("tool_id").and_then(serde_json::Value::as_str) {
                tools.push(tool.to_string());
            }
            if let Some(digest) = json.get("image_digest").and_then(serde_json::Value::as_str) {
                digests.push(digest.to_string());
            }
        }
        let panel_manifest = stage.artifact_dir.join("panel_manifest.json");
        if panel_lock == "none" && panel_manifest.exists() {
            if let Some(json) = read_json(&panel_manifest) {
                panel_lock = json
                    .get("panel")
                    .and_then(|p| p.get("id"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("panel_not_declared")
                    .to_string();
                reference_lock = json
                    .get("map")
                    .and_then(|m| m.get("id"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("map_not_declared")
                    .to_string();
            }
        }
    }
    tools.sort();
    tools.dedup();
    digests.sort();
    digests.dedup();
    format!(
        "tools={} | digests={} | panel_lock={} | reference_lock={}",
        if tools.is_empty() { "none".to_string() } else { tools.join(",") },
        if digests.is_empty() { "none".to_string() } else { digests.join(",") },
        panel_lock,
        reference_lock
    )
}
