use anyhow::Result;
use bijux_dna_domain_fastq::params::correct::QualityEncoding;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_correct_errors_smoke_plans_use_governed_paired_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_fastq::stage_api::local_correct_errors_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed correct-errors smoke should keep one fixture");

    let [case] = plans.as_slice() else {
        panic!("expected exactly one correct-errors smoke case");
    };
    assert_eq!(case.sample_id, "paired-dry-run");
    assert_eq!(case.r1, PathBuf::from("assets/toy/core-v1/fastq/reads_1.fastq"));
    assert_eq!(case.r2, Some(PathBuf::from("assets/toy/core-v1/fastq/reads_2.fastq")));
    assert_eq!(case.quality_encoding, QualityEncoding::Phred33);
    assert!(!case.conservative_mode);

    assert_eq!(case.plan.stage_id.as_str(), "fastq.correct_errors");
    assert_eq!(case.plan.tool_id.as_str(), "rcorrector");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/fastq.correct_errors/paired-dry-run/rcorrector")
    );
    assert_eq!(case.plan.resources.threads, 1);
    assert_eq!(case.plan.effective_params["paired_mode"], serde_json::json!("paired_end"));
    assert_eq!(case.plan.effective_params["correction_engine"], serde_json::json!("rcorrector"));
    assert_eq!(case.plan.effective_params["quality_encoding"], serde_json::json!("phred33"));
    assert_eq!(case.plan.effective_params["conservative_mode"], serde_json::json!(false));
    assert_eq!(
        case.plan.params["input_r1"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_1.fastq")
    );
    assert_eq!(
        case.plan.params["input_r2"],
        serde_json::json!("assets/toy/core-v1/fastq/reads_2.fastq")
    );
    assert_eq!(
        case.plan.params["output_r1"],
        serde_json::json!(
            "target/local-smoke/fastq.correct_errors/paired-dry-run/rcorrector/reads_r1.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["output_r2"],
        serde_json::json!(
            "target/local-smoke/fastq.correct_errors/paired-dry-run/rcorrector/reads_r2.fastq.gz"
        )
    );
    assert_eq!(
        case.plan.params["report_json"],
        serde_json::json!(
            "target/local-smoke/fastq.correct_errors/paired-dry-run/rcorrector/correct_report.json"
        )
    );
    assert!(
        case.plan.command.template.iter().any(|part| part.contains("run_rcorrector.pl"))
            && case.plan.command.template.iter().any(|part| {
                part.contains("assets/toy/core-v1/fastq/reads_1.fastq")
                    && part.contains("assets/toy/core-v1/fastq/reads_2.fastq")
            }),
        "local correct-errors dry-run plan must carry the governed rcorrector command"
    );

    Ok(())
}

#[test]
fn local_correct_errors_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_fastq::LocalCorrectErrorsSmokeCasePlan>> =
        bijux_dna_planner_fastq::stage_api::local_correct_errors_smoke_plans;
}
