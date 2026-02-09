use anyhow::Result;
use bijux_dna_api::v1::api::run::{dry_run, execute, DryRunRequest, ExecuteRequest, RuntimeKind};
use bijux_dna_core::contract::{ExecutionGraph, PlanPolicy};

#[test]
fn dry_run_emits_manifest_and_graph_without_execution() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "test-planner",
        PlanPolicy::PreferAccuracy,
        Vec::new(),
        Vec::new(),
    )?;
    let request = DryRunRequest {
        graph,
        run_dir: temp.path().to_path_buf(),
        profile_id: "fastq-to-fastq__default__v1".to_string(),
    };
    let response = dry_run(&request)?;
    assert!(response.graph_path.exists());
    assert!(response.manifest_path.exists());
    assert!(temp.path().join("run_summary.json").exists());
    Ok(())
}

#[test]
fn execute_emits_run_summary_artifact() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "test-planner",
        PlanPolicy::PreferAccuracy,
        Vec::new(),
        Vec::new(),
    )?;
    let response = execute(&ExecuteRequest {
        graph,
        runner: RuntimeKind::Docker,
        run_dir: temp.path().to_path_buf(),
    })?;
    let run_dir = response
        .manifest_path
        .parent()
        .expect("manifest has parent directory")
        .to_path_buf();
    assert!(run_dir.join("summary").join("run_summary.json").exists());
    Ok(())
}
