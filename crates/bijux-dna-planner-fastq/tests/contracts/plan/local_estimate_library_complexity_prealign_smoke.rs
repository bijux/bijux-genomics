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
fn local_estimate_library_complexity_prealign_smoke_plans_use_governed_toy_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_estimate_library_complexity_prealign_smoke_plans(
            &repo_root,
        )?;
    assert_eq!(
        plans.len(),
        2,
        "governed local-smoke config must keep complexity-hit and clear coverage"
    );

    let complexity_hit = plans
        .iter()
        .find(|case| case.sample_id == "complexity-hit-pe")
        .unwrap_or_else(|| panic!("complexity-hit-pe case missing from local complexity smoke plans"));
    assert_eq!(
        complexity_hit.plan.stage_id.as_str(),
        "fastq.estimate_library_complexity_prealign"
    );
    assert_eq!(complexity_hit.plan.tool_id.as_str(), "bijux_dna");
    assert_eq!(
        complexity_hit.r1,
        PathBuf::from("assets/toy/core-v1/fastq/duplicate_pairs_R1.fastq")
    );
    assert_eq!(
        complexity_hit.r2,
        Some(PathBuf::from("assets/toy/core-v1/fastq/duplicate_pairs_R2.fastq"))
    );
    assert_eq!(complexity_hit.kmer_size, 4);
    assert_eq!(
        complexity_hit.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.estimate_library_complexity_prealign/complexity-hit-pe/bijux_dna"
        )
    );
    assert_eq!(complexity_hit.plan.resources.threads, 1);
    assert_eq!(
        complexity_hit.plan.params["library_complexity_report"],
        serde_json::json!(
            "target/local-smoke/fastq.estimate_library_complexity_prealign/complexity-hit-pe/bijux_dna/library_complexity_report.json"
        )
    );

    let complexity_clear = plans
        .iter()
        .find(|case| case.sample_id == "complexity-clear-pe")
        .unwrap_or_else(|| panic!("complexity-clear-pe case missing from local complexity smoke plans"));
    assert_eq!(
        complexity_clear.r1,
        PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq")
    );
    assert_eq!(
        complexity_clear.r2,
        Some(PathBuf::from("assets/toy/core-v1/fastq/reads_2.fastq"))
    );
    assert_eq!(
        complexity_clear.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.estimate_library_complexity_prealign/complexity-clear-pe/bijux_dna"
        )
    );
    assert_eq!(
        complexity_clear.plan.effective_params["paired_mode"],
        serde_json::json!("paired_end")
    );
    assert_eq!(
        complexity_clear.plan.effective_params["advisory_only"],
        serde_json::json!(true)
    );
    assert_eq!(
        complexity_clear.plan.effective_params["kmer_size"],
        serde_json::json!(4)
    );

    Ok(())
}

#[test]
fn local_estimate_library_complexity_prealign_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalEstimateLibraryComplexityPrealignSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_estimate_library_complexity_prealign_smoke_plans;
}
