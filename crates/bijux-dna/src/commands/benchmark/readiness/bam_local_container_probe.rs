use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus, BamCommandAdapterCoverageRow,
};
use crate::commands::benchmark::local_stage_commands::{
    bam_stage_smoke_command, bam_stage_smoke_support_path, governed_bam_local_smoke_tool_id,
    has_bam_local_ready_only_contract,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH: &str =
    "benchmarks/readiness/bam/bam-local-container-smoke.tsv";
const BAM_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_local_container_smoke.v1";

const BAM_REGISTRY_PATHS: &[&str] = &[
    "configs/ci/registry/tool_registry.toml",
    "configs/ci/registry/tool_registry_container_experimental.toml",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct BamLocalContainerSmokeRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) registered_binary: String,
    pub(crate) tool_status: String,
    pub(crate) benchmark_status: String,
    pub(crate) support_status: String,
    pub(crate) adapter_status: String,
    pub(crate) parser_status: String,
    pub(crate) corpus_status: String,
    pub(crate) smoke_path_kind: String,
    pub(crate) smoke_runtime: String,
    pub(crate) smoke_tool_id: String,
    pub(crate) smoke_command: String,
    pub(crate) smoke_support_path: String,
    pub(crate) smoke_minimal_cmd: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamLocalContainerSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) host_stage_smoke_row_count: usize,
    pub(crate) container_smoke_row_count: usize,
    pub(crate) runtime_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<BamLocalContainerSmokeRow>,
}

#[derive(Debug, Clone, Default)]
struct BamRegistrySmokeRecord {
    tool_id: String,
    registered_binary: String,
    tool_status: String,
    dockerfile: String,
    apptainer_def: String,
    smoke_minimal_cmd: String,
    smoke_minimal_rationale: String,
}

pub(crate) fn run_render_bam_local_container_smoke(
    args: &parse::BenchReadinessRenderBamLocalContainerSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_bam_local_container_smoke(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_BAM_LOCAL_CONTAINER_SMOKE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_bam_local_container_smoke(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<BamLocalContainerSmokeReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_bam_local_container_smoke_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_bam_local_container_smoke_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut runtime_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *runtime_counts.entry(row.smoke_runtime.clone()).or_default() += 1;
    }
    let host_stage_smoke_row_count =
        rows.iter().filter(|row| row.smoke_path_kind == "host_stage_smoke").count();
    let container_smoke_row_count = rows.len().saturating_sub(host_stage_smoke_row_count);

    Ok(BamLocalContainerSmokeReport {
        schema_version: BAM_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        tool_count: rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len(),
        host_stage_smoke_row_count,
        container_smoke_row_count,
        runtime_counts,
        rows,
    })
}

pub(crate) fn collect_bam_local_container_smoke_rows(
    repo_root: &Path,
) -> Result<Vec<BamLocalContainerSmokeRow>> {
    let (_, _, retained_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let retained_rows = retained_rows
        .into_iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
        .collect::<Vec<_>>();
    let retained_tool_ids =
        retained_rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>();
    let registry_by_tool = load_bam_registry_smoke_records(repo_root, &retained_tool_ids)?;

    let mut rows = Vec::with_capacity(retained_rows.len());
    for retained_row in &retained_rows {
        rows.push(build_bam_local_container_smoke_row(repo_root, retained_row, &registry_by_tool)?);
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_bam_local_container_smoke_contract(repo_root, &rows)?;
    Ok(rows)
}

fn build_bam_local_container_smoke_row(
    repo_root: &Path,
    retained_row: &BamCommandAdapterCoverageRow,
    registry_by_tool: &BTreeMap<String, BamRegistrySmokeRecord>,
) -> Result<BamLocalContainerSmokeRow> {
    let registry_row = registry_by_tool.get(retained_row.tool_id.as_str()).ok_or_else(|| {
        anyhow!(
            "BAM local-container smoke is missing registry coverage for retained tool `{}`",
            retained_row.tool_id
        )
    })?;

    let governed_local_smoke_tool_id =
        governed_bam_local_smoke_tool_id(repo_root, &retained_row.stage_id)?;

    if governed_local_smoke_tool_id.as_deref() == Some(retained_row.tool_id.as_str()) {
        if let Some(smoke_support_path) =
            bam_stage_smoke_support_path(repo_root, &retained_row.stage_id)?
        {
            return Ok(BamLocalContainerSmokeRow {
                stage_id: retained_row.stage_id.clone(),
                tool_id: retained_row.tool_id.clone(),
                registered_binary: registry_row.registered_binary.clone(),
                tool_status: registry_row.tool_status.clone(),
                benchmark_status: benchmark_status_label(retained_row.benchmark_status).to_string(),
                support_status: retained_row.support_status.clone(),
                adapter_status: retained_row.adapter_status.clone(),
                parser_status: retained_row.parser_status.clone(),
                corpus_status: retained_row.corpus_status.clone(),
                smoke_path_kind: "host_stage_smoke".to_string(),
                smoke_runtime: "host".to_string(),
                smoke_tool_id: retained_row.tool_id.clone(),
                smoke_command: format!(
                    "bijux-dna bench local {}",
                    bam_stage_smoke_command(&retained_row.stage_id)?
                        .trim_start_matches("bijux-dna bench local ")
                ),
                smoke_support_path,
                smoke_minimal_cmd: String::new(),
                reason: format!(
                    "binding `{}` / `{}` matches the governed BAM local-smoke contract tool, so the exact tiny-fixture stage smoke wrapper is available on host",
                    retained_row.stage_id, retained_row.tool_id
                ),
            });
        }
    }

    let (smoke_path_kind, smoke_runtime, smoke_command, smoke_support_path) =
        resolve_container_smoke_wrapper(repo_root, registry_row)?;
    let host_clause = match governed_local_smoke_tool_id.as_deref() {
        Some(host_tool_id) => format!(
            "binding `{}` / `{}` does not match the governed BAM local-smoke contract tool `{host_tool_id}`",
            retained_row.stage_id, retained_row.tool_id
        ),
        None if has_bam_local_ready_only_contract(&retained_row.stage_id) => format!(
            "stage `{}` keeps governed local-ready plan coverage but no BAM tiny-fixture smoke wrapper",
            retained_row.stage_id
        ),
        None if registry_row.registered_binary != retained_row.tool_id => format!(
            "retained tool `{}` resolves through registered binary `{}`",
            retained_row.tool_id, registry_row.registered_binary
        ),
        None => format!(
            "retained tool `{}` has no exact BAM tiny-fixture smoke wrapper",
            retained_row.tool_id
        ),
    };
    let rationale_clause = if registry_row.smoke_minimal_rationale.trim().is_empty() {
        String::new()
    } else {
        format!("; {}", registry_row.smoke_minimal_rationale.trim())
    };

    Ok(BamLocalContainerSmokeRow {
        stage_id: retained_row.stage_id.clone(),
        tool_id: retained_row.tool_id.clone(),
        registered_binary: registry_row.registered_binary.clone(),
        tool_status: registry_row.tool_status.clone(),
        benchmark_status: benchmark_status_label(retained_row.benchmark_status).to_string(),
        support_status: retained_row.support_status.clone(),
        adapter_status: retained_row.adapter_status.clone(),
        parser_status: retained_row.parser_status.clone(),
        corpus_status: retained_row.corpus_status.clone(),
        smoke_path_kind,
        smoke_runtime,
        smoke_tool_id: registry_row.registered_binary.clone(),
        smoke_command,
        smoke_support_path,
        smoke_minimal_cmd: registry_row.smoke_minimal_cmd.clone(),
        reason: format!(
            "{host_clause}, so the governed container smoke wrapper is the available local exercise path for `{}` / `{}`{rationale_clause}",
            retained_row.stage_id, retained_row.tool_id
        ),
    })
}

fn resolve_container_smoke_wrapper(
    repo_root: &Path,
    registry_row: &BamRegistrySmokeRecord,
) -> Result<(String, String, String, String)> {
    let dockerfile = repo_root.join(&registry_row.dockerfile);
    if !registry_row.dockerfile.trim().is_empty() && dockerfile.is_file() {
        return Ok((
            "docker_container_smoke".to_string(),
            "docker-arm64".to_string(),
            format!("bijux-dna env smoke docker-arm64 {}", registry_row.registered_binary),
            path_relative_to_repo(repo_root, &dockerfile),
        ));
    }
    let apptainer_def = repo_root.join(&registry_row.apptainer_def);
    if !registry_row.apptainer_def.trim().is_empty() && apptainer_def.is_file() {
        return Ok((
            "apptainer_container_smoke".to_string(),
            "apptainer".to_string(),
            format!("bijux-dna env smoke apptainer {}", registry_row.registered_binary),
            path_relative_to_repo(repo_root, &apptainer_def),
        ));
    }

    bail!(
        "BAM local-container smoke could not resolve a checked-in container wrapper for retained tool `{}` (dockerfile=`{}`, apptainer_def=`{}`)",
        registry_row.tool_id,
        registry_row.dockerfile,
        registry_row.apptainer_def
    );
}

fn load_bam_registry_smoke_records(
    repo_root: &Path,
    retained_tool_ids: &BTreeSet<String>,
) -> Result<BTreeMap<String, BamRegistrySmokeRecord>> {
    let mut records = BTreeMap::<String, BamRegistrySmokeRecord>::new();

    for relative_path in BAM_REGISTRY_PATHS {
        let absolute_path = repo_root.join(relative_path);
        let raw = fs::read_to_string(&absolute_path)
            .with_context(|| format!("read {}", absolute_path.display()))?;
        let parsed: toml::Value =
            toml::from_str(&raw).with_context(|| format!("parse {}", absolute_path.display()))?;
        let entries = parsed
            .get("tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing tools in {}", absolute_path.display()))?;

        for entry in entries {
            let tool_id = entry
                .get("id")
                .and_then(toml::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("tool entry in {} is missing id", absolute_path.display()))?
                .to_string();
            if !retained_tool_ids.contains(&tool_id) {
                continue;
            }
            if relative_path.ends_with("tool_registry_container_experimental.toml")
                && records.contains_key(&tool_id)
            {
                continue;
            }
            let record = records.entry(tool_id.clone()).or_insert_with(|| BamRegistrySmokeRecord {
                tool_id: tool_id.clone(),
                ..BamRegistrySmokeRecord::default()
            });
            merge_registry_string(
                &mut record.registered_binary,
                string_field(entry, "expected_bin"),
                "expected_bin",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.tool_status,
                string_field(entry, "status"),
                "status",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.dockerfile,
                string_field(entry, "dockerfile"),
                "dockerfile",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.apptainer_def,
                string_field(entry, "apptainer_def"),
                "apptainer_def",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.smoke_minimal_cmd,
                string_field(entry, "smoke_minimal_cmd"),
                "smoke_minimal_cmd",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.smoke_minimal_rationale,
                string_field(entry, "smoke_minimal_rationale"),
                "smoke_minimal_rationale",
                &tool_id,
            )?;
        }
    }

    for (tool_id, record) in &records {
        if record.registered_binary.trim().is_empty() {
            bail!("BAM local-container smoke registry row `{tool_id}` is missing expected_bin");
        }
        if record.tool_status.trim().is_empty() {
            bail!("BAM local-container smoke registry row `{tool_id}` is missing status");
        }
    }
    Ok(records)
}

fn merge_registry_string(
    target: &mut String,
    incoming: String,
    field: &str,
    tool_id: &str,
) -> Result<()> {
    if incoming.trim().is_empty() {
        return Ok(());
    }
    if target.trim().is_empty() {
        *target = incoming;
        return Ok(());
    }
    if target != &incoming {
        return Err(anyhow!(
            "BAM local-container smoke registry field `{field}` drifted for `{tool_id}` (`{target}` vs `{incoming}`)"
        ));
    }
    Ok(())
}

fn string_field(value: &toml::Value, key: &str) -> String {
    value.get(key).and_then(toml::Value::as_str).unwrap_or_default().trim().to_string()
}

fn ensure_bam_local_container_smoke_contract(
    repo_root: &Path,
    rows: &[BamLocalContainerSmokeRow],
) -> Result<()> {
    let host_contract_row_count = rows
        .iter()
        .filter(|row| {
            governed_bam_local_smoke_tool_id(repo_root, &row.stage_id)
                .ok()
                .flatten()
                .is_some_and(|tool_id| tool_id == row.tool_id)
                && bam_stage_smoke_support_path(repo_root, &row.stage_id)
                    .is_ok_and(|path| path.is_some())
        })
        .count();
    if rows.len() != 49 {
        return Err(anyhow!(
            "BAM local-container smoke report drifted from the governed retained surface (expected 49 rows, found {})",
            rows.len()
        ));
    }
    let stage_count = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len();
    if stage_count != 24 {
        return Err(anyhow!(
            "BAM local-container smoke report drifted from the governed retained stage surface (expected 24 stages, found {stage_count})"
        ));
    }
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    if tool_count != 25 {
        return Err(anyhow!(
            "BAM local-container smoke report drifted from the governed retained tool surface (expected 25 tools, found {tool_count})"
        ));
    }

    let host_stage_smoke_row_count =
        rows.iter().filter(|row| row.smoke_path_kind == "host_stage_smoke").count();
    if host_stage_smoke_row_count != host_contract_row_count {
        return Err(anyhow!(
            "BAM local-container smoke host wrapper count drifted from the governed BAM local-smoke contract surface (expected {host_contract_row_count}, found {host_stage_smoke_row_count})"
        ));
    }

    let expected_rows = [
        (
            "bam.validate",
            "samtools",
            "samtools",
            "production",
            "benchmark_ready",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-bam-mini",
        ),
        (
            "bam.authenticity",
            "authenticct",
            "authenticct",
            "production",
            "benchmark_ready",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-adna-damage-mini",
        ),
        (
            "bam.coverage",
            "samtools",
            "samtools",
            "production",
            "benchmark_ready",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-bam-mini",
        ),
        (
            "bam.align",
            "bwa",
            "bwa",
            "production",
            "benchmark_ready",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-mini",
        ),
        (
            "bam.contamination",
            "verifybamid2",
            "verifybamid2",
            "production",
            "benchmark_ready",
            "supported",
            "runnable",
            "parser_fixture_validated",
            "fixture:corpus-01-adna-bam-mini",
        ),
    ];

    for (
        stage_id,
        tool_id,
        registered_binary,
        tool_status,
        benchmark_status,
        support_status,
        adapter_status,
        parser_status,
        corpus_status,
    ) in expected_rows
    {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
            .ok_or_else(|| {
                anyhow!("BAM local-container smoke report is missing `{stage_id}` / `{tool_id}`")
            })?;
        if row.registered_binary != registered_binary
            || row.tool_status != tool_status
            || row.benchmark_status != benchmark_status
            || row.support_status != support_status
            || row.adapter_status != adapter_status
            || row.parser_status != parser_status
            || row.corpus_status != corpus_status
        {
            return Err(anyhow!(
                "BAM local-container smoke row `{stage_id}` / `{tool_id}` drifted from the governed wrapper contract"
            ));
        }
        if !repo_root.join(&row.smoke_support_path).exists() {
            return Err(anyhow!(
                "BAM local-container smoke row `{stage_id}` / `{tool_id}` points at missing support path `{}`",
                row.smoke_support_path
            ));
        }
    }

    Ok(())
}

fn render_bam_local_container_smoke_tsv(rows: &[BamLocalContainerSmokeRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tregistered_binary\ttool_status\tbenchmark_status\tsupport_status\tadapter_status\tparser_status\tcorpus_status\tsmoke_path_kind\tsmoke_runtime\tsmoke_tool_id\tsmoke_command\tsmoke_support_path\tsmoke_minimal_cmd\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.registered_binary),
            sanitize_tsv(&row.tool_status),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.support_status),
            sanitize_tsv(&row.adapter_status),
            sanitize_tsv(&row.parser_status),
            sanitize_tsv(&row.corpus_status),
            sanitize_tsv(&row.smoke_path_kind),
            sanitize_tsv(&row.smoke_runtime),
            sanitize_tsv(&row.smoke_tool_id),
            sanitize_tsv(&row.smoke_command),
            sanitize_tsv(&row.smoke_support_path),
            sanitize_tsv(&row.smoke_minimal_cmd),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn benchmark_status_label(status: BamBenchmarkStatus) -> &'static str {
    match status {
        BamBenchmarkStatus::BenchmarkReady => "benchmark_ready",
        BamBenchmarkStatus::NotBenchmarkReady => "not_benchmark_ready",
    }
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}
