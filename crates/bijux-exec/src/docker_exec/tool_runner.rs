use std::time::Duration;

use anyhow::Result;
use bijux_environment::api::ResolvedImage;

use super::run_tool::ExecutionOutput;
use bijux_runner::docker::executor::{
    assess_execution, execute_plan, execute_plan_with_timeout, ExecutionOutput as RunnerOutput,
    StageExecutionPlan,
};

pub trait ToolRunner {
    fn run(
        &self,
        plan: &StageExecutionPlan,
        image: &ResolvedImage,
        input_mount: &std::path::Path,
        output_mount: &std::path::Path,
        container_name: &str,
    ) -> Result<ExecutionOutput>;

    fn run_with_timeout(
        &self,
        plan: &StageExecutionPlan,
        image: &ResolvedImage,
        input_mount: &std::path::Path,
        output_mount: &std::path::Path,
        container_name: &str,
        timeout: Duration,
    ) -> Result<ExecutionOutput>;
}

pub struct DockerToolRunner;

impl ToolRunner for DockerToolRunner {
    fn run(
        &self,
        plan: &StageExecutionPlan,
        image: &ResolvedImage,
        input_mount: &std::path::Path,
        output_mount: &std::path::Path,
        container_name: &str,
    ) -> Result<ExecutionOutput> {
        let mut output = execute_plan(plan, image, input_mount, output_mount, container_name)?;
        let assessment = assess_execution(output.exit_code, &plan.expected_outputs);
        if !assessment.success {
            output.exit_code = 1;
            output.stderr = assessment
                .reason
                .unwrap_or_else(|| "execution_failed".to_string());
        }
        Ok(ExecutionOutput {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            output_fastq: plan.output_fastq.clone(),
            command: output.command,
        })
    }

    fn run_with_timeout(
        &self,
        plan: &StageExecutionPlan,
        image: &ResolvedImage,
        input_mount: &std::path::Path,
        output_mount: &std::path::Path,
        container_name: &str,
        timeout: Duration,
    ) -> Result<ExecutionOutput> {
        let mut output: RunnerOutput = execute_plan_with_timeout(
            plan,
            image,
            input_mount,
            output_mount,
            container_name,
            timeout,
        )?;
        let assessment = assess_execution(output.exit_code, &plan.expected_outputs);
        if !assessment.success {
            output.exit_code = 1;
            output.stderr = assessment
                .reason
                .unwrap_or_else(|| "execution_failed".to_string());
        }
        Ok(ExecutionOutput {
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            output_fastq: plan.output_fastq.clone(),
            command: output.command,
        })
    }
}
