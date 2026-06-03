use anyhow::Result;
use bijux_dna_domain_fastq::params::remove_duplicates::DedupMode;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_remove_duplicates_smoke_plans_use_governed_duplicate_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_remove_duplicates_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed remove-duplicates smoke should keep one fixture");

    let [case] = plans.as_slice() else {
        panic!("expected exactly one remove-duplicates smoke case");
    };
    assert_eq!(case.sample_id, "duplicate-hit-se");
    assert_eq!(case.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_with_duplicates.fastq"));
    assert_eq!(case.dedup_mode, DedupMode::Exact);
    assert!(case.keep_order);

    assert_eq!(case.plan.stage_id.as_str(), "fastq.remove_duplicates");
    assert_eq!(case.plan.tool_id.as_str(), "clumpify");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.remove_duplicates/duplicate-hit-se/clumpify")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("single_end"));
    assert_eq!(case.plan.effective_params["dedup_mode"], serde_json::json!("exact"));
    assert_eq!(case.plan.effective_params["keep_order"], serde_json::json!(true));
    assert_eq!(
        case.plan.params["input_r1"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_with_duplicates.fastq")
    );
    assert_eq!(
        case.plan.params["output_r1"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/duplicate-hit-se/clumpify/clumpify.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["duplicate_classes_tsv"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/duplicate-hit-se/clumpify/duplicate_classes.tsv"
        )
    );
    assert_eq!(
        case.plan.params["duplicate_provenance_json"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/duplicate-hit-se/clumpify/duplicate_provenance.json"
        )
    );
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/duplicate-hit-se/clumpify/deduplicate_report.json"
        )
    );

    Ok(())
}

#[test]
fn local_remove_duplicates_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalRemoveDuplicatesSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_remove_duplicates_smoke_plans;
}
