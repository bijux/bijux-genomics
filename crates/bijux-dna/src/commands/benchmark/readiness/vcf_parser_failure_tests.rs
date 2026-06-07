use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    parse_bcftools_stage_metrics, parse_eigensoft_stage_metrics, parse_imputation_stage_metrics,
    parse_phasing_stage_metrics, parse_segment_stage_metrics, VcfDomainStage,
};
use serde::Serialize;

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH: &str =
    "benchmarks/readiness/vcf-parser-failure-tests.json";
const VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_parser_failure_tests.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfParserFailureTestRow {
    pub(crate) case_id: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) parser_id: String,
    pub(crate) failure_reason: String,
    pub(crate) fixture_path: String,
    pub(crate) probe_artifact_path: String,
    pub(crate) expected_error_fragment: String,
    pub(crate) observed_error: String,
    pub(crate) passed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfParserFailureTestsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) passed_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) failure_reason_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfParserFailureTestRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VcfParserFailureReason {
    EmptyOutput,
    MalformedVcf,
    MissingIndex,
    MissingSampleColumn,
    MalformedPcaTable,
    MalformedImputationQualityFile,
    MalformedSegmentFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VcfParserFailureCase {
    case_id: &'static str,
    stage: VcfDomainStage,
    tool_id: &'static str,
    parser_id: &'static str,
    fixture_dir: &'static str,
    failure_reason: VcfParserFailureReason,
    probe_artifact: &'static str,
    expected_error_fragment: &'static str,
}

const VCF_PARSER_FAILURE_CASES: &[VcfParserFailureCase] = &[
    VcfParserFailureCase {
        case_id: "empty-output",
        stage: VcfDomainStage::Call,
        tool_id: "bcftools",
        parser_id: "parse_bcftools_call_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call",
        failure_reason: VcfParserFailureReason::EmptyOutput,
        probe_artifact: "raw.calls.vcf",
        expected_error_fragment: "raw VCF is missing #CHROM header",
    },
    VcfParserFailureCase {
        case_id: "malformed-vcf",
        stage: VcfDomainStage::Call,
        tool_id: "bcftools",
        parser_id: "parse_bcftools_call_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call",
        failure_reason: VcfParserFailureReason::MalformedVcf,
        probe_artifact: "raw.calls.vcf",
        expected_error_fragment: "malformed raw VCF record at line",
    },
    VcfParserFailureCase {
        case_id: "missing-index",
        stage: VcfDomainStage::Postprocess,
        tool_id: "bcftools",
        parser_id: "parse_bcftools_postprocess_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.postprocess",
        failure_reason: VcfParserFailureReason::MissingIndex,
        probe_artifact: "raw.postprocess.vcf.tbi",
        expected_error_fragment: "required tabix index for postprocess output is missing",
    },
    VcfParserFailureCase {
        case_id: "missing-sample-column",
        stage: VcfDomainStage::Phasing,
        tool_id: "shapeit5",
        parser_id: "parse_shapeit5_phasing_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/phasing/shapeit5",
        failure_reason: VcfParserFailureReason::MissingSampleColumn,
        probe_artifact: "raw.phased.vcf",
        expected_error_fragment: "phased VCF row is missing sample columns",
    },
    VcfParserFailureCase {
        case_id: "malformed-pca-table",
        stage: VcfDomainStage::Pca,
        tool_id: "eigensoft",
        parser_id: "parse_eigensoft_pca_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/eigensoft/pca",
        failure_reason: VcfParserFailureReason::MalformedPcaTable,
        probe_artifact: "raw.evec",
        expected_error_fragment: "does not contain any numeric components",
    },
    VcfParserFailureCase {
        case_id: "malformed-imputation-quality-file",
        stage: VcfDomainStage::Impute,
        tool_id: "beagle",
        parser_id: "parse_beagle_impute_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.impute",
        failure_reason: VcfParserFailureReason::MalformedImputationQualityFile,
        probe_artifact: "raw.imputation_qc.json",
        expected_error_fragment: "raw.imputation_qc.json",
    },
    VcfParserFailureCase {
        case_id: "malformed-segment-file",
        stage: VcfDomainStage::Ibd,
        tool_id: "germline",
        parser_id: "parse_germline_ibd_segment_metrics",
        fixture_dir:
            "benchmarks/tests/fixtures/bench/parsers/vcf/segments/germline/vcf.ibd/complete",
        failure_reason: VcfParserFailureReason::MalformedSegmentFile,
        probe_artifact: "raw.ibd_filtered_segments.tsv",
        expected_error_fragment: "must have 7 columns",
    },
];

pub(crate) fn run_render_vcf_parser_failure_tests(
    args: &parse::BenchReadinessRenderVcfParserFailureTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_parser_failure_tests(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_parser_failure_tests(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfParserFailureTestsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_parser_failure_rows(repo_root)?;
    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let mut failure_reason_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *failure_reason_counts.entry(row.failure_reason.clone()).or_default() += 1;
    }
    if failed_row_count != 0 {
        let failed_cases = rows
            .iter()
            .filter(|row| !row.passed)
            .map(|row| format!("{}:{}:{}", row.stage_id, row.tool_id, row.failure_reason))
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF parser failure tests must pass for every governed case, failed rows: {}",
            failed_cases.join(", ")
        ));
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfParserFailureTestsReport {
        schema_version: VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        failure_reason_counts,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_vcf_parser_failure_rows(repo_root: &Path) -> Result<Vec<VcfParserFailureTestRow>> {
    let scratch_root = repo_root.join("artifacts/bench-readiness/vcf-parser-failure-tests");
    fs::create_dir_all(&scratch_root)
        .with_context(|| format!("create {}", scratch_root.display()))?;

    let mut rows = VCF_PARSER_FAILURE_CASES
        .iter()
        .map(|case| evaluate_case(repo_root, &scratch_root, case))
        .collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.stage_id
            .cmp(&right.stage_id)
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.parser_id.cmp(&right.parser_id))
            .then_with(|| left.failure_reason.cmp(&right.failure_reason))
    });
    Ok(rows)
}

fn evaluate_case(
    repo_root: &Path,
    scratch_root: &Path,
    case: &VcfParserFailureCase,
) -> Result<VcfParserFailureTestRow> {
    let temp = bijux_dna_infra::temp_dir_in(scratch_root, "vcf-parser-failure-")
        .map_err(anyhow::Error::from)
        .context("create VCF parser failure probe root")?;
    let fixture_source = repo_root.join(case.fixture_dir);
    let probe_root = temp.path().join("fixture");
    copy_fixture_dir(&fixture_source, &probe_root)?;
    let probe_artifact_path = materialize_failure_probe(&probe_root, case)?;
    let observed = parse_case(case, &probe_root);
    let observed_error = match observed {
        Ok(()) => format!("parser unexpectedly accepted {}", probe_artifact_path.display()),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(case.expected_error_fragment)
        && !observed_error.starts_with("parser unexpectedly accepted");

    Ok(VcfParserFailureTestRow {
        case_id: case.case_id.to_string(),
        stage_id: case.stage.as_str().to_string(),
        tool_id: case.tool_id.to_string(),
        parser_id: case.parser_id.to_string(),
        failure_reason: failure_reason_label(case.failure_reason).to_string(),
        fixture_path: path_relative_to_repo(repo_root, &fixture_source),
        probe_artifact_path: path_relative_to_repo(repo_root, &probe_artifact_path),
        expected_error_fragment: case.expected_error_fragment.to_string(),
        observed_error,
        passed,
    })
}

fn materialize_failure_probe(probe_root: &Path, case: &VcfParserFailureCase) -> Result<PathBuf> {
    let path = probe_root.join(case.probe_artifact);
    match case.failure_reason {
        VcfParserFailureReason::EmptyOutput => {
            fs::write(&path, "").with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedVcf => {
            fs::write(
                &path,
                "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample_a\nchr1\n",
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MissingIndex => {
            if path.exists() {
                fs::remove_file(&path).with_context(|| format!("remove {}", path.display()))?;
            }
        }
        VcfParserFailureReason::MissingSampleColumn => {
            fs::write(
                &path,
                "##fileformat=VCFv4.3\n##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tsample_a\nchr1\t10\t.\tA\tG\t60\tPASS\t.\tGT\n",
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedPcaTable => {
            fs::write(&path, "sample_a cohort_alpha\n")
                .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedImputationQualityFile => {
            fs::write(&path, "{ malformed json")
                .with_context(|| format!("write {}", path.display()))?;
        }
        VcfParserFailureReason::MalformedSegmentFile => {
            fs::write(
                &path,
                "sample_a\tsample_b\tcontig\tstart\tend\tlength_cm\tmarker_count\nsample_a\tsample_b\tchr1\t100\t200\t3.0\n",
            )
            .with_context(|| format!("write {}", path.display()))?;
        }
    }
    Ok(path)
}

fn parse_case(case: &VcfParserFailureCase, probe_root: &Path) -> Result<()> {
    match (case.tool_id, case.stage) {
        ("bcftools", VcfDomainStage::Call) => {
            parse_bcftools_stage_metrics(VcfDomainStage::Call, probe_root)?;
        }
        ("bcftools", VcfDomainStage::Postprocess) => {
            parse_bcftools_stage_metrics(VcfDomainStage::Postprocess, probe_root)?;
        }
        ("shapeit5", VcfDomainStage::Phasing) => {
            parse_phasing_stage_metrics("shapeit5", probe_root)?;
        }
        ("eigensoft", VcfDomainStage::Pca) => {
            parse_eigensoft_stage_metrics(VcfDomainStage::Pca, probe_root)?;
        }
        ("beagle", VcfDomainStage::Impute) => {
            parse_imputation_stage_metrics("beagle", VcfDomainStage::Impute, probe_root)?;
        }
        ("germline", VcfDomainStage::Ibd) => {
            parse_segment_stage_metrics("germline", VcfDomainStage::Ibd, probe_root)?;
        }
        _ => {
            return Err(anyhow!(
                "unsupported VCF parser failure case `{}` / `{}`",
                case.tool_id,
                case.stage.as_str()
            ));
        }
    }
    Ok(())
}

fn failure_reason_label(value: VcfParserFailureReason) -> &'static str {
    match value {
        VcfParserFailureReason::EmptyOutput => "empty_output",
        VcfParserFailureReason::MalformedVcf => "malformed_vcf",
        VcfParserFailureReason::MissingIndex => "missing_index",
        VcfParserFailureReason::MissingSampleColumn => "missing_sample_column",
        VcfParserFailureReason::MalformedPcaTable => "malformed_pca_table",
        VcfParserFailureReason::MalformedImputationQualityFile => {
            "malformed_imputation_quality_file"
        }
        VcfParserFailureReason::MalformedSegmentFile => "malformed_segment_file",
    }
}

fn copy_fixture_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to).with_context(|| format!("create {}", to.display()))?;
    for entry in fs::read_dir(from).with_context(|| format!("read {}", from.display()))? {
        let entry = entry.with_context(|| format!("read entry under {}", from.display()))?;
        let source_path = entry.path();
        let target_path = to.join(entry.file_name());
        if source_path.is_dir() {
            copy_fixture_dir(&source_path, &target_path)?;
        } else {
            fs::copy(&source_path, &target_path).with_context(|| {
                format!("copy {} to {}", source_path.display(), target_path.display())
            })?;
        }
    }
    Ok(())
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_parser_failure_tests, DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH,
        VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_vcf_parser_failure_tests_reports_governed_case_set() {
        let root = repo_root();
        let report = render_vcf_parser_failure_tests(
            &root,
            PathBuf::from(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH),
        )
        .expect("render VCF parser failure tests");

        assert_eq!(report.schema_version, VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH);
        assert_eq!(report.row_count, 7);
        assert_eq!(report.passed_row_count, 7);
        assert_eq!(report.failed_row_count, 0);
        assert_eq!(report.failure_reason_counts.get("empty_output"), Some(&1));
        assert_eq!(report.failure_reason_counts.get("malformed_vcf"), Some(&1));
        assert_eq!(report.failure_reason_counts.get("missing_index"), Some(&1));
        assert_eq!(report.failure_reason_counts.get("missing_sample_column"), Some(&1));
        assert_eq!(report.failure_reason_counts.get("malformed_pca_table"), Some(&1));
        assert_eq!(report.failure_reason_counts.get("malformed_imputation_quality_file"), Some(&1));
        assert_eq!(report.failure_reason_counts.get("malformed_segment_file"), Some(&1));
        assert!(report.rows.iter().all(|row| row.passed));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.postprocess"
                && row.tool_id == "bcftools"
                && row.failure_reason == "missing_index"
                && row
                    .observed_error
                    .contains("required tabix index for postprocess output is missing")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.phasing"
                && row.tool_id == "shapeit5"
                && row.failure_reason == "missing_sample_column"
                && row.observed_error.contains("phased VCF row is missing sample columns")
        }));
    }
}
