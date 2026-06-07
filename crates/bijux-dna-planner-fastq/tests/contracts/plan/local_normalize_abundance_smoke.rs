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
fn local_normalize_abundance_smoke_plans_use_governed_abundance_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_normalize_abundance_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed normalize-abundance smoke should keep one focused case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "corpus-03-otu-abundance-table");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.normalize_abundance");
    assert_eq!(case.plan.tool_id.as_str(), "seqkit");
    assert_eq!(
        case.abundance_table,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/tables/corpus-03-otu-abundance.tsv"
        )
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.normalize_abundance/corpus-03-otu-abundance-table/seqkit"
        )
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.method, "relative_abundance");
    assert_eq!(case.plan.effective_params["method"], serde_json::json!("relative_abundance"));
    assert_eq!(
        case.plan.effective_params["normalized_value_column"],
        serde_json::json!("normalized_abundance")
    );

    Ok(())
}

#[test]
fn local_normalize_abundance_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_fastq::LocalNormalizeAbundanceSmokeCasePlan>,
    > = bijux_dna_planner_fastq::stage_api::local_normalize_abundance_smoke_plans;
}
