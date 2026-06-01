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
fn local_profile_overrepresented_sequences_smoke_plans_use_governed_repeat_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_profile_overrepresented_sequences_smoke_plans(
        &repo_root,
    )?;
    assert_eq!(plans.len(), 1, "governed overrepresented smoke should keep one focused case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "known-repeat-se");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.profile_overrepresented_sequences");
    assert_eq!(case.plan.tool_id.as_str(), "seqkit");
    assert_eq!(
        case.r1,
        PathBuf::from("assets/toy/core-v1/fastq/reads_with_overrepresented_sequences.fastq")
    );
    assert_eq!(case.r2, None);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.profile_overrepresented_sequences/known-repeat-se/seqkit"
        )
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("single_end"));
    assert_eq!(case.plan.effective_params["top_k"], serde_json::json!(5));
    assert_eq!(
        case.plan.params["input_r1"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_with_overrepresented_sequences.fastq")
    );

    Ok(())
}

#[test]
fn local_profile_overrepresented_sequences_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalProfileOverrepresentedSequencesSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_profile_overrepresented_sequences_smoke_plans;
}
