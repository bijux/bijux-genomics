use crate::support::ServingFixture;
use anyhow::{Context, Result};
use bijux_dna_bench_bindings::{
    observations::{Domain, FastqAssessment},
    serving::{collect, render_bam_tool_serving_map, render_fastq_tool_serving_map},
};
#[test]
fn bam_parser_precedence_and_integrated_selection() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut fixture = ServingFixture::new();
    for (flags, label) in [
        ((true, true, true), "parser_fixture_validated"),
        ((false, true, true), "scientific_report_contract"),
        ((false, false, true), "artifact_contract_only"),
        ((false, false, false), "missing"),
    ] {
        fixture.flags = flags;
        let rows = collect(&fixture.context(root.path()), Domain::Bam)?;
        assert_eq!(rows.stage_count, 1);
        assert_eq!(rows.tool_count, 2);
        assert_eq!(
            rows.rows.iter().map(|row| row.tool_id.as_str()).collect::<Vec<_>>(),
            vec!["alpha", "zeta"]
        );
        assert!(rows.rows.iter().all(|row| row.parser_status == label));
    }
    assert!(!fixture.calls.borrow().iter().any(|name| name == "unintegrated"));
    Ok(())
}
#[test]
fn ambiguous_corpus_or_serving_binding_preserves_report() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut fixture = ServingFixture::new();
    let path = root.path().join("serving.tsv");
    render_bam_tool_serving_map(&fixture.context(root.path()), &path)?;
    let before = std::fs::read(&path)?;
    fixture.declarations.push(fixture.declarations.first().context("fixture declaration")?.clone());
    let error = render_bam_tool_serving_map(&fixture.context(root.path()), &path)
        .err()
        .context("duplicate declaration rejected")?;
    assert!(error.to_string().contains("duplicate corpus compatibility stage"));
    assert_eq!(std::fs::read(&path)?, before);
    fixture.declarations.pop();
    fixture.tools.push("alpha".into());
    let error = render_bam_tool_serving_map(&fixture.context(root.path()), &path)
        .err()
        .context("duplicate binding rejected")?;
    assert!(error.to_string().contains("duplicate BAM serving binding"));
    assert_eq!(std::fs::read(&path)?, before);
    Ok(())
}
#[test]
fn missing_fixture_identity_rejects_before_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut fixture = ServingFixture::new();
    fixture.declarations.first_mut().context("fixture declaration")?.fixture_id = None;
    let path = root.path().join("missing.tsv");
    let error = render_bam_tool_serving_map(&fixture.context(root.path()), &path)
        .err()
        .context("missing fixture rejected")?;
    assert!(error.to_string().contains("without a fixture_id"));
    assert!(!path.exists());
    Ok(())
}
#[test]
fn fastq_capability_precedence_and_active_exclusion() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut fixture = ServingFixture::new();
    let mut extra = FastqAssessment {
        readiness: "planned_contract",
        active: true,
        runnable: true,
        plannable: true,
        declared: true,
        comparable: true,
        benchmark_normalized: true,
        parse_normalized: true,
    };
    for (execution, normalization, adapter, parser) in [
        ((true, true, true), (true, true, true), "runnable", "comparable"),
        ((false, true, true), (false, true, true), "plannable", "benchmark_normalized"),
        ((false, false, true), (false, false, true), "declared_only", "parse_normalized"),
        ((false, false, false), (false, false, false), "missing", "not_normalized"),
    ] {
        (extra.runnable, extra.plannable, extra.declared) = execution;
        (extra.comparable, extra.benchmark_normalized, extra.parse_normalized) = normalization;
        fixture.extra = Some(extra.clone());
        let observed = collect(&fixture.context(root.path()), Domain::Fastq)?;
        let row =
            observed.rows.iter().find(|row| row.tool_id == "alpha").context("extra binding")?;
        assert_eq!(row.adapter_status, adapter);
        assert_eq!(row.parser_status, parser);
    }
    extra.active = false;
    fixture.extra = Some(extra);
    let observed = collect(&fixture.context(root.path()), Domain::Fastq)?;
    assert_eq!(observed.rows.len(), 5);
    Ok(())
}
#[test]
fn serving_acquisition_is_once_per_request_and_later_requests_are_fresh() -> Result<()> {
    let root = tempfile::tempdir()?;
    let fixture = ServingFixture::new();
    let context = fixture.context(root.path());
    render_fastq_tool_serving_map(&context, Path::new("nested/map.tsv"))?;
    render_fastq_tool_serving_map(&context, Path::new("nested/map.tsv"))?;
    for name in ["corpus", "integrated", "inventory"] {
        assert_eq!(fixture.calls.borrow().iter().filter(|value| value.as_str() == name).count(), 2);
    }
    let payload = std::fs::read_to_string(root.path().join("nested/map.tsv"))?;
    assert_eq!(payload.lines().count(), 6);
    assert!(payload.starts_with(
        "tool_id\tstage_id\tsupport_status\tadapter_status\tparser_status\tcorpus_status\n"
    ));
    Ok(())
}
use std::path::Path;
