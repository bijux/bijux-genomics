use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::{run_dir, Profile, RunSpec};
use bijux_dna_core::contract::{ToolExecutionSpecV1, ToolRegistry};
use bijux_dna_core::ids::{RunId, StageVersion};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1};

use crate::stage_plan::{PlannedArtifactV1, StagePlanV1};
use bijux_dna_core::contract::{ArtifactRef, ArtifactRole, StageIO};
use bijux_dna_core::ids::ArtifactId;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    let run_dir = run_dir(
        &profile.run_base_dir,
        &run_id,
        &run_spec.stage,
        &run_spec.tool,
    );
    let logs_dir = run_dir.join("logs");
    let artifacts_dir = run_dir.join("artifacts");
    if stage_spec.outputs.is_empty() {
        return Err(anyhow!(
            "stage {} has no declared outputs; planning requires explicit output contract",
            run_spec.stage.0
        ));
    }
    for output in &stage_spec.outputs {
        if output.name.trim().is_empty() || output.data_type.trim().is_empty() {
            return Err(anyhow!(
                "stage {} has invalid output contract entry (name/data_type must be non-empty)",
                run_spec.stage.0
            ));
        }
    }

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
        params: serde_json::to_value(&run_spec.params).map_err(|err| {
            anyhow!(
                "failed to serialize run parameters for {}: {err}",
                run_spec.stage.0
            )
        })?,
        effective_params: serde_json::json!({}),
        aux_images: BTreeMap::new(),
        reason: crate::stage_plan::PlanDecisionReason::default(),
    };

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
        planned_artifacts,
        stage,
        tool,
    })
}

fn artifact_kind_schema(role: &str) -> (&'static str, &'static str) {
    match role {
        "reads" | "trimmed_reads" => ("fastq", "bijux.artifact.fastq.v1"),
        "bam" | "dedup_bam" => ("bam", "bijux.artifact.bam.v1"),
        "report_json" | "metrics_json" | "summary_json" => {
            ("json", "bijux.artifact.report_json.v1")
        }
        "summary_tsv" => ("tsv", "bijux.artifact.summary_tsv.v1"),
        "report_html" => ("html", "bijux.artifact.report_html.v1"),
        "log" => ("log", "bijux.artifact.log.v1"),
        "index" => ("index", "bijux.artifact.index.v1"),
        "metrics_envelope" => ("json", "bijux.metrics.envelope.v1"),
        "stage_report" => ("json", "bijux.stage_report.v1"),
        _ => ("file", "bijux.artifact.file.v1"),
    }
}
