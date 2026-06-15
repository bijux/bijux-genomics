use anyhow::Result;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_index_reference_plan_uses_governed_repo_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_fastq::stage_api::local_index_reference_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "fastq.index_reference");
    assert_eq!(plan.tool_id.as_str(), "bowtie2_build");
    assert_eq!(plan.resources.threads, 4);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(
        plan.out_dir,
        PathBuf::from("benchmarks/readiness/local-ready/fastq.index_reference")
    );

    let input = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference_fasta")
        .unwrap_or_else(|| panic!("reference_fasta input missing from local-ready plan"));
    assert_eq!(input.path, PathBuf::from("assets/reference/contaminants/references/phix174.fasta"));

    let reference_index = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference_index")
        .unwrap_or_else(|| panic!("reference_index output missing from local-ready plan"));
    assert_eq!(
        reference_index.path,
        PathBuf::from("benchmarks/readiness/local-ready/fastq.index_reference/reference_index/bowtie2/reference")
    );

    let report_json = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "report_json")
        .unwrap_or_else(|| panic!("report_json output missing from local-ready plan"));
    assert_eq!(
        report_json.path,
        PathBuf::from(
            "benchmarks/readiness/local-ready/fastq.index_reference/index_reference_report.json"
        )
    );

    assert_eq!(
        plan.params["reference_fasta"],
        serde_json::json!("assets/reference/contaminants/references/phix174.fasta")
    );
    assert_eq!(plan.params["tool"], serde_json::json!("bowtie2_build"));
    assert_eq!(plan.params["threads"], serde_json::json!(4));
    assert_eq!(plan.resources.mem_gb, 8);
    assert!(
        plan.command.template[2].contains(
            "bowtie2-build --threads 4 'assets/reference/contaminants/references/phix174.fasta' 'benchmarks/readiness/local-ready/fastq.index_reference/reference_index/bowtie2/reference'"
        ),
        "local-ready plan command must materialize the governed bowtie2-build dry-run command"
    );
    Ok(())
}

#[test]
fn local_index_reference_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_index_reference_plan;
}
