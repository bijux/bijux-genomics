use anyhow::Result;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_filter_reads_smoke_plans_use_governed_corpus_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_filter_reads_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed filter smoke should keep curated SE and PE cases");

    let se_case = plans
        .iter()
        .find(|case| case.sample_id == "n-and-complexity-se")
        .unwrap_or_else(|| panic!("single-end filter smoke case missing"));
    assert_eq!(se_case.plan.stage_id.as_str(), "fastq.filter_reads");
    assert_eq!(se_case.plan.tool_id.as_str(), "fastp");
    assert_eq!(
        se_case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz"
        )
    );
    assert_eq!(se_case.r2, None);
    assert_eq!(se_case.max_n_count, Some(1));
    assert_eq!(se_case.low_complexity_threshold, Some(20.0));
    assert_eq!(
        se_case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.filter_reads/n-and-complexity-se/fastp")
    );
    assert_eq!(se_case.plan.effective_params["max_n_count"], serde_json::json!(1));
    assert_eq!(se_case.plan.effective_params["low_complexity_threshold"], serde_json::json!(20.0));
    assert_eq!(se_case.plan.effective_params["paired_mode"], serde_json::json!("single_end"));

    let pe_case = plans
        .iter()
        .find(|case| case.sample_id == "human-like-pe-layout")
        .unwrap_or_else(|| panic!("paired-end filter smoke case missing"));
    assert_eq!(
        pe_case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_distinct_pairs_R1.fastq.gz"
        )
    );
    assert_eq!(
        pe_case.r2,
        Some(PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_distinct_pairs_R2.fastq.gz"
        ))
    );
    assert_eq!(
        pe_case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.filter_reads/human-like-pe-layout/fastp")
    );
    assert_eq!(pe_case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));

    Ok(())
}

#[test]
fn local_filter_reads_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalFilterReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_filter_reads_smoke_plans;
}
