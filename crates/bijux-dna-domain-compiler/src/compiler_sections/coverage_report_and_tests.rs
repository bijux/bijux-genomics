/// # Errors
/// Returns an error if domain indexes/tools/stages cannot be parsed.
#[allow(clippy::too_many_lines)]
pub fn domain_coverage_report(domain_dir: &Path) -> Result<serde_json::Value> {
    let mut by_domain = BTreeMap::new();
    for dom in ["fastq", "bam"] {
        let index_path = domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;

        let mut supported_stages_with_defaults = BTreeMap::new();
        let mut supported_tools_with_stage_mappings = BTreeMap::new();

        for stage_id in &index.stage_ids {
            let suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs)
                .replace('.', "_");
            let stage_path = domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{suffix}.yaml"));
            let stage: DomainStage = read_yaml(&stage_path)?;
            if stage.status != "supported" {
                continue;
            }
            let default_tool = index.active_defaults.get(stage_id).cloned();
            supported_stages_with_defaults.insert(
                stage_id.clone(),
                serde_json::json!({
                    "default_tool": default_tool,
                    "has_default": default_tool.is_some(),
                }),
            );
        }

        let tool_dir = domain_dir.join(dom).join("tools");
        let mut tool_path_by_id: BTreeMap<String, PathBuf> = BTreeMap::new();
        for entry in
            std::fs::read_dir(&tool_dir).with_context(|| format!("read {}", tool_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml")
                || path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml")
            {
                continue;
            }
            let tool: DomainToolLoose = read_yaml(&path)?;
            if !tool.tool_id.is_empty() {
                tool_path_by_id.insert(tool.tool_id, path);
            }
        }
        for other_dom in ["fastq", "bam"] {
            if other_dom == dom {
                continue;
            }
            let other_tool_dir = domain_dir.join(other_dom).join("tools");
            if !other_tool_dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&other_tool_dir)
                .with_context(|| format!("read {}", other_tool_dir.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml")
                    || path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml")
                {
                    continue;
                }
                let tool: DomainToolLoose = read_yaml(&path)?;
                if !tool.tool_id.is_empty() {
                    tool_path_by_id.entry(tool.tool_id).or_insert(path);
                }
            }
        }

        for tool_id in &index.tool_ids {
            let tool_path = tool_path_by_id
                .get(tool_id)
                .ok_or_else(|| anyhow!("missing tool file for {tool_id}"))?;
            let tool: DomainToolLoose = read_yaml(tool_path)?;
            if tool.status != "supported" {
                continue;
            }
            let mut stage_mappings = index
                .stage_tool_compatibility
                .iter()
                .filter_map(|(stage_id, tools)| {
                    if tools.iter().any(|tool| tool == tool_id) {
                        Some(stage_id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            stage_mappings.sort();
            supported_tools_with_stage_mappings.insert(
                tool_id.clone(),
                serde_json::json!({
                    "stage_mappings": stage_mappings,
                    "mapping_count": stage_mappings.len(),
                }),
            );
        }

        by_domain.insert(
            dom.to_string(),
            serde_json::json!({
                "supported_stage_count": supported_stages_with_defaults.len(),
                "supported_tool_count": supported_tools_with_stage_mappings.len(),
                "supported_stages_with_defaults": supported_stages_with_defaults,
                "supported_tools_with_stage_mappings": supported_tools_with_stage_mappings,
            }),
        );
    }
    Ok(serde_json::json!({ "domain_coverage": by_domain }))
}

#[cfg(test)]
mod tests {
    use super::validate_tool_output_subset;
    use std::path::Path;

    #[test]
    fn tool_output_validation_rejects_unknown_output_name() {
        let tool = r"
tool_id: fastp
outputs:
  - name: trimmed_reads
  - name: rogue_output
";
        let stage = r"
stage_id: fastq.trim
outputs:
  - name: trimmed_reads
";
        let Err(err) =
            validate_tool_output_subset(tool, stage, Path::new("tool.yaml"), "fastq.trim")
        else {
            panic!("must reject unknown output");
        };
        assert!(
            err.to_string().contains("rogue_output"),
            "unexpected error: {err}"
        );
    }
}
