use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::bam_command_adapter_coverage::{
    collect_bam_command_adapter_coverage_rows, BamBenchmarkStatus,
};
use super::fastq_command_adapter_coverage::{
    collect_fastq_command_adapter_coverage_rows, FastqBenchmarkStatus,
};
use super::tool_execution_modes::{
    load_runtime_probe, validate_tool_execution_modes_path, DEFAULT_TOOL_EXECUTION_MODES_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_STAGE_TOOL_CONTAINERS_PATH: &str =
    "configs/bench/local/stage-tool-containers.toml";
pub(crate) const LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_VERSION: &str =
    "bijux.bench.local_stage_tool_containers.v1";
const STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.stage_tool_containers.v1";
const STAGE_TOOL_CONTAINERS_SCOPE: &str = "benchmark_ready_runtime_declarations";
const STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN: &str =
    "runtime_probe_with_primary_operator_runtime";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageToolContainersConfig {
    pub(crate) schema_version: String,
    pub(crate) classification_scope: String,
    pub(crate) rows: Vec<StageToolContainerRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageToolContainerRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) install_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) container_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) command_entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) host_binary_mode: Option<String>,
    pub(crate) declaration_origin: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct StageToolContainersReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) classification_scope: &'static str,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) external_row_count: usize,
    pub(crate) container_declared_row_count: usize,
    pub(crate) command_entrypoint_row_count: usize,
    pub(crate) host_binary_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) execution_mode_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<StageToolContainerRow>,
}

pub(crate) fn run_render_stage_tool_containers(
    args: &parse::BenchReadinessRenderStageToolContainersArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_stage_tool_containers(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_STAGE_TOOL_CONTAINERS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn render_stage_tool_containers(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<StageToolContainersReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_stage_tool_container_rows(repo_root)?;

    let config = StageToolContainersConfig {
        schema_version: LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_VERSION.to_string(),
        classification_scope: STAGE_TOOL_CONTAINERS_SCOPE.to_string(),
        rows: rows.clone(),
    };
    let rendered = render_stage_tool_containers_toml(&config)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut execution_mode_counts = BTreeMap::<String, usize>::new();
    let mut external_row_count = 0_usize;
    let mut container_declared_row_count = 0_usize;
    let mut command_entrypoint_row_count = 0_usize;
    let mut host_binary_row_count = 0_usize;
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *execution_mode_counts.entry(row.execution_mode.clone()).or_default() += 1;
        if row.install_kind != "workspace_binary" {
            external_row_count += 1;
        }
        if row.container_id.is_some() {
            container_declared_row_count += 1;
        }
        if row.command_entrypoint.is_some() {
            command_entrypoint_row_count += 1;
        }
        if row.host_binary_mode.is_some() {
            host_binary_row_count += 1;
        }
    }

    Ok(StageToolContainersReport {
        schema_version: STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, &output_path),
        classification_scope: STAGE_TOOL_CONTAINERS_SCOPE,
        row_count: rows.len(),
        benchmark_ready_row_count: rows.len(),
        external_row_count,
        container_declared_row_count,
        command_entrypoint_row_count,
        host_binary_row_count,
        domain_counts,
        execution_mode_counts,
        rows,
    })
}

fn collect_stage_tool_container_rows(repo_root: &Path) -> Result<Vec<StageToolContainerRow>> {
    let (_, _, fastq_rows) = collect_fastq_command_adapter_coverage_rows(repo_root)?;
    let (_, _, bam_rows) = collect_bam_command_adapter_coverage_rows(repo_root)?;
    let execution_mode_report = validate_tool_execution_modes_path(
        repo_root,
        &repo_root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
    )?;
    let execution_modes = execution_mode_report
        .rows
        .into_iter()
        .map(|row| (row.tool_id.clone(), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();
    for row in fastq_rows
        .into_iter()
        .filter(|row| row.benchmark_status == FastqBenchmarkStatus::BenchmarkReady)
    {
        rows.push(render_stage_tool_container_row(
            repo_root,
            "fastq",
            &row.stage_id,
            &row.tool_id,
            &execution_modes,
        )?);
    }

    for row in bam_rows
        .into_iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
    {
        rows.push(render_stage_tool_container_row(
            repo_root,
            "bam",
            &row.stage_id,
            &row.tool_id,
            &execution_modes,
        )?);
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_stage_tool_rows(&rows)?;
    Ok(rows)
}

fn render_stage_tool_container_row(
    repo_root: &Path,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    execution_modes: &BTreeMap<String, super::tool_execution_modes::ToolExecutionModeRow>,
) -> Result<StageToolContainerRow> {
    let execution_mode = execution_modes.get(tool_id).ok_or_else(|| {
        anyhow!("missing execution mode declaration for benchmark-ready tool `{tool_id}`")
    })?;
    let probe = load_runtime_probe(repo_root, domain, tool_id).with_context(|| {
        format!("load runtime probe for benchmark-ready row `{stage_id}` / `{tool_id}`")
    })?;

    let install_kind = probe.install_kind().to_string();
    if install_kind != execution_mode.expected_install_kind {
        return Err(anyhow!(
            "runtime install_kind `{}` for benchmark-ready row `{}` / `{}` does not match execution-mode expectation `{}`",
            install_kind,
            stage_id,
            tool_id,
            execution_mode.expected_install_kind
        ));
    }

    let host_binary_mode = match install_kind.as_str() {
        "workspace_binary" | "host_binary" => Some(install_kind.clone()),
        _ => None,
    };
    let container_id = probe.container_id();
    let command_entrypoint = probe.command_entrypoint();

    if install_kind == "container" && container_id.is_none() && command_entrypoint.is_none() {
        return Err(anyhow!(
            "benchmark-ready external row `{}` / `{}` must declare a container image or a command entrypoint",
            stage_id,
            tool_id
        ));
    }
    if install_kind == "workspace_binary" && host_binary_mode.is_none() {
        return Err(anyhow!(
            "benchmark-ready workspace row `{}` / `{}` must declare an explicit host-binary mode",
            stage_id,
            tool_id
        ));
    }
    if container_id.is_none() && command_entrypoint.is_none() && host_binary_mode.is_none() {
        return Err(anyhow!(
            "benchmark-ready row `{}` / `{}` must declare runtime location details",
            stage_id,
            tool_id
        ));
    }

    Ok(StageToolContainerRow {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        execution_mode: execution_mode.execution_mode.clone(),
        install_kind,
        container_id,
        command_entrypoint,
        host_binary_mode,
        declaration_origin: STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN.to_string(),
    })
}

fn ensure_unique_stage_tool_rows(rows: &[StageToolContainerRow]) -> Result<()> {
    let mut seen = BTreeSet::<(String, String)>::new();
    for row in rows {
        let pair = (row.stage_id.clone(), row.tool_id.clone());
        if !seen.insert(pair.clone()) {
            return Err(anyhow!(
                "stage-tool container rows repeat benchmark-ready pair `{}` / `{}`",
                pair.0,
                pair.1
            ));
        }
    }
    Ok(())
}

fn render_stage_tool_containers_toml(config: &StageToolContainersConfig) -> Result<String> {
    let body = toml::to_string_pretty(config).context("serialize stage-tool containers config")?;
    Ok(format!(
        "# schema_version = 1\n\
         # owner = bijux-dna-bench\n\
         # purpose = Governed runtime declarations for benchmark-ready FASTQ and BAM stage-tool commands.\n\
         # authority = bijux-dna-bench\n\
         # stability = evolving\n\n{body}"
    ))
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_containers_reports_governed_benchmark_ready_row_slice() {
        use super::{render_stage_tool_containers, DEFAULT_STAGE_TOOL_CONTAINERS_PATH};

        let root = repo_root();
        let report =
            render_stage_tool_containers(&root, PathBuf::from(DEFAULT_STAGE_TOOL_CONTAINERS_PATH))
                .expect("render stage-tool containers");

        assert_eq!(report.schema_version, "bijux.bench.readiness.stage_tool_containers.v1");
        assert_eq!(report.config_path, "configs/bench/local/stage-tool-containers.toml");
        assert_eq!(report.classification_scope, "benchmark_ready_runtime_declarations");
        assert_eq!(report.row_count, 55);
        assert_eq!(report.benchmark_ready_row_count, 55);
        assert_eq!(report.external_row_count, 55);
        assert_eq!(report.domain_counts.get("fastq"), Some(&47));
        assert_eq!(report.domain_counts.get("bam"), Some(&8));
        assert!(report.rows.iter().all(|row| {
            row.container_id.is_some()
                || row.command_entrypoint.is_some()
                || row.host_binary_mode.is_some()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.normalize_primers"
                && row.tool_id == "cutadapt"
                && row
                    .container_id
                    .as_deref()
                    .is_some_and(|value| value.starts_with("bijuxdna/cutadapt@sha256:"))
                && row.command_entrypoint.as_deref() == Some("cutadapt")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("fastqc")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789"
                    )
        }));
    }

    #[cfg(feature = "bam_downstream")]
    #[test]
    fn render_stage_tool_containers_writes_governed_toml_contract() {
        use super::{
            render_stage_tool_containers, StageToolContainersConfig,
            DEFAULT_STAGE_TOOL_CONTAINERS_PATH, LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_VERSION,
            STAGE_TOOL_CONTAINERS_SCOPE,
        };

        let root = repo_root();
        let output_path = root.join(DEFAULT_STAGE_TOOL_CONTAINERS_PATH);
        let _report =
            render_stage_tool_containers(&root, PathBuf::from(DEFAULT_STAGE_TOOL_CONTAINERS_PATH))
                .expect("render stage-tool containers");

        let raw = std::fs::read_to_string(&output_path).expect("read rendered config");
        let config: StageToolContainersConfig =
            toml::from_str(&raw).expect("parse rendered stage-tool container config");

        assert_eq!(config.schema_version, LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_VERSION);
        assert_eq!(config.classification_scope, STAGE_TOOL_CONTAINERS_SCOPE);
        assert_eq!(config.rows.len(), 55);
        assert!(config.rows.iter().all(|row| {
            row.container_id.is_some()
                || row.command_entrypoint.is_some()
                || row.host_binary_mode.is_some()
        }));
        assert!(config.rows.iter().any(|row| {
            row.stage_id == "fastq.detect_adapters"
                && row.tool_id == "fastqc"
                && row.execution_mode == "java"
                && row.command_entrypoint.as_deref() == Some("fastqc")
                && row.container_id.as_deref()
                    == Some(
                        "bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789"
                    )
        }));
    }
}
