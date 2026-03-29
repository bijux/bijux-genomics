use super::super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn load_domain_stages(
    domain_dir: &Path,
    domain: &str,
    index: &DomainIndex,
    active_scope: &str,
    stage_to_tools: &mut StageToolMap,
    stage_planned: &mut StagePlannedMap,
    stage_statuses: &mut StageStatusMap,
    stage_output_kinds: &mut StageOutputKindsMap,
) -> Result<()> {
    let stages_dir = domain_dir.join(domain).join("stages");
    for stage_id in &index.stage_ids {
        let stage_suffix = stage_id
            .as_str()
            .split_once('.')
            .map_or(stage_id.as_str(), |(_, suffix)| suffix);
        let stage_file = stage_suffix.replace('.', "_");
        let path = stages_dir.join(format!("{stage_file}.yaml"));
        if !path.exists() {
            return Err(anyhow!(
                "index references missing stage file for {} at {}",
                stage_id,
                path.display()
            ));
        }
        let stage: DomainStage = read_yaml(&path)?;
        let stage_raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if stage.stage_id.trim().is_empty() {
            return Err(anyhow!("{} missing stage_id", path.display()));
        }
        if stage.scope.trim().is_empty() {
            return Err(anyhow!("{} missing scope", path.display()));
        }
        ensure_status(&stage.status, &path)?;
        if has_supported_placeholder_forbidden_token(&stage_raw)
            && !placeholders_allowed(&stage.status)
        {
            bail!(
                "{} contains placeholder token; placeholders are allowed only under status=planned",
                path.display()
            );
        }
        if !scope_active(&stage.scope, active_scope) || stage.status != "supported" {
            continue;
        }
        stage_to_tools.entry(stage.stage_id.clone()).or_default();
        let mut kinds = stage
            .outputs
            .iter()
            .map(|port| port.data_type.clone())
            .collect::<Vec<_>>();
        kinds.sort();
        kinds.dedup();
        stage_output_kinds.insert(stage.stage_id.clone(), kinds);
        stage_statuses.insert(stage.stage_id.clone(), stage.status.clone());
        stage_planned.insert(stage.stage_id, stage.planned_out_of_scope);
    }
    Ok(())
}
