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
fn local_normalize_primers_smoke_plans_use_governed_amplicon_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_fastq::stage_api::local_normalize_primers_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed normalize-primers smoke should keep one curated case");

    let case = &plans[0];
    assert_eq!(case.sample_id, "amplicon-16s-se");
    assert_eq!(case.plan.stage_id.as_str(), "fastq.normalize_primers");
    assert_eq!(case.plan.tool_id.as_str(), "cutadapt");
    assert_eq!(case.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_with_primers.fastq"));
    assert_eq!(case.r2, None);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.normalize_primers/amplicon-16s-se/cutadapt")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "target/local-smoke/fastq.normalize_primers/amplicon-16s-se/cutadapt/normalize_primers_report.json"
        )
    );
    assert_eq!(case.plan.effective_params["primer_set_id"], serde_json::json!("16S_universal_v1"));
    assert_eq!(case.plan.effective_params["marker_id"], serde_json::json!("16S"));
    assert_eq!(
        case.plan.effective_params["primer_fasta"],
        serde_json::json!("assets/reference/primers/16S_universal_v1.fasta")
    );
    assert_eq!(
        case.plan.effective_params["orientation_policy"],
        serde_json::json!("normalize_to_forward_primer")
    );
    assert_eq!(case.plan.effective_params["min_overlap_bp"], serde_json::json!(6));

    Ok(())
}

#[test]
fn local_normalize_primers_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalNormalizePrimersSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_normalize_primers_smoke_plans;
}
