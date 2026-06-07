use anyhow::Result;
use bijux_dna_domain_fastq::params::umi::{UmiFailedExtractionPolicy, UmiReadNameTransform};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_extract_umis_smoke_plans_use_governed_known_umi_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_extract_umis_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed extract-umis smoke should keep one fixture");

    let [case] = plans.as_slice() else {
        panic!("expected exactly one extract-umis smoke case");
    };
    assert_eq!(case.sample_id, "human_like_pe_umi_prefix_signals");
    assert_eq!(
        case.r1,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_umi_prefix_signals_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.r2,
        PathBuf::from(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_umi_prefix_signals_R2.fastq.gz"
        )
    );
    assert_eq!(case.umi_pattern, "NNNN");
    assert_eq!(case.read_name_transform, UmiReadNameTransform::AppendToHeader);
    assert_eq!(case.failed_extraction_policy, UmiFailedExtractionPolicy::RetainUnmodified);

    assert_eq!(case.plan.stage_id.as_str(), "fastq.extract_umis");
    assert_eq!(case.plan.tool_id.as_str(), "umi_tools");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/fastq.extract_umis/human_like_pe_umi_prefix_signals/umi_tools"
        )
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(case.plan.effective_params["umi_pattern"], serde_json::json!("NNNN"));
    assert_eq!(
        case.plan.effective_params["read_name_transform"],
        serde_json::json!("append_to_header")
    );
    assert_eq!(
        case.plan.effective_params["failed_extraction_policy"],
        serde_json::json!("retain_unmodified")
    );
    assert_eq!(
        case.plan.params["r1"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_umi_prefix_signals_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["r2"],
        serde_json::json!(
            "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_umi_prefix_signals_R2.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["output_r1"],
        serde_json::json!(
            "target/local-smoke/fastq.extract_umis/human_like_pe_umi_prefix_signals/umi_tools/umi_tagged_R1.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["output_r2"],
        serde_json::json!(
            "target/local-smoke/fastq.extract_umis/human_like_pe_umi_prefix_signals/umi_tools/umi_tagged_R2.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "target/local-smoke/fastq.extract_umis/human_like_pe_umi_prefix_signals/umi_tools/umi_report.json"
        )
    );
    assert_eq!(
        case.plan.params["raw_backend_report"],
        serde_json::json!(
            "target/local-smoke/fastq.extract_umis/human_like_pe_umi_prefix_signals/umi_tools/umi_tools.extract.log"
        )
    );

    Ok(())
}

#[test]
fn local_extract_umis_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalExtractUmisSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_extract_umis_smoke_plans;
}
