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
fn local_filter_low_complexity_smoke_plans_use_governed_low_complexity_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_filter_low_complexity_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed low-complexity smoke should keep SE and PE fixtures");

    let se_case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_se_filter_signals")
        .unwrap_or_else(|| panic!("single-end low-complexity smoke case missing"));
    assert_eq!(
        se_case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz"
        )
    );
    assert_eq!(se_case.r2, None);
    assert!((se_case.entropy_threshold - 0.6).abs() < f64::EPSILON);
    assert_eq!(se_case.polyx_threshold, Some(8));

    assert_eq!(se_case.plan.stage_id.as_str(), "fastq.filter_low_complexity");
    assert_eq!(se_case.plan.tool_id.as_str(), "bbduk");
    assert_eq!(
        se_case.plan.out_dir,
        PathBuf::from(
            "runs/bench/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk"
        )
    );
    assert_eq!(se_case.plan.resources.threads, 1);
    assert_eq!(se_case.plan.effective_params["paired_mode"], serde_json::json!("single_end"));
    assert_eq!(se_case.plan.effective_params["entropy_threshold"], serde_json::json!(0.6));
    assert_eq!(se_case.plan.effective_params["polyx_threshold"], serde_json::json!(8));
    assert_eq!(
        se_case.plan.params["input_r1"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz"
        )
    );
    assert_eq!(
        se_case.plan.params["output_r1"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk/bbduk.fastq.gz"
        )
    );
    assert_eq!(
        se_case.plan.params["report_json"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk/low_complexity_report.json"
        )
    );
    assert_eq!(
        se_case.plan.params["raw_backend_report"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk/bbduk.low_complexity.stats"
        )
    );

    let pe_case = plans
        .iter()
        .find(|case| case.sample_id == "human_like_pe_layout")
        .unwrap_or_else(|| panic!("paired-end low-complexity smoke case missing"));
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
        PathBuf::from(
            "runs/bench/local-smoke/fastq.filter_low_complexity/human_like_pe_layout/bbduk"
        )
    );
    assert_eq!(pe_case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));

    Ok(())
}

#[test]
fn local_filter_low_complexity_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalFilterLowComplexitySmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_filter_low_complexity_smoke_plans;
}
