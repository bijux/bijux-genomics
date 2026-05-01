use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use bijux_dna_core::contract::{
    BackendVersionPolicy, ExecutionContract, ImageRequirements, StageId, StageOperatingMode,
    StageParameterSpec, StageSpec, ToolConstraints, ToolManifest, ToolRegistry,
};
use bijux_dna_core::ids::ToolId;
use bijux_dna_core::prelude::tooling::{ReadCountChangePolicy, StageBehavior, StageMetricSpec};

use super::classification::{
    artifact_kind_from_stage, declared_file_name, output_artifact_kind_from_stage, parse_tool_role,
    stable_produced_artifacts, stage_family_from_id, stage_semantic_from_id, to_ports,
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
pub(super) fn read_domain_registry(domain_dir: &Path) -> Result<ToolRegistry> {
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
                    stage_family: stage_family_from_id(&stage.stage_id),
                    semantic_kind: stage_semantic_from_id(&stage.stage_id),
                    input_kind: artifact_kind_from_stage(&stage.stage_id),
                    output_kind: output_artifact_kind_from_stage(&stage.stage_id),
                    produced_artifacts: stable_produced_artifacts(
                        &stage.stage_id,
                        output_artifact_kind_from_stage(&stage.stage_id),
                    ),
                    stage_semver: "1.0.0".to_string(),
                    runtime_scale: bijux_dna_core::contract::RuntimeScale::Small,
                    inputs: to_ports(stage.inputs),
                    outputs: to_ports(stage.outputs),
                    parameters: stage.parameters,
                    metrics: stage.metrics,
                    description: stage.description,
                    environment_requirements: Default::default(),
                    report_contracts: Vec::new(),
                    capability_contract: Default::default(),
                    refusal_codes: Vec::new(),
                    operating_mode: if stage.report_only {
                        StageOperatingMode::Advisory
                    } else {
                        StageOperatingMode::Enforced
                    },
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
                        supported_modes: vec![
                            StageOperatingMode::Simulation,
                            StageOperatingMode::Advisory,
                            StageOperatingMode::Enforced,
                        ],
                        backend_version_policy: BackendVersionPolicy::Pinned,
                        capability_contract: Default::default(),
                    });
                }
            }
        }
    }
    registry.sort_tools_for_determinism();
    Ok(registry)
}
