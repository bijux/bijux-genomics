mod artifact_catalog;
mod planner_contract;
mod stage_builder;

use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::{run_dir, Profile, RunSpec};
use bijux_dna_core::contract::{ToolExecutionSpecV1, ToolRegistry};
use bijux_dna_core::ids::RunId;

use crate::stage_plan::{PlannedArtifactV1, StagePlanV1};
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole};
use bijux_dna_core::ids::ArtifactId;

pub use artifact_catalog::artifact_kind_schema;
pub use planner_contract::*;
pub use stage_builder::{build_stage_plan, build_tool_execution_spec, validate_stage_outputs};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunExecutionPlan {
    pub run_id: RunId,
    pub run_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub artifacts_dir: PathBuf,
    pub planned_artifacts: Vec<PlannedArtifactV1>,
    pub stage: StagePlanV1,
    pub tool: ToolExecutionSpecV1,
}

pub trait Executor {
    /// # Errors
    /// Returns an error if execution fails.
    fn run(&self, plan: &RunExecutionPlan) -> Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub struct DryRunExecutor;

impl Executor for DryRunExecutor {
    fn run(&self, _plan: &RunExecutionPlan) -> Result<()> {
        Ok(())
    }
}

/// # Errors
/// Returns an error if the registry is missing the requested stage or tool.
pub fn build_run_execution_plan(
    run_spec: &RunSpec,
    registry: &ToolRegistry,
    profile: &Profile,
    run_id: RunId,
) -> Result<RunExecutionPlan> {
    let stage_spec = registry
        .stages()
        .get(&run_spec.stage)
        .ok_or_else(|| anyhow!("missing stage {}", run_spec.stage.0))?;
    let tool_manifest = registry
        .tool_by_id(&run_spec.stage, &run_spec.tool)
        .ok_or_else(|| anyhow!("missing tool {} for {}", run_spec.tool.0, run_spec.stage.0))?;

    let run_dir = run_dir(&profile.run_base_dir, &run_id, &run_spec.stage, &run_spec.tool);
    let logs_dir = run_dir.join("logs");
    let artifacts_dir = run_dir.join("artifacts");
    validate_stage_outputs(stage_spec, run_spec)?;

    let inputs = stage_spec
        .inputs
        .iter()
        .map(|port| {
            ArtifactRef::required(
                ArtifactId::new(port.name.clone()),
                PathBuf::from(&port.name),
                ArtifactRole::Unknown,
            )
        })
        .collect();
    let outputs = stage_spec
        .outputs
        .iter()
        .map(|port| {
            ArtifactRef::required(
                ArtifactId::new(port.name.clone()),
                PathBuf::from(&port.name),
                ArtifactRole::Unknown,
            )
        })
        .collect();

    let stage = build_stage_plan(run_spec, tool_manifest, stage_spec, &run_dir, inputs, outputs)?;

    let planned_artifacts = stage
        .io
        .outputs
        .iter()
        .map(|artifact| {
            let role = artifact.role.as_str().to_string();
            let (kind, schema) = artifact_kind_schema(&role);
            PlannedArtifactV1 {
                artifact_id: artifact.name.0.to_string(),
                role,
                path: artifact.path.to_string_lossy().to_string(),
                kind: kind.to_string(),
                schema: schema.to_string(),
            }
        })
        .collect();

    let tool = build_tool_execution_spec(run_spec, tool_manifest);

    Ok(RunExecutionPlan {
        run_id,
        run_dir,
        logs_dir,
        artifacts_dir,
        planned_artifacts,
        stage,
        tool,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use bijux_dna_core::contract::ToolRole;
    use bijux_dna_core::contract::{
        ArtifactKind, Cardinality, ExecutionContract, PathSpec, PortSpec, RunSpec, RuntimeScale,
        StageSemanticKind, StageSpec, ToolConstraints, ToolManifest,
    };
    use bijux_dna_core::id_catalog;
    use bijux_dna_core::ids::{StageId, ToolId};
    use bijux_dna_core::prelude::tooling::StageBehavior;

    #[test]
    fn build_stage_plan_copies_run_params_into_effective_params() {
        let run_spec = RunSpec {
            stage: StageId::from_static(id_catalog::FASTQ_TRIM),
            tool: ToolId::from_static("fastp"),
            paths: PathSpec { input: Vec::new(), output: Vec::new(), work: PathBuf::from("work") },
            params: BTreeMap::from([
                ("quality".to_string(), "20".to_string()),
                ("trim_poly_g".to_string(), "true".to_string()),
            ]),
        };
        let tool_manifest = ToolManifest {
            tool_id: ToolId::from_static("fastp"),
            stage_id: StageId::from_static(id_catalog::FASTQ_TRIM),
            role: ToolRole::Authoritative,
            command_template: vec!["fastp".to_string()],
            outputs: Vec::new(),
            metrics_parser: None,
            constraints: ToolConstraints::default(),
            execution_contract: ExecutionContract::default(),
        };
        let stage_spec = StageSpec {
            stage_id: StageId::from_static(id_catalog::FASTQ_TRIM),
            semantic_kind: StageSemanticKind::Transform,
            input_kind: ArtifactKind::Fastq,
            output_kind: ArtifactKind::Fastq,
            produced_artifacts: vec!["trimmed_reads".to_string()],
            stage_semver: "1.0.0".to_string(),
            runtime_scale: RuntimeScale::Small,
            inputs: Vec::new(),
            outputs: vec![PortSpec {
                name: "trimmed_reads".to_string(),
                data_type: "fastq".to_string(),
                cardinality: Cardinality::One,
            }],
            parameters: Vec::new(),
            metrics: Vec::new(),
            description: None,
            behavior: StageBehavior::default(),
            image_requirements: None,
            extends: None,
        };

        let plan = match super::build_stage_plan(
            &run_spec,
            &tool_manifest,
            &stage_spec,
            &PathBuf::from("runs/run-1"),
            Vec::new(),
            Vec::new(),
        ) {
            Ok(plan) => plan,
            Err(error) => panic!("build stage plan failed: {error}"),
        };

        assert_eq!(plan.params, serde_json::json!({"quality": "20", "trim_poly_g": "true"}));
        assert_eq!(plan.effective_params, plan.params);
    }

    #[test]
    fn validate_stage_outputs_rejects_stage_id_mismatch() {
        let run_spec = RunSpec {
            stage: StageId::from_static(id_catalog::FASTQ_TRIM),
            tool: ToolId::from_static("fastp"),
            paths: PathSpec { input: Vec::new(), output: Vec::new(), work: PathBuf::from("work") },
            params: BTreeMap::new(),
        };
        let stage_spec = StageSpec {
            stage_id: StageId::from_static(id_catalog::FASTQ_FILTER),
            semantic_kind: StageSemanticKind::Transform,
            input_kind: ArtifactKind::Fastq,
            output_kind: ArtifactKind::Fastq,
            produced_artifacts: vec!["trimmed_reads".to_string()],
            stage_semver: "1.0.0".to_string(),
            runtime_scale: RuntimeScale::Small,
            inputs: Vec::new(),
            outputs: vec![PortSpec {
                name: "trimmed_reads".to_string(),
                data_type: "fastq".to_string(),
                cardinality: Cardinality::One,
            }],
            parameters: Vec::new(),
            metrics: Vec::new(),
            description: None,
            behavior: StageBehavior::default(),
            image_requirements: None,
            extends: None,
        };

        let error = match super::validate_stage_outputs(&stage_spec, &run_spec) {
            Ok(()) => panic!("stage output contract must match the requested run stage"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("output contract belongs to"));
    }

    #[test]
    fn validate_stage_outputs_rejects_duplicate_output_names() {
        let run_spec = RunSpec {
            stage: StageId::from_static(id_catalog::FASTQ_TRIM),
            tool: ToolId::from_static("fastp"),
            paths: PathSpec { input: Vec::new(), output: Vec::new(), work: PathBuf::from("work") },
            params: BTreeMap::new(),
        };
        let stage_spec = StageSpec {
            stage_id: StageId::from_static(id_catalog::FASTQ_TRIM),
            semantic_kind: StageSemanticKind::Transform,
            input_kind: ArtifactKind::Fastq,
            output_kind: ArtifactKind::Fastq,
            produced_artifacts: vec!["trimmed_reads".to_string()],
            stage_semver: "1.0.0".to_string(),
            runtime_scale: RuntimeScale::Small,
            inputs: Vec::new(),
            outputs: vec![
                PortSpec {
                    name: "trimmed_reads".to_string(),
                    data_type: "fastq".to_string(),
                    cardinality: Cardinality::One,
                },
                PortSpec {
                    name: "trimmed_reads".to_string(),
                    data_type: "fastq".to_string(),
                    cardinality: Cardinality::One,
                },
            ],
            parameters: Vec::new(),
            metrics: Vec::new(),
            description: None,
            behavior: StageBehavior::default(),
            image_requirements: None,
            extends: None,
        };

        let error = match super::validate_stage_outputs(&stage_spec, &run_spec) {
            Ok(()) => panic!("duplicate output names must fail validation"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("duplicate output contract entry"));
    }
}
