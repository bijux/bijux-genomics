use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    validate_all_domain_failure_classification, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1,
    REQUIRED_ALL_DOMAIN_FAILURE_CLASSES,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    AllDomainFailureClassificationReport, AllDomainFailureClassificationRow,
};
use bijux_dna_api::v1::api::run::run_command_with_context;

use super::all_domain_expected_benchmark_results::{
    collect_all_domain_expected_benchmark_result_rows, AllDomainExpectedBenchmarkResultRow,
};
use super::all_domain_stage_tool_table::{
    render_all_domain_stage_tool_table, AllDomainStageToolTableReport,
    DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
};
use super::catalog::load_external_artifact_tool_contracts;
use super::vcf_adapter_missing_input_audit::{
    render_vcf_adapter_missing_input_audit, VcfAdapterMissingInputTestsReport,
    DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH,
};
use super::vcf_parser_failure_audit::{
    render_vcf_parser_failure_audit, VcfParserFailureTestsReport,
    DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH,
};
use crate::commands::benchmark::local_stage_fake_runs::path_relative_to_repo;
use crate::commands::benchmark::path_resolution::{
    ensure_path_stays_within_benchmark_simulated_run_root, BenchmarkPathResolver,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH: &str =
    "artifacts/benchmark-smoke/readiness/failure-classification-all-domains.json";
const DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_FIXTURE_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/failure-classification";
const INSUFFICIENT_DATA_FIXTURE_PATH: &str =
    "benchmarks/tests/fixtures/bench/parsers/vcf/segments/ibdne/vcf.demography/insufficient_data/expected.normalized.json";

pub(crate) const REQUIRED_FAILURE_CLASSES: [&str; 8] = REQUIRED_ALL_DOMAIN_FAILURE_CLASSES;

const UNSUPPORTED_PAIR_DOMAIN: &str = "vcf";
const UNSUPPORTED_PAIR_STAGE_ID: &str = "vcf.filter";
const UNSUPPORTED_PAIR_TOOL_ID: &str = "samtools";

pub(crate) fn run_render_all_domain_failure_classification(
    args: &parse::BenchReadinessRenderAllDomainFailureClassificationArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let report = render_all_domain_failure_classification(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            benchmark_paths
                .benchmark_readiness_root()
                .join("failure-classification-all-domains.json")
        }),
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
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }

    let fixture_root =
        benchmark_paths.benchmark_readiness_probe_root().join("all-domains/failure-classification");
    ensure_path_stays_within_benchmark_simulated_run_root(
        repo_root,
        &fixture_root,
        "all-domain failure-classification fixture root",
    )?;
    if fixture_root.exists() {
        bijux_dna_infra::remove_dir_all(&fixture_root)
            .with_context(|| format!("remove {}", fixture_root.display()))?;
    }
    bijux_dna_infra::ensure_dir(&fixture_root)
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
    let mut report = AllDomainFailureClassificationReport {
        schema_version: ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1.to_string(),
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
    validate_all_domain_failure_classification(&mut report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn collect_all_domain_failure_classification_rows(
    repo_root: &Path,
    fixture_root: &Path,
) -> Result<Vec<AllDomainFailureClassificationRow>> {
    let missing_input_report = render_vcf_adapter_missing_input_audit(
        repo_root,
        PathBuf::from(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH),
    )?;
    let parser_failure_report = render_vcf_parser_failure_audit(
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
        classify_blocked_external_access(repo_root)?,
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

fn classify_blocked_external_access(repo_root: &Path) -> Result<AllDomainFailureClassificationRow> {
    let contract = load_external_artifact_tool_contracts(repo_root)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("blocked-external-access probe requires an external artifact"))?;
    let stage_id = contract.stage_ids.first().cloned().ok_or_else(|| {
        anyhow!("blocked-external-access probe tool `{}` has no declared stage", contract.tool_id)
    })?;
    let domain = stage_id
        .split_once('.')
        .map(|(domain, _)| domain.to_string())
        .ok_or_else(|| anyhow!("blocked-external-access probe has malformed stage `{stage_id}`"))?;

    Ok(AllDomainFailureClassificationRow {
        class_id: "blocked_external_access".to_string(),
        domain,
        stage_id,
        tool_id: contract.tool_id,
        result_id: None,
        source_surface: "external_artifact_admission_contract".to_string(),
        evidence_path: "containers/contracts/catalog.toml".to_string(),
        observed_status: "blocked_external_access".to_string(),
        observed_error: "no checksum-admitted operator artifact was supplied to repository-local readiness execution".to_string(),
        detail: "supported non-redistributable tooling remains in the comparison and is blocked until an exact external artifact admission is available".to_string(),
        triggered: true,
    })
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
        source_surface: "vcf_adapter_missing_input_audit".to_string(),
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
    bijux_dna_infra::ensure_dir(&probe_root)
        .with_context(|| format!("create {}", probe_root.display()))?;
    let missing_executable_path = probe_root.join("missing-tool");
    let command_script_path = probe_root.join("command.sh");
    let command_script = format!("{} --version\n", missing_executable_path.display());
    bijux_dna_infra::write_bytes(&command_script_path, command_script.as_bytes())
        .with_context(|| format!("write {}", command_script_path.display()))?;

    let observed = std::fs::metadata(&missing_executable_path);
    let (triggered, observed_status, observed_error) = match observed {
        Ok(_) => (
            false,
            "unexpected_success".to_string(),
            format!(
                "tool-not-found probe unexpectedly resolved `{}`",
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
    bijux_dna_infra::ensure_dir(&probe_root)
        .with_context(|| format!("create {}", probe_root.display()))?;
    let command_script_path = probe_root.join("command.sh");
    bijux_dna_infra::write_bytes(
        &command_script_path,
        b"printf 'governed command-failed probe\\n' >&2\nexit 23\n",
    )
    .with_context(|| format!("write {}", command_script_path.display()))?;
    let output = run_command_with_context(
        "sh",
        &[command_script_path.display().to_string()],
        Some(repo_root),
        None,
    )
    .with_context(|| format!("run {}", command_script_path.display()))?;
    let stderr_path = probe_root.join("stderr.txt");
    bijux_dna_infra::write_bytes(&stderr_path, output.stderr.as_bytes())
        .with_context(|| format!("write {}", stderr_path.display()))?;
    let observed_error = output.stderr.trim().to_string();
    let exit_code = output.exit_code;
    Ok(AllDomainFailureClassificationRow {
        class_id: "command_failed".to_string(),
        domain: binding.domain,
        stage_id: binding.stage_id,
        tool_id: binding.tool_id,
        result_id: Some(binding.result_id),
        source_surface: "governed_command_failed_probe".to_string(),
        evidence_path: path_relative_to_repo(repo_root, &stderr_path),
        observed_status: if exit_code == 23 {
            "command_failed".to_string()
        } else {
            "unexpected_success".to_string()
        },
        observed_error,
        detail: "governed shell probe exits with code 23 after writing stderr".to_string(),
        triggered: exit_code == 23,
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
    bijux_dna_infra::ensure_dir(&probe_root)
        .with_context(|| format!("create {}", probe_root.display()))?;
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
        source_surface: "vcf_parser_failure_audit".to_string(),
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
            "governed unsupported-pair probe drifted because `{UNSUPPORTED_PAIR_DOMAIN}` / `{UNSUPPORTED_PAIR_STAGE_ID}` / `{UNSUPPORTED_PAIR_TOOL_ID}` is now supported"
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
            "benchmark pair `{UNSUPPORTED_PAIR_STAGE_ID}` / `{UNSUPPORTED_PAIR_TOOL_ID}` is absent from the governed all-domain stage-tool table"
        ),
        detail: "unsupported-pair classification must remain explicit instead of collapsing into a generic failed status"
            .to_string(),
        triggered: true,
    })
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
        render_all_domain_failure_classification, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1,
        DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH,
    };

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn render_all_domain_failure_classification_reports_each_required_class_once() {
        let root = repo_root();
        let report = render_all_domain_failure_classification(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH),
        )
        .expect("render all-domain failure classification");

        assert_eq!(report.schema_version, ALL_DOMAIN_FAILURE_CLASSIFICATION_SCHEMA_V1);
        assert_eq!(report.output_path, DEFAULT_ALL_DOMAIN_FAILURE_CLASSIFICATION_PATH);
        assert_eq!(
            report.fixture_root,
            "artifacts/benchmark-smoke/simulated-runs/readiness-probes/all-domains/failure-classification"
        );
        assert_eq!(report.row_count, 8);
        assert_eq!(report.triggered_row_count, 8);
        assert_eq!(report.required_class_count, 8);
        assert_eq!(report.triggered_class_count, 8);
        assert_eq!(report.missing_class_count, 0);
        assert!(report.missing_class_ids.is_empty());
        assert!(report.passes_behavior_test);
        assert_eq!(
            report.rows.iter().map(|row| row.class_id.as_str()).collect::<BTreeSet<_>>(),
            [
                "missing_input",
                "blocked_external_access",
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
            row.class_id == "blocked_external_access"
                && row.observed_status == "blocked_external_access"
                && row.evidence_path == "containers/contracts/catalog.toml"
        }));
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
