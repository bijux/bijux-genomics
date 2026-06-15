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
fn local_validate_reads_smoke_plans_use_governed_toy_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_validate_reads_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed local-smoke config must keep SE and PE coverage");

    let single_end = plans
        .iter()
        .find(|case| case.sample_id == "toy-se")
        .unwrap_or_else(|| panic!("toy-se case missing from local validate smoke plans"));
    assert_eq!(single_end.plan.stage_id.as_str(), "fastq.validate_reads");
    assert_eq!(single_end.plan.tool_id.as_str(), "fastqvalidator");
    assert_eq!(single_end.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));
    assert_eq!(single_end.r2, None);
    assert_eq!(
        single_end.validation_mode,
        bijux_dna_domain_fastq::params::validate::ValidationMode::Strict
    );
    assert_eq!(
        single_end.pair_sync_policy,
        bijux_dna_domain_fastq::params::validate::PairSyncPolicy::NotApplicable
    );
    assert_eq!(
        single_end.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.validate_reads/toy-se/fastqvalidator")
    );
    assert_eq!(single_end.plan.resources.threads, 4);

    let paired_end = plans
        .iter()
        .find(|case| case.sample_id == "toy-pe")
        .unwrap_or_else(|| panic!("toy-pe case missing from local validate smoke plans"));
    assert_eq!(paired_end.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));
    assert_eq!(paired_end.r2, Some(PathBuf::from("assets/toy/core-v1/fastq/reads_2.fastq")));
    assert_eq!(
        paired_end.pair_sync_policy,
        bijux_dna_domain_fastq::params::validate::PairSyncPolicy::RequireHeaderSync
    );
    assert_eq!(
        paired_end.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.validate_reads/toy-pe/fastqvalidator")
    );
    assert_eq!(paired_end.plan.params["validation_mode"], serde_json::json!("strict"));
    assert_eq!(
        paired_end.plan.params["pair_sync_policy"],
        serde_json::json!("require_header_sync")
    );

    Ok(())
}

#[test]
fn local_validate_reads_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalValidateReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_validate_reads_smoke_plans;
}
