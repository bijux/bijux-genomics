use super::{
    bail, is_umbrella_stage, read_yaml, BTreeMap, Context, DomainIndex, DomainStage,
    DomainToolLoose, Result, ValidateOptions,
};

pub(super) fn validate_domain_index_inventory(
    options: &ValidateOptions,
    dom: &str,
    index: &DomainIndex,
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    let index_path = options.domain_dir.join(dom).join("index.yaml");
    if index.domain != dom {
        bail!(
            "{} has domain {} but expected {}",
            index_path.display(),
            index.domain,
            dom
        );
    }
    if index.stage_ids.is_empty() || index.tool_ids.is_empty() {
        bail!("{} missing stage_ids/tool_ids", index_path.display());
    }
    for stage_id in &index.stage_ids {
        if is_umbrella_stage(stage_id) {
            bail!(
                "{} contains umbrella stage {}. Use explicit stage IDs (e.g. fastq.validate_reads, fastq.profile_reads, ...).",
                index_path.display(),
                stage_id
            );
        }
        if !stage_ids.contains_key(stage_id) {
            bail!(
                "{} references unknown stage {}",
                index_path.display(),
                stage_id
            );
        }
    }
    for tool_id in &index.tool_ids {
        if !tool_ids.contains_key(tool_id) {
            bail!(
                "{} references unknown tool {}",
                index_path.display(),
                tool_id
            );
        }
    }

    let stage_dir = options.domain_dir.join(dom).join("stages");
    for entry in
        std::fs::read_dir(&stage_dir).with_context(|| format!("read {}", stage_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|value| value.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let stage: DomainStage = read_yaml(&path)?;
        if !index.stage_ids.contains(&stage.stage_id) {
            bail!(
                "{} stage {} exists in file system but is not listed in index.yaml",
                path.display(),
                stage.stage_id
            );
        }
    }

    let tool_dir = options.domain_dir.join(dom).join("tools");
    for entry in
        std::fs::read_dir(&tool_dir).with_context(|| format!("read {}", tool_dir.display()))?
    {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|value| value.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let tool: DomainToolLoose = read_yaml(&path)?;
        if !index.tool_ids.contains(&tool.tool_id) {
            bail!(
                "{} tool {} exists in file system but is not listed in index.yaml",
                path.display(),
                tool.tool_id
            );
        }
    }

    let mut stage_status_by_id = BTreeMap::new();
    for stage_id in &index.stage_ids {
        let stage_suffix = stage_id
            .split_once('.')
            .map_or(stage_id.as_str(), |(_, rhs)| rhs);
        let stage_path = options
            .domain_dir
            .join(dom)
            .join("stages")
            .join(format!("{}.yaml", stage_suffix.replace('.', "_")));
        let stage: DomainStage = read_yaml(&stage_path)?;
        stage_status_by_id.insert(stage_id.clone(), stage.status);
    }

    Ok(stage_status_by_id)
}
