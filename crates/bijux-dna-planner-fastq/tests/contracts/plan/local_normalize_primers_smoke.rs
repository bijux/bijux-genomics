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
    assert_eq!(plans.len(), 2, "governed normalize-primers smoke should cover curated single-end and paired-end amplicon cases");

    let case = plans
        .iter()
        .find(|case| case.sample_id == "amplicon-16s-se")
        .unwrap_or_else(|| panic!("single-end normalize-primers smoke case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "fastq.normalize_primers");
    assert_eq!(case.plan.tool_id.as_str(), "cutadapt");
    assert_eq!(
        case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/amplicon-16s-se.fastq.gz"
        )
    );
    assert_eq!(case.r2, None);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("runs/bench/local-smoke/fastq.normalize_primers/amplicon-16s-se/cutadapt")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "runs/bench/local-smoke/fastq.normalize_primers/amplicon-16s-se/cutadapt/normalize_primers_report.json"
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

    let paired_case = plans
        .iter()
        .find(|case| case.sample_id == "amplicon-16s-pe")
        .unwrap_or_else(|| panic!("paired-end normalize-primers smoke case missing"));
    assert_eq!(
        paired_case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/amplicon-16s-pe_R1.fastq.gz"
        )
    );
    assert_eq!(
        paired_case.r2,
        Some(PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/normalized/amplicon-16s-pe_R2.fastq.gz"
        ))
    );

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
