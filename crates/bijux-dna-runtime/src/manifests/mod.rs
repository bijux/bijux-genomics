mod classification;

use std::path::Path;

use anyhow::{anyhow, Context, Result};

use bijux_dna_core::contract::{
    Cardinality, ExecutionContract, ImageRequirements, PortSpec, RuntimeScale, StageId,
    StageParameterSpec, StageSpec, ToolConstraints, ToolManifest, ToolRegistry, ToolRole,
};
use bijux_dna_core::ids::ToolId;
use bijux_dna_core::prelude::tooling::{ReadCountChangePolicy, StageBehavior, StageMetricSpec};
use serde::Deserialize;

use self::classification::{
    artifact_kind_from_stage, declared_file_name, experimental_manifests_enabled, find_domain_dir,
    list_strings, output_artifact_kind_from_stage, parse_stage_semver, parse_tool_role,
    stable_produced_artifacts, stage_scale_from_row, stage_semantic_from_id, to_ports,
    DomainPortYaml,
};

#[derive(Debug, Deserialize, Default)]
struct DomainStageYaml {
    stage_id: String,
    #[serde(default)]
    inputs: Vec<DomainPortYaml>,
    #[serde(default)]
    outputs: Vec<DomainPortYaml>,
    #[serde(default)]
    parameters: Vec<StageParameterSpec>,
    #[serde(default)]
    metrics: Vec<StageMetricSpec>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    mutates_fastq: bool,
    #[serde(default)]
    report_only: bool,
    #[serde(default)]
    may_change_read_count: bool,
    #[serde(default)]
    image_requirements: Option<ImageRequirements>,
    #[serde(default)]
    extends: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct DomainToolYaml {
    tool_id: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    planned_stage_ids: Vec<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    command_template: Vec<String>,
    #[serde(default)]
    outputs: Vec<DomainPortYaml>,
    #[serde(default)]
    metrics_parser: Option<String>,
    #[serde(default)]
    constraints: Option<ToolConstraints>,
    #[serde(default)]
    execution_contract: Option<ExecutionContract>,
}

fn load_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

#[allow(clippy::too_many_lines)]
fn read_domain_registry(domain_dir: &Path) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::default();
    for domain_name in ["fastq", "bam"] {
        let stages_dir = domain_dir.join(domain_name).join("stages");
        if stages_dir.exists() {
            for entry in std::fs::read_dir(&stages_dir)
                .with_context(|| format!("read {}", stages_dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                    continue;
                }
                let name = declared_file_name(&path)?;
                if name.starts_with('_') {
                    continue;
                }
                let stage: DomainStageYaml = load_yaml(&path)?;
                if stage.stage_id.trim().is_empty() {
                    return Err(anyhow!("{} missing stage_id", path.display()));
                }
                let stage_id = StageId::try_from(stage.stage_id.as_str())
                    .map_err(|err| anyhow!("invalid stage id `{}`: {err}", stage.stage_id))?;
                let extends = match stage.extends.as_deref() {
                    Some(v) if !v.is_empty() && !v.starts_with('_') => StageId::try_from(v).ok(),
                    _ => None,
                };
                registry.insert_stage(StageSpec {
                    stage_id,
                    semantic_kind: stage_semantic_from_id(&stage.stage_id),
                    input_kind: artifact_kind_from_stage(&stage.stage_id),
                    output_kind: output_artifact_kind_from_stage(&stage.stage_id),
                    produced_artifacts: stable_produced_artifacts(
                        &stage.stage_id,
                        output_artifact_kind_from_stage(&stage.stage_id),
                    ),
                    stage_semver: "1.0.0".to_string(),
                    runtime_scale: RuntimeScale::Small,
                    inputs: to_ports(stage.inputs),
                    outputs: to_ports(stage.outputs),
                    parameters: stage.parameters,
                    metrics: stage.metrics,
                    description: stage.description,
                    behavior: StageBehavior {
                        idempotent: true,
                        mutates_fastq: stage.mutates_fastq,
                        report_only: stage.report_only,
                        read_count_change: ReadCountChangePolicy::from_bool(
                            stage.may_change_read_count,
                        ),
                    },
                    image_requirements: stage.image_requirements,
                    extends,
                });
            }
        }

        let tools_dir = domain_dir.join(domain_name).join("tools");
        if tools_dir.exists() {
            for entry in std::fs::read_dir(&tools_dir)
                .with_context(|| format!("read {}", tools_dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
                    continue;
                }
                let name = declared_file_name(&path)?;
                if name.starts_with('_') {
                    continue;
                }
                let tool: DomainToolYaml = load_yaml(&path)?;
                if tool.tool_id.trim().is_empty() {
                    return Err(anyhow!("{} missing tool_id", path.display()));
                }
                let tool_id = ToolId::try_from(tool.tool_id.as_str())
                    .map_err(|err| anyhow!("invalid tool id `{}`: {err}", tool.tool_id))?;
                let mut stage_ids = Vec::new();
                if let Some(stage_id) = tool.stage_id {
                    stage_ids.push(stage_id);
                }
                stage_ids.extend(tool.stage_ids);
                let mut declared_stage_ids = stage_ids.clone();
                declared_stage_ids.extend(tool.planned_stage_ids);
                if declared_stage_ids.is_empty() {
                    return Err(anyhow!("{} missing stage_id(s)", path.display()));
                }
                if matches!(tool.status.as_deref(), Some("supported")) && stage_ids.is_empty() {
                    return Err(anyhow!(
                        "{} missing governed stage_ids for supported tool {}",
                        path.display(),
                        tool.tool_id
                    ));
                }
                if stage_ids.is_empty() {
                    continue;
                }
                for stage_id_raw in stage_ids {
                    let stage_id = StageId::try_from(stage_id_raw.as_str())
                        .map_err(|err| anyhow!("invalid stage id `{stage_id_raw}`: {err}"))?;
                    registry.insert_tool(ToolManifest {
                        tool_id: tool_id.clone(),
                        stage_id,
                        role: parse_tool_role(tool.role.as_deref()),
                        command_template: tool.command_template.clone(),
                        outputs: to_ports(tool.outputs.clone()),
                        metrics_parser: tool.metrics_parser.clone(),
                        constraints: tool.constraints.clone().unwrap_or_default(),
                        execution_contract: tool.execution_contract.clone().unwrap_or_default(),
                    });
                }
            }
        }
    }
    registry.sort_tools_for_determinism();
    Ok(registry)
}

/// # Errors
/// Returns an error if registry config cannot be read or parsed.
#[allow(clippy::too_many_lines)]
pub fn load_manifests(source_path: &Path) -> Result<ToolRegistry> {
    if let Some(domain_dir) = find_domain_dir(source_path) {
        return read_domain_registry(&domain_dir);
    }

    let mut registry = ToolRegistry::default();
    let registry_path = if source_path.is_dir() {
        bijux_dna_infra::configs_file(source_path, "ci/registry/tool_registry.toml")
    } else {
        source_path.to_path_buf()
    };
    if !registry_path.exists() {
        return Err(anyhow!(
            "registry file {} does not exist",
            registry_path.display()
        ));
    }
    let raw = std::fs::read_to_string(&registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut parsed: toml::Value = raw
        .parse()
        .with_context(|| format!("parse {}", registry_path.display()))?;
    if experimental_manifests_enabled() {
        let experimental_path = registry_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("tool_registry_experimental.toml");
        if experimental_path.exists() {
            let exp_raw = std::fs::read_to_string(&experimental_path)
                .with_context(|| format!("read {}", experimental_path.display()))?;
            let exp: toml::Value = exp_raw
                .parse()
                .with_context(|| format!("parse {}", experimental_path.display()))?;
            if let Some(exp_tools) = exp.get("tools").and_then(toml::Value::as_array) {
                let current = parsed
                    .as_table_mut()
                    .and_then(|table| table.get_mut("tools"))
                    .and_then(toml::Value::as_array_mut);
                if let Some(current_tools) = current {
                    current_tools.extend(exp_tools.iter().cloned());
                }
            }
        }
    }

    for stage in parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(stage_id_raw) = stage
            .get("id")
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Err(anyhow!("generated stage registry row missing declared id"));
        };
        let stage_id = StageId::try_from(stage_id_raw)
            .map_err(|err| anyhow!("invalid stage id `{stage_id_raw}`: {err}"))?;
        let input_kind = artifact_kind_from_stage(stage_id_raw);
        let output_kind = output_artifact_kind_from_stage(stage_id_raw);
        let spec = StageSpec {
            stage_id,
            semantic_kind: stage_semantic_from_id(stage_id_raw),
            input_kind,
            output_kind,
            produced_artifacts: stable_produced_artifacts(stage_id_raw, output_kind),
            stage_semver: parse_stage_semver(&stage),
            runtime_scale: stage_scale_from_row(&stage),
            inputs: vec![PortSpec {
                name: format!("{}_in", stage_id_raw.replace('.', "_")),
                data_type: format!("{input_kind:?}").to_lowercase(),
                cardinality: Cardinality::Many,
            }],
            outputs: vec![PortSpec {
                name: format!("{}_out", stage_id_raw.replace('.', "_")),
                data_type: format!("{output_kind:?}").to_lowercase(),
                cardinality: Cardinality::Many,
            }],
            parameters: Vec::new(),
            metrics: Vec::new(),
            description: Some("generated from configs/ci/registry/tool_registry.toml".to_string()),
            behavior: StageBehavior {
                idempotent: stage
                    .get("idempotent")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(true),
                mutates_fastq: false,
                report_only: false,
                read_count_change: ReadCountChangePolicy::Stable,
            },
            image_requirements: None,
            extends: None,
        };
        registry.insert_stage(spec);
    }

    for tool in parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(tool_id_raw) = tool
            .get("id")
            .and_then(toml::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Err(anyhow!("generated tool registry row missing declared id"));
        };
        for stage_id_raw in list_strings(&tool, "stage_ids") {
            let tool_id = ToolId::try_from(tool_id_raw)
                .map_err(|err| anyhow!("invalid tool id `{tool_id_raw}`: {err}"))?;
            let stage_id = StageId::try_from(stage_id_raw.as_str())
                .map_err(|err| anyhow!("invalid stage id `{stage_id_raw}`: {err}"))?;
            registry.insert_tool(ToolManifest {
                tool_id,
                stage_id,
                role: ToolRole::default(),
                command_template: Vec::new(),
                outputs: Vec::new(),
                metrics_parser: None,
                constraints: ToolConstraints::default(),
                execution_contract: ExecutionContract::default(),
            });
        }
    }
    registry.sort_tools_for_determinism();
    Ok(registry)
}
