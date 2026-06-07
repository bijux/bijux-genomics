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
fn local_trim_polyg_tails_smoke_plans_use_governed_corpus_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_trim_polyg_tails_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed polyG smoke should keep one curated case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "polyg-hit-se");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.trim_polyg_tails");
    assert_eq!(case.plan.tool_id.as_str(), "fastp");
    assert_eq!(
        case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_polyg_trim_signals_R1.fastq.gz"
        )
    );
    assert_eq!(case.r2, None);
    assert_eq!(case.min_polyg_run, 6);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.trim_polyg_tails/polyg-hit-se/fastp")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.trim_polyg_tails/polyg-hit-se/fastp/trim_polyg_tails_report.json"
        )
    );
    assert_eq!(case.plan.effective_params["trim_polyg"], serde_json::json!(true));
    assert_eq!(case.plan.effective_params["min_polyg_run"], serde_json::json!(6));

    Ok(())
}

#[test]
fn local_trim_polyg_tails_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalTrimPolygTailsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_trim_polyg_tails_smoke_plans;
}
