use super::{bail, read_yaml, BTreeMap, DomainIndex, Result, ValidateOptions};

#[derive(Debug, serde::Deserialize, Default)]
struct ToolStageClaims {
    #[serde(default)]
    status: String,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    planned_stage_ids: Vec<String>,
}

#[derive(Debug, serde::Deserialize, Default)]
struct StageStatusDoc {
    #[serde(default)]
    stage_id: String,
    #[serde(default)]
    status: String,
}

pub(super) fn validate_fixture_consistency(
    options: &ValidateOptions,
    stage_ids: &BTreeMap<String, String>,
    tool_ids: &BTreeMap<String, String>,
) -> Result<()> {
    for dom in ["fastq", "bam", "vcf"] {
        let index_path = options.domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        let fixtures_dir = options.domain_dir.join(dom).join("fixtures");
        let mut tool_stage_claims = BTreeMap::<String, Vec<String>>::new();
        let mut tool_statuses = BTreeMap::<String, String>::new();
        let tools_dir = options.domain_dir.join(dom).join("tools");
        if tools_dir.exists() {
            for entry in std::fs::read_dir(&tools_dir)? {
                let path = entry?.path();
                if path.extension().and_then(|value| value.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|value| value.to_str()).is_some_and(|name| name.starts_with('_'))
                {
                    continue;
                }
                let claims: ToolStageClaims = read_yaml(&path)?;
                let Some(tool_id) = path.file_stem().and_then(|value| value.to_str()) else {
                    continue;
                };
                let mut stages = claims.stage_ids;
                stages.extend(claims.planned_stage_ids);
                tool_stage_claims.insert(tool_id.to_string(), stages);
                tool_statuses.insert(tool_id.to_string(), claims.status);
            }
        }
        let mut stage_statuses = BTreeMap::<String, String>::new();
        let stages_dir = options.domain_dir.join(dom).join("stages");
        if stages_dir.exists() {
            for entry in std::fs::read_dir(&stages_dir)? {
                let path = entry?.path();
                if path.extension().and_then(|value| value.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|value| value.to_str()).is_some_and(|name| name.starts_with('_'))
                {
                    continue;
                }
                let doc: StageStatusDoc = read_yaml(&path)?;
                if doc.stage_id.trim().is_empty() {
                    continue;
                }
                stage_statuses.insert(doc.stage_id, doc.status);
            }
        }

        for (stage_id, compatible_tools) in &index.stage_tool_compatibility {
            for tool_id in compatible_tools {
                let stage_status = stage_statuses.get(stage_id).map_or("", String::as_str);
                let tool_status = tool_statuses.get(tool_id).map_or("", String::as_str);
                if stage_status != "supported" || tool_status != "supported" {
                    continue;
                }
                let fixture = fixtures_dir.join(stage_id).join(format!("{tool_id}.txt"));
                if !fixture.exists() {
                    bail!(
                        "{} declares stage/tool pair {} / {} but fixture {} is missing",
                        index_path.display(),
                        stage_id,
                        tool_id,
                        fixture.display()
                    );
                }
            }
        }

        if !fixtures_dir.exists() {
            continue;
        }

        for stage_entry in std::fs::read_dir(&fixtures_dir)? {
            let stage_entry = stage_entry?;
            if !stage_entry.file_type()?.is_dir() {
                continue;
            }
            let Some(stage_id) = stage_entry.file_name().to_str().map(ToString::to_string) else {
                continue;
            };
            if !stage_ids.contains_key(&stage_id) {
                bail!(
                    "{} fixture directory {} does not map to a declared stage",
                    fixtures_dir.display(),
                    stage_id
                );
            }
            let declared_tools = index.stage_tool_compatibility.get(&stage_id).cloned().unwrap_or_default();
            for tool_entry in std::fs::read_dir(stage_entry.path())? {
                let tool_entry = tool_entry?;
                if !tool_entry.file_type()?.is_file() {
                    continue;
                }
                let path = tool_entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("txt") {
                    continue;
                }
                let Some(tool_id) = path.file_stem().and_then(|value| value.to_str()) else {
                    continue;
                };
                if !tool_ids.contains_key(tool_id) {
                    bail!("{} fixture references undeclared tool {}", path.display(), tool_id);
                }
                let claims_stage = tool_stage_claims
                    .get(tool_id)
                    .is_some_and(|stages| stages.iter().any(|candidate| candidate == &stage_id));
                if !declared_tools.iter().any(|candidate| candidate == tool_id) && !claims_stage {
                    bail!(
                        "{} fixture points to undeclared stage/tool pair {} / {}",
                        path.display(),
                        stage_id,
                        tool_id
                    );
                }
            }
        }
    }
    Ok(())
}
