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
fn local_align_plan_uses_governed_repo_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_bam::stage_api::local_align_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "bam.align");
    assert_eq!(plan.tool_id.as_str(), "bowtie2");
    assert_eq!(plan.resources.threads, 4);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("target/local-ready/bam.align"));

    let input_r1 = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "fastq_r1")
        .unwrap_or_else(|| panic!("fastq_r1 input missing from local-ready plan"));
    assert_eq!(input_r1.path, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));

    let input_r2 = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "fastq_r2")
        .unwrap_or_else(|| panic!("fastq_r2 input missing from local-ready plan"));
    assert_eq!(input_r2.path, PathBuf::from("assets/toy/core-v1/fastq/reads_2.fastq"));

    let reference = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference")
        .unwrap_or_else(|| panic!("reference input missing from local-ready plan"));
    assert_eq!(
        reference.path,
        PathBuf::from("assets/reference/host/references/toy_host_reference.fasta")
    );

    let reference_index = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reference_index")
        .unwrap_or_else(|| panic!("reference_index input missing from local-ready plan"));
    assert_eq!(
        reference_index.path,
        PathBuf::from("assets/reference/host/references/toy_host_reference")
    );

    let aligned_bam = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "align_bam")
        .unwrap_or_else(|| panic!("align_bam output missing from local-ready plan"));
    assert_eq!(aligned_bam.path, PathBuf::from("target/local-ready/bam.align/align.bam"));

    let aligned_bai = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "align_bai")
        .unwrap_or_else(|| panic!("align_bai output missing from local-ready plan"));
    assert_eq!(aligned_bai.path, PathBuf::from("target/local-ready/bam.align/align.bam.bai"));

    assert_eq!(
        plan.params["reference"],
        serde_json::json!("assets/reference/host/references/toy_host_reference.fasta")
    );
    assert_eq!(
        plan.params["reference_index"],
        serde_json::json!("assets/reference/host/references/toy_host_reference")
    );
    assert_eq!(
        plan.params["reference_fai"],
        serde_json::json!("assets/reference/host/references/toy_host_reference.fasta.fai")
    );
    assert_eq!(
        plan.params["reference_dict"],
        serde_json::json!("assets/reference/host/references/toy_host_reference.dict")
    );
    assert_eq!(plan.params["sample_id"], serde_json::json!("core-v1-align"));
    assert_eq!(plan.params["tool"], serde_json::json!("bowtie2"));
    assert_eq!(plan.params["threads"], serde_json::json!(4));
    assert_eq!(plan.effective_params["build_indices"], serde_json::json!(false));
    assert_eq!(
        plan.effective_params["strategy_id"],
        serde_json::json!("bowtie2_very_sensitive_local")
    );

    let command = plan
        .command
        .template
        .iter()
        .last()
        .unwrap_or_else(|| panic!("bam.align command template must contain a shell body"));
    assert!(
        command.contains("-x assets/reference/host/references/toy_host_reference")
            && command.contains("assets/reference/host/references/toy_host_reference.fasta")
            && command.contains("target/local-ready/bam.align/align.bam"),
        "local-ready plan command must carry the governed Bowtie2 index prefix, FASTA path, and BAM output"
    );

    Ok(())
}

#[test]
fn local_align_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_bam::stage_api::local_align_plan;
}
