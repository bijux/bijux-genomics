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
fn local_remove_chimeras_smoke_plans_use_governed_corpus_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_remove_chimeras_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed chimera smoke should keep one focused case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "chimera-control-se");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.remove_chimeras");
    assert_eq!(case.plan.tool_id.as_str(), "vsearch");
    assert_eq!(
        case.reads,
        PathBuf::from("assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq")
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.remove_chimeras/chimera-control-se/vsearch")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["input_layout"], serde_json::json!("single_stream"));
    assert_eq!(
        case.plan.params["input_reads"],
        serde_json::json!("assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq")
    );

    Ok(())
}

#[test]
fn local_remove_chimeras_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalRemoveChimerasSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_remove_chimeras_smoke_plans;
}
