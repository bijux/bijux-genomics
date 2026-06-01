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
fn local_infer_asvs_smoke_plans_use_governed_corpus_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_infer_asvs_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed infer-asvs smoke should keep one focused case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "corpus-03-amplicon-se");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.infer_asvs");
    assert_eq!(case.plan.tool_id.as_str(), "dada2");
    assert_eq!(
        case.reads,
        PathBuf::from("assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq")
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.infer_asvs/corpus-03-amplicon-se/dada2")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.denoising_method, "dada2");
    assert_eq!(case.pooling_mode, "independent");
    assert_eq!(case.chimera_policy, "remove_bimera_denovo");
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("single_end"));

    Ok(())
}

#[test]
fn local_infer_asvs_smoke_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalInferAsvsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_infer_asvs_smoke_plans;
}
