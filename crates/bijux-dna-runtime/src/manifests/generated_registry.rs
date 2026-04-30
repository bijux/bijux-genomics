use std::path::Path;

use anyhow::{anyhow, Context, Result};

use bijux_dna_core::contract::{
    ArtifactRole, BackendVersionPolicy, Cardinality, ExecutionContract, PortSpec, StageId,
    StageOperatingMode, StageSpec, ToolConstraints, ToolManifest, ToolRegistry, ToolRole,
};
use bijux_dna_core::ids::ToolId;
use bijux_dna_core::prelude::tooling::{ReadCountChangePolicy, StageBehavior};

use super::classification::{
    artifact_kind_from_stage, list_strings, output_artifact_kind_from_stage, parse_stage_semver,
    stable_produced_artifacts, stage_family_from_id, stage_scale_from_row, stage_semantic_from_id,
};
use super::source::experimental_manifests_enabled;

#[allow(clippy::too_many_lines)]
pub(super) fn read_generated_registry(registry_path: &Path) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::default();
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut parsed: toml::Value =
        raw.parse().with_context(|| format!("parse {}", registry_path.display()))?;
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

    for stage in parsed.get("stages").and_then(toml::Value::as_array).cloned().unwrap_or_default() {
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
            stage_family: stage_family_from_id(stage_id_raw),
            semantic_kind: stage_semantic_from_id(stage_id_raw),
            input_kind,
            output_kind,
            produced_artifacts: stable_produced_artifacts(stage_id_raw, output_kind),
            stage_semver: parse_stage_semver(&stage),
            runtime_scale: stage_scale_from_row(&stage),
            inputs: vec![PortSpec {
                artifact_role: match input_kind {
                    bijux_dna_core::contract::ArtifactKind::Fastq => ArtifactRole::Reads,
                    bijux_dna_core::contract::ArtifactKind::Bam => ArtifactRole::Bam,
                    bijux_dna_core::contract::ArtifactKind::Vcf => ArtifactRole::Variant,
                    bijux_dna_core::contract::ArtifactKind::Index => ArtifactRole::Index,
                    bijux_dna_core::contract::ArtifactKind::Metrics => ArtifactRole::MetricsJson,
                    bijux_dna_core::contract::ArtifactKind::Report => ArtifactRole::ReportJson,
                    bijux_dna_core::contract::ArtifactKind::Unknown => ArtifactRole::Unknown,
                },
                name: format!("{}_in", stage_id_raw.replace('.', "_")),
                data_type: format!("{input_kind:?}").to_lowercase(),
                cardinality: Cardinality::Many,
            }],
            outputs: vec![PortSpec {
                artifact_role: match output_kind {
                    bijux_dna_core::contract::ArtifactKind::Fastq => ArtifactRole::Reads,
                    bijux_dna_core::contract::ArtifactKind::Bam => ArtifactRole::Bam,
                    bijux_dna_core::contract::ArtifactKind::Vcf => ArtifactRole::Variant,
                    bijux_dna_core::contract::ArtifactKind::Index => ArtifactRole::Index,
                    bijux_dna_core::contract::ArtifactKind::Metrics => ArtifactRole::MetricsJson,
                    bijux_dna_core::contract::ArtifactKind::Report => ArtifactRole::ReportJson,
                    bijux_dna_core::contract::ArtifactKind::Unknown => ArtifactRole::Unknown,
                },
                name: format!("{}_out", stage_id_raw.replace('.', "_")),
                data_type: format!("{output_kind:?}").to_lowercase(),
                cardinality: Cardinality::Many,
            }],
            parameters: Vec::new(),
            metrics: Vec::new(),
            description: Some("generated from configs/ci/registry/tool_registry.toml".to_string()),
            environment_requirements: Default::default(),
            report_contracts: Vec::new(),
            capability_contract: Default::default(),
            refusal_codes: Vec::new(),
            operating_mode: StageOperatingMode::Enforced,
            behavior: StageBehavior {
                idempotent: stage.get("idempotent").and_then(toml::Value::as_bool).unwrap_or(true),
                mutates_fastq: false,
                report_only: false,
                read_count_change: ReadCountChangePolicy::Stable,
            },
            image_requirements: None,
            extends: None,
        };
        registry.insert_stage(spec);
    }

    for tool in parsed.get("tools").and_then(toml::Value::as_array).cloned().unwrap_or_default() {
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
    registry.sort_tools_for_determinism();
    Ok(registry)
}
