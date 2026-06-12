use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::tool_metrics::{
    parse_adapterremoval_metrics, parse_fastp_metrics, parse_fastqc_summary_metrics,
    parse_multiqc_general_stats_metrics, parse_seqkit_tool_metrics,
};
use super::{
    parse_bbduk_reads_removed, parse_deduplicate_report, parse_fastqvalidator_count,
    parse_length_histogram, parse_low_complexity_report, parse_screen_summary_tsv,
    parse_seqkit_stats,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FastqRawParserFailureClass {
    MissingRawOutput,
    EmptyRawOutput,
    MalformedRawOutput,
    UnexpectedSuccess,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqRawParserFailureContractRow {
    pub expected_failure_class: FastqRawParserFailureClass,
    pub observed_failure_class: FastqRawParserFailureClass,
    pub stage_id: String,
    pub tool_id: String,
    pub parser_id: String,
    pub observed_error: String,
    pub passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FastqRawParserFixtureCase {
    stage_id: &'static str,
    tool_id: &'static str,
    parser_id: &'static str,
    raw_file: &'static str,
}

const FASTQ_RAW_PARSER_FIXTURE_CASES: &[FastqRawParserFixtureCase] = &[
    FastqRawParserFixtureCase {
        stage_id: "fastq.validate_reads",
        tool_id: "fastqvalidator",
        parser_id: "parse_fastqvalidator_count",
        raw_file: "raw.fastqvalidator.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.validate_reads",
        tool_id: "seqkit",
        parser_id: "parse_seqkit_stats",
        raw_file: "raw.seqkit.tsv",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.profile_reads",
        tool_id: "seqkit",
        parser_id: "parse_seqkit_tool_metrics",
        raw_file: "raw.seqkit.tsv",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.profile_read_lengths",
        tool_id: "seqkit",
        parser_id: "parse_length_histogram",
        raw_file: "raw.seqkit-fx2tab.tsv",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.trim_reads",
        tool_id: "fastp",
        parser_id: "parse_fastp_metrics",
        raw_file: "raw.fastp.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.trim_terminal_damage",
        tool_id: "adapterremoval",
        parser_id: "parse_adapterremoval_metrics",
        raw_file: "raw.adapterremoval.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.detect_adapters",
        tool_id: "fastqc",
        parser_id: "parse_fastqc_summary_metrics",
        raw_file: "raw.fastqc-summary.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.report_qc",
        tool_id: "multiqc",
        parser_id: "parse_multiqc_general_stats_metrics",
        raw_file: "raw.multiqc-general-stats.json",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "bbduk",
        parser_id: "parse_low_complexity_report",
        raw_file: "raw.low-complexity.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "bbduk",
        parser_id: "parse_bbduk_reads_removed",
        raw_file: "raw.bbduk-summary.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "dustmasker",
        parser_id: "parse_low_complexity_report",
        raw_file: "raw.low-complexity.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.filter_low_complexity",
        tool_id: "prinseq",
        parser_id: "parse_low_complexity_report",
        raw_file: "raw.low-complexity.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.remove_duplicates",
        tool_id: "clumpify",
        parser_id: "parse_deduplicate_report",
        raw_file: "raw.deduplicate.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.remove_duplicates",
        tool_id: "fastuniq",
        parser_id: "parse_deduplicate_report",
        raw_file: "raw.deduplicate.txt",
    },
    FastqRawParserFixtureCase {
        stage_id: "fastq.screen_taxonomy",
        tool_id: "fastq_screen",
        parser_id: "parse_screen_summary_tsv",
        raw_file: "raw.screen-summary.tsv",
    },
];

pub fn evaluate_fastq_raw_parser_failure_contracts(
    repo_root: &Path,
    scratch_root: &Path,
) -> Result<Vec<FastqRawParserFailureContractRow>> {
    FASTQ_RAW_PARSER_FIXTURE_CASES
        .iter()
        .flat_map(|case| {
            [
                FastqRawParserFailureClass::MissingRawOutput,
                FastqRawParserFailureClass::EmptyRawOutput,
                FastqRawParserFailureClass::MalformedRawOutput,
            ]
            .into_iter()
            .map(move |failure_class| evaluate_case(repo_root, scratch_root, case, failure_class))
        })
        .collect()
}

fn evaluate_case(
    repo_root: &Path,
    scratch_root: &Path,
    case: &FastqRawParserFixtureCase,
    expected_failure_class: FastqRawParserFailureClass,
) -> Result<FastqRawParserFailureContractRow> {
    let temp = bijux_dna_infra::temp_dir_in(scratch_root, "fastq-parser-failure-")
        .map_err(anyhow::Error::from)
        .context("create FASTQ parser failure probe root")?;
    let raw_path = materialize_failure_probe(repo_root, temp.path(), case, expected_failure_class)?;
    let observed = match fs::read_to_string(&raw_path) {
        Ok(raw) => match parse_case(case, &raw) {
            Ok(()) => Ok(()),
            Err(error) => Err(error),
        },
        Err(error) => Err(anyhow!(error)).context(format!("read {}", raw_path.display())),
    };
    let (observed_failure_class, observed_error) = classify_observed_failure(&raw_path, observed);
    let passed =
        observed_failure_class == expected_failure_class && !observed_error.trim().is_empty();

    Ok(FastqRawParserFailureContractRow {
        expected_failure_class,
        observed_failure_class,
        stage_id: case.stage_id.to_string(),
        tool_id: case.tool_id.to_string(),
        parser_id: case.parser_id.to_string(),
        observed_error,
        passed,
    })
}

fn materialize_failure_probe(
    repo_root: &Path,
    temp_root: &Path,
    case: &FastqRawParserFixtureCase,
    failure_class: FastqRawParserFailureClass,
) -> Result<PathBuf> {
    let path = temp_root.join(case.raw_file);
    match failure_class {
        FastqRawParserFailureClass::MissingRawOutput => Ok(path),
        FastqRawParserFailureClass::EmptyRawOutput => {
            fs::write(&path, "").with_context(|| format!("write {}", path.display()))?;
            Ok(path)
        }
        FastqRawParserFailureClass::MalformedRawOutput => {
            let payload = malformed_payload(repo_root, case)?;
            fs::write(&path, payload).with_context(|| format!("write {}", path.display()))?;
            Ok(path)
        }
        FastqRawParserFailureClass::UnexpectedSuccess => Ok(path),
    }
}

fn malformed_payload(repo_root: &Path, case: &FastqRawParserFixtureCase) -> Result<String> {
    let fixture_path = fixture_dir(repo_root, case).join(case.raw_file);
    let payload = if case.raw_file.ends_with(".json") {
        "{ malformed json".to_string()
    } else if case.raw_file.ends_with(".tsv") {
        "not\ta\tvalid\tparser\tfixture".to_string()
    } else {
        "not a valid parser fixture".to_string()
    };
    if payload.is_empty() {
        return Err(anyhow!("malformed payload generation failed for {}", fixture_path.display()));
    }
    Ok(payload)
}

fn classify_observed_failure(
    raw_path: &Path,
    observed: Result<()>,
) -> (FastqRawParserFailureClass, String) {
    match observed {
        Ok(()) => (
            FastqRawParserFailureClass::UnexpectedSuccess,
            format!("parser unexpectedly accepted {}", raw_path.display()),
        ),
        Err(error) => {
            let failure_class = if !raw_path.exists() {
                FastqRawParserFailureClass::MissingRawOutput
            } else if raw_path.metadata().map(|metadata| metadata.len()).unwrap_or(1) == 0 {
                FastqRawParserFailureClass::EmptyRawOutput
            } else {
                FastqRawParserFailureClass::MalformedRawOutput
            };
            (failure_class, error.to_string())
        }
    }
}

fn parse_case(case: &FastqRawParserFixtureCase, raw: &str) -> Result<()> {
    match case.parser_id {
        "parse_fastqvalidator_count" => {
            parse_fastqvalidator_count(raw)?;
        }
        "parse_seqkit_stats" => {
            parse_seqkit_stats(raw)?;
        }
        "parse_seqkit_tool_metrics" => {
            parse_seqkit_tool_metrics(raw)?;
        }
        "parse_length_histogram" => {
            parse_length_histogram(raw)?;
        }
        "parse_fastp_metrics" => {
            parse_fastp_metrics(raw)?;
        }
        "parse_adapterremoval_metrics" => {
            parse_adapterremoval_metrics(raw)?;
        }
        "parse_fastqc_summary_metrics" => {
            parse_fastqc_summary_metrics(raw)?;
        }
        "parse_multiqc_general_stats_metrics" => {
            parse_multiqc_general_stats_metrics(raw)?;
        }
        "parse_low_complexity_report" => {
            parse_low_complexity_report(raw)?;
        }
        "parse_bbduk_reads_removed" => {
            parse_bbduk_reads_removed(raw)?;
        }
        "parse_deduplicate_report" => {
            parse_deduplicate_report(raw)?;
        }
        "parse_screen_summary_tsv" => {
            parse_screen_summary_tsv(raw)?;
        }
        parser_id => {
            return Err(anyhow!(
                "unsupported FASTQ raw parser failure contract parser_id `{parser_id}`"
            ));
        }
    }
    Ok(())
}

fn fixture_dir(repo_root: &Path, case: &FastqRawParserFixtureCase) -> PathBuf {
    repo_root
        .join("benchmarks/tests/fixtures/bench/parsers/fastq")
        .join(case.stage_id)
        .join(case.tool_id)
}

#[cfg(test)]
mod tests {
    use super::{evaluate_fastq_raw_parser_failure_contracts, FastqRawParserFailureClass};
    use anyhow::Result;
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn governed_fastq_raw_parser_failure_contracts_classify_missing_empty_and_malformed_inputs(
    ) -> Result<()> {
        let root = repo_root();
        let scratch_root = root.join("artifacts/contract-probes");
        let rows = evaluate_fastq_raw_parser_failure_contracts(&root, &scratch_root)?;

        assert_eq!(rows.len(), 45);
        assert!(rows.iter().all(|row| row.passed));
        assert_eq!(
            rows.iter()
                .filter(|row| {
                    row.expected_failure_class == FastqRawParserFailureClass::MissingRawOutput
                })
                .count(),
            15
        );
        assert_eq!(
            rows.iter()
                .filter(|row| {
                    row.expected_failure_class == FastqRawParserFailureClass::EmptyRawOutput
                })
                .count(),
            15
        );
        assert_eq!(
            rows.iter()
                .filter(|row| {
                    row.expected_failure_class == FastqRawParserFailureClass::MalformedRawOutput
                })
                .count(),
            15
        );
        assert!(rows.iter().all(|row| !row.observed_error.trim().is_empty()));
        assert!(rows.iter().any(|row| {
            row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.expected_failure_class == FastqRawParserFailureClass::MalformedRawOutput
                && row.observed_error.contains("fastqc total sequences missing")
        }));
        assert!(rows.iter().any(|row| {
            row.stage_id == "fastq.trim_terminal_damage"
                && row.tool_id == "adapterremoval"
                && row.expected_failure_class == FastqRawParserFailureClass::EmptyRawOutput
                && row
                    .observed_error
                    .contains("adapterremoval line for `Total number of read pairs` is missing")
        }));
        Ok(())
    }
}
