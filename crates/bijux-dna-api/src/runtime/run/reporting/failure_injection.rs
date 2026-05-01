use super::Result;
use crate::request_args::{ExecuteRequest, ExecuteResponse};
use anyhow::anyhow;
use bijux_dna_core::contract::{ExecutionGraph, ExecutionStep, PlanPolicy};
use bijux_dna_core::ids::{ArtifactId, StageId, StepId};
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1, StageIO, ToolConstraints,
};
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runtime::run_layout::RunExecutionModeV1;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

/// Execute a local failure-injection scenario and return captured failure evidence.
///
/// # Errors
/// Returns an error if scenario preparation or execution fails.
pub fn run_local_failure_injection(run_dir: &Path, scenario: &str) -> Result<serde_json::Value> {
    bijux_dna_infra::ensure_dir(run_dir)?;
    let input = run_dir.join("reads.fastq");
    bijux_dna_infra::write_bytes(&input, b"@read\nACGT\n+\n!!!!\n")?;
    let output = run_dir.join("out").join("result.json");
    let command = scenario_command(scenario, &output)?;
    let graph = failure_graph(run_dir, &input, &output, scenario, command)?;

    let execute_request = ExecuteRequest {
        graph,
        runner: RuntimeKind::Local,
        run_dir: run_dir.join("run"),
        mode: RunExecutionModeV1::Enforced,
    };
    let should_cancel = scenario == "cancel";
    let cancel_target_dir = execute_request.run_dir.clone();
    let cancel_thread = should_cancel.then(|| {
        std::thread::spawn(move || {
            let started = std::time::Instant::now();
            while started.elapsed() < Duration::from_secs(5) {
                if cancel_target_dir.join("run_control.json").exists() {
                    let _ = super::cancel_run(&cancel_target_dir);
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        })
    });
    let response = super::execute(&execute_request)?;
    if let Some(thread) = cancel_thread {
        let _ = thread.join();
    }
    summarize_failure_injection(scenario, &response)
}

fn summarize_failure_injection(
    scenario: &str,
    response: &ExecuteResponse,
) -> Result<serde_json::Value> {
    let failure = response
        .failure_path
        .as_ref()
        .and_then(|path| std::fs::read(path).ok())
        .and_then(|raw| serde_json::from_slice::<serde_json::Value>(&raw).ok());
    Ok(serde_json::json!({
        "schema_version": "bijux.failure_injection.v1",
        "scenario": scenario,
        "state": response.state,
        "failure": failure,
        "run_dir": response
            .manifest_path
            .parent()
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        "failure_path": response.failure_path.as_ref().map(|path| path.display().to_string()),
    }))
}

fn scenario_command(scenario: &str, output: &Path) -> Result<Vec<String>> {
    let script = match scenario {
        "timeout" => format!("set -eu; sleep 2; printf '{{\"ok\":true}}' > '{}'", output.display()),
        "cancel" => format!("set -eu; sleep 2; printf '{{\"ok\":true}}' > '{}'", output.display()),
        "missing_output" => {
            "set -eu; printf 'no output declared artifact written' > /dev/null".to_string()
        }
        "corrupt_output" => {
            format!("set -eu; printf '{{\"unterminated\"' > '{}'", output.display())
        }
        "nonzero_exit" => "set -eu; exit 7".to_string(),
        "interrupted_process" => "set -eu; kill -INT $$".to_string(),
        "partial_files" => {
            format!("set -eu; printf '{{\"partial\":' > '{}'; exit 1", output.display())
        }
        _ => return Err(anyhow!("unknown failure injection scenario: {scenario}")),
    };
    Ok(vec!["sh".to_string(), "-c".to_string(), script])
}

fn failure_graph(
    run_dir: &Path,
    input: &Path,
    output: &Path,
    scenario: &str,
    command: Vec<String>,
) -> Result<ExecutionGraph> {
    let role =
        if scenario == "corrupt_output" { ArtifactRole::ReportJson } else { ArtifactRole::Reads };
    let step = ExecutionStep {
        step_id: StepId::new("fastq.validate_reads"),
        stage_id: StageId::new("fastq.validate_reads"),
        command: CommandSpecV1 { template: command },
        image: ContainerImageRefV1 {
            image: "example/failure-injection:1".to_string(),
            digest: Some("sha256:deadbeef".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![ArtifactSpec::required(
                ArtifactId::new("reads"),
                input.to_path_buf(),
                ArtifactRole::Reads,
            )],
            outputs: vec![ArtifactSpec::required(
                ArtifactId::new("result"),
                output.to_path_buf(),
                role,
            )],
        },
        out_dir: run_dir.join("step-out"),
        aux_images: BTreeMap::default(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    };
    let mut graph = ExecutionGraph::new(
        "fastq-to-fastq__failure_injection__v1",
        "api.failure.injection",
        PlanPolicy::PreferAccuracy,
        vec![step],
        Vec::new(),
    )?;
    if scenario == "timeout" {
        graph = graph.with_step_timeout(Some(1));
    }
    Ok(graph)
}
