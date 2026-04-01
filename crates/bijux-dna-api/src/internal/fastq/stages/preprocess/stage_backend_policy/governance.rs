use super::{anyhow, ExecutionStep, NetworkPolicy, Result};

pub(super) fn stage_network_policy(stage_id: &str) -> NetworkPolicy {
    match stage_id {
        "fastq.validate_reads"
        | "fastq.detect_adapters"
        | "fastq.trim_terminal_damage"
        | "fastq.trim_reads"
        | "fastq.merge_pairs"
        | "fastq.remove_duplicates"
        | "fastq.correct_errors"
        | "fastq.filter_reads"
        | "fastq.filter_low_complexity"
        | "fastq.trim_polyg_tails"
        | "fastq.screen_taxonomy" => NetworkPolicy::Forbid,
        _ => NetworkPolicy::Allow,
    }
}

pub(super) fn fastq_backend_allowlist(stage_id: &str) -> Option<Vec<String>> {
    if !stage_id.starts_with("fastq.") {
        return None;
    }
    let tools = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(
        &bijux_dna_core::ids::StageId::new(stage_id.to_string()),
    );
    Some(
        tools
            .into_iter()
            .map(|tool| tool.to_string())
            .collect::<Vec<_>>(),
    )
}

pub(super) fn enforce_fastq_backend_allowlist(stage_id: &str, tool_id: &str) -> Result<()> {
    let Some(allowed) = fastq_backend_allowlist(stage_id) else {
        return Ok(());
    };
    if allowed.iter().any(|allowed_tool| allowed_tool == tool_id) {
        return Ok(());
    }
    Err(anyhow!(
        "unsupported backend for {stage_id}: `{tool_id}` not in allowlist {}",
        allowed.join(",")
    ))
}

pub(super) fn required_fastq_tools() -> Result<std::collections::BTreeSet<String>> {
    let raw = std::fs::read_to_string(
        crate::support::workspace::resolve_repo_root()?
            .join("configs/ci/tools/required_tools.toml"),
    )?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let mut set = std::collections::BTreeSet::new();
    let items = parsed
        .get("required_tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing required_tools in required_tools.toml"))?;
    for item in items {
        if let Some(id) = item.as_str() {
            set.insert(id.to_string());
        }
    }
    Ok(set)
}

pub(super) fn enforce_screen_db_governance(planned: &ExecutionStep) -> Result<()> {
    let stage = planned.step_id.as_str();
    if !matches!(
        stage,
        "fastq.screen_taxonomy"
            | "fastq.deplete_rrna"
            | "fastq.deplete_host"
            | "fastq.deplete_reference_contaminants"
    ) {
        return Ok(());
    }
    let template = planned.command.template.join(" ");
    if template.contains("http://") || template.contains("https://") {
        return Err(anyhow!(
            "{stage} may not fetch databases over network at runtime; use pre-mounted references"
        ));
    }
    if template.contains("download") || template.contains("pull") {
        return Err(anyhow!(
            "{stage} command contains database fetch verbs; require immutable pre-resolved DB paths"
        ));
    }
    Ok(())
}
