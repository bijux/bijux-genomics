use anyhow::{anyhow, Context, Result};

use bijux_dna_core::metrics::ToolInvocationV1;

use super::manifest_identity::{detect_run_context, input_hash_from_many};
use super::{run_artifacts_dir, write_canonical_json, RunDirs};

pub(super) struct ReproducibilityContext {
    pub tool_invocations: Vec<ToolInvocationV1>,
    pub replay_tool_image_ref: Option<String>,
    pub replay_tool_image_digest: Option<String>,
    pub replay_tool_version_output: Option<String>,
    pub replay_tool_id: Option<String>,
    pub run_context: crate::RunContextV1,
    pub reproducibility_tuple: serde_json::Value,
}

pub(super) fn prepare_reproducibility_context(
    run_dirs: &RunDirs,
    run_provenance: &crate::RunProvenanceV1,
) -> Result<ReproducibilityContext> {
    let tool_invocations = load_tool_invocations(run_dirs)?;
    let replay_tool_image_ref = tool_invocations.first().map(|inv| inv.tool_id.to_string());
    let replay_tool_image_digest = tool_invocations
        .first()
        .map(|inv| inv.image_digest.clone())
        .or_else(|| run_provenance.tool_image_digest.clone());
    let replay_tool_version_output = tool_invocations
        .first()
        .and_then(|inv| inv.resolved_tool_version.clone())
        .or_else(|| Some(run_provenance.tool_version.clone()));
    let replay_tool_id = tool_invocations.first().map(|inv| inv.tool_id.to_string());
    let reproducibility_tuple = serde_json::json!({
        "schema_version": "bijux.repro_tuple.v1",
        "tool_digests": tool_invocations
            .iter()
            .map(|inv| serde_json::json!({
                "stage_id": inv.stage_id,
                "tool_id": inv.tool_id,
                "image_digest": inv.image_digest,
            }))
            .collect::<Vec<_>>(),
        "bank_hashes": serde_json::json!({}),
        "profile_hash": std::env::var("BIJUX_PROFILE_HASH").ok(),
    });
    let run_context = detect_run_context()?;
    validate_reproducibility_contract(&run_context, &reproducibility_tuple)?;
    Ok(ReproducibilityContext {
        tool_invocations,
        replay_tool_image_ref,
        replay_tool_image_digest,
        replay_tool_version_output,
        replay_tool_id,
        run_context,
        reproducibility_tuple,
    })
}

pub(super) fn write_reproducibility_report(
    run_dirs: &RunDirs,
    pipeline_id: &str,
    graph_hash: Option<&String>,
    run_provenance: &crate::RunProvenanceV1,
    declared_tool_image_digest: &str,
    tool_invocations: &[ToolInvocationV1],
    replay_tool_image_digest: Option<&String>,
    reproducibility_tuple: &serde_json::Value,
) -> Result<()> {
    let reproducibility_dir = run_artifacts_dir(run_dirs)?.join("reproducibility");
    bijux_dna_infra::ensure_dir(&reproducibility_dir).context("create reproducibility dir")?;
    let reproducibility_report_path = reproducibility_dir.join("report.json");
    let reproducibility_identity = bijux_dna_core::prelude::ReproducibilityIdentityV1 {
        image_digest: replay_tool_image_digest
            .cloned()
            .unwrap_or_else(|| declared_tool_image_digest.to_string()),
        tool_version: run_provenance.tool_version.clone(),
        params_hash: run_provenance.params_hash.clone(),
        input_hash: input_hash_from_many(&run_provenance.input_hashes),
        bank_hashes: serde_json::json!({}),
    };
    write_canonical_json(
        &reproducibility_report_path,
        &serde_json::json!({
            "schema_version": "bijux.reproducibility_report.v1",
            "pipeline_id": pipeline_id,
            "plan_hash": graph_hash,
            "params_hash": run_provenance.params_hash,
            "input_hashes": run_provenance.input_hashes,
            "tool_version": run_provenance.tool_version,
            "tool_image_digest": run_provenance.tool_image_digest,
            "tool_invocations": tool_invocations,
            "reproducibility_identity": reproducibility_identity,
            "reproducibility_tuple": reproducibility_tuple,
        }),
    )
    .context("write reproducibility report")?;
    Ok(())
}

fn load_tool_invocations(run_dirs: &RunDirs) -> Result<Vec<ToolInvocationV1>> {
    let path = run_dirs.artifacts_dir.join("tool_invocation.json");
    if path.exists() {
        let raw = std::fs::read_to_string(&path)?;
        let parsed: ToolInvocationV1 = serde_json::from_str(&raw)?;
        Ok(vec![parsed])
    } else {
        Ok(Vec::new())
    }
}

fn validate_reproducibility_contract(
    run_context: &crate::RunContextV1,
    reproducibility_tuple: &serde_json::Value,
) -> Result<()> {
    if matches!(run_context, crate::RunContextV1::Hpc { .. }) {
        let has_tool_digests = reproducibility_tuple
            .get("tool_digests")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|v| !v.is_empty());
        let has_profile_hash = reproducibility_tuple
            .get("profile_hash")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|v| !v.trim().is_empty());
        if !has_tool_digests || !has_profile_hash {
            return Err(anyhow!(
                "missing reproducibility tuple for HPC run (tool digests + profile hash required)"
            ));
        }
    }
    Ok(())
}
