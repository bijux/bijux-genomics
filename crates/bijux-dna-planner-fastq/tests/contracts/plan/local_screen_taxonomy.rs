use anyhow::Result;
use std::path::{Path, PathBuf};

use bijux_dna_stage_contract::StagePlanV1;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

fn input_path<'a>(plan: &'a StagePlanV1, name: &str) -> &'a Path {
    plan.io
        .inputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .unwrap_or_else(|| panic!("{name} input missing from local-ready taxonomy plan"))
        .path
        .as_path()
}

fn output_path<'a>(plan: &'a StagePlanV1, name: &str) -> &'a Path {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == name)
        .unwrap_or_else(|| panic!("{name} output missing from local-ready taxonomy plan"))
        .path
        .as_path()
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
    for (name, expected) in [
        ("reads_r1", "assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq"),
        ("reads_r2", "assets/toy/corpus-02-edna-mini/fastq/mock_community_reads_R2.fastq"),
        ("taxonomy_database_root", "assets/reference/taxonomy/references/mock_community_taxonomy"),
    ] {
        assert_eq!(input_path(&plan, name), Path::new(expected));
    }
    for (name, expected) in [
        (
            "screen_report_tsv",
            "benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.report.tsv",
        ),
        (
            "classification_report_json",
            "benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.classifications.json",
        ),
        (
            "unclassified_reads_r1",
            "benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads_1.fastq",
        ),
        (
            "unclassified_reads_r2",
            "benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads_2.fastq",
        ),
    ] {
        assert_eq!(output_path(&plan, name), Path::new(expected));
    }

    for (key, expected) in [
        (
            "database_root",
            serde_json::json!("assets/reference/taxonomy/references/mock_community_taxonomy"),
        ),
        (
            "input_r2",
            serde_json::json!("assets/toy/corpus-02-edna-mini/fastq/mock_community_reads_R2.fastq"),
        ),
        ("tool", serde_json::json!("kraken2")),
        ("threads", serde_json::json!(4)),
    ] {
        assert_eq!(plan.params[key], expected);
    }
    assert_eq!(plan.effective_params["emit_unclassified"], serde_json::json!(true));
    assert_eq!(
        plan.effective_params["database_catalog_id"],
        serde_json::json!("taxonomy_reference")
    );
    assert!(
        plan.command.template[2].contains("--db 'assets/reference/taxonomy/references/mock_community_taxonomy/kraken2'")
            && plan.command.template[2].contains("'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.report.tsv'")
            && plan.command.template[2].contains("'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.classifications.native.tsv'")
            && plan.command.template[2].contains("--paired 'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq' 'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads_R2.fastq'")
            && plan.command.template[2].contains("--unclassified-out 'benchmarks/readiness/local-ready/fastq.screen_taxonomy/kraken2.unclassified_reads_#.fastq'"),
        "local-ready taxonomy plan command must carry the governed database root and output paths"
    );
    Ok(())
}

#[test]
fn local_screen_taxonomy_plan_stage_api_surface_stays_callable() {
    let _: fn(&std::path::Path) -> anyhow::Result<bijux_dna_stage_contract::StagePlanV1> =
        bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_plan;
    let _: fn(&std::path::Path) -> anyhow::Result<Vec<bijux_dna_stage_contract::StagePlanV1>> =
        bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_output_contract_plans;
}

#[test]
fn local_screen_taxonomy_output_contract_plans_cover_all_governed_tools() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_screen_taxonomy_output_contract_plans(
        &repo_root,
    )?;
    let tool_ids = plans.iter().map(|plan| plan.tool_id.as_str()).collect::<Vec<_>>();
    assert_eq!(tool_ids, vec!["centrifuge", "kaiju", "kraken2", "krakenuniq"]);
    assert!(
        plans.iter().any(|plan| {
            plan.tool_id.as_str() == "centrifuge"
                && plan
                    .io
                    .inputs
                    .iter()
                    .any(|artifact| artifact.name.as_str() == "taxonomy_database_root")
        }),
        "Centrifuge proof plans must keep the governed taxonomy database root input"
    );
    Ok(())
}
