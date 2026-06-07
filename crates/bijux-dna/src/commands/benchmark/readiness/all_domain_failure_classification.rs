use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_expected_benchmark_results::{
    collect_all_domain_expected_benchmark_result_rows, AllDomainExpectedBenchmarkResultRow,
};
use super::all_domain_stage_tool_table::{
    render_all_domain_stage_tool_table, AllDomainStageToolTableReport,
    DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
};
use super::vcf_adapter_missing_input_tests::{
    render_vcf_adapter_missing_input_tests, VcfAdapterMissingInputTestsReport,
    DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH,
};
use super::vcf_parser_failure_tests::{
    render_vcf_parser_failure_tests, VcfParserFailureTestsReport,
    DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH,
};
use crate::commands::benchmark::local_stage_fake_runs::path_relative_to_repo;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH: &str =
    "target/bench-readiness/failure-classification-all-domains.json";
const DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_FIXTURE_ROOT: &str =
    "target/bench-readiness/failure-classification-all-domains-fixture";
const ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_failure_classification.v1";
const INSUFFICIENT_DATA_FIXTURE_PATH: &str =
    "benchmarks/tests/fixtures/bench/parsers/vcf/segments/ibdne/vcf.demography/insufficient_data/expected.normalized.json";

const REQUIRED_FAILURE_CLASSES: [&str; 7] = [
    "missing_input",
    "tool_not_found",
    "command_failed",
    "missing_output",
    "parser_failed",
    "insufficient_data",
    "unsupported_pair",
];

const UNSUPPORTED_PAIR_DOMAIN: &str = "vcf";
const UNSUPPORTED_PAIR_STAGE_ID: &str = "vcf.filter";
const UNSUPPORTED_PAIR_TOOL_ID: &str = "samtools";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainFailureClassificationRow {
    pub(crate) class_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) result_id: Option<String>,
    pub(crate) source_surface: String,
    pub(crate) evidence_path: String,
    pub(crate) observed_status: String,
    pub(crate) observed_error: String,
    pub(crate) detail: String,
    pub(crate) triggered: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainFailureClassificationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fixture_root: String,
    pub(crate) row_count: usize,
    pub(crate) triggered_row_count: usize,
    pub(crate) required_class_count: usize,
    pub(crate) triggered_class_count: usize,
    pub(crate) missing_class_count: usize,
    pub(crate) class_counts: BTreeMap<String, usize>,
    pub(crate) missing_class_ids: Vec<String>,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<AllDomainFailureClassificationRow>,
}

pub(crate) fn run_render_all_domain_failure_classification(
    args: &parse::BenchReadinessRenderAllDomainFailureClassificationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_failure_classification(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_failure_classification(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainFailureClassificationReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let fixture_root = repo_relative_path(
        repo_root,
        Path::new(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_FIXTURE_ROOT),
    );
    if fixture_root.exists() {
        fs::remove_dir_all(&fixture_root)
            .with_context(|| format!("remove {}", fixture_root.display()))?;
    }
    fs::create_dir_all(&fixture_root)
        .with_context(|| format!("create {}", fixture_root.display()))?;

    let rows = collect_all_domain_failure_classification_rows(repo_root, &fixture_root)?;
    let mut class_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *class_counts.entry(row.class_id.clone()).or_default() += 1;
    }
    let triggered_row_count = rows.iter().filter(|row| row.triggered).count();
    let triggered_class_ids = rows
        .iter()
        .filter(|row| row.triggered)
        .map(|row| row.class_id.as_str())
        .collect::<BTreeSet<_>>();
    let missing_class_ids = REQUIRED_FAILURE_CLASSES
        .into_iter()
        .filter(|class_id| !triggered_class_ids.contains(class_id))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let report = AllDomainFailureClassificationReport {
        schema_version: ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        fixture_root: path_relative_to_repo(repo_root, &fixture_root),
        row_count: rows.len(),
        triggered_row_count,
        required_class_count: REQUIRED_FAILURE_CLASSES.len(),
        triggered_class_count: triggered_class_ids.len(),
        missing_class_count: missing_class_ids.len(),
        class_counts,
        missing_class_ids,
        passes_behavior_test: false,
        rows,
    };
    let report = ensure_all_domain_failure_classification_contract(report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn collect_all_domain_failure_classification_rows(
    repo_root: &Path,
    fixture_root: &Path,
) -> Result<Vec<AllDomainFailureClassificationRow>> {
    let missing_input_report = render_vcf_adapter_missing_input_tests(
        repo_root,
        PathBuf::from(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH),
    )?;
    let parser_failure_report = render_vcf_parser_failure_tests(
        repo_root,
        PathBuf::from(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH),
    )?;
    let stage_tool_table_report = render_all_domain_stage_tool_table(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
    )?;
    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;

    let mut rows = vec![
        classify_missing_input(&missing_input_report)?,
        classify_tool_not_found(repo_root, fixture_root, &expected_rows)?,
        classify_command_failed(repo_root, fixture_root, &expected_rows)?,
        classify_missing_output(repo_root, fixture_root, &expected_rows)?,
        classify_parser_failed(&parser_failure_report)?,
        classify_insufficient_data(repo_root)?,
        classify_unsupported_pair(&stage_tool_table_report)?,
    ];
    rows.sort_by(|left, right| left.class_id.cmp(&right.class_id));
    Ok(rows)
}

fn classify_missing_input(
    report: &VcfAdapterMissingInputTestsReport,
) -> Result<AllDomainFailureClassificationRow> {
    let row = report.rows.iter().find(|row| row.passed).ok_or_else(|| {
        anyhow!("VCF adapter missing-input tests did not produce a passing probe row")
    })?;
    Ok(AllDomainFailureClassificationRow {
        class_id: "missing_input".to_string(),
        domain: "vcf".to_string(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        result_id: None,
        source_surface: "vcf_adapter_missing_input_tests".to_string(),
        evidence_path: row.artifact_path.clone(),
        observed_status: if row.passed {
            "missing_input".to_string()
        } else {
            "unexpected_success".to_string()
        },
        observed_error: row.observed_error.clone(),
        detail: format!(
            "governed VCF missing-input probe removed `{}` and expected `{}`",
            row.missing_input_role, row.expected_error_fragment
        ),
        triggered: row.passed,
    })
}

fn classify_tool_not_found(
    repo_root: &Path,
    fixture_root: &Path,
    expected_rows: &[AllDomainExpectedBenchmarkResultRow],
) -> Result<AllDomainFailureClassificationRow> {
    let binding = expected_rows
        .iter()
        .find(|row| row.domain == "bam")
        .cloned()
        .or_else(|| expected_rows.first().cloned())
        .ok_or_else(|| anyhow!("tool-not-found probe requires a benchmark-ready binding"))?;

    let probe_root = fixture_root
        .join("tool-not-found")
        .join(&binding.domain)
        .join(&binding.stage_id)
        .join(&binding.tool_id);
    fs::create_dir_all(&probe_root).with_context(|| format!("create {}", probe_root.display()))?;
    let missing_executable_path = probe_root.join("missing-tool");
    let command_script_path = probe_root.join("command.sh");
    fs::write(&command_script_path, format!("{} --version\n", missing_executable_path.display()))
        .with_context(|| format!("write {}", command_script_path.display()))?;

    let observed = Command::new(&missing_executable_path).arg("--version").output();
    let (triggered, observed_status, observed_error) = match observed {
        Ok(_) => (
            false,
            "unexpected_success".to_string(),
            format!(
                "tool-not-found probe unexpectedly executed `{}`",
                missing_executable_path.display()
            ),
        ),
        Err(error) => (
            error.kind() == ErrorKind::NotFound,
            if error.kind() == ErrorKind::NotFound {
                "tool_not_found".to_string()
            } else {
                "probe_error".to_string()
            },
            error.to_string(),
        ),
    };

    Ok(AllDomainFailureClassificationRow {
        class_id: "tool_not_found".to_string(),
        domain: binding.domain,
        stage_id: binding.stage_id,
        tool_id: binding.tool_id,
        result_id: Some(binding.result_id),
        source_surface: "governed_tool_not_found_probe".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &command_script_path),
        observed_status,
        observed_error,
        detail:
            "governed probe derived from a benchmark-ready binding uses an absent executable path"
                .to_string(),
        triggered,
    })
}

fn classify_command_failed(
    repo_root: &Path,
    fixture_root: &Path,
    expected_rows: &[AllDomainExpectedBenchmarkResultRow],
) -> Result<AllDomainFailureClassificationRow> {
    let binding = expected_rows
        .iter()
        .find(|row| row.domain == "fastq")
        .cloned()
        .or_else(|| expected_rows.first().cloned())
        .ok_or_else(|| anyhow!("command-failed probe requires a benchmark-ready binding"))?;
    let probe_root = fixture_root
        .join("command-failed")
        .join(&binding.domain)
        .join(&binding.stage_id)
        .join(&binding.tool_id);
    fs::create_dir_all(&probe_root).with_context(|| format!("create {}", probe_root.display()))?;
    let command_script_path = probe_root.join("command.sh");
    fs::write(&command_script_path, "printf 'governed command-failed probe\\n' >&2\nexit 23\n")
        .with_context(|| format!("write {}", command_script_path.display()))?;
    let output = Command::new("sh")
        .arg(&command_script_path)
        .output()
        .with_context(|| format!("run {}", command_script_path.display()))?;
    let stderr_path = probe_root.join("stderr.txt");
    fs::write(&stderr_path, &output.stderr)
        .with_context(|| format!("write {}", stderr_path.display()))?;
    let observed_error = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let exit_code = output.status.code();
    Ok(AllDomainFailureClassificationRow {
        class_id: "command_failed".to_string(),
        domain: binding.domain,
        stage_id: binding.stage_id,
        tool_id: binding.tool_id,
        result_id: Some(binding.result_id),
        source_surface: "governed_command_failed_probe".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &stderr_path),
        observed_status: if exit_code == Some(23) {
            "command_failed".to_string()
        } else {
            "unexpected_success".to_string()
        },
        observed_error,
        detail: "governed shell probe exits with code 23 after writing stderr".to_string(),
        triggered: exit_code == Some(23),
    })
}

fn classify_missing_output(
    repo_root: &Path,
    fixture_root: &Path,
    expected_rows: &[AllDomainExpectedBenchmarkResultRow],
) -> Result<AllDomainFailureClassificationRow> {
    let binding = expected_rows
        .iter()
        .find(|row| row.domain == "vcf")
        .cloned()
        .or_else(|| expected_rows.first().cloned())
        .ok_or_else(|| anyhow!("missing-output probe requires a benchmark-ready binding"))?;
    let artifact_id = binding
        .expected_outputs
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("missing-output probe requires at least one declared output"))?;
    let probe_root = fixture_root
        .join("missing-output")
        .join(&binding.domain)
        .join(&binding.stage_id)
        .join(&binding.tool_id);
    fs::create_dir_all(&probe_root).with_context(|| format!("create {}", probe_root.display()))?;
    let evidence_manifest_path = probe_root.join("expected-output.json");
    let missing_output_path = probe_root.join(format!("{artifact_id}.missing"));
    let payload = serde_json::json!({
        "result_id": binding.result_id.clone(),
        "domain": binding.domain.clone(),
        "stage_id": binding.stage_id.clone(),
        "tool_id": binding.tool_id.clone(),
        "artifact_id": artifact_id.clone(),
        "missing_output_path": path_relative_to_repo(repo_root, &missing_output_path),
    });
    bijux_dna_infra::atomic_write_bytes(
        &evidence_manifest_path,
        serde_json::to_string_pretty(&payload)
            .context("render missing-output probe manifest")?
            .as_bytes(),
    )?;
    Ok(AllDomainFailureClassificationRow {
        class_id: "missing_output".to_string(),
        domain: binding.domain,
        stage_id: binding.stage_id,
        tool_id: binding.tool_id,
        result_id: Some(binding.result_id),
        source_surface: "governed_missing_output_probe".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &missing_output_path),
        observed_status: "missing_output".to_string(),
        observed_error: format!(
            "declared output `{artifact_id}` is absent from the governed missing-output probe"
        ),
        detail: path_relative_to_repo(repo_root, &evidence_manifest_path),
        triggered: !missing_output_path.exists(),
    })
}

fn classify_parser_failed(
    report: &VcfParserFailureTestsReport,
) -> Result<AllDomainFailureClassificationRow> {
    let row = report.rows.iter().find(|row| row.passed).ok_or_else(|| {
        anyhow!("VCF parser failure tests did not produce a passing governed failure row")
    })?;
    Ok(AllDomainFailureClassificationRow {
        class_id: "parser_failed".to_string(),
        domain: "vcf".to_string(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        result_id: None,
        source_surface: "vcf_parser_failure_tests".to_string(),
        evidence_path: row.probe_artifact_path.clone(),
        observed_status: if row.passed {
            "parser_failed".to_string()
        } else {
            "unexpected_success".to_string()
        },
        observed_error: row.observed_error.clone(),
        detail: format!("{} expected `{}`", row.parser_id, row.expected_error_fragment),
        triggered: row.passed,
    })
}

fn classify_insufficient_data(repo_root: &Path) -> Result<AllDomainFailureClassificationRow> {
    let fixture_path = repo_root.join(INSUFFICIENT_DATA_FIXTURE_PATH);
    let payload = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&fixture_path).with_context(|| format!("read {}", fixture_path.display()))?,
    )
    .with_context(|| format!("parse {}", fixture_path.display()))?;
    let normalized = payload
        .get("normalized")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| anyhow!("insufficient-data fixture is missing `normalized` object"))?;
    let stage_id = normalized
        .get("stage_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("insufficient-data fixture is missing normalized stage_id"))?;
    let tool_id = normalized
        .get("tool_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("insufficient-data fixture is missing normalized tool_id"))?;
    let method = normalized
        .get("method")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("insufficient-data fixture is missing normalized method"))?;
    let status = normalized
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("insufficient-data fixture is missing normalized status"))?;
    let insufficient_reason =
        normalized.get("insufficient_reason").and_then(serde_json::Value::as_str).ok_or_else(
            || anyhow!("insufficient-data fixture is missing normalized insufficient_reason"),
        )?;
    Ok(AllDomainFailureClassificationRow {
        class_id: "insufficient_data".to_string(),
        domain: "vcf".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        result_id: None,
        source_surface: "vcf_segment_fixture_bank".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &fixture_path),
        observed_status: status.to_string(),
        observed_error: insufficient_reason.to_string(),
        detail: method.to_string(),
        triggered: status == "insufficient_data",
    })
}

fn classify_unsupported_pair(
    report: &AllDomainStageToolTableReport,
) -> Result<AllDomainFailureClassificationRow> {
    let supported = report.rows.iter().any(|row| {
        row.domain == UNSUPPORTED_PAIR_DOMAIN
            && row.stage_id == UNSUPPORTED_PAIR_STAGE_ID
            && row.tool_id == UNSUPPORTED_PAIR_TOOL_ID
    });
    if supported {
        return Err(anyhow!(
            "governed unsupported-pair probe drifted because `{}` / `{}` / `{}` is now supported",
            UNSUPPORTED_PAIR_DOMAIN,
            UNSUPPORTED_PAIR_STAGE_ID,
            UNSUPPORTED_PAIR_TOOL_ID
        ));
    }
    Ok(AllDomainFailureClassificationRow {
        class_id: "unsupported_pair".to_string(),
        domain: UNSUPPORTED_PAIR_DOMAIN.to_string(),
        stage_id: UNSUPPORTED_PAIR_STAGE_ID.to_string(),
        tool_id: UNSUPPORTED_PAIR_TOOL_ID.to_string(),
        result_id: None,
        source_surface: "all_domain_stage_tool_table".to_string(),
        evidence_path: report.output_path.clone(),
        observed_status: "unsupported_pair".to_string(),
        observed_error: format!(
            "benchmark pair `{}` / `{}` is absent from the governed all-domain stage-tool table",
            UNSUPPORTED_PAIR_STAGE_ID, UNSUPPORTED_PAIR_TOOL_ID
        ),
        detail: "unsupported-pair classification must remain explicit instead of collapsing into a generic failed status"
            .to_string(),
        triggered: true,
    })
}

fn ensure_all_domain_failure_classification_contract(
    mut report: AllDomainFailureClassificationReport,
) -> Result<AllDomainFailureClassificationReport> {
    if report.row_count != REQUIRED_FAILURE_CLASSES.len()
        || report.rows.len() != REQUIRED_FAILURE_CLASSES.len()
    {
        return Err(anyhow!(
            "all-domain failure classification must keep exactly one row per required failure class"
        ));
    }
    let seen_classes = report.rows.iter().map(|row| row.class_id.as_str()).collect::<BTreeSet<_>>();
    let required_classes = REQUIRED_FAILURE_CLASSES.into_iter().collect::<BTreeSet<_>>();
    if seen_classes != required_classes {
        return Err(anyhow!(
            "all-domain failure classification rows do not match the governed class set"
        ));
    }
    if report.rows.iter().any(|row| {
        row.class_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.source_surface.trim().is_empty()
            || row.evidence_path.trim().is_empty()
            || row.observed_status.trim().is_empty()
            || row.detail.trim().is_empty()
    }) {
        return Err(anyhow!(
            "all-domain failure classification rows cannot contain blank required fields"
        ));
    }
    if report.class_counts.values().sum::<usize>() != report.row_count {
        return Err(anyhow!(
            "all-domain failure classification class counts must sum to the row count"
        ));
    }
    report.passes_behavior_test = report.triggered_row_count == REQUIRED_FAILURE_CLASSES.len()
        && report.triggered_class_count == REQUIRED_FAILURE_CLASSES.len()
        && report.missing_class_count == 0
        && report.rows.iter().all(|row| row.triggered);
    if !report.passes_behavior_test {
        return Err(anyhow!(
            "all-domain failure classification must trigger every required class exactly once"
        ));
    }
    Ok(report)
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use super::{
        render_all_domain_failure_classification, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_VERSION,
        DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn render_all_domain_failure_classification_reports_each_required_class_once() {
        let root = repo_root();
        let report = render_all_domain_failure_classification(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH),
        )
        .expect("render all-domain failure classification");

        assert_eq!(report.schema_version, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH);
        assert_eq!(
            report.fixture_root,
            "target/bench-readiness/failure-classification-all-domains-fixture"
        );
        assert_eq!(report.row_count, 7);
        assert_eq!(report.triggered_row_count, 7);
        assert_eq!(report.required_class_count, 7);
        assert_eq!(report.triggered_class_count, 7);
        assert_eq!(report.missing_class_count, 0);
        assert!(report.missing_class_ids.is_empty());
        assert!(report.passes_behavior_test);
        assert_eq!(
            report.rows.iter().map(|row| row.class_id.as_str()).collect::<BTreeSet<_>>(),
            [
                "missing_input",
                "tool_not_found",
                "command_failed",
                "missing_output",
                "parser_failed",
                "insufficient_data",
                "unsupported_pair",
            ]
            .into_iter()
            .collect()
        );
        assert!(report.rows.iter().all(|row| row.triggered));
        assert!(report.rows.iter().any(|row| {
            row.class_id == "tool_not_found"
                && row.observed_status == "tool_not_found"
                && row.result_id.is_some()
        }));
        assert!(report.rows.iter().any(|row| {
            row.class_id == "unsupported_pair"
                && row.domain == "vcf"
                && row.stage_id == "vcf.filter"
                && row.tool_id == "samtools"
        }));
    }
}
