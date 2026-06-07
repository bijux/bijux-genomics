use std::path::{Path, PathBuf};

use super::*;
use anyhow::{Context, Result};

const FASTQ_RAW_PARSER_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.fastq_raw_parser.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FastqRawParserFixtureCase {
    stage_id: &'static str,
    tool_id: &'static str,
    parser_id: &'static str,
    raw_file: &'static str,
    expected_file: &'static str,
}

const FASTQ_RAW_PARSER_FIXTURE_CASES: &[FastqRawParserFixtureCase] = &[
    FastqRawParserFixtureCase {
        stage_id: "fastq.validate_reads",
        tool_id: "fastqvalidator",
        parser_id: "parse_fastqvalidator_count",
        raw_file: "raw.fastqvalidator.txt",
        expected_file: "expected.fastqvalidator-count.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.validate_reads",
        tool_id: "seqkit",
        parser_id: "parse_seqkit_stats",
        raw_file: "raw.seqkit.tsv",
        expected_file: "expected.seqkit-stats.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.profile_reads",
        tool_id: "seqkit",
        parser_id: "parse_seqkit_tool_metrics",
        raw_file: "raw.seqkit.tsv",
        expected_file: "expected.seqkit-tool-metrics.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.profile_read_lengths",
        tool_id: "seqkit",
        parser_id: "parse_length_histogram",
        raw_file: "raw.seqkit-fx2tab.tsv",
        expected_file: "expected.length-histogram.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.trim_reads",
        tool_id: "fastp",
        parser_id: "parse_fastp_metrics",
        raw_file: "raw.fastp.json",
        expected_file: "expected.fastp-metrics.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.trim_terminal_damage",
        tool_id: "adapterremoval",
        parser_id: "parse_adapterremoval_metrics",
        raw_file: "raw.adapterremoval.txt",
        expected_file: "expected.adapterremoval-metrics.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.detect_adapters",
        tool_id: "fastqc",
        parser_id: "parse_fastqc_summary_metrics",
        raw_file: "raw.fastqc-summary.txt",
        expected_file: "expected.fastqc-metrics.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.report_qc",
        tool_id: "multiqc",
        parser_id: "parse_multiqc_general_stats_metrics",
        raw_file: "raw.multiqc-general-stats.json",
        expected_file: "expected.multiqc-metrics.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "bbduk",
        parser_id: "parse_low_complexity_report",
        raw_file: "raw.low-complexity.txt",
        expected_file: "expected.low-complexity.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "bbduk",
        parser_id: "parse_bbduk_reads_removed",
        raw_file: "raw.bbduk-summary.txt",
        expected_file: "expected.bbduk-reads-removed.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "dustmasker",
        parser_id: "parse_low_complexity_report",
        raw_file: "raw.low-complexity.txt",
        expected_file: "expected.low-complexity.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "prinseq",
        parser_id: "parse_low_complexity_report",
        raw_file: "raw.low-complexity.txt",
        expected_file: "expected.low-complexity.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.remove_duplicates",
        tool_id: "clumpify",
        parser_id: "parse_deduplicate_report",
        raw_file: "raw.deduplicate.txt",
        expected_file: "expected.deduplicate.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.remove_duplicates",
        tool_id: "fastuniq",
        parser_id: "parse_deduplicate_report",
        raw_file: "raw.deduplicate.txt",
        expected_file: "expected.deduplicate.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.screen_taxonomy",
        tool_id: "fastq_screen",
        parser_id: "parse_screen_summary_tsv",
        raw_file: "raw.screen-summary.tsv",
        expected_file: "expected.screen-summary.json",
    },
];

#[test]
fn fastq_raw_parser_fixture_bank_covers_every_governed_case() {
    assert_eq!(FASTQ_RAW_PARSER_FIXTURE_CASES.len(), 15);
    for case in FASTQ_RAW_PARSER_FIXTURE_CASES {
        let dir = fixture_dir(case);
        assert!(dir.exists(), "missing fixture directory: {}", dir.display());
        assert!(
            dir.join(case.raw_file).exists(),
            "missing raw fixture `{}` for {} / {} / {}",
            case.raw_file,
            case.stage_id,
            case.tool_id,
            case.parser_id
        );
        assert!(
            dir.join(case.expected_file).exists(),
            "missing expected fixture `{}` for {} / {} / {}",
            case.expected_file,
            case.stage_id,
            case.tool_id,
            case.parser_id
        );
    }
}

#[test]
fn fastq_raw_parser_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in FASTQ_RAW_PARSER_FIXTURE_CASES {
        let raw =
            std::fs::read_to_string(fixture_dir(case).join(case.raw_file)).with_context(|| {
                format!("read raw fixture for {} / {}", case.stage_id, case.tool_id)
            })?;
        let expected = read_expected_json(case)?;
        let observed = parse_case(case, &raw)?;
        assert_eq!(
            observed, expected,
            "FASTQ raw parser fixture mismatch for {} / {} / {}",
            case.stage_id, case.tool_id, case.parser_id
        );
    }
    Ok(())
}

fn parse_case(case: &FastqRawParserFixtureCase, raw: &str) -> Result<serde_json::Value> {
    let normalized = match case.parser_id {
        "parse_fastqvalidator_count" => {
            serde_json::json!({ "total_reads": parse_fastqvalidator_count(raw)? })
        }
        "parse_seqkit_stats" => {
            let parsed = parse_seqkit_stats(raw)?;
            serde_json::json!({
                "reads": parsed.reads,
                "bases": parsed.bases,
                "mean_q": parsed.mean_q,
                "gc_percent": parsed.gc_percent,
            })
        }
        "parse_seqkit_tool_metrics" => serde_json::to_value(parse_seqkit_tool_metrics(raw)?)?,
        "parse_length_histogram" => {
            let parsed = parse_length_histogram(raw)?
                .into_iter()
                .map(|(length, count)| serde_json::json!({ "length": length, "count": count }))
                .collect::<Vec<_>>();
            serde_json::json!(parsed)
        }
        "parse_fastp_metrics" => serde_json::to_value(parse_fastp_metrics(raw)?)?,
        "parse_adapterremoval_metrics" => serde_json::to_value(parse_adapterremoval_metrics(raw)?)?,
        "parse_fastqc_summary_metrics" => serde_json::to_value(parse_fastqc_summary_metrics(raw)?)?,
        "parse_multiqc_general_stats_metrics" => {
            serde_json::to_value(parse_multiqc_general_stats_metrics(raw)?)?
        }
        "parse_low_complexity_report" => {
            serde_json::json!({ "reads_removed_low_complexity": parse_low_complexity_report(raw)? })
        }
        "parse_bbduk_reads_removed" => {
            serde_json::json!({ "reads_removed": parse_bbduk_reads_removed(raw)? })
        }
        "parse_deduplicate_report" => {
            let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
            serde_json::json!({ "reads_in": reads_in, "reads_out": reads_out })
        }
        "parse_screen_summary_tsv" => serde_json::to_value(parse_screen_summary_tsv(raw)?)?,
        parser_id => {
            return Err(anyhow::anyhow!(
                "unsupported FASTQ raw parser fixture parser_id `{parser_id}`"
            ));
        }
    };

    Ok(serde_json::json!({
        "schema_version": FASTQ_RAW_PARSER_FIXTURE_SCHEMA_VERSION,
        "stage_id": case.stage_id,
        "tool_id": case.tool_id,
        "parser_id": case.parser_id,
        "normalized": normalized,
    }))
}

fn read_expected_json(case: &FastqRawParserFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join(case.expected_file);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &FastqRawParserFixtureCase) -> PathBuf {
    repo_root().join("benchmarks/tests/fixtures/bench/parsers/fastq").join(case.stage_id).join(case.tool_id)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}
