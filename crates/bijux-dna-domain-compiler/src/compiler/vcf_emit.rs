use super::*;

fn vcf_output_kinds(stage: &DomainStage) -> Vec<String> {
    let mut kinds = stage.outputs.iter().map(|port| port.data_type.clone()).collect::<Vec<_>>();
    kinds.sort();
    kinds.dedup();
    kinds
}

fn vcf_stage_default_tools(
    index: &DomainIndex,
    stage: &DomainStage,
    tools: &[String],
) -> Vec<String> {
    index
        .active_defaults
        .get(&stage.stage_id)
        .cloned()
        .filter(|tool_id| tools.iter().any(|candidate| candidate == tool_id))
        .or_else(|| tools.first().cloned())
        .into_iter()
        .collect()
}

fn vcf_stage_default_rationale(index: &DomainIndex, stage: &DomainStage) -> String {
    index.active_default_rationale.get(&stage.stage_id).cloned().unwrap_or_default()
}

fn vcf_metrics_schema(stage: &DomainStage) -> String {
    format!("bijux.{}.v1", stage.stage_id)
}

fn vcf_registry_stage_ids(index: &DomainIndex, tool: &DomainToolLoose) -> Vec<String> {
    let mut stage_ids = tool.stage_ids.clone();
    stage_ids.extend(
        tool.planned_stage_ids
            .iter()
            .filter(|stage_id| {
                index
                    .active_defaults
                    .get(stage_id.as_str())
                    .is_some_and(|default_tool| default_tool == &tool.tool_id)
            })
            .cloned(),
    );
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

/// # Errors
/// Returns an error when VCF post-processing domain views cannot be read or written.
pub(super) fn write_vcf_generated_views(
    domain_dir: &Path,
    ci_registry_dir: &Path,
    ci_stages_dir: &Path,
    ci_tools_dir: &Path,
    container_catalog: &BTreeMap<String, ContainerToolContract>,
    source_commit: &str,
) -> Result<()> {
    let tool_registry_vcf_path = ci_registry_dir.join("tool_registry_vcf.toml");
    let downstream_registry_path = ci_registry_dir.join("tool_registry_vcf_downstream.toml");
    let downstream_required_tools_path = ci_tools_dir.join("required_tools_vcf_downstream.toml");
    let stages_vcf_path = ci_stages_dir.join("stages_vcf.toml");
    let vcf_header =
        generated_header("domain/vcf/** + containers/contracts/catalog.toml", source_commit);
    let vcf_index_path = domain_dir.join("vcf").join("index.yaml");
    let vcf_index: DomainIndex = read_yaml(&vcf_index_path)?;

    let mut stages_vcf_toml = vcf_header.clone();
    let mut registry_stage_toml = String::new();
    let vcf_stage_dir = domain_dir.join("vcf").join("stages");
    for path in collect_yaml_files(&vcf_stage_dir)? {
        let stage: DomainStage = read_yaml(&path)?;
        if stage.scope != "post_vcf" || stage.status == "out_of_scope" {
            continue;
        }
        let tools = if stage.compatible_tools.is_empty() {
            vec!["bcftools".to_string()]
        } else {
            stage.compatible_tools.clone()
        };
        let output_kinds = vcf_output_kinds(&stage);
        let metrics_schema = vcf_metrics_schema(&stage);
        let _ = writeln!(stages_vcf_toml, "[[stages]]");
        let _ = writeln!(stages_vcf_toml, "id = \"{}\"", stage.stage_id);
        let _ = writeln!(stages_vcf_toml, "status = \"{}\"", stage.status);
        let _ = writeln!(stages_vcf_toml, "output_kinds = {}", toml_array(&output_kinds));
        let _ = writeln!(stages_vcf_toml, "experimental = true");
        let _ = writeln!(stages_vcf_toml, "metrics_schema = \"{}\"", metrics_schema);
        let _ = writeln!(stages_vcf_toml, "smoke_required = true");
        let _ = writeln!(stages_vcf_toml, "tools = {}", toml_array(&tools));
        stages_vcf_toml.push('\n');

        if stage.status != "supported" {
            continue;
        }

        let primary = vcf_stage_default_tools(&vcf_index, &stage, &tools);
        let optional = tools
            .iter()
            .filter(|tool_id| !primary.iter().any(|selected| selected == *tool_id))
            .cloned()
            .collect::<Vec<_>>();
        let rationale = vcf_stage_default_rationale(&vcf_index, &stage).replace('"', "'");
        let _ = writeln!(registry_stage_toml, "[[stages]]");
        let _ = writeln!(registry_stage_toml, "id = \"{}\"", stage.stage_id);
        let _ = writeln!(
            registry_stage_toml,
            "required_tool_roles = {}",
            toml_array(&required_tool_roles_for_stage(&stage.stage_id))
        );
        let _ = writeln!(registry_stage_toml, "primary_tools = {}", toml_array(&primary));
        let _ = writeln!(registry_stage_toml, "optional_alternatives = {}", toml_array(&optional));
        registry_stage_toml.push_str("validation_tools = []\n");
        registry_stage_toml.push_str("reporting_tools = []\n");
        let _ = writeln!(
            registry_stage_toml,
            "planned_out_of_scope = {}",
            toml_array(&stage.planned_out_of_scope)
        );
        let _ = writeln!(registry_stage_toml, "default_rationale = \"{rationale}\"");
        registry_stage_toml.push_str("requires_validation = false\n");
        registry_stage_toml.push_str("requires_reporting = false\n\n");
    }

    let mut tools_vcf_toml = vcf_header.clone();
    let mut downstream_vcf_toml = vcf_header;
    let mut downstream_tool_ids = Vec::new();
    let vcf_tool_dir = domain_dir.join("vcf").join("tools");
    for path in collect_yaml_files(&vcf_tool_dir)? {
        let tool: DomainToolLoose = read_yaml(&path)?;
        if tool.scope != "post_vcf" || tool.status == "out_of_scope" {
            continue;
        }
        let contract = container_catalog.get(&tool.tool_id).ok_or_else(|| {
            anyhow!(
                "VCF domain tool `{}` is absent from containers/contracts/catalog.toml",
                tool.tool_id
            )
        })?;
        let stage_ids = vcf_registry_stage_ids(&vcf_index, &tool);
        let _ = writeln!(tools_vcf_toml, "[[tools]]");
        let _ = writeln!(tools_vcf_toml, "id = \"{}\"", tool.tool_id);
        let _ = writeln!(tools_vcf_toml, "tool_id = \"{}\"", tool.tool_id);
        let _ = writeln!(tools_vcf_toml, "domain = \"vcf\"");
        let vcf_status = match contract.support_disposition.as_str() {
            "unsupported" | "retired" | "excluded" => contract.support_disposition.as_str(),
            _ if tool.status == "supported" => "production",
            _ => "planned",
        };
        let _ = writeln!(tools_vcf_toml, "status = \"{vcf_status}\"");
        let _ = writeln!(tools_vcf_toml, "stage_ids = {}", toml_array(&stage_ids));
        let _ = writeln!(tools_vcf_toml, "version = \"{}\"", contract.version);
        let _ = writeln!(tools_vcf_toml, "default_version = \"{}\"", contract.version);
        let _ = writeln!(tools_vcf_toml, "upstream = \"{}\"", contract.upstream);
        let _ = writeln!(tools_vcf_toml, "version_rule = \"pinned\"");
        let _ = writeln!(tools_vcf_toml, "license = \"{}\"", contract.license);
        let _ = writeln!(tools_vcf_toml, "citation = \"{}\"", tool.citation);
        let _ = writeln!(tools_vcf_toml, "pinned_commit = \"{}\"", contract.source_identity());
        let _ = writeln!(tools_vcf_toml, "pin_strategy = \"{}\"", contract.source_pin_strategy());
        let runtimes = contract.runtimes();
        let container_ref = contract.container_ref();
        let _ = writeln!(tools_vcf_toml, "container_ref = \"{container_ref}\"");
        let _ = writeln!(tools_vcf_toml, "runtimes = {}", toml_array(&runtimes));
        let _ = writeln!(tools_vcf_toml, "container = {}", !runtimes.is_empty());
        let _ = writeln!(tools_vcf_toml, "version_cmd = \"{}\"", contract.version_command);
        let _ = writeln!(tools_vcf_toml, "help_cmd = \"{}\"", contract.help_command);
        let _ = writeln!(tools_vcf_toml, "smoke_version_cmd = \"{}\"", contract.version_command);
        let _ = writeln!(tools_vcf_toml, "smoke_help_cmd = \"{}\"", contract.help_command);
        let _ = writeln!(
            tools_vcf_toml,
            "expected_version_regex = \"{}\"",
            contract.expected_version_regex
        );
        let _ = writeln!(tools_vcf_toml, "healthcheck_cmd = \"{}\"", contract.help_command);
        let _ = writeln!(tools_vcf_toml, "expected_bin = \"{}\"", contract.expected_bin);
        let _ = writeln!(
            tools_vcf_toml,
            "expected_artifacts = {}",
            toml_array(&tool.expected_artifacts)
        );
        let _ = writeln!(tools_vcf_toml, "metrics_schema = \"{}\"", tool.metrics_schema_id);
        let comparability_notes = if contract.support_reason.is_empty() {
            &tool.comparability_notes
        } else {
            &contract.support_reason
        };
        let _ = writeln!(
            tools_vcf_toml,
            "comparability_notes = \"{}\"",
            comparability_notes.replace('"', "'")
        );
        let _ = writeln!(tools_vcf_toml, "dockerfile = \"{}\"", contract.dockerfile);
        let _ = writeln!(tools_vcf_toml, "apptainer_def = \"{}\"", contract.apptainer_definition);
        let _ = writeln!(tools_vcf_toml, "require_labels = {}\n", !runtimes.is_empty());

        if tool.tool_id != "bcftools" {
            let Some(row_start) = tools_vcf_toml.rfind("[[tools]]") else {
                bail!("VCF tool row rendering lost the current tool boundary");
            };
            downstream_vcf_toml.push_str(&tools_vcf_toml[row_start..]);
            if vcf_status == "planned" {
                let _ = writeln!(
                    downstream_vcf_toml,
                    "smoke_minimal_rationale = \"No deterministic stage fixture is admitted for this planned tool; help/version probes remain non-qualification checks.\"\n"
                );
            }
            downstream_tool_ids.push(tool.tool_id.clone());
        }
    }
    tools_vcf_toml.push_str(&registry_stage_toml);

    let tools_vcf_toml = normalize_generated_text(&tools_vcf_toml);
    let downstream_vcf_toml = normalize_generated_text(&downstream_vcf_toml);
    let stages_vcf_toml = normalize_generated_text(&stages_vcf_toml);
    write_string(&tool_registry_vcf_path, &tools_vcf_toml)
        .with_context(|| format!("write {}", tool_registry_vcf_path.display()))?;
    write_string(&stages_vcf_path, &stages_vcf_toml)
        .with_context(|| format!("write {}", stages_vcf_path.display()))?;
    write_string(&downstream_registry_path, &downstream_vcf_toml)
        .with_context(|| format!("write {}", downstream_registry_path.display()))?;
    downstream_tool_ids.sort();
    let mut required_tools = generated_header("domain/vcf/**", source_commit);
    required_tools.push_str("schema_version = \"bijux.required_tools.v1\"\n");
    let _ = writeln!(required_tools, "required_tools = {}", toml_array(&downstream_tool_ids));
    write_string(&downstream_required_tools_path, &normalize_generated_text(&required_tools))
        .with_context(|| format!("write {}", downstream_required_tools_path.display()))?;
    println!("generated: {}", tool_registry_vcf_path.display());
    println!("generated: {}", downstream_registry_path.display());
    println!("generated: {}", downstream_required_tools_path.display());
    println!("generated: {}", stages_vcf_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn vcf_registry_stage_ids_exclude_planned_stage_claims() {
        let index = DomainIndex {
            active_defaults: BTreeMap::from([
                ("vcf.impute".to_string(), "beagle".to_string()),
                ("vcf.prepare_reference_panel".to_string(), "bcftools".to_string()),
                ("vcf.phasing".to_string(), "shapeit5".to_string()),
            ]),
            ..DomainIndex::default()
        };
        let tool = DomainToolLoose {
            tool_id: "beagle".to_string(),
            stage_ids: vec!["vcf.imputation_metrics".to_string(), "vcf.impute".to_string()],
            planned_stage_ids: vec![
                "vcf.phasing".to_string(),
                "vcf.prepare_reference_panel".to_string(),
            ],
            ..DomainToolLoose::default()
        };

        assert_eq!(
            vcf_registry_stage_ids(&index, &tool),
            vec!["vcf.imputation_metrics".to_string(), "vcf.impute".to_string()]
        );
    }

    #[test]
    fn vcf_registry_stage_ids_promote_default_planned_claims() {
        let index = DomainIndex {
            active_defaults: BTreeMap::from([
                ("vcf.prepare_reference_panel".to_string(), "bcftools".to_string()),
                ("vcf.postprocess".to_string(), "bcftools".to_string()),
            ]),
            ..DomainIndex::default()
        };
        let tool = DomainToolLoose {
            tool_id: "bcftools".to_string(),
            stage_ids: vec!["vcf.call".to_string()],
            planned_stage_ids: vec![
                "vcf.prepare_reference_panel".to_string(),
                "vcf.postprocess".to_string(),
                "vcf.imputation_metrics".to_string(),
            ],
            ..DomainToolLoose::default()
        };

        assert_eq!(
            vcf_registry_stage_ids(&index, &tool),
            vec![
                "vcf.call".to_string(),
                "vcf.postprocess".to_string(),
                "vcf.prepare_reference_panel".to_string(),
            ]
        );
    }
}
