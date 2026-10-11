use super::{build_real_output_parser_smoke_report, DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH};
use anyhow::Result;

#[test]
fn build_real_output_parser_smoke_report_covers_active_families() -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let output_path = repo_root.join(DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH);
    let probe_root = repo_root.join(
        "artifacts/benchmark-smoke/simulated-runs/readiness-probes/tests/real-output-parser-smoke",
    );
    if probe_root.exists() {
        std::fs::remove_dir_all(&probe_root)?;
    }
    std::fs::create_dir_all(&probe_root)?;
    let report = build_real_output_parser_smoke_report(&repo_root, &output_path, &probe_root)?;
    assert!(!report.rows.is_empty());
    assert_eq!(report.family_count, report.rows.len());
    assert_eq!(report.failed_family_count, 0);
    assert!(report.rows.iter().all(|row| row.passed));
    Ok(())
}
#[test]
fn alignment_probe_resolves_inputs_without_changing_working_directory() -> Result<()> {
    let repo_root = crate::commands::support::workspace_root::resolve_repo_root()?;
    let working_directory = std::env::current_dir()?;
    let directory = tempfile::tempdir()?;
    let plan = bijux_dna_planner_bam::stage_api::local_align_plan(&repo_root)?;
    let declared_reference = super::inputs::required_input_path(&plan, "reference_fasta")
        .or_else(|_| super::inputs::required_input_path(&plan, "reference"))?;
    let proof = super::bam::generate_bam_align_probe(&repo_root, directory.path())?;
    let provenance: bijux_dna_domain_bam::BamAlignmentProvenanceV1 =
        serde_json::from_slice(&std::fs::read(proof.proof_path)?)?;
    assert_eq!(provenance.reference_fasta, declared_reference);
    assert_eq!(provenance.backend_tool_id, plan.tool_id.as_str());
    assert_eq!(std::env::current_dir()?, working_directory);
    assert!(directory.path().join("bam_align/alignment.provenance.json").is_file());
    Ok(())
}
#[test]
fn kinship_parser_requires_only_its_declared_fixture() -> Result<()> {
    let source = crate::commands::support::workspace_root::resolve_repo_root()?;
    let directory = tempfile::tempdir()?;
    let root = directory.path();
    let config: super::bam::LocalBamKinshipConfig =
        super::inputs::read_toml_document(&source.join(super::bam::LOCAL_BAM_KINSHIP_CONFIG_PATH))?;
    let case = config
        .cases
        .iter()
        .find(|case| case.expected_status == "ok")
        .ok_or_else(|| anyhow::anyhow!("missing governed successful kinship case"))?;
    let config_path = root.join(super::bam::LOCAL_BAM_KINSHIP_CONFIG_PATH);
    let fixture = root.join(&case.bam);
    std::fs::create_dir_all(config_path.parent().ok_or_else(|| anyhow::anyhow!("config parent"))?)?;
    std::fs::create_dir_all(fixture.parent().ok_or_else(|| anyhow::anyhow!("fixture parent"))?)?;
    std::fs::copy(source.join(super::bam::LOCAL_BAM_KINSHIP_CONFIG_PATH), config_path)?;
    std::fs::copy(source.join(&case.bam), &fixture)?;
    let working_directory = std::env::current_dir()?;
    let row = super::render_bam_kinship_parser_smoke_row(root)?;
    assert_eq!(row.representative_stage_id, "bam.kinship");
    assert_eq!(row.representative_tool_id, "king");
    assert!(row.passed);
    assert_eq!(
        row.normalized_snapshot.get("status").and_then(serde_json::Value::as_str),
        Some("ok")
    );
    assert!(root.join(row.proof_path).is_file());
    assert!(!root.join("assets").exists());
    assert!(!root.join(super::DEFAULT_REAL_OUTPUT_PARSER_SMOKE_PATH).exists());
    assert_eq!(std::env::current_dir()?, working_directory);
    std::fs::remove_file(fixture)?;
    assert!(super::render_bam_kinship_parser_smoke_row(root).is_err());
    Ok(())
}
