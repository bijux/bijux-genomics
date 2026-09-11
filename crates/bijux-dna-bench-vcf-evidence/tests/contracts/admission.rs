use super::observations::evidence;
use anyhow::Result;
use bijux_dna_bench_vcf_evidence::{
    model::parsers::VcfParserFixtureCoverageStatus,
    snapshot::{VcfEvidenceRows, VcfEvidenceSnapshot},
};

#[test]
fn repeated_selection_reuses_admitted_observations() -> Result<()> {
    let snapshot = VcfEvidenceSnapshot::admit(evidence())?;
    let rows = snapshot.stage_bindings("vcf.call")?;
    assert!(rows[0].active_row.is_some());
    assert!(rows[0].expected_row.is_some());
    assert!(std::ptr::eq(rows, snapshot.stage_bindings("vcf.call")?));
    assert!(snapshot.stage_bindings("vcf.impute").is_err());
    Ok(())
}

#[test]
fn missing_surfaces_remain_explicit() -> Result<()> {
    let snapshot = VcfEvidenceSnapshot::admit(VcfEvidenceRows {
        retained: evidence().retained,
        ..VcfEvidenceRows::default()
    })?;
    let row = &snapshot.stage_bindings("vcf.call")?[0];
    assert!(row.active_row.is_none());
    assert!(row.command_row.is_none());
    assert!(row.output_row.is_none());
    assert!(row.parser_row.is_none());
    assert!(row.expected_row.is_none());
    assert!(row.report_row.is_none());
    Ok(())
}

#[test]
fn every_surface_rejects_duplicate_identity() {
    for surface in
        ["retained", "active", "command", "output", "parser", "expected result", "report"]
    {
        let mut rows = evidence();
        match surface {
            "retained" => rows.retained.push(rows.retained[0].clone()),
            "active" => rows.active.push(rows.active[0].clone()),
            "command" => rows.commands.push(rows.commands[0].clone()),
            "output" => rows.outputs.push(rows.outputs[0].clone()),
            "parser" => rows.parsers.push(rows.parsers[0].clone()),
            "expected result" => rows.expected.push(rows.expected[0].clone()),
            _ => rows.reports.push(rows.reports[0].clone()),
        }
        let error = VcfEvidenceSnapshot::admit(rows).err().map(|e| e.to_string());
        assert!(error.is_some_and(|e| e.contains(&format!("duplicate {surface}"))), "{surface}");
    }
}

#[test]
fn corpus_specific_parsers_are_never_joined_by_tool_alone() -> Result<()> {
    let mut rows = evidence();
    let mut retained = rows.retained[0].clone();
    retained.corpus_id = "second_corpus".into();
    rows.retained.push(retained);
    let mut parser = rows.parsers[0].clone();
    parser.corpus_id = "second_corpus".into();
    parser.raw_fixture_paths = vec!["second/raw.vcf".into()];
    rows.parsers.push(parser);
    let snapshot = VcfEvidenceSnapshot::admit(rows)?;
    let joined = snapshot.stage_bindings("vcf.call")?;
    assert_eq!(joined.len(), 2);
    assert_eq!(joined[0].retained_row.corpus_id, "bam_mini");
    assert_eq!(joined[1].retained_row.corpus_id, "second_corpus");
    assert_eq!(
        joined[1].parser_row.as_ref().map(|r| r.raw_fixture_paths.as_slice()),
        Some(["second/raw.vcf".to_owned()].as_slice())
    );
    assert!(joined[1].active_row.is_none());
    assert!(joined[1].expected_row.is_none());
    Ok(())
}

#[test]
fn unretained_and_foreign_evidence_is_rejected() {
    let mut rows = evidence();
    rows.active[0].key.corpus_id = "foreign".into();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
    let mut rows = evidence();
    rows.expected[0].domain = "bam".into();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
    let mut rows = evidence();
    rows.outputs[0].tool_id = "another_tool".into();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
    let mut rows = evidence();
    rows.retained[0].stage_id = "bam.call".into();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
}

#[test]
fn active_and_parser_contract_mismatches_are_rejected() {
    let mut rows = evidence();
    rows.active[0].adapter_id = "different".into();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
    let mut rows = evidence();
    rows.parsers[0].schema_id = "different".into();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
    let mut rows = evidence();
    rows.parsers[0].parser_fixture_schema_id = String::new();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
}

#[test]
fn incomplete_fixture_is_an_observation_not_an_admission_failure() -> Result<()> {
    let mut rows = evidence();
    rows.parsers[0].coverage_status = VcfParserFixtureCoverageStatus::MissingFixtureInventory;
    rows.parsers[0].parser_fixture_schema_id = "-".into();
    rows.parsers[0].parser_fixture_parser_id = "-".into();
    assert!(VcfEvidenceSnapshot::admit(rows)?.stage_bindings("vcf.call")?[0].parser_row.is_some());
    Ok(())
}

#[test]
fn report_and_fixture_count_contradictions_are_rejected() {
    let mut rows = evidence();
    rows.reports[0].section_id = "unrelated".into();
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
    let mut rows = evidence();
    rows.parsers[0].raw_fixture_count = 2;
    assert!(VcfEvidenceSnapshot::admit(rows).is_err());
}

#[test]
fn blank_or_padded_binding_identity_is_rejected() {
    for value in ["", " value", "value ", "line\nbreak"] {
        let mut rows = evidence();
        rows.retained[0].corpus_id = value.into();
        assert!(VcfEvidenceSnapshot::admit(rows).is_err(), "{value:?}");
    }
}

#[test]
fn native_fixture_contracts_keep_their_own_schema_namespace() -> Result<()> {
    let mut rows = evidence();
    rows.parsers[0].parser_fixture_parser_id = "parse_bcftools_call_metrics".into();
    rows.parsers[0].parser_fixture_schema_id = "bijux.vcf.call.v1".into();
    assert!(VcfEvidenceSnapshot::admit(rows)?.stage_bindings("vcf.call")?[0].parser_row.is_some());
    Ok(())
}
