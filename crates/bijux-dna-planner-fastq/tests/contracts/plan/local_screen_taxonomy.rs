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
fn local_screen_taxonomy_plan_uses_governed_corpus02_inputs() -> Result<()> {
    let repo_root = repo_root();
    let plan = bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_plan(&repo_root)?;

    assert_eq!(plan.stage_id.as_str(), "fastq.screen_taxonomy");
    assert_eq!(plan.tool_id.as_str(), "kraken2");
    assert_eq!(plan.resources.threads, 4);
    assert_eq!(plan.resources.mem_gb, 16);
    assert_eq!(
        plan.out_dir,
        PathBuf::from("benchmarks/readiness/local-ready/fastq.screen_taxonomy")
    );

    let input_r1 = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "reads_r1")
        .unwrap_or_else(|| panic!("reads_r1 input missing from local-ready taxonomy plan"));
    assert_eq!(
        input_r1.path,
        PathBuf::from("assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq")
    );

    let taxonomy_root = plan
        .io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "taxonomy_database_root")
        .unwrap_or_else(|| {
            panic!("taxonomy_database_root input missing from local-ready taxonomy plan")
        });
    assert_eq!(
        taxonomy_root.path,
        PathBuf::from("assets/reference/taxonomy/references/mock_community_taxonomy")
    );

    let summary_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "screen_report_tsv")
        .unwrap_or_else(|| {
            panic!("screen_report_tsv output missing from local-ready taxonomy plan")
        });
    assert_eq!(
        summary_output.path,
        PathBuf::from("benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.report.tsv")
    );

    let classification_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "classification_report_json")
        .unwrap_or_else(|| {
            panic!("classification_report_json output missing from local-ready taxonomy plan")
        });
    assert_eq!(
        classification_output.path,
        PathBuf::from(
            "benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.classifications.json"
        )
    );

    let unclassified_r1_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "unclassified_reads_r1")
        .unwrap_or_else(|| {
            panic!("unclassified_reads_r1 output missing from local-ready taxonomy plan")
        });
    assert_eq!(
        unclassified_r1_output.path,
        PathBuf::from("benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads.fastq")
    );

    assert_eq!(
        plan.params["database_root"],
        serde_json::json!("assets/reference/taxonomy/references/mock_community_taxonomy")
    );
    assert_eq!(plan.params["tool"], serde_json::json!("kraken2"));
    assert_eq!(plan.params["threads"], serde_json::json!(4));
    assert_eq!(plan.effective_params["emit_unclassified"], serde_json::json!(true));
    assert_eq!(
        plan.effective_params["database_catalog_id"],
        serde_json::json!("taxonomy_reference")
    );
    assert!(
        plan.command.template[2].contains("--db 'assets/reference/taxonomy/references/mock_community_taxonomy/kraken2'")
            && plan.command.template[2].contains("'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.report.tsv'")
            && plan.command.template[2].contains("'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.classifications.native.tsv'")
            && plan.command.template[2].contains("--unclassified-out 'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads.fastq'"),
        "local-ready taxonomy plan command must carry the governed database root and output paths"
    );
    Ok(())
}

#[test]
fn local_screen_taxonomy_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_plan;
}
