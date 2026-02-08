use anyhow::Result;
use bijux_dna_api::v1::api::run::{dry_run, DryRunRequest};
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
    Ok(())
}
