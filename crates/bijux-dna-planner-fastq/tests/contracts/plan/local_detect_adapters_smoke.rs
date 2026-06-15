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
fn local_detect_adapters_smoke_plans_use_governed_corpus_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_detect_adapters_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        2,
        "governed local-smoke config must keep corpus-owned hit and clear coverage"
    );

    let adapter_hit = plans
        .iter()
        .find(|case| case.sample_id == "adapter-hit-se")
        .unwrap_or_else(|| panic!("adapter-hit-se case missing from local adapter smoke plans"));
    assert_eq!(adapter_hit.plan.stage_id.as_str(), "fastq.detect_adapters");
    assert_eq!(adapter_hit.plan.tool_id.as_str(), "fastqc");
    assert_eq!(
        adapter_hit.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_adapter_hit_R1.fastq.gz"
        )
    );
    assert_eq!(adapter_hit.r2, None);
    assert_eq!(
        adapter_hit.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.detect_adapters/adapter-hit-se/fastqc")
    );
    assert_eq!(adapter_hit.plan.resources.threads, 4);

    let adapter_clear = plans
        .iter()
        .find(|case| case.sample_id == "adapter-clear-se")
        .unwrap_or_else(|| panic!("adapter-clear-se case missing from local adapter smoke plans"));
    assert_eq!(
        adapter_clear.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz"
        )
    );
    assert_eq!(adapter_clear.r2, None);
    assert_eq!(
        adapter_clear.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.detect_adapters/adapter-clear-se/fastqc")
    );
    assert_eq!(
        adapter_clear.plan.params["adapter_evidence_dir"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.detect_adapters/adapter-clear-se/fastqc/fastqc"
        )
    );

    Ok(())
}

#[test]
fn local_detect_adapters_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalDetectAdaptersSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_detect_adapters_smoke_plans;
}
