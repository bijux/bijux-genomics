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
fn local_deplete_rrna_plan_uses_governed_repo_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "fastq.deplete_rrna");
    assert_eq!(plan.tool_id.as_str(), "sortmerna");
    assert_eq!(plan.resources.threads, 4);
    assert_eq!(plan.resources.mem_gb, 8);
    assert_eq!(plan.out_dir, PathBuf::from("target/local-ready/fastq.deplete_rrna"));

    let input_r1 = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reads_r1")
        .unwrap_or_else(|| panic!("reads_r1 input missing from local-ready plan"));
    assert_eq!(input_r1.path, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));

    let rrna_reference = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "rrna_reference")
        .unwrap_or_else(|| panic!("rrna_reference input missing from local-ready plan"));
    assert_eq!(
        rrna_reference.path,
        PathBuf::from("assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta")
    );

    let retained_reads = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "rrna_filtered_reads_r1")
        .unwrap_or_else(|| panic!("rrna_filtered_reads_r1 output missing from local-ready plan"));
    assert_eq!(
        retained_reads.path,
        PathBuf::from("target/local-ready/fastq.deplete_rrna/rrna_filtered.fastq.gz")
    );

    let removed_reads = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "rrna_removed_reads_r1")
        .unwrap_or_else(|| panic!("rrna_removed_reads_r1 output missing from local-ready plan"));
    assert_eq!(
        removed_reads.path,
        PathBuf::from("target/local-ready/fastq.deplete_rrna/removed_rrna.fastq.gz")
    );

    assert_eq!(
        plan.params["rrna_db"],
        serde_json::json!("assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta")
    );
    assert_eq!(
        plan.params["removed_reads_r1"],
        serde_json::json!("target/local-ready/fastq.deplete_rrna/removed_rrna.fastq.gz")
    );
    assert_eq!(plan.params["tool"], serde_json::json!("sortmerna"));
    assert_eq!(plan.params["threads"], serde_json::json!(4));
    assert_eq!(plan.effective_params["emit_removed_reads"], serde_json::json!(true));
    assert!(
        plan.command.template[2].contains("sortmerna")
            && plan.command.template[2]
                .contains("assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta"),
        "local-ready plan command must materialize the governed SortMeRNA reference path"
    );
    Ok(())
}

#[test]
fn local_deplete_rrna_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_deplete_rrna_plan;
}
