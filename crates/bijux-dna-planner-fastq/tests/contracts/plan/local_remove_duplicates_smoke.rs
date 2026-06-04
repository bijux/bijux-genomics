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
    assert_eq!(case.sample_id, "human_like_pe_duplicate_signals");
    assert_eq!(
        case.r1,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.r2,
        Some(PathBuf::from(
            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz"
        ))
    );
    assert_eq!(case.dedup_mode, DedupMode::Exact);
    assert!(case.keep_order);

    assert_eq!(case.plan.stage_id.as_str(), "fastq.remove_duplicates");
    assert_eq!(case.plan.tool_id.as_str(), "clumpify");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.remove_duplicates/human_like_pe_duplicate_signals/clumpify"
        )
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(case.plan.effective_params["dedup_mode"], serde_json::json!("exact"));
    assert_eq!(case.plan.effective_params["keep_order"], serde_json::json!(true));
    assert_eq!(
        case.plan.params["input_r1"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["input_r2"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["output_r1"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/human_like_pe_duplicate_signals/clumpify/clumpify.dedup.R1.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["output_r2"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/human_like_pe_duplicate_signals/clumpify/clumpify.dedup.R2.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["duplicate_classes_tsv"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/human_like_pe_duplicate_signals/clumpify/duplicate_classes.tsv"
        )
    );
    assert_eq!(
        case.plan.params["duplicate_provenance_json"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/human_like_pe_duplicate_signals/clumpify/duplicate_provenance.json"
        )
    );
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "target/local-smoke/fastq.remove_duplicates/human_like_pe_duplicate_signals/clumpify/deduplicate_report.json"
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
