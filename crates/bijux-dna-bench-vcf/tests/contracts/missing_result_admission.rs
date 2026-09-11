use anyhow::Result;
use bijux_dna_bench_vcf::{
    adapters::observations::VcfAdapterObservations,
    evidence::{
        expectations::collect_vcf_expected_benchmark_result_rows,
        results::missing::render_vcf_missing_result_report,
    },
    inputs::VcfAdapterInputs,
};
use bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow;
use std::fs;
fn governed() -> Result<(VcfAdapterInputs, Vec<VcfExpectedBenchmarkResultRow>)> {
    let repo = crate::support::repo_root();
    let inputs = crate::support::inputs(&repo);
    let rows =
        collect_vcf_expected_benchmark_result_rows(&VcfAdapterObservations::new(&inputs, &repo))?;
    Ok((inputs, rows))
}
#[test]
fn malformed_missing_result_expectations_preserve_existing_report_and_evidence() -> Result<()> {
    let (inputs, rows) = governed()?;
    let root = tempfile::tempdir()?;
    let report = root.path().join("report.json");
    fs::write(&report, b"prior report")?;
    let mut duplicate = rows.clone();
    duplicate.push(rows[0].clone());
    let mut foreign = rows.clone();
    foreign[0].corpus_id = "foreign".to_owned();
    let mut escaped = rows.clone();
    escaped[0].expected_outputs.push("../escaped".to_owned());
    let mut repeated = rows.clone();
    repeated[0].expected_outputs.push(rows[0].expected_outputs[0].clone());
    let mut absent = rows.clone();
    absent.pop();
    for candidate in [duplicate, foreign, escaped, repeated, absent] {
        assert!(
            render_vcf_missing_result_report(&inputs, &candidate, root.path(), &report).is_err()
        );
        assert_eq!(fs::read(&report)?, b"prior report");
        assert!(!root.path().join("artifacts").exists());
    }
    Ok(())
}
#[test]
fn repeated_missing_result_audit_preserves_unrelated_files_and_marks_one_absence() -> Result<()> {
    let (inputs, rows) = governed()?;
    let root = tempfile::tempdir()?;
    let fixtures =
        root.path().join("artifacts/benchmark-smoke/readiness/missing-result/vcf/results");
    fs::create_dir_all(&fixtures)?;
    let evidence = fixtures.join("retained-evidence.txt");
    fs::write(&evidence, b"historical evidence")?;
    let report = root.path().join("report.json");
    for _ in 0..2 {
        let actual = render_vcf_missing_result_report(&inputs, &rows, root.path(), &report)?;
        assert_eq!(actual.missing_result_row_count, 1);
        assert_eq!(actual.present_result_row_count + 1, rows.len());
        assert!(!root.path().join(actual.removed_manifest_path).exists());
        assert_eq!(fs::read(&evidence)?, b"historical evidence");
    }
    Ok(())
}
#[cfg(unix)]
#[test]
fn missing_result_publication_refuses_symlink_ancestry_without_touching_target() -> Result<()> {
    let (inputs, rows) = governed()?;
    let root = tempfile::tempdir()?;
    let external = tempfile::tempdir()?;
    let fixtures =
        root.path().join("artifacts/benchmark-smoke/readiness/missing-result/vcf/results");
    fs::create_dir_all(&fixtures)?;
    fs::write(external.path().join("retained.txt"), b"external")?;
    std::os::unix::fs::symlink(external.path(), fixtures.join(&rows[0].corpus_id))?;
    let report = root.path().join("report.json");
    fs::write(&report, b"prior report")?;
    assert!(render_vcf_missing_result_report(&inputs, &rows, root.path(), &report).is_err());
    assert_eq!(fs::read(&report)?, b"prior report");
    assert_eq!(fs::read(external.path().join("retained.txt"))?, b"external");
    assert_eq!(fs::read_dir(external.path())?.count(), 1);
    Ok(())
}
