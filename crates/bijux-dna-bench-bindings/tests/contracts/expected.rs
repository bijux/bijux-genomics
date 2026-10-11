use crate::support::ExpectedFixture;
use anyhow::{Context, Result};
use bijux_dna_bench_bindings::expected::{
    collect_expected_benchmark_result_rows, render_expected_benchmark_results,
};
use bijux_dna_bench_bindings::observations::AssignmentKind;
use bijux_dna_core::id_catalog::{FASTQ_INDEX_REFERENCE, FASTQ_SCREEN};
use std::path::Path;
#[test]
fn expected_rows_keep_artifact_roles_and_explicit_result_identity() -> Result<()> {
    let root = tempfile::tempdir()?;
    let fixture = ExpectedFixture::new();
    let report = render_expected_benchmark_results(
        &fixture.context(root.path()),
        Path::new("nested/expected.tsv"),
    )?;
    assert_eq!(report.row_count, 5);
    assert_eq!(report.domain_counts.get("bam"), Some(&2));
    assert_eq!(report.domain_counts.get("fastq"), Some(&3));
    assert_eq!(report.output_path, "nested/expected.tsv");
    let row =
        report.rows.iter().find(|row| row.stage_id == FASTQ_SCREEN).context("taxonomy row")?;
    assert_eq!(
        row.result_row_id,
        bijux_dna_core::ids::build_sample_scoped_benchmark_result_id(
            "fastq",
            "corpus-02-edna-mini",
            FASTQ_SCREEN,
            "sample-set",
            "kraken2"
        )
    );
    assert_eq!(row.result_root,format!("chosen/results/runs/declared-run/corpus-02-edna-mini/{FASTQ_SCREEN}/sample-set/kraken2"));
    assert_eq!(row.stage_result_manifest_path, format!("{}/stage-result.json", row.result_root));
    assert_eq!(row.raw_output_artifact_ids, vec!["native_output"]);
    assert_eq!(
        row.expected_output_artifact_ids,
        vec!["native_output", "classification_report_json"]
    );
    let asset = report
        .rows
        .iter()
        .find(|row| row.stage_id == FASTQ_INDEX_REFERENCE)
        .context("asset row")?;
    assert_eq!(asset.fixture_id, "reference-index-assets");
    assert_eq!(asset.corpus_family_id, "asset_backed");
    assert_eq!(asset.sample_scope, "asset-set");
    let text = std::fs::read_to_string(root.path().join("nested/expected.tsv"))?;
    assert_eq!(text.lines().count(), 6);
    assert!(text.lines().all(|line| line.split('\t').count() == 16));
    Ok(())
}
#[test]
fn expected_sources_are_observed_once_in_order_and_reacquired() -> Result<()> {
    let root = tempfile::tempdir()?;
    let fixture = ExpectedFixture::new();
    collect_expected_benchmark_result_rows(&fixture.context(root.path()))?;
    collect_expected_benchmark_result_rows(&fixture.context(root.path()))?;
    let expected = [
        "scopes",
        "fastq outputs",
        "bam outputs",
        "fastq corpus",
        "bam corpus",
        "fastq kinds",
        "bam kinds",
    ];
    assert_eq!(
        *fixture.calls.borrow(),
        expected.into_iter().chain(expected).map(str::to_owned).collect::<Vec<_>>()
    );
    Ok(())
}
#[test]
fn duplicate_output_binding_preserves_published_report() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut fixture = ExpectedFixture::new();
    let path = root.path().join("expected.tsv");
    render_expected_benchmark_results(&fixture.context(root.path()), &path)?;
    let before = std::fs::read(&path)?;
    fixture.fastq_outputs.push(fixture.fastq_outputs.first().context("output row")?.clone());
    let error = render_expected_benchmark_results(&fixture.context(root.path()), &path)
        .err()
        .context("duplicate rejected")?;
    assert!(error.to_string().contains("duplicate FASTQ adapter output contract row"));
    assert_eq!(std::fs::read(&path)?, before);
    Ok(())
}
#[test]
fn incomplete_or_missing_output_contract_rejects() -> Result<()> {
    let root = tempfile::tempdir()?;
    for missing in [false, true] {
        let mut fixture = ExpectedFixture::new();
        if missing {
            fixture.bam_outputs.remove(0);
        } else {
            fixture.bam_outputs.first_mut().context("BAM output")?.complete = false;
        }
        let error = collect_expected_benchmark_result_rows(&fixture.context(root.path()))
            .err()
            .context("invalid output rejected")?;
        assert!(error.to_string().contains(if missing {
            "missing BAM adapter output contract row"
        } else {
            "missing a complete adapter output contract"
        }));
    }
    Ok(())
}
#[test]
fn missing_assignment_identity_or_sample_scope_rejects() -> Result<()> {
    let root = tempfile::tempdir()?;
    for field in ["fixture", "family", "scope", "sample", "kind"] {
        let mut fixture = ExpectedFixture::new();
        match field {
            "fixture" => fixture.fastq_corpus.first_mut().context("corpus row")?.fixture_id = None,
            "family" => {
                fixture.fastq_corpus.first_mut().context("corpus row")?.corpus_family_id = None;
            }
            "scope" => {
                fixture
                    .fastq_corpus
                    .iter_mut()
                    .find(|row| row.assignment_status == AssignmentKind::AssetBacked)
                    .context("asset row")?
                    .benchmark_scope_id = None;
            }
            "sample" => fixture.scopes.clear(),
            _ => fixture.kinds.clear(),
        }
        assert!(collect_expected_benchmark_result_rows(&fixture.context(root.path())).is_err());
    }
    Ok(())
}
#[test]
fn excluded_assignment_does_not_require_output_contract() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut fixture = ExpectedFixture::new();
    let mut excluded = fixture.fastq_corpus.first().context("corpus row")?.clone();
    excluded.tool_id = "excluded-tool".into();
    excluded.assignment_status = AssignmentKind::Excluded;
    fixture.fastq_corpus.push(excluded);
    let rows = collect_expected_benchmark_result_rows(&fixture.context(root.path()))?;
    assert_eq!(rows.len(), 5);
    Ok(())
}
#[test]
fn duplicate_expected_identity_rejects_before_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut fixture = ExpectedFixture::new();
    fixture.bam_corpus.push(fixture.bam_corpus.first().context("BAM corpus")?.clone());
    let path = root.path().join("expected.tsv");
    let error = render_expected_benchmark_results(&fixture.context(root.path()), &path)
        .err()
        .context("duplicate expected ID rejected")?;
    assert!(error.to_string().contains("unique result_row_id"));
    assert!(!path.exists());
    Ok(())
}
