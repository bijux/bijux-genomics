use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, Result};
use bijux_environment::api::{ResolvedImage, RunnerKind};
use uuid::Uuid;

use crate::api::{
    cleanup_execution, execution_memory_mb, run_merge_execution, run_multiqc_execution,
    run_tool_execution, run_validate_execution,
};

#[derive(Debug, Clone)]
pub struct StagePlan {
    pub stage_id: String,
    pub tool: String,
    pub image: ResolvedImage,
    pub runner: RunnerKind,
    pub inputs: Vec<PathBuf>,
    pub out_dir: PathBuf,
    pub outputs: Vec<PathBuf>,
    pub params: serde_json::Value,
    pub aux_images: HashMap<String, ResolvedImage>,
}

#[derive(Debug, Clone)]
pub struct StageResult {
    pub exit_code: i32,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub outputs: Vec<PathBuf>,
    pub metrics_path: Option<PathBuf>,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[derive(Debug)]
struct ExecutionEnvelope {
    exit_code: i32,
    stdout: String,
    stderr: String,
    command: String,
}

/// Execute a single stage plan.
///
/// # Errors
/// Returns an error if the execution fails or the plan is invalid.
#[allow(clippy::too_many_lines)]
pub fn execute_stage_plan(plan: &StagePlan) -> Result<StageResult> {
    let (r1, r2) = match plan.inputs.as_slice() {
        [] => (None, None),
        [r1] => (Some(r1.as_path()), None),
        [r1, r2, ..] => (Some(r1.as_path()), Some(r2.as_path())),
    };
    let r1 = r1.ok_or_else(|| anyhow!("plan inputs missing r1"))?;
    let r1_dir = r1
        .parent()
        .ok_or_else(|| anyhow!("input r1 has no parent directory"))?;
    let container_name = format!("bijux-stage-{}-{}", plan.stage_id, Uuid::new_v4());
    let start = Instant::now();
    let mut outputs_override: Option<Vec<PathBuf>> = None;
    let execution = match plan.stage_id.as_str() {
        "fastq.merge" => {
            let r2 = r2.ok_or_else(|| anyhow!("merge requires r2 input"))?;
            let exec = run_merge_execution(
                &plan.tool,
                &plan.image,
                r1_dir,
                r1,
                r2,
                &plan.out_dir,
                &container_name,
            )?;
            outputs_override = Some(vec![
                exec.merged_fastq.clone(),
                exec.unmerged_r1.clone(),
                exec.unmerged_r2.clone(),
            ]);
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.qc_post" if plan.tool == "multiqc" => {
            let fastqc_image = plan
                .aux_images
                .get("fastqc")
                .ok_or_else(|| anyhow!("fastqc image missing for multiqc qc_post"))?;
            let fastqc_dir = plan.out_dir.join("fastqc");
            std::fs::create_dir_all(&fastqc_dir)?;
            let fastqc_container = format!("bijux-stage-fastqc-{}", Uuid::new_v4());
            let fastqc_exec = run_validate_execution(
                "fastqc",
                fastqc_image,
                r1_dir,
                r1,
                &fastqc_dir,
                &fastqc_container,
            )?;
            cleanup_execution(&fastqc_container)?;
            if fastqc_exec.exit_code != 0 {
                return Err(anyhow!("fastqc exit code {}", fastqc_exec.exit_code));
            }
            let exec =
                run_multiqc_execution(&plan.image, &fastqc_dir, &plan.out_dir, &container_name)?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.validate_pre" | "fastq.qc_post" => {
            let exec = run_validate_execution(
                &plan.tool,
                &plan.image,
                r1_dir,
                r1,
                &plan.out_dir,
                &container_name,
            )?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
        "fastq.screen" => {
            return Err(anyhow!("fastq.screen execution is not implemented yet"));
        }
        _ => {
            let exec = run_tool_execution(
                &plan.tool,
                &plan.image,
                r1_dir,
                r1,
                &plan.out_dir,
                &container_name,
            )?;
            ExecutionEnvelope {
                exit_code: exec.exit_code,
                stdout: exec.stdout,
                stderr: exec.stderr,
                command: exec.command,
            }
        }
    };
    let runtime_s = start.elapsed().as_secs_f64();
    let memory_mb = execution_memory_mb(&container_name)?;
    cleanup_execution(&container_name)?;
    Ok(StageResult {
        exit_code: execution.exit_code,
        runtime_s,
        memory_mb,
        outputs: outputs_override.unwrap_or_else(|| plan.outputs.clone()),
        metrics_path: None,
        stdout: execution.stdout,
        stderr: execution.stderr,
        command: execution.command,
    })
}
