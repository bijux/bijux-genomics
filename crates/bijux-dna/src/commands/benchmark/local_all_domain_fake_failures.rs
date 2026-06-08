use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_all_domain_fake_runs::{
    declared_output_ids, output_relative_path, output_role, render_result_command_script,
};
use super::local_stage_fake_runs::path_relative_to_repo;
use super::path_resolution::{
    ensure_path_stays_outside_benchmark_readiness_root, BenchmarkPathResolver,
};
use super::readiness::all_domain_expected_benchmark_results::{
    collect_all_domain_expected_benchmark_result_rows, AllDomainExpectedBenchmarkResultRow,
};
use super::readiness::all_domain_output_declarations::{
    collect_all_domain_output_declaration_rows, AllDomainOutputDeclarationRow,
    AllDomainOutputDeclarationStatus,
};
use super::readiness::all_domain_rendered_commands::{
    collect_all_domain_rendered_command_rows, AllDomainRenderedCommandRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT: &str =
    "runs/bench/local-fake-runs/all-domains-failures";
const ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_fake_failures.v1";
const ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_fake_failure_record.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainFakeFailureOutputEntry {
    pub(crate) artifact_id: String,
    pub(crate) declared_output: String,
    pub(crate) expected_fake_run_path: String,
    pub(crate) role: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainFakeFailureRecord {
    pub(crate) schema_version: &'static str,
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_source: String,
    pub(crate) command: String,
    pub(crate) exit_code: i32,
    pub(crate) command_script_path: String,
    pub(crate) stderr_path: String,
    pub(crate) failure_record_path: String,
    pub(crate) failed_output_count: usize,
    pub(crate) failed_outputs: Vec<AllDomainFakeFailureOutputEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainFakeFailuresManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) failure_root: String,
    pub(crate) root_manifest_path: String,
    pub(crate) result_count: usize,
    pub(crate) failed_output_count: usize,
    pub(crate) exit_code: i32,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) failures: Vec<AllDomainFakeFailureRecord>,
}

pub(crate) fn run_fake_run_all_domain_failures(
    args: &parse::BenchLocalFakeRunAllDomainFailuresArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let benchmark_paths = BenchmarkPathResolver::new(&repo_root, None);
    let manifest = fake_run_all_domain_failures(
        &repo_root,
        args.output_root.clone().unwrap_or_else(|| {
            benchmark_paths.benchmark_local_fake_run_root().join("all-domains-failures")
        }),
        args.exit_code,
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.failure_root);
    }
    Ok(())
}

pub(crate) fn fake_run_all_domain_failures(
    repo_root: &Path,
    output_root: PathBuf,
    exit_code: i32,
) -> Result<AllDomainFakeFailuresManifest> {
    let absolute_output_root = repo_relative_path(repo_root, &output_root);
    ensure_path_stays_outside_benchmark_readiness_root(
        repo_root,
        &absolute_output_root,
        "all-domain fake-failure output root",
    )?;
    fs::create_dir_all(&absolute_output_root)
        .with_context(|| format!("create {}", absolute_output_root.display()))?;

    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let command_rows = collect_all_domain_rendered_command_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let output_rows = collect_all_domain_output_declaration_rows(repo_root)?
        .into_iter()
        .map(|row| (row.result_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    ensure_fake_failure_row_alignment(&expected_rows, &command_rows, &output_rows)?;

    let mut failures = Vec::with_capacity(expected_rows.len());
    let mut failed_output_count = 0usize;
    let mut domain_counts = BTreeMap::<String, usize>::new();

    for expected in expected_rows.values() {
        let command = command_rows.get(&expected.result_id).ok_or_else(|| {
            anyhow!(
                "all-domain fake-failure runner is missing a rendered command row for `{}`",
                expected.result_id
            )
        })?;
        let outputs = output_rows.get(&expected.result_id).ok_or_else(|| {
            anyhow!(
                "all-domain fake-failure runner is missing an output declaration row for `{}`",
                expected.result_id
            )
        })?;
        let record = fake_fail_result(
            repo_root,
            &absolute_output_root,
            expected,
            command,
            outputs,
            exit_code,
        )?;
        failed_output_count += record.failed_output_count;
        *domain_counts.entry(record.domain.clone()).or_default() += 1;
        failures.push(record);
    }

    failures.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let root_manifest_path = absolute_output_root.join("manifest.json");
    let manifest = AllDomainFakeFailuresManifest {
        schema_version: ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION,
        failure_root: path_relative_to_repo(repo_root, &absolute_output_root),
        root_manifest_path: path_relative_to_repo(repo_root, &root_manifest_path),
        result_count: failures.len(),
        failed_output_count,
        exit_code,
        domain_counts,
        failures,
    };
    ensure_all_domain_fake_failure_contract(&manifest)?;
    bijux_dna_infra::atomic_write_json(&root_manifest_path, &manifest)?;
    Ok(manifest)
}

fn fake_fail_result(
    repo_root: &Path,
    failure_root: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
    outputs: &AllDomainOutputDeclarationRow,
    exit_code: i32,
) -> Result<AllDomainFakeFailureRecord> {
    if outputs.status != AllDomainOutputDeclarationStatus::Complete {
        return Err(anyhow!(
            "all-domain fake-failure runner requires complete output declarations for `{}`",
            expected.result_id
        ));
    }

    let result_root = failure_root
        .join(&expected.domain)
        .join(&expected.corpus_id)
        .join(&expected.stage_id)
        .join(&expected.asset_profile_id)
        .join(&expected.tool_id);
    fs::create_dir_all(&result_root)
        .with_context(|| format!("create {}", result_root.display()))?;

    let command_script_path = result_root.join("command.sh");
    fs::write(&command_script_path, render_result_command_script(expected, command))
        .with_context(|| format!("write {}", command_script_path.display()))?;

    let stderr_path = result_root.join("stderr.txt");
    fs::write(
        &stderr_path,
        format!(
            "fake local all-domain benchmark failure\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\nexit_code={exit_code}\ncommand={}\n",
            expected.result_id,
            expected.domain,
            expected.stage_id,
            expected.tool_id,
            command.script_commands.join(" && ")
        ),
    )
    .with_context(|| format!("write {}", stderr_path.display()))?;

    let failed_outputs = declared_output_ids(outputs)
        .into_iter()
        .map(|artifact_id| AllDomainFakeFailureOutputEntry {
            expected_fake_run_path: path_relative_to_repo(
                repo_root,
                &result_root.join("declared-outputs").join(output_relative_path(&artifact_id)),
            ),
            role: output_role(&artifact_id).to_string(),
            declared_output: artifact_id.clone(),
            artifact_id,
        })
        .collect::<Vec<_>>();

    let failure_record_path = result_root.join("failure.json");
    let record = AllDomainFakeFailureRecord {
        schema_version: ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION,
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_source: command.command_source.clone(),
        command: command.script_commands.join("\n"),
        exit_code,
        command_script_path: path_relative_to_repo(repo_root, &command_script_path),
        stderr_path: path_relative_to_repo(repo_root, &stderr_path),
        failure_record_path: path_relative_to_repo(repo_root, &failure_record_path),
        failed_output_count: failed_outputs.len(),
        failed_outputs,
    };
    ensure_failure_record_contract(&record)?;
    bijux_dna_infra::atomic_write_json(&failure_record_path, &record)?;
    Ok(record)
}

fn ensure_fake_failure_row_alignment(
    expected_rows: &BTreeMap<String, AllDomainExpectedBenchmarkResultRow>,
    command_rows: &BTreeMap<String, AllDomainRenderedCommandRow>,
    output_rows: &BTreeMap<String, AllDomainOutputDeclarationRow>,
) -> Result<()> {
    if expected_rows.len() != 125 || command_rows.len() != 125 || output_rows.len() != 125 {
        return Err(anyhow!(
            "all-domain fake-failure runner requires exactly 125 expected-result, command, and output rows"
        ));
    }
    let expected_ids = expected_rows.keys().cloned().collect::<BTreeSet<_>>();
    let command_ids = command_rows.keys().cloned().collect::<BTreeSet<_>>();
    let output_ids = output_rows.keys().cloned().collect::<BTreeSet<_>>();
    if expected_ids != command_ids || expected_ids != output_ids {
        return Err(anyhow!(
            "all-domain fake-failure runner requires exact result_id alignment across expected results, rendered commands, and output declarations"
        ));
    }
    Ok(())
}

fn ensure_failure_record_contract(record: &AllDomainFakeFailureRecord) -> Result<()> {
    if record.result_id.trim().is_empty()
        || record.domain.trim().is_empty()
        || record.stage_id.trim().is_empty()
        || record.tool_id.trim().is_empty()
        || record.corpus_id.trim().is_empty()
        || record.asset_profile_id.trim().is_empty()
        || record.command.trim().is_empty()
        || record.command_script_path.trim().is_empty()
        || record.stderr_path.trim().is_empty()
        || record.failure_record_path.trim().is_empty()
    {
        return Err(anyhow!(
            "all-domain fake-failure record is missing required fields for `{}`",
            record.result_id
        ));
    }
    if record.failed_output_count == 0 || record.failed_output_count != record.failed_outputs.len()
    {
        return Err(anyhow!(
            "all-domain fake-failure record `{}` must enumerate every failed output",
            record.result_id
        ));
    }
    Ok(())
}

fn ensure_all_domain_fake_failure_contract(report: &AllDomainFakeFailuresManifest) -> Result<()> {
    if report.result_count != 125 || report.failures.len() != 125 {
        return Err(anyhow!(
            "all-domain fake-failure runner must cover exactly 125 governed benchmark-ready results"
        ));
    }
    let unique_result_ids =
        report.failures.iter().map(|failure| failure.result_id.as_str()).collect::<BTreeSet<_>>();
    if unique_result_ids.len() != report.result_count {
        return Err(anyhow!(
            "all-domain fake-failure runner cannot repeat benchmark result identifiers"
        ));
    }
    if report.failures.iter().map(|failure| failure.failed_output_count).sum::<usize>()
        != report.failed_output_count
    {
        return Err(anyhow!(
            "all-domain fake-failure root counts must match per-result failed output counts"
        ));
    }
    Ok(())
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
