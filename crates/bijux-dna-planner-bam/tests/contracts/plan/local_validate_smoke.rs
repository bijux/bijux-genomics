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
fn local_validate_smoke_plans_use_governed_bam_fixtures() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_validate_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 2, "governed local-smoke config must keep pass and refusal coverage");

    let passing = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-coordinate-pass")
        .unwrap_or_else(|| panic!("passing BAM validation case missing"));
    assert_eq!(passing.plan.stage_id.as_str(), "bam.validate");
    assert_eq!(passing.plan.tool_id.as_str(), "samtools");
    assert_eq!(passing.plan.resources.threads, 4);
    assert_eq!(passing.bam, PathBuf::from("assets/toy/core-v1/bam/validation_pass.bam"));
    assert_eq!(
        passing.alignment_fixture_encoding,
        bijux_dna_planner_bam::stage_api::LocalValidateAlignmentFixtureEncoding::BinaryBam
    );
    assert_eq!(
        passing.bam_index,
        Some(PathBuf::from("assets/toy/core-v1/bam/validation_pass.bam.bai"))
    );
    assert_eq!(
        passing.reference_fasta,
        Some(PathBuf::from("assets/toy/core-v1/bam/validation_reference.fasta"))
    );
    assert!(passing.expect_pass);
    assert!(passing.required_refusal_codes.is_empty());
    assert_eq!(
        passing.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.validate/core-v1-coordinate-pass/samtools")
    );

    let refusal = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-malformed-refusal")
        .unwrap_or_else(|| panic!("refusal BAM validation case missing"));
    assert_eq!(refusal.bam, PathBuf::from("assets/toy/core-v1/bam/validation_malformed.bam"));
    assert_eq!(
        refusal.alignment_fixture_encoding,
        bijux_dna_planner_bam::stage_api::LocalValidateAlignmentFixtureEncoding::BinaryBam
    );
    assert_eq!(refusal.bam_index, None);
    assert_eq!(refusal.reference_fasta, None);
    assert!(!refusal.expect_pass);
    assert_eq!(refusal.required_refusal_codes, vec!["malformed_alignment_record".to_string()]);
    assert_eq!(
        refusal.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.validate/core-v1-malformed-refusal/samtools")
    );

    let validation_output = refusal
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "validation_report")
        .unwrap_or_else(|| panic!("validation_report output missing from refusal plan"));
    assert_eq!(
        validation_output.path,
        PathBuf::from(
            "target/local-smoke/bam.validate/core-v1-malformed-refusal/samtools/validation.json"
        )
    );

    Ok(())
}

#[test]
fn local_validate_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalValidateSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_validate_smoke_plans;
}
