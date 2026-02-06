use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use bijux_core::contract::{run_dir, Profile, RunSpec};
use bijux_core::contract::{ToolExecutionSpecV1, ToolRegistry};
use bijux_core::foundation::{CommandSpecV1, ContainerImageRefV1};
use bijux_core::ids::{RunId, StageVersion};

use crate::stage_plan::StagePlanV1;
use bijux_core::contract::{ArtifactRef, ArtifactRole, StageIO};
use bijux_core::ids::ArtifactId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunExecutionPlan {
    pub run_id: RunId,
    pub run_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub artifacts_dir: PathBuf,
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

    let run_dir = run_dir(
        &profile.run_base_dir,
        &run_id,
        &run_spec.stage,
        &run_spec.tool,
    );
    let logs_dir = run_dir.join("logs");
    let artifacts_dir = run_dir.join("artifacts");

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

    let stage = StagePlanV1 {
        stage_id: run_spec.stage.clone(),
        stage_version: StageVersion(1),
        tool_id: run_spec.tool.clone(),
        tool_version: "unknown".to_string(),
        image: ContainerImageRefV1 {
            image: tool_manifest.tool_id.to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: tool_manifest.command_template.clone(),
        },
        resources: tool_manifest.constraints.clone(),
        io: StageIO { inputs, outputs },
        out_dir: run_dir.join("stage"),
        params: serde_json::to_value(&run_spec.params).unwrap_or_else(|_| serde_json::json!({})),
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        reason: crate::stage_plan::PlanDecisionReason::default(),
    };

    let tool = ToolExecutionSpecV1 {
        tool_id: run_spec.tool.clone(),
        tool_version: "unknown".to_string(),
        image: ContainerImageRefV1 {
            image: tool_manifest.tool_id.to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: tool_manifest.command_template.clone(),
        },
        resources: tool_manifest.constraints.clone(),
    };

    Ok(RunExecutionPlan {
        run_id,
        run_dir,
        logs_dir,
        artifacts_dir,
        stage,
        tool,
    })
}
