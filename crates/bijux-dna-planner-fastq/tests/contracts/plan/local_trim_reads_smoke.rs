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
fn local_trim_reads_smoke_plans_use_governed_toy_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_trim_reads_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed trim smoke should keep curated SE and PE cases");

    let se_case = plans
        .iter()
        .find(|case| case.sample_id == "adapter-quality-se")
        .unwrap_or_else(|| panic!("single-end trim smoke case missing"));
    assert_eq!(se_case.plan.stage_id.as_str(), "fastq.trim_reads");
    assert_eq!(se_case.plan.tool_id.as_str(), "fastp");
    assert_eq!(se_case.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_with_trim_signals.fastq"));
    assert_eq!(se_case.r2, None);
    assert_eq!(se_case.min_length, 4);
    assert_eq!(se_case.quality_cutoff, Some(20));
    assert_eq!(
        se_case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.trim_reads/adapter-quality-se/fastp")
    );
    assert_eq!(se_case.plan.effective_params["adapter_policy"], serde_json::json!("bank"));
    assert_eq!(se_case.plan.effective_params["min_len"], serde_json::json!(4));
    assert_eq!(se_case.plan.effective_params["q_cutoff"], serde_json::json!(20));

    let pe_case = plans
        .iter()
        .find(|case| case.sample_id == "adapter-quality-pe")
        .unwrap_or_else(|| panic!("paired-end trim smoke case missing"));
    assert_eq!(
        pe_case.r1,
        PathBuf::from("assets/toy/core-v1/fastq/reads_with_trim_signals_R1.fastq")
    );
    assert_eq!(
        pe_case.r2,
        Some(PathBuf::from("assets/toy/core-v1/fastq/reads_with_trim_signals_R2.fastq"))
    );
    assert_eq!(
        pe_case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.trim_reads/adapter-quality-pe/fastp")
    );
    assert_eq!(pe_case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));

    Ok(())
}

#[test]
fn local_trim_reads_smoke_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalTrimReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_trim_reads_smoke_plans;
}
