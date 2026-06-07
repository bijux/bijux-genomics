use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_vcf::{parse_segment_stage_metrics, VcfDomainStage};

const VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.vcf_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfSegmentFixtureCase {
    tool_id: &'static str,
    stage: VcfDomainStage,
    case_id: &'static str,
    parser_id: &'static str,
    required_files: &'static [&'static str],
}

const ROH_REQUIRED_FILES: &[&str] =
    &["raw.command.json", "raw.hom", "raw.log", "expected.normalized.json"];
const IBD_REQUIRED_FILES: &[&str] = &[
    "raw.command.json",
    "raw.ibd_filtered_segments.tsv",
    "raw.ibd_summary.json",
    "raw.ibd_metrics.json",
    "raw.log",
    "expected.normalized.json",
];
const DEMOGRAPHY_REQUIRED_FILES: &[&str] = &[
    "raw.command.json",
    "raw.ne_trajectory.tsv",
    "raw.demography.json",
    "raw.demography_metrics.json",
    "raw.log",
    "expected.normalized.json",
];

const VCF_SEGMENT_FIXTURE_CASES: &[VcfSegmentFixtureCase] = &[
    VcfSegmentFixtureCase {
        tool_id: "plink2",
        stage: VcfDomainStage::Roh,
        case_id: "complete",
        parser_id: "parse_plink2_roh_segment_metrics",
        required_files: ROH_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "germline",
        stage: VcfDomainStage::Ibd,
        case_id: "complete",
        parser_id: "parse_germline_ibd_segment_metrics",
        required_files: IBD_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "germline",
        stage: VcfDomainStage::Ibd,
        case_id: "insufficient_marker_overlap",
        parser_id: "parse_germline_ibd_segment_metrics",
        required_files: IBD_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "ibdseq",
        stage: VcfDomainStage::Ibd,
        case_id: "complete",
        parser_id: "parse_ibdseq_ibd_segment_metrics",
        required_files: IBD_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "ibdseq",
        stage: VcfDomainStage::Ibd,
        case_id: "insufficient_marker_overlap",
        parser_id: "parse_ibdseq_ibd_segment_metrics",
        required_files: IBD_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "ibdhap",
        stage: VcfDomainStage::Ibd,
        case_id: "complete",
        parser_id: "parse_ibdhap_ibd_segment_metrics",
        required_files: IBD_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "ibdhap",
        stage: VcfDomainStage::Ibd,
        case_id: "insufficient_marker_overlap",
        parser_id: "parse_ibdhap_ibd_segment_metrics",
        required_files: IBD_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "ibdne",
        stage: VcfDomainStage::Demography,
        case_id: "complete",
        parser_id: "parse_ibdne_demography_metrics",
        required_files: DEMOGRAPHY_REQUIRED_FILES,
    },
    VcfSegmentFixtureCase {
        tool_id: "ibdne",
        stage: VcfDomainStage::Demography,
        case_id: "insufficient_data",
        parser_id: "parse_ibdne_demography_metrics",
        required_files: DEMOGRAPHY_REQUIRED_FILES,
    },
];

#[test]
fn vcf_segment_fixture_bank_covers_governed_descent_rows() {
    let governed_rows = VCF_SEGMENT_FIXTURE_CASES
        .iter()
        .map(|case| format!("{}:{}:{}", case.tool_id, case.stage.as_str(), case.case_id))
        .collect::<Vec<_>>();
    assert_eq!(
        governed_rows,
        vec![
            "plink2:vcf.roh:complete",
            "germline:vcf.ibd:complete",
            "germline:vcf.ibd:insufficient_marker_overlap",
            "ibdseq:vcf.ibd:complete",
            "ibdseq:vcf.ibd:insufficient_marker_overlap",
            "ibdhap:vcf.ibd:complete",
            "ibdhap:vcf.ibd:insufficient_marker_overlap",
            "ibdne:vcf.demography:complete",
            "ibdne:vcf.demography:insufficient_data",
        ]
    );
    for case in VCF_SEGMENT_FIXTURE_CASES {
        let dir = fixture_dir(case);
        assert!(dir.exists(), "missing fixture directory: {}", dir.display());
        for file_name in case.required_files {
            assert!(
                dir.join(file_name).exists(),
                "missing fixture file `{file_name}` for {} / {} / {}",
                case.tool_id,
                case.stage.as_str(),
                case.case_id
            );
        }
    }
}

#[test]
fn vcf_segment_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in VCF_SEGMENT_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(observed, expected, case);
    }
    Ok(())
}

#[test]
fn vcf_segment_fixture_bank_parses_insufficient_cases_as_structured_reports() -> Result<()> {
    for case in VCF_SEGMENT_FIXTURE_CASES.iter().filter(|case| case.case_id != "complete") {
        let normalized = parse_segment_stage_metrics(case.tool_id, case.stage, &fixture_dir(case))
            .with_context(|| {
                format!(
                    "parse insufficient segment fixture {} / {} / {}",
                    case.tool_id,
                    case.stage.as_str(),
                    case.case_id
                )
            })?;
        let status =
            normalized.get("status").and_then(serde_json::Value::as_str).unwrap_or_default();
        assert_eq!(
            status, case.case_id,
            "insufficient segment fixture must normalize into a structured status"
        );
    }
    Ok(())
}

fn render_case(case: &VcfSegmentFixtureCase) -> Result<serde_json::Value> {
    let normalized = parse_segment_stage_metrics(case.tool_id, case.stage, &fixture_dir(case))
        .with_context(|| {
            format!(
                "parse segment fixture {} / {} / {}",
                case.tool_id,
                case.stage.as_str(),
                case.case_id
            )
        })?;
    Ok(serde_json::json!({
        "schema_version": VCF_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage.as_str(),
        "tool_id": case.tool_id,
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &VcfSegmentFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join("expected.normalized.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &VcfSegmentFixtureCase) -> PathBuf {
    repo_root()
        .join("benchmarks/tests/fixtures/bench/parsers/vcf/segments")
        .join(case.tool_id)
        .join(case.stage.as_str())
        .join(case.case_id)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn assert_json_matches(
    observed: serde_json::Value,
    expected: serde_json::Value,
    case: &VcfSegmentFixtureCase,
) {
    assert!(
        observed == expected,
        "VCF segment parser fixture mismatch for {} / {} / {}\nobserved: {observed:#}\nexpected: {expected:#}",
        case.tool_id,
        case.stage.as_str(),
        case.case_id
    );
}
