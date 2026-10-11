use anyhow::Result;
use bijux_dna_bench_vcf::evidence::parsers::failure::render_vcf_parser_failure_audit;
use std::{fs, path::Path};

fn copy_fixtures(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_fixtures(&entry.path(), &to.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.join(entry.file_name()))?;
        }
    }
    Ok(())
}
fn populate(root: &Path) -> Result<()> {
    let relative = "benchmarks/tests/fixtures/bench/parsers/vcf";
    copy_fixtures(&crate::support::repo_root().join(relative), &root.join(relative))
}
#[test]
fn missing_late_fixture_preserves_prior_report_before_any_probe_effects() -> Result<()> {
    let root = tempfile::tempdir()?;
    populate(root.path())?;
    fs::remove_file(root.path().join("benchmarks/tests/fixtures/bench/parsers/vcf/segments/germline/vcf.ibd/complete/raw.ibd_filtered_segments.tsv"))?;
    let report = root.path().join("report.json");
    fs::write(&report, b"prior report")?;
    assert!(render_vcf_parser_failure_audit(root.path(), &report).is_err());
    assert_eq!(fs::read(report)?, b"prior report");
    assert!(!root.path().join("artifacts").exists());
    Ok(())
}
#[cfg(unix)]
#[test]
fn source_and_destination_links_are_rejected_before_probe_effects() -> Result<()> {
    for destination in [false, true] {
        let root = tempfile::tempdir()?;
        populate(root.path())?;
        let external = root.path().join("external.txt");
        fs::write(&external, b"preserved external bytes")?;
        let relative = if destination {
            "artifacts/bench-readiness/vcf-parser-failure-tests/empty-output/fixture/raw.calls.vcf"
        } else {
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call/raw.calls.vcf"
        };
        let link = root.path().join(relative);
        fs::create_dir_all(link.parent().expect("probe parent"))?;
        if link.exists() {
            fs::remove_file(&link)?;
        }
        std::os::unix::fs::symlink(&external, &link)?;
        let report = root.path().join("report.json");
        fs::write(&report, b"prior report")?;
        assert!(render_vcf_parser_failure_audit(root.path(), &report).is_err());
        assert_eq!(fs::read(report)?, b"prior report");
        assert_eq!(fs::read(external)?, b"preserved external bytes");
        assert!(!root
            .path()
            .join("artifacts/bench-readiness/vcf-parser-failure-tests/runs")
            .exists());
    }
    Ok(())
}
#[test]
fn repeated_failure_audit_keeps_unrelated_evidence_and_removes_only_governed_absent_index(
) -> Result<()> {
    let root = tempfile::tempdir()?;
    populate(root.path())?;
    let published = root.path().join("artifacts/bench-readiness/vcf-parser-failure-tests");
    let probe = published.join("missing-index/fixture");
    fs::create_dir_all(&probe)?;
    let unrelated = probe.join("retained-evidence.txt");
    fs::write(&unrelated, b"historical evidence")?;
    let absent = probe.join("raw.postprocess.vcf.tbi");
    let report_path = root.path().join("report.json");
    for _ in 0..2 {
        fs::write(&absent, b"stale index")?;
        let report = render_vcf_parser_failure_audit(root.path(), &report_path)?;
        assert_eq!(report.passed_row_count, 7);
        assert!(!absent.exists());
        assert_eq!(fs::read(&unrelated)?, b"historical evidence");
        assert!(report.rows.iter().all(|row| !row.observed_error.contains("/runs/run-")));
    }
    Ok(())
}
