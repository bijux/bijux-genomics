use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_expected_benchmark_results::{
    collect_all_domain_expected_benchmark_result_rows, AllDomainExpectedBenchmarkResultRow,
};
use super::all_domain_output_declarations::{
    collect_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
};
use crate::commands::benchmark::local_all_domain_fake_runs::{
    declared_output_ids, fake_run_all_domain_benchmark_results, AllDomainFakeRunResultReport,
};
use crate::commands::benchmark::local_stage_fake_runs::path_relative_to_repo;
use crate::commands::benchmark::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, BenchStageResultManifestV1, BenchStageResultStatus,
};
use crate::commands::benchmark::path_resolution::ensure_path_stays_within_benchmark_runs_root;
use crate::commands::benchmark::path_resolution::BenchmarkPathResolver;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_PATH: &str =
    "benchmarks/readiness/completion-check-all-domains.json";
const DEFAULT_ALL_DOMAIN_COMPLETION_CHECK_FIXTURE_ROOT: &str =
    "runs/bench/readiness-probes/all-domains/completion-check";
const ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_completion_check.v1";

const MUTATION_MISSING_DECLARED_OUTPUT: &str = "missing_declared_output";
const MUTATION_MISSING_NORMALIZED_METRICS: &str = "missing_normalized_metrics";
const MUTATION_MISSING_MANIFEST: &str = "missing_manifest";
const MUTATION_REQUIRED_FILE_EMPTY: &str = "required_file_empty";
const MUTATION_EXECUTION_NOT_SUCCESSFUL: &str = "execution_not_successful";

const REASON_MISSING_MANIFEST: &str = "missing_manifest";
const REASON_INVALID_MANIFEST: &str = "invalid_manifest";
const REASON_REQUIRED_FILES_INCOMPLETE: &str = "required_files_incomplete";
const REASON_MISSING_DECLARED_OUTPUTS: &str = "missing_declared_outputs";
const REASON_MISSING_NORMALIZED_METRICS: &str = "missing_normalized_metrics";
const REASON_EXECUTION_NOT_SUCCESSFUL: &str = "execution_not_successful";
const REASON_MANIFEST_OUTPUT_MISMATCH: &str = "manifest_output_mismatch";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AllDomainCompletionStatus {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainCompletionRequiredFile {
    pub(crate) file_id: String,
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) non_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainCompletionSeededMutation {
    pub(crate) mutation_id: String,
    pub(crate) result_id: String,
    pub(crate) evidence_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainCompletionCheckRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_script_path: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_path: String,
    pub(crate) manifest_exists: bool,
    pub(crate) manifest_valid: bool,
    pub(crate) manifest_output_match: bool,
    pub(crate) exit_code_zero: bool,
    pub(crate) runtime_status: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) declared_output_count: usize,
    pub(crate) present_declared_output_count: usize,
    pub(crate) missing_declared_output_ids: Vec<String>,
    pub(crate) normalized_metrics_count: usize,
    pub(crate) present_normalized_metrics_count: usize,
    pub(crate) missing_normalized_metric_ids: Vec<String>,
    pub(crate) required_file_count: usize,
    pub(crate) ready_required_file_count: usize,
    pub(crate) required_files: Vec<AllDomainCompletionRequiredFile>,
    pub(crate) completion_status: AllDomainCompletionStatus,
    pub(crate) failure_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainCompletionCheckReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fixture_root: String,
    pub(crate) row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) failure_reason_counts: BTreeMap<String, usize>,
    pub(crate) passes_behavior_test: bool,
    pub(crate) seeded_mutations: Vec<AllDomainCompletionSeededMutation>,
    pub(crate) rows: Vec<AllDomainCompletionCheckRow>,
}

pub(crate) fn run_render_all_domain_completion_check(
    args: &parse::BenchReadinessRenderAllDomainCompletionCheckArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let report = render_all_domain_completion_check(
        &repo_root,
        args.output.clone().unwrap_or_else(|| {
            benchmark_paths.benchmark_readiness_root().join("completion-check-all-domains.json")
        }),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_completion_check(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainCompletionCheckReport> {
    let benchmark_paths = BenchmarkPathResolver::new(repo_root, None);
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let fixture_root =
        benchmark_paths.benchmark_readiness_probe_root().join("all-domains/completion-check");
    ensure_path_stays_within_benchmark_runs_root(
        repo_root,
        &fixture_root,
        "all-domain completion fixture root",
    )?;
    if fixture_root.exists() {
        fs::remove_dir_all(&fixture_root)
            .with_context(|| format!("remove {}", fixture_root.display()))?;
    }
    let fake_runs = fake_run_all_domain_benchmark_results(repo_root, fixture_root.clone())
        .with_context(|| {
            format!("materialize all-domain completion fixture under {}", fixture_root.display())
        })?;

    let seeded_mutations = seed_all_domain_completion_mutations(repo_root, &fake_runs.results)
        .with_context(|| {
            format!("seed all-domain completion mutations under {}", fixture_root.display())
        })?;
    let rows = collect_all_domain_completion_rows(repo_root, &fake_runs.results)?;

    let complete_row_count = rows
        .iter()
        .filter(|row| row.completion_status == AllDomainCompletionStatus::Complete)
        .count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut failure_reason_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        for reason in &row.failure_reasons {
            *failure_reason_counts.entry(reason.clone()).or_default() += 1;
        }
    }

    let report = AllDomainCompletionCheckReport {
        schema_version: ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        fixture_root: path_relative_to_repo(repo_root, &fixture_root),
        row_count: rows.len(),
        complete_row_count,
        incomplete_row_count,
        domain_counts,
        failure_reason_counts,
        passes_behavior_test: false,
        seeded_mutations,
        rows,
    };
    let report = ensure_all_domain_completion_check_contract(report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn seed_all_domain_completion_mutations(
    repo_root: &Path,
    results: &[AllDomainFakeRunResultReport],
) -> Result<Vec<AllDomainCompletionSeededMutation>> {
    let mut mutations = Vec::with_capacity(5);

    let missing_declared_output =
        find_result(results, "fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats")?;
    let missing_declared_output_path =
        artifact_fake_run_path(repo_root, missing_declared_output, "qc_tsv")?;
    fs::remove_file(&missing_declared_output_path)
        .with_context(|| format!("remove {}", missing_declared_output_path.display()))?;
    mutations.push(AllDomainCompletionSeededMutation {
        mutation_id: MUTATION_MISSING_DECLARED_OUTPUT.to_string(),
        result_id: missing_declared_output.result_id.clone(),
        evidence_path: path_relative_to_repo(repo_root, &missing_declared_output_path),
    });

    let missing_normalized_metrics =
        find_result(results, "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2")?;
    let missing_normalized_metrics_path = artifact_fake_run_path(
        repo_root,
        missing_normalized_metrics,
        "classification_report_json",
    )?;
    fs::remove_file(&missing_normalized_metrics_path)
        .with_context(|| format!("remove {}", missing_normalized_metrics_path.display()))?;
    mutations.push(AllDomainCompletionSeededMutation {
        mutation_id: MUTATION_MISSING_NORMALIZED_METRICS.to_string(),
        result_id: missing_normalized_metrics.result_id.clone(),
        evidence_path: path_relative_to_repo(repo_root, &missing_normalized_metrics_path),
    });

    let missing_manifest =
        find_result(results, "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools")?;
    let missing_manifest_path = repo_root.join(&missing_manifest.stage_result_path);
    fs::remove_file(&missing_manifest_path)
        .with_context(|| format!("remove {}", missing_manifest_path.display()))?;
    mutations.push(AllDomainCompletionSeededMutation {
        mutation_id: MUTATION_MISSING_MANIFEST.to_string(),
        result_id: missing_manifest.result_id.clone(),
        evidence_path: path_relative_to_repo(repo_root, &missing_manifest_path),
    });

    let empty_required_file =
        find_result(results, "bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools")?;
    let empty_required_file_path = repo_root.join(&empty_required_file.command_script_path);
    fs::write(&empty_required_file_path, b"")
        .with_context(|| format!("empty {}", empty_required_file_path.display()))?;
    mutations.push(AllDomainCompletionSeededMutation {
        mutation_id: MUTATION_REQUIRED_FILE_EMPTY.to_string(),
        result_id: empty_required_file.result_id.clone(),
        evidence_path: path_relative_to_repo(repo_root, &empty_required_file_path),
    });

    let execution_not_successful =
        find_result(results, "bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc")?;
    let execution_manifest_path = repo_root.join(&execution_not_successful.stage_result_path);
    let mut execution_manifest =
        load_validated_stage_result_manifest_path(&execution_manifest_path)
            .with_context(|| format!("load {}", execution_manifest_path.display()))?;
    execution_manifest.runtime.exit_code = 23;
    execution_manifest.runtime.status = BenchStageResultStatus::Failed;
    bijux_dna_infra::atomic_write_json(&execution_manifest_path, &execution_manifest)?;
    mutations.push(AllDomainCompletionSeededMutation {
        mutation_id: MUTATION_EXECUTION_NOT_SUCCESSFUL.to_string(),
        result_id: execution_not_successful.result_id.clone(),
        evidence_path: path_relative_to_repo(repo_root, &execution_manifest_path),
    });

    mutations.sort_by(|left, right| left.mutation_id.cmp(&right.mutation_id));
    Ok(mutations)
}

fn collect_all_domain_completion_rows(
    repo_root: &Path,
    fake_run_results: &[AllDomainFakeRunResultReport],
) -> Result<Vec<AllDomainCompletionCheckRow>> {
    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let output_rows = collect_all_domain_output_declaration_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let fake_run_rows = fake_run_results
        .iter()
        .cloned()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    ensure_completion_row_alignment(&expected_rows, &output_rows, &fake_run_rows)?;

    let mut rows = Vec::with_capacity(expected_rows.len());
    for expected in expected_rows.values() {
        let output_row = output_rows.get(&expected.result_id).ok_or_else(|| {
            anyhow!(
                "all-domain completion checker is missing output declarations for `{}`",
                expected.result_id
            )
        })?;
        let fake_run_row = fake_run_rows.get(&expected.result_id).ok_or_else(|| {
            anyhow!(
                "all-domain completion checker is missing fake-run coverage for `{}`",
                expected.result_id
            )
        })?;
        rows.push(collect_completion_row(repo_root, expected, output_row, fake_run_row)?);
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    Ok(rows)
}

fn collect_completion_row(
    repo_root: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    output_row: &AllDomainOutputDeclarationRow,
    fake_run_row: &AllDomainFakeRunResultReport,
) -> Result<AllDomainCompletionCheckRow> {
    let command_script_path = repo_root.join(&fake_run_row.command_script_path);
    let stdout_path = repo_root.join(&fake_run_row.stdout_path);
    let stderr_path = repo_root.join(&fake_run_row.stderr_path);
    let metrics_path = repo_root.join(&fake_run_row.metrics_path);
    let stage_result_path = repo_root.join(&fake_run_row.stage_result_path);

    let required_files = vec![
        required_file_status(repo_root, "command_script", &command_script_path),
        required_file_status(repo_root, "stdout", &stdout_path),
        required_file_status(repo_root, "stderr", &stderr_path),
        required_file_status(repo_root, "metrics", &metrics_path),
    ];
    let ready_required_file_count =
        required_files.iter().filter(|file| file.exists && file.non_empty).count();

    let expected_declared_output_ids = declared_output_ids(output_row);
    let output_paths = fake_run_row
        .outputs
        .iter()
        .map(|output| (output.artifact_id.clone(), repo_root.join(&output.fake_run_path)))
        .collect::<BTreeMap<_, _>>();
    let mut missing_declared_output_ids = Vec::new();
    for artifact_id in &expected_declared_output_ids {
        let output_path = output_paths.get(artifact_id).ok_or_else(|| {
            anyhow!(
                "all-domain completion checker fake-run row `{}` is missing declared output path for `{artifact_id}`",
                expected.result_id
            )
        })?;
        if !output_path.exists() {
            missing_declared_output_ids.push(artifact_id.clone());
        }
    }
    let present_declared_output_count =
        expected_declared_output_ids.len().saturating_sub(missing_declared_output_ids.len());

    let mut missing_normalized_metric_ids = Vec::new();
    for artifact_id in &output_row.normalized_metrics {
        let output_path = output_paths.get(artifact_id).ok_or_else(|| {
            anyhow!(
                "all-domain completion checker fake-run row `{}` is missing normalized metric output path for `{artifact_id}`",
                expected.result_id
            )
        })?;
        if !path_has_content(output_path)? {
            missing_normalized_metric_ids.push(artifact_id.clone());
        }
    }
    let present_normalized_metrics_count =
        output_row.normalized_metrics.len().saturating_sub(missing_normalized_metric_ids.len());

    let manifest_exists = stage_result_path.is_file();
    let mut manifest_valid = false;
    let mut manifest_output_match = false;
    let mut runtime_status = None;
    let mut exit_code = None;
    let mut execution_successful = false;
    if manifest_exists {
        match load_validated_stage_result_manifest_path(&stage_result_path) {
            Ok(manifest) => {
                let manifest_identity_matches =
                    manifest.stage_id == expected.stage_id && manifest.tool.id == expected.tool_id;
                let manifest_status_succeeded =
                    manifest.runtime.status == BenchStageResultStatus::Succeeded;
                runtime_status = Some(status_label(&manifest.runtime.status).to_string());
                exit_code = Some(manifest.runtime.exit_code);
                execution_successful = manifest.runtime.exit_code == 0 && manifest_status_succeeded;
                manifest_valid = manifest_identity_matches;
                if manifest_identity_matches {
                    manifest_output_match = manifest_outputs_match(
                        repo_root,
                        expected,
                        &manifest,
                        &expected_declared_output_ids,
                        &output_paths,
                    )?;
                }
            }
            Err(_) => {
                manifest_valid = false;
            }
        }
    }

    let mut failure_reasons = Vec::<String>::new();
    if !manifest_exists {
        failure_reasons.push(REASON_MISSING_MANIFEST.to_string());
    } else if !manifest_valid {
        failure_reasons.push(REASON_INVALID_MANIFEST.to_string());
    }
    if manifest_exists && manifest_valid && !execution_successful {
        failure_reasons.push(REASON_EXECUTION_NOT_SUCCESSFUL.to_string());
    }
    if ready_required_file_count != required_files.len() {
        failure_reasons.push(REASON_REQUIRED_FILES_INCOMPLETE.to_string());
    }
    if !missing_declared_output_ids.is_empty() {
        failure_reasons.push(REASON_MISSING_DECLARED_OUTPUTS.to_string());
    }
    if !missing_normalized_metric_ids.is_empty() {
        failure_reasons.push(REASON_MISSING_NORMALIZED_METRICS.to_string());
    }
    if manifest_exists && manifest_valid && !manifest_output_match {
        failure_reasons.push(REASON_MANIFEST_OUTPUT_MISMATCH.to_string());
    }

    let completion_status = if manifest_exists
        && manifest_valid
        && manifest_output_match
        && execution_successful
        && ready_required_file_count == required_files.len()
        && missing_declared_output_ids.is_empty()
        && missing_normalized_metric_ids.is_empty()
    {
        AllDomainCompletionStatus::Complete
    } else {
        AllDomainCompletionStatus::Incomplete
    };

    Ok(AllDomainCompletionCheckRow {
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_script_path: fake_run_row.command_script_path.clone(),
        stdout_path: fake_run_row.stdout_path.clone(),
        stderr_path: fake_run_row.stderr_path.clone(),
        metrics_path: fake_run_row.metrics_path.clone(),
        stage_result_path: fake_run_row.stage_result_path.clone(),
        manifest_exists,
        manifest_valid,
        manifest_output_match,
        exit_code_zero: exit_code == Some(0),
        runtime_status,
        exit_code,
        declared_output_count: expected_declared_output_ids.len(),
        present_declared_output_count,
        missing_declared_output_ids,
        normalized_metrics_count: output_row.normalized_metrics.len(),
        present_normalized_metrics_count,
        missing_normalized_metric_ids,
        required_file_count: required_files.len(),
        ready_required_file_count,
        required_files,
        completion_status,
        failure_reasons,
    })
}

fn manifest_outputs_match(
    repo_root: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    manifest: &BenchStageResultManifestV1,
    expected_declared_output_ids: &[String],
    output_paths: &BTreeMap<String, PathBuf>,
) -> Result<bool> {
    let manifest_outputs = manifest
        .outputs
        .iter()
        .map(|output| (output.artifact_id.as_str(), output))
        .collect::<BTreeMap<_, _>>();
    let manifest_ids = manifest_outputs.keys().copied().collect::<BTreeSet<_>>();
    let expected_ids =
        expected_declared_output_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if manifest_ids != expected_ids {
        return Ok(false);
    }
    for artifact_id in expected_declared_output_ids {
        let manifest_output = manifest_outputs.get(artifact_id.as_str()).ok_or_else(|| {
            anyhow!(
                "completion checker manifest for `{}` is missing `{artifact_id}` despite matching id set",
                expected.result_id
            )
        })?;
        let expected_realized_path = output_paths.get(artifact_id).ok_or_else(|| {
            anyhow!(
                "completion checker is missing fake-run path for `{}` / `{artifact_id}`",
                expected.result_id
            )
        })?;
        if !manifest_output.exists
            || manifest_output.realized_path
                != path_relative_to_repo(repo_root, expected_realized_path)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn ensure_completion_row_alignment(
    expected_rows: &BTreeMap<String, AllDomainExpectedBenchmarkResultRow>,
    output_rows: &BTreeMap<String, AllDomainOutputDeclarationRow>,
    fake_run_rows: &BTreeMap<String, AllDomainFakeRunResultReport>,
) -> Result<()> {
    if expected_rows.len() != 126 || output_rows.len() != 126 || fake_run_rows.len() != 126 {
        return Err(anyhow!(
            "all-domain completion checker requires exactly 126 expected-result, output-declaration, and fake-run rows"
        ));
    }
    let expected_ids = expected_rows.keys().cloned().collect::<BTreeSet<_>>();
    let output_ids = output_rows.keys().cloned().collect::<BTreeSet<_>>();
    let fake_run_ids = fake_run_rows.keys().cloned().collect::<BTreeSet<_>>();
    if expected_ids != output_ids || expected_ids != fake_run_ids {
        return Err(anyhow!(
            "all-domain completion checker requires exact result_id alignment across expected results, output declarations, and fake runs"
        ));
    }
    Ok(())
}

fn ensure_all_domain_completion_check_contract(
    mut report: AllDomainCompletionCheckReport,
) -> Result<AllDomainCompletionCheckReport> {
    if report.row_count != 126 {
        return Err(anyhow!(
            "all-domain completion checker must report exactly 126 rows, found {}",
            report.row_count
        ));
    }
    if report.complete_row_count + report.incomplete_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain completion checker row counts do not sum to the total row count"
        ));
    }
    let passes_behavior_test = {
        let row_by_result_id =
            report.rows.iter().map(|row| (row.result_id.as_str(), row)).collect::<BTreeMap<_, _>>();
        let mutation_by_id = report
            .seeded_mutations
            .iter()
            .map(|mutation| (mutation.mutation_id.as_str(), mutation.result_id.as_str()))
            .collect::<BTreeMap<_, _>>();

        report.complete_row_count == 121
            && report.incomplete_row_count == 5
            && matches_seeded_reason(
                &row_by_result_id,
                mutation_by_id.get(MUTATION_MISSING_DECLARED_OUTPUT).copied(),
                REASON_MISSING_DECLARED_OUTPUTS,
            )
            && matches_seeded_reason(
                &row_by_result_id,
                mutation_by_id.get(MUTATION_MISSING_NORMALIZED_METRICS).copied(),
                REASON_MISSING_NORMALIZED_METRICS,
            )
            && matches_seeded_reason(
                &row_by_result_id,
                mutation_by_id.get(MUTATION_MISSING_MANIFEST).copied(),
                REASON_MISSING_MANIFEST,
            )
            && matches_seeded_reason(
                &row_by_result_id,
                mutation_by_id.get(MUTATION_REQUIRED_FILE_EMPTY).copied(),
                REASON_REQUIRED_FILES_INCOMPLETE,
            )
            && matches_seeded_reason(
                &row_by_result_id,
                mutation_by_id.get(MUTATION_EXECUTION_NOT_SUCCESSFUL).copied(),
                REASON_EXECUTION_NOT_SUCCESSFUL,
            )
    };

    report.passes_behavior_test = passes_behavior_test;

    if !report.passes_behavior_test {
        return Err(anyhow!("all-domain completion checker failed the governed behavior test"));
    }
    Ok(report)
}

fn matches_seeded_reason(
    rows: &BTreeMap<&str, &AllDomainCompletionCheckRow>,
    result_id: Option<&str>,
    required_reason: &str,
) -> bool {
    result_id.and_then(|result_id| rows.get(result_id)).is_some_and(|row| {
        row.completion_status == AllDomainCompletionStatus::Incomplete
            && row.failure_reasons.iter().any(|reason| reason == required_reason)
    })
}

fn find_result<'a>(
    results: &'a [AllDomainFakeRunResultReport],
    result_id: &str,
) -> Result<&'a AllDomainFakeRunResultReport> {
    results
        .iter()
        .find(|result| result.result_id == result_id)
        .ok_or_else(|| anyhow!("missing governed all-domain fake-run row `{result_id}`"))
}

fn artifact_fake_run_path(
    repo_root: &Path,
    result: &AllDomainFakeRunResultReport,
    artifact_id: &str,
) -> Result<PathBuf> {
    result
        .outputs
        .iter()
        .find(|output| output.artifact_id == artifact_id)
        .map(|output| repo_root.join(&output.fake_run_path))
        .ok_or_else(|| {
            anyhow!(
                "all-domain fake-run row `{}` is missing artifact `{artifact_id}`",
                result.result_id
            )
        })
}

fn required_file_status(
    repo_root: &Path,
    file_id: &str,
    path: &Path,
) -> AllDomainCompletionRequiredFile {
    AllDomainCompletionRequiredFile {
        file_id: file_id.to_string(),
        path: path_relative_to_repo(repo_root, path),
        exists: path.exists(),
        non_empty: path_has_content(path).unwrap_or(false),
    }
}

fn path_has_content(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    if path.is_dir() {
        return Ok(fs::read_dir(path)
            .with_context(|| format!("read {}", path.display()))?
            .next()
            .transpose()?
            .is_some());
    }
    Ok(fs::metadata(path).with_context(|| format!("stat {}", path.display()))?.len() > 0)
}

fn status_label(status: &BenchStageResultStatus) -> &'static str {
    match status {
        BenchStageResultStatus::Succeeded => "succeeded",
        BenchStageResultStatus::Failed => "failed",
    }
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
