use anyhow::Result;
use bijux_dna_bench_vcf::{
    evidence::parsers::{
        collect_vcf_parser_fixture_coverage_rows_from_retained, render_vcf_parser_fixture_coverage,
    },
    inputs::VcfAdapterInputs,
};
use bijux_dna_bench_vcf_evidence::model::{
    parsers::VcfParserFixtureCoverageStatus, scope::VcfActiveStageToolMatrixRow,
};
use bijux_dna_domain_vcf::{find_vcf_parser_fixture_inventory_row, VcfDomainStage};

fn single_binding() -> (VcfAdapterInputs, VcfActiveStageToolMatrixRow) {
    let inputs = crate::support::inputs(&crate::support::repo_root());
    let binding = inputs
        .bindings()
        .iter()
        .find(|row| row.stage_id == "vcf.call" && row.tool_id == "bcftools")
        .expect("call binding");
    let descriptor=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors().expect("schema descriptors").into_iter().find(|row|row.stage_id==binding.stage_id).expect("call schema");
    let row = VcfActiveStageToolMatrixRow {
        stage_id: binding.stage_id.clone(),
        tool_id: binding.tool_id.clone(),
        corpus_id: binding.corpus_id.clone(),
        asset_profile_id: binding.asset_profile_id.clone(),
        adapter_id: binding.adapter_id.clone(),
        parser_id: binding.parser_id.clone(),
        schema_id: descriptor.schema_id,
        tool_status: "production".to_owned(),
        stage_support_status: "supported".to_owned(),
        scope_state: "active".to_owned(),
        scope_detail: "governed binding".to_owned(),
        scope_proof_path: String::new(),
        reason: String::new(),
    };
    (inputs, row)
}
#[test]
fn malformed_retained_identity_or_metadata_preserves_previous_coverage() -> Result<()> {
    let (inputs, row) = single_binding();
    let root = tempfile::tempdir()?;
    let output = root.path().join("coverage.tsv");
    std::fs::write(&output, b"previous coverage")?;
    let mut foreign = row.clone();
    foreign.corpus_id = "foreign".to_owned();
    let mut wrong_schema = row.clone();
    wrong_schema.schema_id = "bijux.vcf.call.v1".to_owned();
    let mut wrong_parser = row.clone();
    wrong_parser.parser_id = "foreign.parser".to_owned();
    for rows in [vec![foreign], vec![wrong_schema], vec![wrong_parser], vec![row.clone(), row]] {
        let error = render_vcf_parser_fixture_coverage(&inputs, root.path(), &rows, &output)
            .err()
            .map(|error| error.to_string());
        assert!(error.is_some_and(|message| message.contains("parser fixture coverage")));
        assert_eq!(std::fs::read(&output)?, b"previous coverage");
    }
    Ok(())
}
#[test]
fn missing_fixture_publishes_incomplete_coverage_diagnostic() -> Result<()> {
    let (inputs, row) = single_binding();
    let root = tempfile::tempdir()?;
    let output = root.path().join("coverage.tsv");
    let error = render_vcf_parser_fixture_coverage(&inputs, root.path(), &[row], &output)
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("coverage must be complete")));
    let report = std::fs::read_to_string(output)?;
    assert!(report.contains("missing_fixture_directory"));
    assert!(report.contains("vcf.call\tbcftools"));
    Ok(())
}
#[test]
fn non_object_normalized_payload_cannot_be_covered() -> Result<()> {
    let (inputs, row) = single_binding();
    let root = tempfile::tempdir()?;
    let fixture =
        find_vcf_parser_fixture_inventory_row("bcftools", VcfDomainStage::try_from("vcf.call")?)
            .expect("call fixture");
    let directory = root.path().join(fixture.fixture_path);
    std::fs::create_dir_all(&directory)?;
    std::fs::write(directory.join("raw.call.vcf"), b"raw fixture")?;
    for normalized in [serde_json::Value::Null, serde_json::json!([]), serde_json::json!("scalar")]
    {
        let envelope = serde_json::json!({"schema_version":"bijux.fixture.vcf_raw_parser.v1","stage_id":"vcf.call","tool_id":"bcftools","parser_id":fixture.parser_id,"normalized":normalized});
        std::fs::write(directory.join("expected.normalized.json"), serde_json::to_vec(&envelope)?)?;
        let (_, _, rows) = collect_vcf_parser_fixture_coverage_rows_from_retained(
            &inputs,
            root.path(),
            std::slice::from_ref(&row),
        )?;
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].coverage_status,
            VcfParserFixtureCoverageStatus::InvalidExpectedNormalizedJson
        );
    }
    Ok(())
}
#[cfg(unix)]
#[test]
fn fixture_symlink_loop_refuses_without_replacing_coverage() -> Result<()> {
    let (inputs, row) = single_binding();
    let root = tempfile::tempdir()?;
    let fixture =
        find_vcf_parser_fixture_inventory_row("bcftools", VcfDomainStage::try_from("vcf.call")?)
            .expect("call fixture");
    let directory = root.path().join(fixture.fixture_path);
    std::fs::create_dir_all(&directory)?;
    std::os::unix::fs::symlink(&directory, directory.join("loop"))?;
    let output = root.path().join("coverage.tsv");
    std::fs::write(&output, b"previous coverage")?;
    let error = render_vcf_parser_fixture_coverage(&inputs, root.path(), &[row], &output)
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("refuses symbolic links")));
    assert_eq!(std::fs::read(output)?, b"previous coverage");
    Ok(())
}
