use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_stage_fake_runs::path_relative_to_repo;
use super::local_stage_result_manifest::{
    validate_stage_result_manifest, BenchStageResultCommandV1, BenchStageResultManifestV1,
    BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
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

pub(crate) const DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT: &str = "target/local-fake-runs/all-domains";
const ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION: &str = "bijux.bench.local_all_domain_fake_runs.v1";
const ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_fake_run_metrics.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainFakeRunOutputEntry {
    pub(crate) artifact_id: String,
    pub(crate) declared_output: String,
    pub(crate) fake_run_path: String,
    pub(crate) role: String,
    pub(crate) exists: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainFakeRunMetrics {
    pub(crate) schema_version: &'static str,
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_step_count: usize,
    pub(crate) declared_output_count: usize,
    pub(crate) expected_metric_count: usize,
    pub(crate) expected_metrics: Vec<String>,
    pub(crate) materialized_byte_count: u64,
    pub(crate) simulated_elapsed_seconds: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainFakeRunResultReport {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_source: String,
    pub(crate) command_step_count: usize,
    pub(crate) declared_output_count: usize,
    pub(crate) created_output_count: usize,
    pub(crate) expected_metric_count: usize,
    pub(crate) command_script_path: String,
    pub(crate) stdout_path: String,
    pub(crate) stderr_path: String,
    pub(crate) metrics_path: String,
    pub(crate) stage_result_path: String,
    pub(crate) outputs: Vec<AllDomainFakeRunOutputEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainFakeRunsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fake_run_root: String,
    pub(crate) root_manifest_path: String,
    pub(crate) result_count: usize,
    pub(crate) created_output_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) results: Vec<AllDomainFakeRunResultReport>,
}

struct ResultFakeRunArtifacts {
    command_script_path: PathBuf,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
    metrics_path: PathBuf,
    stage_result_path: PathBuf,
}

pub(crate) fn run_fake_run_all_domains(
    args: &parse::BenchLocalFakeRunAllDomainsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let manifest = fake_run_all_domain_benchmark_results(
        &repo_root,
        args.output_root.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_FAKE_RUN_ROOT)),
    )?;
    if args.json {
        render::json::print_pretty(&manifest)?;
    } else {
        println!("{}", manifest.fake_run_root);
    }
    Ok(())
}

pub(crate) fn fake_run_all_domain_benchmark_results(
    repo_root: &Path,
    output_root: PathBuf,
) -> Result<AllDomainFakeRunsReport> {
    let absolute_output_root = repo_relative_path(repo_root, &output_root);
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

    ensure_fake_run_row_alignment(&expected_rows, &command_rows, &output_rows)?;

    let mut results = Vec::with_capacity(expected_rows.len());
    let mut created_output_count = 0usize;
    let mut domain_counts = BTreeMap::<String, usize>::new();

    for expected in expected_rows.values() {
        let command = command_rows.get(&expected.result_id).ok_or_else(|| {
            anyhow!(
                "all-domain fake-runner is missing a rendered command row for `{}`",
                expected.result_id
            )
        })?;
        let outputs = output_rows.get(&expected.result_id).ok_or_else(|| {
            anyhow!(
                "all-domain fake-runner is missing an output declaration row for `{}`",
                expected.result_id
            )
        })?;
        let report = fake_run_result(repo_root, &absolute_output_root, expected, command, outputs)?;
        created_output_count += report.created_output_count;
        *domain_counts.entry(report.domain.clone()).or_default() += 1;
        results.push(report);
    }

    results.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let root_manifest_path = absolute_output_root.join("manifest.json");
    let manifest = AllDomainFakeRunsReport {
        schema_version: ALL_DOMAIN_FAKE_RUNS_SCHEMA_VERSION,
        fake_run_root: path_relative_to_repo(repo_root, &absolute_output_root),
        root_manifest_path: path_relative_to_repo(repo_root, &root_manifest_path),
        result_count: results.len(),
        created_output_count,
        domain_counts,
        results,
    };
    ensure_all_domain_fake_run_contract(&manifest)?;
    bijux_dna_infra::atomic_write_json(&root_manifest_path, &manifest)?;
    Ok(manifest)
}

fn fake_run_result(
    repo_root: &Path,
    fake_run_root: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
    outputs: &AllDomainOutputDeclarationRow,
) -> Result<AllDomainFakeRunResultReport> {
    if outputs.status != AllDomainOutputDeclarationStatus::Complete {
        return Err(anyhow!(
            "all-domain fake-runner requires complete output declarations for `{}`",
            expected.result_id
        ));
    }

    let result_root = fake_run_root
        .join(&expected.domain)
        .join(&expected.corpus_id)
        .join(&expected.stage_id)
        .join(&expected.asset_profile_id)
        .join(&expected.tool_id);
    fs::create_dir_all(&result_root)
        .with_context(|| format!("create {}", result_root.display()))?;

    let artifacts = ResultFakeRunArtifacts {
        command_script_path: result_root.join("command.sh"),
        stdout_path: result_root.join("stdout.txt"),
        stderr_path: result_root.join("stderr.txt"),
        metrics_path: result_root.join("metrics.json"),
        stage_result_path: result_root.join("stage-result.json"),
    };

    fs::write(&artifacts.command_script_path, render_result_command_script(expected, command))
        .with_context(|| format!("write {}", artifacts.command_script_path.display()))?;
    fs::write(&artifacts.stdout_path, render_result_stdout(expected, command, outputs))
        .with_context(|| format!("write {}", artifacts.stdout_path.display()))?;
    fs::write(
        &artifacts.stderr_path,
        format!(
            "fake local all-domain benchmark run produced no stderr\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\n",
            expected.result_id, expected.domain, expected.stage_id, expected.tool_id
        ),
    )
    .with_context(|| format!("write {}", artifacts.stderr_path.display()))?;

    let declared_outputs = declared_output_ids(outputs);
    let mut output_entries = Vec::with_capacity(declared_outputs.len());
    let mut materialized_byte_count = 0u64;
    for artifact_id in declared_outputs {
        let output_path =
            result_root.join("declared-outputs").join(output_relative_path(&artifact_id));
        materialize_fake_run_output(&output_path, expected, &artifact_id)
            .with_context(|| format!("materialize `{artifact_id}` for `{}`", expected.result_id))?;
        let exists = output_path.exists();
        if exists {
            materialized_byte_count += materialized_path_size(&output_path)?;
        }
        output_entries.push(AllDomainFakeRunOutputEntry {
            artifact_id: artifact_id.clone(),
            declared_output: artifact_id.clone(),
            fake_run_path: path_relative_to_repo(repo_root, &output_path),
            role: output_role(&artifact_id).to_string(),
            exists,
        });
    }

    let metrics = AllDomainFakeRunMetrics {
        schema_version: ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION,
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_step_count: command.command_steps.len(),
        declared_output_count: output_entries.len(),
        expected_metric_count: expected.expected_metrics.len(),
        expected_metrics: expected.expected_metrics.clone(),
        materialized_byte_count,
        simulated_elapsed_seconds: simulated_elapsed_seconds(command.command_steps.len()),
    };
    bijux_dna_infra::atomic_write_json(&artifacts.metrics_path, &metrics)?;

    let stage_result = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: expected.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: expected.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: command.script_commands.join("\n") },
        runtime: BenchStageResultRuntimeV1 {
            mode: "benchmark_fake_run".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: metrics.simulated_elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: output_entries
            .iter()
            .map(|output| BenchStageResultOutputV1 {
                artifact_id: output.artifact_id.clone(),
                declared_path: output.declared_output.clone(),
                realized_path: output.fake_run_path.clone(),
                role: output.role.clone(),
                optional: false,
                exists: output.exists,
            })
            .collect(),
    };
    validate_stage_result_manifest(&stage_result)?;
    bijux_dna_infra::atomic_write_json(&artifacts.stage_result_path, &stage_result)?;

    let report = AllDomainFakeRunResultReport {
        result_id: expected.result_id.clone(),
        domain: expected.domain.clone(),
        stage_id: expected.stage_id.clone(),
        tool_id: expected.tool_id.clone(),
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        command_source: command.command_source.clone(),
        command_step_count: command.command_steps.len(),
        declared_output_count: output_entries.len(),
        created_output_count: output_entries.iter().filter(|output| output.exists).count(),
        expected_metric_count: expected.expected_metrics.len(),
        command_script_path: path_relative_to_repo(repo_root, &artifacts.command_script_path),
        stdout_path: path_relative_to_repo(repo_root, &artifacts.stdout_path),
        stderr_path: path_relative_to_repo(repo_root, &artifacts.stderr_path),
        metrics_path: path_relative_to_repo(repo_root, &artifacts.metrics_path),
        stage_result_path: path_relative_to_repo(repo_root, &artifacts.stage_result_path),
        outputs: output_entries,
    };
    ensure_result_report_contract(&report)?;
    Ok(report)
}

fn declared_output_ids(outputs: &AllDomainOutputDeclarationRow) -> Vec<String> {
    let mut seen = BTreeSet::<&str>::new();
    let mut ordered = Vec::<String>::new();
    for artifact_id in outputs
        .raw_outputs
        .iter()
        .chain(outputs.normalized_metrics.iter())
        .chain(outputs.index_outputs.iter())
    {
        if seen.insert(artifact_id.as_str()) {
            ordered.push(artifact_id.clone());
        }
    }
    ordered
}

fn ensure_fake_run_row_alignment(
    expected_rows: &BTreeMap<String, AllDomainExpectedBenchmarkResultRow>,
    command_rows: &BTreeMap<String, AllDomainRenderedCommandRow>,
    output_rows: &BTreeMap<String, AllDomainOutputDeclarationRow>,
) -> Result<()> {
    if expected_rows.len() != 120 || command_rows.len() != 120 || output_rows.len() != 120 {
        return Err(anyhow!(
            "all-domain fake-runner requires exactly 120 expected-result, command, and output rows"
        ));
    }
    let expected_ids = expected_rows.keys().cloned().collect::<BTreeSet<_>>();
    let command_ids = command_rows.keys().cloned().collect::<BTreeSet<_>>();
    let output_ids = output_rows.keys().cloned().collect::<BTreeSet<_>>();
    if expected_ids != command_ids || expected_ids != output_ids {
        return Err(anyhow!(
            "all-domain fake-runner requires exact result_id alignment across expected results, rendered commands, and output declarations"
        ));
    }
    Ok(())
}

fn render_result_command_script(
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str(&format!(
        "# all-domain benchmark fake-run script\n# result_id={}\n# domain={}\n# stage_id={}\n# tool_id={}\n# corpus_id={}\n# asset_profile_id={}\n\n",
        expected.result_id,
        expected.domain,
        expected.stage_id,
        expected.tool_id,
        expected.corpus_id,
        expected.asset_profile_id
    ));
    for script_command in &command.script_commands {
        rendered.push_str(script_command);
        rendered.push('\n');
    }
    rendered
}

fn render_result_stdout(
    expected: &AllDomainExpectedBenchmarkResultRow,
    command: &AllDomainRenderedCommandRow,
    outputs: &AllDomainOutputDeclarationRow,
) -> String {
    let mut rendered = format!(
        "fake local all-domain benchmark run\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\ncorpus_id={}\nasset_profile_id={}\ncommand_source={}\n",
        expected.result_id,
        expected.domain,
        expected.stage_id,
        expected.tool_id,
        expected.corpus_id,
        expected.asset_profile_id,
        command.command_source
    );
    rendered.push_str("declared_outputs=\n");
    for artifact_id in declared_output_ids(outputs) {
        rendered.push_str(&format!("  - {artifact_id}\n"));
    }
    rendered.push_str("expected_metrics=\n");
    for metric in &expected.expected_metrics {
        rendered.push_str(&format!("  - {metric}\n"));
    }
    rendered.push_str("commands=\n");
    for script_command in &command.script_commands {
        rendered.push_str(&format!("  - {script_command}\n"));
    }
    rendered
}

fn output_relative_path(artifact_id: &str) -> PathBuf {
    let file_name = if artifact_id.ends_with("_bundle") {
        return PathBuf::from(artifact_id);
    } else if artifact_id.ends_with("_dir") {
        return PathBuf::from(artifact_id);
    } else if let Some(stem) = artifact_id.strip_suffix("_vcf_tbi") {
        format!("{stem}.vcf.tbi")
    } else if let Some(stem) = artifact_id.strip_suffix("_vcf") {
        format!("{stem}.vcf")
    } else if let Some(stem) = artifact_id.strip_suffix("_bam") {
        format!("{stem}.bam")
    } else if let Some(stem) = artifact_id.strip_suffix("_bai") {
        format!("{stem}.bai")
    } else if let Some(stem) = artifact_id.strip_suffix("_json") {
        format!("{stem}.json")
    } else if let Some(stem) = artifact_id.strip_suffix("_tsv") {
        format!("{stem}.tsv")
    } else if artifact_id.contains("reads") {
        format!("{artifact_id}.fastq")
    } else if artifact_id.contains("representatives") {
        format!("{artifact_id}.fasta")
    } else if artifact_id.contains("classification") || artifact_id.ends_with("_table") {
        format!("{artifact_id}.tsv")
    } else if artifact_id.contains("report")
        || artifact_id.contains("metrics")
        || artifact_id.contains("summary")
        || artifact_id.contains("decision")
        || artifact_id.contains("manifest")
    {
        format!("{artifact_id}.json")
    } else {
        format!("{artifact_id}.txt")
    };
    PathBuf::from(file_name)
}

fn materialize_fake_run_output(
    output_path: &Path,
    expected: &AllDomainExpectedBenchmarkResultRow,
    artifact_id: &str,
) -> Result<()> {
    if output_path_is_directory(artifact_id) {
        fs::create_dir_all(output_path)
            .with_context(|| format!("create {}", output_path.display()))?;
        let sentinel = output_path.join(".bijux-all-domain-fake-run-placeholder");
        fs::write(
            &sentinel,
            format!(
                "all-domain fake-run directory placeholder\nresult_id={}\nstage_id={}\nartifact_id={artifact_id}\n",
                expected.result_id, expected.stage_id
            ),
        )
        .with_context(|| format!("write {}", sentinel.display()))?;
        return Ok(());
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(output_path, fake_output_bytes(expected, artifact_id, output_path)?)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(())
}

fn fake_output_bytes(
    expected: &AllDomainExpectedBenchmarkResultRow,
    artifact_id: &str,
    output_path: &Path,
) -> Result<Vec<u8>> {
    if binary_output_extension(output_path) {
        return Ok(Vec::new());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("json") {
        return serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.bench.local_all_domain_fake_output.v1",
            "result_id": expected.result_id,
            "domain": expected.domain,
            "stage_id": expected.stage_id,
            "tool_id": expected.tool_id,
            "artifact_id": artifact_id,
        }))
        .context("serialize all-domain fake JSON output");
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("tsv") {
        return Ok(format!(
            "result_id\tdomain\tstage_id\ttool_id\tartifact_id\n{}\t{}\t{}\t{}\t{artifact_id}\n",
            expected.result_id, expected.domain, expected.stage_id, expected.tool_id
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("html") {
        return Ok(format!(
            "<html><body><h1>fake local all-domain benchmark output</h1><p>{}</p><p>{artifact_id}</p></body></html>\n",
            expected.result_id
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("fastq") {
        return Ok(format!(
            "@{}_{}\nACGT\n+\nIIII\n",
            expected.stage_id.replace('.', "_"),
            artifact_id.replace('.', "_")
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("fasta") {
        return Ok(format!(
            ">{}_{}\nACGTACGT\n",
            expected.stage_id.replace('.', "_"),
            artifact_id.replace('.', "_")
        )
        .into_bytes());
    }
    if output_path.extension().and_then(|ext| ext.to_str()) == Some("vcf") {
        return Ok(format!(
            "##fileformat=VCFv4.3\n#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\nchr1\t1\t{}\tA\tG\t.\tPASS\tRESULT_ID={};STAGE_ID={};TOOL_ID={}\n",
            artifact_id.replace('.', "_"),
            expected.result_id,
            expected.stage_id,
            expected.tool_id
        )
        .into_bytes());
    }

    Ok(format!(
        "fake local all-domain benchmark output\nresult_id={}\ndomain={}\nstage_id={}\ntool_id={}\nartifact_id={artifact_id}\n",
        expected.result_id, expected.domain, expected.stage_id, expected.tool_id
    )
    .into_bytes())
}

fn output_path_is_directory(artifact_id: &str) -> bool {
    artifact_id.ends_with("_bundle") || artifact_id.ends_with("_dir")
}

fn binary_output_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext, "bam" | "bai" | "bcf" | "gz" | "pdf" | "tbi" | "zip"))
}

fn materialized_path_size(path: &Path) -> Result<u64> {
    if path.is_dir() {
        let mut total = 0u64;
        for entry in fs::read_dir(path).with_context(|| format!("read {}", path.display()))? {
            let entry = entry?;
            total += materialized_path_size(&entry.path())?;
        }
        return Ok(total);
    }
    Ok(fs::metadata(path).with_context(|| format!("stat {}", path.display()))?.len())
}

fn output_role(artifact_id: &str) -> &'static str {
    if artifact_id.contains("manifest") {
        "manifest"
    } else if artifact_id.contains("metrics") {
        "metrics"
    } else if artifact_id.contains("report") || artifact_id.contains("summary") {
        "report"
    } else if artifact_id.ends_with("_vcf") {
        "vcf"
    } else if artifact_id.ends_with("_vcf_tbi") || artifact_id.ends_with("_bai") {
        "index"
    } else if artifact_id.ends_with("_bam") {
        "bam"
    } else if artifact_id.contains("reads") {
        "reads"
    } else if artifact_id.contains("table") || artifact_id.contains("classification") {
        "table"
    } else if artifact_id.contains("representatives") {
        "sequences"
    } else if artifact_id.ends_with("_bundle") || artifact_id.ends_with("_dir") {
        "bundle"
    } else {
        "artifact"
    }
}

fn simulated_elapsed_seconds(command_step_count: usize) -> f64 {
    1.0 + (command_step_count as f64 * 0.25)
}

fn ensure_result_report_contract(report: &AllDomainFakeRunResultReport) -> Result<()> {
    if report.declared_output_count == 0
        || report.created_output_count != report.declared_output_count
    {
        return Err(anyhow!(
            "all-domain fake-runner result `{}` did not materialize every declared output",
            report.result_id
        ));
    }
    for relative_path in [
        &report.command_script_path,
        &report.stdout_path,
        &report.stderr_path,
        &report.metrics_path,
        &report.stage_result_path,
    ] {
        if relative_path.trim().is_empty() {
            return Err(anyhow!(
                "all-domain fake-runner result `{}` has an empty artifact path",
                report.result_id
            ));
        }
    }
    if report.outputs.iter().any(|output| !output.exists || output.fake_run_path.trim().is_empty())
    {
        return Err(anyhow!(
            "all-domain fake-runner result `{}` is missing a materialized output path",
            report.result_id
        ));
    }
    Ok(())
}

fn ensure_all_domain_fake_run_contract(report: &AllDomainFakeRunsReport) -> Result<()> {
    if report.result_count != 120 || report.results.len() != 120 {
        return Err(anyhow!(
            "all-domain fake-runner must cover exactly 120 governed benchmark-ready results"
        ));
    }
    let unique_result_ids =
        report.results.iter().map(|result| result.result_id.as_str()).collect::<BTreeSet<_>>();
    if unique_result_ids.len() != report.result_count {
        return Err(anyhow!("all-domain fake-runner cannot repeat benchmark result identifiers"));
    }
    if report.results.iter().map(|result| result.created_output_count).sum::<usize>()
        != report.created_output_count
    {
        return Err(anyhow!(
            "all-domain fake-runner root counts must match per-result created output counts"
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

#[cfg(test)]
mod tests {
    use super::{output_relative_path, output_role};

    #[test]
    fn output_relative_path_keeps_expected_all_domain_shapes() {
        assert_eq!(output_relative_path("called_vcf").to_string_lossy(), "called.vcf");
        assert_eq!(output_relative_path("called_vcf_tbi").to_string_lossy(), "called.vcf.tbi");
        assert_eq!(
            output_relative_path("classification_report_json").to_string_lossy(),
            "classification_report.json"
        );
        assert_eq!(output_relative_path("qc_bundle").to_string_lossy(), "qc_bundle");
    }

    #[test]
    fn output_role_classifies_all_domain_symbols() {
        assert_eq!(output_role("called_vcf"), "vcf");
        assert_eq!(output_role("called_vcf_tbi"), "index");
        assert_eq!(output_role("trim_metrics"), "metrics");
        assert_eq!(output_role("classification_table"), "table");
        assert_eq!(output_role("qc_bundle"), "bundle");
    }
}
