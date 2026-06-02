use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::tool_serving_map::{
    render_bam_tool_serving_map, render_fastq_tool_serving_map, ToolServingMapRow,
    DEFAULT_BAM_TOOL_SERVING_MAP_PATH, DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_TOOL_EXECUTION_MODES_PATH: &str =
    "configs/bench/local/tool-execution-modes.toml";
pub(crate) const LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION: &str =
    "bijux.bench.local_tool_execution_modes.v1";
const TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.tool_execution_modes_validation.v1";
const TOOL_EXECUTION_CLASSIFICATION_SCOPE: &str = "primary_operator_runtime";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolExecutionModesConfig {
    schema_version: String,
    classification_scope: String,
    modes: Vec<ToolExecutionModeDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolExecutionModeDefinition {
    execution_mode: String,
    expected_install_kind: String,
    required_runtime_fields: Vec<String>,
    summary: String,
    tool_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ToolExecutionModeRow {
    pub(crate) tool_id: String,
    pub(crate) execution_mode: String,
    pub(crate) expected_install_kind: String,
    pub(crate) domains: Vec<String>,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) required_runtime_fields: Vec<String>,
    pub(crate) summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolExecutionModesValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) classification_scope: String,
    pub(crate) mode_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) multidomain_tool_count: usize,
    pub(crate) mode_counts: BTreeMap<String, usize>,
    pub(crate) valid: bool,
    pub(crate) rows: Vec<ToolExecutionModeRow>,
}

pub(crate) fn run_validate_tool_execution_modes(
    args: &parse::BenchReadinessValidateToolExecutionModesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let config_path = match &args.config {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
    };
    let report = validate_tool_execution_modes_path(&repo_root, &config_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn validate_tool_execution_modes_path(
    repo_root: &Path,
    config_path: &Path,
) -> Result<ToolExecutionModesValidationReport> {
    let config = load_tool_execution_modes_config(config_path)?;
    validate_tool_execution_modes_contract(&config)?;

    let fastq_map = render_fastq_tool_serving_map(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
    )?;
    let bam_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;
    let benchmark_tool_index = build_benchmark_tool_index(&fastq_map.rows, &bam_map.rows);
    let assignments = build_mode_assignment_index(&config)?;

    let configured_tool_ids = assignments.keys().cloned().collect::<BTreeSet<_>>();
    let benchmark_tool_ids = benchmark_tool_index.keys().cloned().collect::<BTreeSet<_>>();

    let missing_tool_ids =
        benchmark_tool_ids.difference(&configured_tool_ids).cloned().collect::<Vec<_>>();
    if !missing_tool_ids.is_empty() {
        return Err(anyhow!(
            "tool execution mode config is missing {} benchmark tool ids: {}",
            missing_tool_ids.len(),
            missing_tool_ids.join(", "),
        ));
    }

    let extra_tool_ids =
        configured_tool_ids.difference(&benchmark_tool_ids).cloned().collect::<Vec<_>>();
    if !extra_tool_ids.is_empty() {
        return Err(anyhow!(
            "tool execution mode config declares {} tool ids outside the governed benchmark scope: {}",
            extra_tool_ids.len(),
            extra_tool_ids.join(", "),
        ));
    }

    for (tool_id, assignment) in &assignments {
        let benchmark_entry = benchmark_tool_index
            .get(tool_id)
            .expect("benchmark tool coverage must exist after set validation");
        for domain in &benchmark_entry.domains {
            let probe = load_runtime_probe(repo_root, domain, tool_id)?;
            if probe.install_kind() != assignment.expected_install_kind {
                return Err(anyhow!(
                    "tool `{}` in domain `{}` declares install_kind `{}` but execution mode config expects `{}`",
                    tool_id,
                    domain,
                    probe.install_kind(),
                    assignment.expected_install_kind
                ));
            }
            for field in &assignment.required_runtime_fields {
                if !probe.has_required_field(field) {
                    return Err(anyhow!(
                        "tool `{}` in domain `{}` is missing required runtime field `{}` for execution mode `{}`",
                        tool_id,
                        domain,
                        field,
                        assignment.execution_mode
                    ));
                }
            }
        }
    }

    let mut mode_counts = BTreeMap::<String, usize>::new();
    let mut rows = Vec::with_capacity(benchmark_tool_index.len());
    for (tool_id, benchmark_entry) in benchmark_tool_index {
        let assignment =
            assignments.get(&tool_id).expect("tool execution mode assignment must exist");
        *mode_counts.entry(assignment.execution_mode.clone()).or_default() += 1;
        rows.push(ToolExecutionModeRow {
            tool_id,
            execution_mode: assignment.execution_mode.clone(),
            expected_install_kind: assignment.expected_install_kind.clone(),
            domains: benchmark_entry.domains,
            stage_ids: benchmark_entry.stage_ids,
            required_runtime_fields: assignment.required_runtime_fields.clone(),
            summary: assignment.summary.clone(),
        });
    }
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));

    let multidomain_tool_count = rows.iter().filter(|row| row.domains.len() > 1).count();

    Ok(ToolExecutionModesValidationReport {
        schema_version: TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, config_path),
        classification_scope: config.classification_scope,
        mode_count: config.modes.len(),
        tool_count: rows.len(),
        multidomain_tool_count,
        mode_counts,
        valid: true,
        rows,
    })
}

fn load_tool_execution_modes_config(config_path: &Path) -> Result<ToolExecutionModesConfig> {
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

fn validate_tool_execution_modes_contract(config: &ToolExecutionModesConfig) -> Result<()> {
    if config.schema_version != LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION {
        return Err(anyhow!("unsupported tool execution mode schema `{}`", config.schema_version));
    }
    if config.classification_scope != TOOL_EXECUTION_CLASSIFICATION_SCOPE {
        return Err(anyhow!(
            "unsupported tool execution classification scope `{}`",
            config.classification_scope
        ));
    }
    if config.modes.is_empty() {
        return Err(anyhow!(
            "tool execution mode config must declare at least one `[[modes]]` entry"
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ModeAssignment {
    execution_mode: String,
    expected_install_kind: String,
    required_runtime_fields: Vec<String>,
    summary: String,
}

fn build_mode_assignment_index(
    config: &ToolExecutionModesConfig,
) -> Result<BTreeMap<String, ModeAssignment>> {
    let mut seen_modes = BTreeSet::<String>::new();
    let mut assignments = BTreeMap::<String, ModeAssignment>::new();
    for mode in &config.modes {
        validate_mode_definition(mode)?;
        if !seen_modes.insert(mode.execution_mode.clone()) {
            return Err(anyhow!(
                "tool execution mode config repeats execution_mode `{}`",
                mode.execution_mode
            ));
        }
        for tool_id in &mode.tool_ids {
            if let Some(previous) = assignments.get(tool_id) {
                return Err(anyhow!(
                    "tool `{}` is assigned to both execution mode `{}` and `{}`",
                    tool_id,
                    previous.execution_mode,
                    mode.execution_mode
                ));
            }
            assignments.insert(
                tool_id.clone(),
                ModeAssignment {
                    execution_mode: mode.execution_mode.clone(),
                    expected_install_kind: mode.expected_install_kind.clone(),
                    required_runtime_fields: mode.required_runtime_fields.clone(),
                    summary: mode.summary.clone(),
                },
            );
        }
    }
    Ok(assignments)
}

fn validate_mode_definition(mode: &ToolExecutionModeDefinition) -> Result<()> {
    if !matches!(
        mode.execution_mode.as_str(),
        "internal" | "binary" | "containerized" | "r" | "java" | "python" | "mixed"
    ) {
        return Err(anyhow!(
            "tool execution mode `{}` is not one of the governed allowed values",
            mode.execution_mode
        ));
    }
    if mode.expected_install_kind.trim().is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare a non-empty `expected_install_kind`",
            mode.execution_mode
        ));
    }
    if mode.summary.trim().is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare a non-empty `summary`",
            mode.execution_mode
        ));
    }
    if mode.required_runtime_fields.is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare at least one required runtime field",
            mode.execution_mode
        ));
    }
    let mut sorted_fields = mode.required_runtime_fields.clone();
    sorted_fields.sort();
    if sorted_fields != mode.required_runtime_fields {
        return Err(anyhow!(
            "execution mode `{}` must keep `required_runtime_fields` sorted lexically",
            mode.execution_mode
        ));
    }
    let unique_fields = mode.required_runtime_fields.iter().cloned().collect::<BTreeSet<_>>();
    if unique_fields.len() != mode.required_runtime_fields.len() {
        return Err(anyhow!(
            "execution mode `{}` repeats one or more required runtime fields",
            mode.execution_mode
        ));
    }
    for field in &mode.required_runtime_fields {
        if !matches!(
            field.as_str(),
            "install_kind" | "container.image" | "help_cmd" | "version_cmd" | "command_template"
        ) {
            return Err(anyhow!(
                "execution mode `{}` declares unsupported runtime field `{}`",
                mode.execution_mode,
                field
            ));
        }
    }
    if mode.tool_ids.is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare at least one tool id",
            mode.execution_mode
        ));
    }
    let mut sorted_tool_ids = mode.tool_ids.clone();
    sorted_tool_ids.sort();
    if sorted_tool_ids != mode.tool_ids {
        return Err(anyhow!(
            "execution mode `{}` must keep `tool_ids` sorted lexically",
            mode.execution_mode
        ));
    }
    let unique_tool_ids = mode.tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    if unique_tool_ids.len() != mode.tool_ids.len() {
        return Err(anyhow!(
            "execution mode `{}` repeats one or more tool ids",
            mode.execution_mode
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct RuntimeProbe {
    #[serde(default)]
    install_kind: Option<String>,
    #[serde(default)]
    container: Option<RuntimeProbeContainer>,
    #[serde(default)]
    help_cmd: Option<String>,
    #[serde(default)]
    version_cmd: Option<String>,
    #[serde(default)]
    command_template: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct RuntimeProbeContainer {
    image: String,
}

impl RuntimeProbe {
    fn install_kind(&self) -> &str {
        self.install_kind.as_deref().unwrap_or("container")
    }

    fn has_required_field(&self, field: &str) -> bool {
        match field {
            "install_kind" => {
                self.install_kind.as_deref().map(str::trim).is_some_and(|value| !value.is_empty())
                    || self.container.is_some()
            }
            "container.image" => self
                .container
                .as_ref()
                .map(|container| container.image.trim())
                .is_some_and(|value| !value.is_empty()),
            "help_cmd" => {
                self.help_cmd.as_deref().map(str::trim).is_some_and(|value| !value.is_empty())
            }
            "version_cmd" => {
                self.version_cmd.as_deref().map(str::trim).is_some_and(|value| !value.is_empty())
            }
            "command_template" => !self.command_template.is_empty(),
            _ => false,
        }
    }
}

fn load_runtime_probe(repo_root: &Path, domain: &str, tool_id: &str) -> Result<RuntimeProbe> {
    let mut probe_domains = vec![domain.to_string()];
    for fallback in ["fastq", "bam", "vcf"] {
        if !probe_domains.iter().any(|candidate| candidate == fallback) {
            probe_domains.push(fallback.to_string());
        }
    }

    let mut attempted_paths = Vec::new();
    for probe_domain in probe_domains {
        let yaml_path = repo_root
            .join("domain")
            .join(&probe_domain)
            .join("tools")
            .join(format!("{tool_id}.yaml"));
        attempted_paths.push(yaml_path.display().to_string());
        if !yaml_path.is_file() {
            continue;
        }
        let raw = fs::read_to_string(&yaml_path)
            .with_context(|| format!("read {}", yaml_path.display()))?;
        let probe = bijux_dna_infra::formats::parse_yaml(&raw)
            .with_context(|| format!("parse {}", yaml_path.display()))?;
        return Ok(probe);
    }

    Err(anyhow!(
        "no governed runtime probe found for tool `{}`; looked in {}",
        tool_id,
        attempted_paths.join(", ")
    ))
}

#[derive(Debug, Clone)]
struct BenchmarkToolEntry {
    domains: Vec<String>,
    stage_ids: Vec<String>,
}

fn build_benchmark_tool_index(
    fastq_rows: &[ToolServingMapRow],
    bam_rows: &[ToolServingMapRow],
) -> BTreeMap<String, BenchmarkToolEntry> {
    let mut domains_by_tool = BTreeMap::<String, BTreeSet<String>>::new();
    let mut stages_by_tool = BTreeMap::<String, BTreeSet<String>>::new();

    for row in fastq_rows {
        domains_by_tool.entry(row.tool_id.clone()).or_default().insert("fastq".to_string());
        stages_by_tool.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }
    for row in bam_rows {
        domains_by_tool.entry(row.tool_id.clone()).or_default().insert("bam".to_string());
        stages_by_tool.entry(row.tool_id.clone()).or_default().insert(row.stage_id.clone());
    }

    domains_by_tool
        .into_iter()
        .map(|(tool_id, domains)| {
            let stage_ids =
                stages_by_tool.remove(&tool_id).unwrap_or_default().into_iter().collect::<Vec<_>>();
            (
                tool_id,
                BenchmarkToolEntry { domains: domains.into_iter().collect::<Vec<_>>(), stage_ids },
            )
        })
        .collect()
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        validate_tool_execution_modes_path, DEFAULT_TOOL_EXECUTION_MODES_PATH,
        TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn tool_execution_mode_config_covers_governed_benchmark_tools() {
        let root = repo_root();
        let report = validate_tool_execution_modes_path(
            &root,
            &root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
        )
        .expect("validate tool execution modes");

        assert_eq!(report.schema_version, TOOL_EXECUTION_MODES_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.classification_scope, "primary_operator_runtime");
        assert_eq!(report.mode_count, 6);
        assert_eq!(report.tool_count, 66);
        assert!(report.valid, "tool execution mode config must validate cleanly");

        let bijux_dna =
            report.rows.iter().find(|row| row.tool_id == "bijux_dna").expect("bijux_dna row");
        assert_eq!(bijux_dna.execution_mode, "internal");
        assert_eq!(bijux_dna.expected_install_kind, "workspace_binary");

        let cutadapt =
            report.rows.iter().find(|row| row.tool_id == "cutadapt").expect("cutadapt row");
        assert_eq!(cutadapt.execution_mode, "python");

        let gatk = report.rows.iter().find(|row| row.tool_id == "gatk").expect("gatk row");
        assert_eq!(gatk.execution_mode, "java");
    }

    #[test]
    fn tool_execution_mode_rows_preserve_governed_cross_domain_scope() {
        let root = repo_root();
        let report = validate_tool_execution_modes_path(
            &root,
            &root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
        )
        .expect("validate tool execution modes");

        let bowtie2 = report.rows.iter().find(|row| row.tool_id == "bowtie2").expect("bowtie2 row");
        assert_eq!(bowtie2.execution_mode, "containerized");
        assert_eq!(bowtie2.domains, vec!["bam".to_string(), "fastq".to_string()]);
        assert!(
            bowtie2.stage_ids.contains(&"bam.align".to_string())
                && bowtie2.stage_ids.contains(&"fastq.deplete_host".to_string()),
            "cross-domain execution-mode rows must preserve their governed benchmark stage scope"
        );

        let trim_galore =
            report.rows.iter().find(|row| row.tool_id == "trim_galore").expect("trim_galore row");
        assert_eq!(trim_galore.execution_mode, "mixed");
    }
}
