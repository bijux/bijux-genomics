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
    assert_eq!(plans.len(), 1, "governed low-complexity smoke should keep one fixture");

    let [case] = plans.as_slice() else {
        panic!("expected exactly one low-complexity smoke case");
    };
    assert_eq!(case.sample_id, "human_like_se_filter_signals");
    assert_eq!(
        case.r1,
        PathBuf::from(
            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz"
        )
    );
    assert_eq!(case.r2, None);
    assert!((case.entropy_threshold - 0.6).abs() < f64::EPSILON);
    assert_eq!(case.polyx_threshold, Some(8));

    assert_eq!(case.plan.stage_id.as_str(), "fastq.filter_low_complexity");
    assert_eq!(case.plan.tool_id.as_str(), "bbduk");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk"
        )
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("single_end"));
    assert_eq!(case.plan.effective_params["entropy_threshold"], serde_json::json!(0.6));
    assert_eq!(case.plan.effective_params["polyx_threshold"], serde_json::json!(8));
    assert_eq!(
        case.plan.params["input_r1"],
        serde_json::json!(
            "tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["output_r1"],
        serde_json::json!(
            "target/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk/bbduk.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "target/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk/low_complexity_report.json"
        )
    );
    assert_eq!(
        case.plan.params["raw_backend_report"],
        serde_json::json!(
            "target/local-smoke/fastq.filter_low_complexity/human_like_se_filter_signals/bbduk/bbduk.low_complexity.stats"
        )
    );

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
