use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use bijux_dna_core::contract::{
    ArtifactKind, Cardinality, ExecutionContract, ImageRequirements, PortSpec, RuntimeScale,
    StageId, StageParameterSpec, StageSemanticKind, StageSpec, ToolConstraints, ToolManifest,
    ToolRegistry, ToolRole,
};
use bijux_dna_core::ids::ToolId;
use bijux_dna_core::prelude::tooling::StageMetricSpec;
use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize, Default)]
struct DomainPortYaml {
    name: String,
    data_type: String,
    cardinality: String,
}

#[derive(Debug, Deserialize, Default)]
struct DomainToolYaml {
    tool_id: String,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    stage_ids: Vec<String>,
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

fn to_cardinality(raw: &str) -> Cardinality {
    if raw.eq_ignore_ascii_case("one") {
        Cardinality::One
    } else {
        Cardinality::Many
    }
}

fn to_ports(ports: Vec<DomainPortYaml>) -> Vec<PortSpec> {
    ports
        .into_iter()
        .map(|port| PortSpec {
            name: port.name,
            data_type: port.data_type,
            cardinality: to_cardinality(&port.cardinality),
        })
        .collect()
}

fn parse_tool_role(raw: Option<&str>) -> ToolRole {
    match raw {
        Some("diagnostic") => ToolRole::Diagnostic,
        Some("experimental") => ToolRole::Experimental,
        _ => ToolRole::Authoritative,
    }
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
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
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
                    idempotent: true,
                    stage_semver: "1.0.0".to_string(),
                    runtime_scale: RuntimeScale::Small,
                    inputs: to_ports(stage.inputs),
                    outputs: to_ports(stage.outputs),
                    parameters: stage.parameters,
                    metrics: stage.metrics,
                    description: stage.description,
                    mutates_fastq: stage.mutates_fastq,
                    report_only: stage.report_only,
                    may_change_read_count: stage.may_change_read_count,
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
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();
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
                if stage_ids.is_empty() {
                    return Err(anyhow!("{} missing stage_id(s)", path.display()));
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

fn list_strings(table: &toml::Value, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn has_index_suffix(stage_id: &str) -> bool {
    Path::new(stage_id)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("index"))
}

fn stage_semantic_from_id(stage_id: &str) -> StageSemanticKind {
    if has_index_suffix(stage_id) || stage_id.contains("prepare_reference") {
        StageSemanticKind::Index
    } else if stage_id.contains("qc") || stage_id.contains("stats") || stage_id.contains("summary")
    {
        StageSemanticKind::Qc
    } else if stage_id.contains("report") {
        StageSemanticKind::Report
    } else if stage_id.contains("filter") || stage_id.contains("trim") {
        StageSemanticKind::Filter
    } else if stage_id.contains("annot") || stage_id.contains("haplogroup") {
        StageSemanticKind::Annotate
    } else {
        StageSemanticKind::Transform
    }
}

fn artifact_kind_from_stage(stage_id: &str) -> ArtifactKind {
    if stage_id.starts_with("fastq.") {
        ArtifactKind::Fastq
    } else if stage_id.starts_with("bam.") {
        ArtifactKind::Bam
    } else if stage_id.starts_with("vcf.") {
        ArtifactKind::Vcf
    } else {
        ArtifactKind::Unknown
    }
}

fn output_artifact_kind_from_stage(stage_id: &str) -> ArtifactKind {
    if stage_id.contains("qc") || stage_id.contains("stats") || stage_id.contains("summary") {
        ArtifactKind::Metrics
    } else if has_index_suffix(stage_id) {
        ArtifactKind::Index
    } else {
        artifact_kind_from_stage(stage_id)
    }
}

fn stage_scale_from_row(stage: &toml::Value) -> RuntimeScale {
    let mem = stage
        .get("resource_memory_gb")
        .and_then(toml::Value::as_integer)
        .unwrap_or(4);
    let mins = stage
        .get("resource_time_minutes")
        .and_then(toml::Value::as_integer)
        .unwrap_or(30);
    if mem >= 24 || mins >= 180 {
        RuntimeScale::Large
    } else if mem >= 12 || mins >= 90 {
        RuntimeScale::Medium
    } else if mem >= 4 || mins >= 30 {
        RuntimeScale::Small
    } else {
        RuntimeScale::Tiny
    }
}

fn parse_stage_semver(stage: &toml::Value) -> String {
    stage
        .get("stage_semver")
        .and_then(toml::Value::as_str)
        .unwrap_or("1.0.0")
        .to_string()
}

fn stable_produced_artifacts(stage_id: &str, output_kind: ArtifactKind) -> Vec<String> {
    let base = stage_id.replace('.', "_");
    match output_kind {
        ArtifactKind::Fastq => vec![format!("{base}_fastq_out")],
        ArtifactKind::Bam => vec![format!("{base}_bam_out")],
        ArtifactKind::Vcf => vec![format!("{base}_vcf_out")],
        ArtifactKind::Index => vec![format!("{base}_index_out")],
        ArtifactKind::Metrics => vec![format!("{base}_metrics_out")],
        ArtifactKind::Report => vec![format!("{base}_report_out")],
        ArtifactKind::Unknown => vec![format!("{base}_out")],
    }
}

fn find_domain_dir(path: &Path) -> Option<PathBuf> {
    if path.is_dir()
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "domain")
    {
        return Some(path.to_path_buf());
    }
    if path.file_name().and_then(|n| n.to_str()) == Some("tool_registry.toml") {
        let parent = path.parent()?;
        if parent.file_name().and_then(|n| n.to_str()) == Some("configs") {
            let candidate = parent.parent()?.join("domain");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    if path.is_dir() {
        let candidate = path.join("domain");
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
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
    let experimental_enabled = std::env::var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS")
        .ok()
        .is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
    if experimental_enabled {
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
        let Some(stage_id_raw) = stage.get("id").and_then(toml::Value::as_str) else {
            continue;
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
            idempotent: stage
                .get("idempotent")
                .and_then(toml::Value::as_bool)
                .unwrap_or(true),
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
            mutates_fastq: false,
            report_only: false,
            may_change_read_count: false,
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
        let Some(tool_id_raw) = tool.get("id").and_then(toml::Value::as_str) else {
            continue;
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
