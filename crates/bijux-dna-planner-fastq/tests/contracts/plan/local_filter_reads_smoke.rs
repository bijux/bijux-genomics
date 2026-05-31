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
fn local_filter_reads_smoke_plans_use_governed_toy_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_filter_reads_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed filter smoke should keep exactly one curated case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "n-and-complexity-se");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.filter_reads");
    assert_eq!(case.plan.tool_id.as_str(), "fastp");
    assert_eq!(case.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_with_filter_signals.fastq"));
    assert_eq!(case.r2, None);
    assert_eq!(case.max_n_count, Some(1));
    assert_eq!(case.low_complexity_threshold, Some(20.0));
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.filter_reads/n-and-complexity-se/fastp")
    );
    assert_eq!(case.plan.effective_params["max_n_count"], serde_json::json!(1));
    assert_eq!(case.plan.effective_params["low_complexity_threshold"], serde_json::json!(20.0));
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("single_end"));

    Ok(())
}

#[test]
fn local_filter_reads_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalFilterReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_filter_reads_smoke_plans;
}
