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
fn local_profile_reads_smoke_plans_use_governed_toy_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_profile_reads_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed profile smoke should keep SE and PE coverage");

    let single_end = plans
        .iter()
        .find(|case| case.sample_id == "toy-se")
        .unwrap_or_else(|| panic!("toy-se profile smoke case missing"));
    assert_eq!(single_end.plan.stage_id.as_str(), "fastq.profile_reads");
    assert_eq!(single_end.plan.tool_id.as_str(), "seqkit_stats");
    assert_eq!(single_end.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));
    assert_eq!(single_end.r2, None);
    assert_eq!(
        single_end.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.profile_reads/toy-se/seqkit_stats")
    );
    assert_eq!(single_end.plan.resources.threads, 4);

    let paired_end = plans
        .iter()
        .find(|case| case.sample_id == "toy-pe")
        .unwrap_or_else(|| panic!("toy-pe profile smoke case missing"));
    assert_eq!(paired_end.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));
    assert_eq!(paired_end.r2, Some(PathBuf::from("assets/toy/core-v1/fastq/reads_2.fastq")));
    assert_eq!(
        paired_end.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.profile_reads/toy-pe/seqkit_stats")
    );
    assert_eq!(paired_end.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(
        paired_end.plan.params["input_r2"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_2.fastq")
    );

    Ok(())
}

#[test]
fn local_profile_reads_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalProfileReadsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_profile_reads_smoke_plans;
}
