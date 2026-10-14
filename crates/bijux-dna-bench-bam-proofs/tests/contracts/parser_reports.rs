use anyhow::{Context, Result};
use bijux_dna_bench_bam_proofs::parser_probes::reports::{parse_report, BamReportKind};

#[test]
fn public_typed_reports_keep_raw_schema_refusals() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let path = owner.path().join("report.json");
    for (raw, signal) in [
        ("{}", "missing field `schema_version`"),
        (
            r#"{"schema_version":"first","schema_version":"second"}"#,
            "duplicate field `schema_version`",
        ),
        (r#"{"schema_version":"fixture","foreign":true}"#, "unknown field `foreign`"),
    ] {
        std::fs::write(&path, raw)?;
        let error = parse_report(&path, BamReportKind::ValidationSummary)
            .err()
            .context("invalid typed schema accepted")?;
        let diagnostic = format!("{error:#}");
        assert_eq!(error.to_string(), format!("parse {}", path.display()));
        assert!(diagnostic.contains(signal), "actual schema refusal: {diagnostic}");
    }
    Ok(())
}

#[test]
fn public_report_reads_preserve_kind_specific_error_context() -> Result<()> {
    let owner = tempfile::tempdir()?;
    for (kind, legacy_read) in [
        (BamReportKind::ValidationSummary, None),
        (BamReportKind::ContaminationJson, Some("read contamination json")),
        (BamReportKind::SexJson, Some("read sex json")),
        (BamReportKind::GenotypingJson, None),
    ] {
        let missing = owner.path().join("absent.json");
        let directory = owner.path().join("directory.json");
        std::fs::create_dir_all(&directory)?;
        let invalid_utf8 = owner.path().join("invalid-utf8.json");
        std::fs::write(&invalid_utf8, [0xff])?;
        for path in [&missing, &directory, &invalid_utf8] {
            let error = parse_report(path, kind).err().context("unreadable report admitted")?;
            let expected = if legacy_read.is_some() { "parse" } else { "read" };
            assert_eq!(error.to_string(), format!("{expected} {}", path.display()));
            if let Some(read_context) = legacy_read {
                assert!(format!("{error:#}").contains(read_context));
            }
        }
        let malformed = owner.path().join("malformed.json");
        std::fs::write(&malformed, "{")?;
        let error = parse_report(&malformed, kind).err().context("malformed report admitted")?;
        assert_eq!(error.to_string(), format!("parse {}", malformed.display()));
    }
    Ok(())
}

#[test]
fn public_legacy_report_projection_keeps_untyped_documents() -> Result<()> {
    let owner = tempfile::tempdir()?;
    let path = owner.path().join("legacy.json");
    for raw in ["{}", r#"{"foreign":true,"method":7}"#, r#"{"method":"first","method":"last"}"#] {
        std::fs::write(&path, raw)?;
        let expected: serde_json::Value = serde_json::from_str(raw)?;
        for kind in [BamReportKind::ContaminationJson, BamReportKind::SexJson] {
            assert_eq!(parse_report(&path, kind)?, expected);
        }
    }
    Ok(())
}
