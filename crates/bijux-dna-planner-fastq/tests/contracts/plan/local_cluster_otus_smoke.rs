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
fn local_cluster_otus_smoke_plans_use_governed_corpus_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_cluster_otus_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed cluster-otus smoke should keep one focused case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "corpus-03-otu-cluster-se");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.cluster_otus");
    assert_eq!(case.plan.tool_id.as_str(), "vsearch");
    assert_eq!(
        case.reads,
        PathBuf::from("assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq")
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.cluster_otus/corpus-03-otu-cluster-se/vsearch")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert!((case.otu_identity - 0.97).abs() < f64::EPSILON);
    assert_eq!(case.plan.effective_params["identity_threshold"], serde_json::json!(0.97));

    Ok(())
}

#[test]
fn local_cluster_otus_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalClusterOtusSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_cluster_otus_smoke_plans;
}
