use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_infra::{ensure_dir, write_string};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub domain_dir: PathBuf,
    pub configs_dir: PathBuf,
    pub scope: String,
}

#[derive(Debug, Clone)]
pub struct ValidateOptions {
    pub domain_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
struct DomainTool {
    tool_id: String,
    stage_ids: Vec<String>,
    status: String,
    scope: String,
    default_version: String,
    upstream: String,
    versioning_strategy: String,
    #[serde(default)]
    pin_strategy: String,
    license: String,
    citation: String,
    version_cmd: String,
    help_cmd: String,
    expected_artifacts: Vec<String>,
    metrics_schema_id: String,
    #[serde(default)]
    metrics_schema: String,
    #[serde(default)]
    comparability_notes: String,
}

#[derive(Debug, Deserialize, Default)]
struct DomainToolLoose {
    #[serde(default)]
    tool_id: String,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    status: String,
    #[serde(default)]
    scope: String,
    #[serde(default)]
    default_version: String,
    #[serde(default)]
    upstream: String,
    #[serde(default)]
    pin_strategy: String,
    #[serde(default)]
    license: String,
    #[serde(default)]
    citation: String,
    #[serde(default)]
    version_cmd: String,
    #[serde(default)]
    help_cmd: String,
    #[serde(default)]
    expected_artifacts: Vec<String>,
    #[serde(default)]
    metrics_schema_id: String,
    #[serde(default)]
    comparability_notes: String,
}

#[derive(Debug, Deserialize, Default)]
struct StagePort {
    name: String,
    data_type: String,
    cardinality: String,
}

#[derive(Debug, Deserialize, Default)]
struct StageMetric {
    name: String,
}

#[derive(Debug, Deserialize, Default)]
struct DomainStage {
    stage_id: String,
    status: String,
    scope: String,
    domain: String,
    #[serde(default)]
    inputs: Vec<StagePort>,
    #[serde(default)]
    outputs: Vec<StagePort>,
    #[serde(default)]
    required_inputs: Vec<String>,
    #[serde(default)]
    required_outputs: Vec<String>,
    #[serde(default)]
    metrics: Vec<StageMetric>,
    #[serde(default)]
    compatible_tools: Vec<String>,
    #[serde(default)]
    assumptions: Vec<String>,
    #[serde(default)]
    invariants: Vec<String>,
    #[serde(default)]
    allowed_missingness: Vec<String>,
    #[serde(default)]
    planned_out_of_scope: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct DomainIndex {
    domain: String,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    tool_ids: Vec<String>,
    #[serde(default)]
    stage_tool_compatibility: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    active_defaults: BTreeMap<String, String>,
    #[serde(default)]
    active_default_rationale: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct DomainArtifactVocabulary {
    #[serde(default)]
    domain: String,
    #[serde(default)]
    artifact_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
struct DomainMetricVocabulary {
    #[serde(default)]
    domain: String,
    #[serde(default)]
    metric_ids: Vec<String>,
}

#[derive(Debug)]
struct ToolRow {
    id: String,
    domain: String,
    stage_ids: Vec<String>,
    default_version: String,
    upstream: String,
    pin_strategy: String,
    version_cmd: String,
    help_cmd: String,
    expected_artifacts: Vec<String>,
    metrics_schema: String,
    status: String,
    comparability_notes: String,
}

type ToolMap = BTreeMap<String, ToolRow>;
type StageToolMap = BTreeMap<String, BTreeSet<String>>;
type StagePlannedMap = BTreeMap<String, Vec<String>>;
type StageDefaultMap = BTreeMap<String, String>;
type StageStatusMap = BTreeMap<String, String>;
type StageOutputKindsMap = BTreeMap<String, Vec<String>>;
type StageDefaultRationaleMap = BTreeMap<String, String>;

fn ensure_status(status: &str, path: &Path) -> Result<()> {
    match status {
        "supported" | "planned" | "out_of_scope" => Ok(()),
        _ => Err(anyhow!(
            "{} invalid status `{status}` (expected supported|planned|out_of_scope)",
            path.display()
        )),
    }
}

fn scope_active(entry_scope: &str, active_scope: &str) -> bool {
    entry_scope == active_scope
}

fn read_yaml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

fn toml_array(values: &[String]) -> String {
    let joined = values
        .iter()
        .map(|v| format!("\"{v}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined}]")
}

fn find_git_dir(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        let dot_git = dir.join(".git");
        if dot_git.is_dir() {
            return Some(dot_git);
        }
        if dot_git.is_file() {
            let raw = std::fs::read_to_string(&dot_git).ok()?;
            let line = raw.trim();
            if let Some(path) = line.strip_prefix("gitdir:") {
                let p = path.trim();
                let git_dir = if Path::new(p).is_absolute() {
                    PathBuf::from(p)
                } else {
                    dir.join(p)
                };
                return Some(git_dir);
            }
        }
        current = dir.parent();
    }
    None
}

fn git_head_commit(start: &Path) -> Option<String> {
    let git_dir = find_git_dir(start)?;
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let head = head.trim();
    if let Some(reference) = head.strip_prefix("ref:") {
        let ref_path = git_dir.join(reference.trim());
        return std::fs::read_to_string(ref_path)
            .ok()
            .map(|s| s.trim().to_string());
    }
    Some(head.to_string())
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in
        std::fs::read_dir(dir).with_context(|| format!("read directory {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn domain_content_hash(domain_dir: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_files(domain_dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for file in files {
        let rel = file
            .strip_prefix(domain_dir)
            .unwrap_or(&file)
            .to_string_lossy()
            .into_owned();
        hasher.update(rel.as_bytes());
        hasher.update([0]);
        let file_hash = bijux_dna_infra::hash_file_sha256(&file)
            .with_context(|| format!("hash {}", file.display()))?;
        hasher.update(file_hash.as_bytes());
        hasher.update([0]);
    }
    let hex = format!("{:x}", hasher.finalize());
    Ok(hex.chars().take(40).collect())
}

fn generated_header(source: &str, source_commit: &str) -> String {
    format!(
        "# GENERATED - DO NOT EDIT - source: {source}\n# source_commit: {source_commit}\n# domain_schema_version: bijux.domain.v1\n# Regenerate with: cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs\n\n"
    )
}

fn validate_tool_output_subset(
    tool_raw: &str,
    stage_raw: &str,
    tool_path: &Path,
    stage_id: &str,
) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct NamedOutput {
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct ToolOutputsDoc {
        #[serde(default)]
        outputs: Vec<NamedOutput>,
    }
    #[derive(serde::Deserialize)]
    struct StageOutputsDoc {
        #[serde(default)]
        outputs: Vec<NamedOutput>,
    }

    let parsed_tool: ToolOutputsDoc = bijux_dna_infra::formats::parse_yaml(tool_raw)
        .with_context(|| format!("parse {}", tool_path.display()))?;
    if parsed_tool.outputs.is_empty() {
        return Ok(());
    }
    let output_names = parsed_tool
        .outputs
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<BTreeSet<_>>();
    if output_names.is_empty() {
        bail!(
            "{} outputs section must include named outputs",
            tool_path.display()
        );
    }
    let stage_yaml: StageOutputsDoc = bijux_dna_infra::formats::parse_yaml(stage_raw)
        .with_context(|| format!("parse stage {stage_id}"))?;
    let stage_outputs = stage_yaml
        .outputs
        .iter()
        .map(|entry| entry.name.as_str())
        .collect::<BTreeSet<_>>();
    for output in &output_names {
        if !stage_outputs.contains(output) {
            bail!(
                "{} output `{}` is not declared by stage `{}` outputs",
                tool_path.display(),
                output,
                stage_id
            );
        }
    }
    Ok(())
}

fn has_placeholder_token(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    lower.contains("todo") || lower.contains("tbd") || lower.contains("placeholder")
}

fn is_unspecified(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unspecified")
}

fn load_domain_tools(
    domain_dir: &Path,
    domain: &str,
    index: &DomainIndex,
    active_scope: &str,
    tools: &mut ToolMap,
    stage_to_tools: &mut StageToolMap,
) -> Result<()> {
    let tools_dir = domain_dir.join(domain).join("tools");
    for tool_id_ref in &index.tool_ids {
        let tool_id_normalized = tool_id_ref.replace('-', "_");
        let path_candidates = [
            tools_dir.join(format!("{tool_id_ref}.yaml")),
            tools_dir.join(format!("{tool_id_normalized}.yaml")),
            domain_dir
                .join(if domain == "fastq" { "bam" } else { "fastq" })
                .join("tools")
                .join(format!("{tool_id_ref}.yaml")),
            domain_dir
                .join(if domain == "fastq" { "bam" } else { "fastq" })
                .join("tools")
                .join(format!("{tool_id_normalized}.yaml")),
        ];
        let Some(path) = path_candidates.into_iter().find(|p| p.exists()) else {
            return Err(anyhow!(
                "index references missing tool file for {} in {}",
                tool_id_ref,
                tools_dir.display()
            ));
        };
        let tool: DomainTool = read_yaml(&path)?;
        if tool.tool_id.trim().is_empty() {
            return Err(anyhow!("{} missing tool_id", path.display()));
        }
        if tool.scope.trim().is_empty() {
            return Err(anyhow!("{} missing scope", path.display()));
        }
        ensure_status(&tool.status, &path)?;
        if !scope_active(&tool.scope, active_scope) || tool.status == "out_of_scope" {
            continue;
        }
        if tool.stage_ids.is_empty() {
            return Err(anyhow!("{} missing stage_ids", path.display()));
        }
        if tool.upstream.trim().is_empty()
            || tool.default_version.trim().is_empty()
            || tool.versioning_strategy.trim().is_empty()
            || tool.license.trim().is_empty()
            || tool.citation.trim().is_empty()
            || tool.version_cmd.trim().is_empty()
            || tool.help_cmd.trim().is_empty()
            || tool.expected_artifacts.is_empty()
            || (tool.metrics_schema_id.trim().is_empty() && tool.metrics_schema.trim().is_empty())
        {
            return Err(anyhow!("{} missing required tool fields", path.display()));
        }
        for stage in &tool.stage_ids {
            stage_to_tools
                .entry(stage.clone())
                .or_default()
                .insert(tool.tool_id.clone());
        }
        let tool_id = tool.tool_id.clone();
        if tools.contains_key(&tool_id) {
            continue;
        }
        let resolved_domain = path
            .parent()
            .and_then(Path::parent)
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            .unwrap_or(domain)
            .to_string();
        tools.insert(
            tool_id.clone(),
            ToolRow {
                id: tool_id.clone(),
                domain: resolved_domain,
                stage_ids: tool.stage_ids,
                default_version: tool.default_version,
                upstream: tool.upstream,
                pin_strategy: if tool.pin_strategy.is_empty() {
                    tool.versioning_strategy
                } else {
                    tool.pin_strategy
                },
                version_cmd: tool.version_cmd,
                help_cmd: tool.help_cmd,
                expected_artifacts: tool.expected_artifacts,
                metrics_schema: if tool.metrics_schema_id.is_empty() {
                    tool.metrics_schema
                } else {
                    tool.metrics_schema_id
                },
                status: tool.status,
                comparability_notes: tool.comparability_notes,
            },
        );
    }
    Ok(())
}

fn load_domain_stages(
    domain_dir: &Path,
    domain: &str,
    index: &DomainIndex,
    active_scope: &str,
    stage_to_tools: &mut StageToolMap,
    stage_planned: &mut StagePlannedMap,
    stage_statuses: &mut StageStatusMap,
    stage_output_kinds: &mut StageOutputKindsMap,
) -> Result<()> {
    let stages_dir = domain_dir.join(domain).join("stages");
    for stage_id in &index.stage_ids {
        let stage_suffix = stage_id
            .as_str()
            .split_once('.')
            .map_or(stage_id.as_str(), |(_, suffix)| suffix);
        let stage_file = stage_suffix.replace('.', "_");
        let path = stages_dir.join(format!("{stage_file}.yaml"));
        if !path.exists() {
            return Err(anyhow!(
                "index references missing stage file for {} at {}",
                stage_id,
                path.display()
            ));
        }
        let stage: DomainStage = read_yaml(&path)?;
        if stage.stage_id.trim().is_empty() {
            return Err(anyhow!("{} missing stage_id", path.display()));
        }
        if stage.scope.trim().is_empty() {
            return Err(anyhow!("{} missing scope", path.display()));
        }
        ensure_status(&stage.status, &path)?;
        if !scope_active(&stage.scope, active_scope) || stage.status == "out_of_scope" {
            continue;
        }
        stage_to_tools.entry(stage.stage_id.clone()).or_default();
        let mut kinds = stage
            .outputs
            .iter()
            .map(|port| port.data_type.clone())
            .collect::<Vec<_>>();
        kinds.sort();
        kinds.dedup();
        stage_output_kinds.insert(stage.stage_id.clone(), kinds);
        stage_statuses.insert(stage.stage_id.clone(), stage.status.clone());
        stage_planned.insert(stage.stage_id, stage.planned_out_of_scope);
    }
    Ok(())
}

fn collect_domain_data(
    domain_dir: &Path,
    active_scope: &str,
) -> Result<(
    ToolMap,
    StageToolMap,
    StagePlannedMap,
    StageDefaultMap,
    StageDefaultRationaleMap,
    StageStatusMap,
    StageOutputKindsMap,
)> {
    let mut tools: ToolMap = BTreeMap::new();
    let mut stage_to_tools: StageToolMap = BTreeMap::new();
    let mut stage_planned: StagePlannedMap = BTreeMap::new();
    let mut stage_defaults: StageDefaultMap = BTreeMap::new();
    let mut stage_default_rationale: StageDefaultRationaleMap = BTreeMap::new();
    let mut stage_statuses: StageStatusMap = BTreeMap::new();
    let mut stage_output_kinds: StageOutputKindsMap = BTreeMap::new();
    for domain in ["fastq", "bam"] {
        let index_path = domain_dir.join(domain).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        if index.domain != domain {
            return Err(anyhow!(
                "{} has domain {} but expected {}",
                index_path.display(),
                index.domain,
                domain
            ));
        }
        load_domain_tools(
            domain_dir,
            domain,
            &index,
            active_scope,
            &mut tools,
            &mut stage_to_tools,
        )?;
        load_domain_stages(
            domain_dir,
            domain,
            &index,
            active_scope,
            &mut stage_to_tools,
            &mut stage_planned,
            &mut stage_statuses,
            &mut stage_output_kinds,
        )?;
        for (stage_id, tool_ids) in &index.stage_tool_compatibility {
            if !stage_to_tools.contains_key(stage_id) {
                continue;
            }
            let active_tools = stage_to_tools.entry(stage_id.clone()).or_default();
            active_tools.clear();
            for tool_id in tool_ids {
                if tools.contains_key(tool_id) {
                    active_tools.insert(tool_id.clone());
                }
            }
        }
        for (stage_id, default_tool) in &index.active_defaults {
            if !stage_to_tools.contains_key(stage_id) {
                continue;
            }
            if !stage_to_tools
                .get(stage_id)
                .is_some_and(|set| set.contains(default_tool))
            {
                return Err(anyhow!(
                    "index active default {default_tool} for {stage_id} is not compatible"
                ));
            }
            let rationale = index
                .active_default_rationale
                .get(stage_id)
                .cloned()
                .unwrap_or_default();
            if is_unspecified(&rationale) {
                return Err(anyhow!(
                    "index active_default_rationale for {stage_id} must be non-empty and not unspecified"
                ));
            }
            stage_defaults.insert(stage_id.clone(), default_tool.clone());
            stage_default_rationale.insert(stage_id.clone(), rationale);
        }
    }
    for tool in tools.values() {
        for stage in &tool.stage_ids {
            if !stage_to_tools.contains_key(stage) {
                return Err(anyhow!(
                    "tool {} references unknown stage {}",
                    tool.id,
                    stage
                ));
            }
        }
    }
    Ok((
        tools,
        stage_to_tools,
        stage_planned,
        stage_defaults,
        stage_default_rationale,
        stage_statuses,
        stage_output_kinds,
    ))
}

fn build_tool_registry_toml(
    tools: &ToolMap,
    stage_to_tools: &StageToolMap,
    stage_planned: &StagePlannedMap,
    stage_defaults: &StageDefaultMap,
    stage_default_rationale: &StageDefaultRationaleMap,
    source_commit: &str,
) -> String {
    let mut registry_toml = generated_header("domain/**", source_commit);
    for tool in tools.values() {
        let dockerfile = format!("containers/docker/arm64/Dockerfile.{}", tool.id);
        let apptainer_def = format!("containers/apptainer/{}.def", tool.id);
        let docker_exists = Path::new(&dockerfile).exists();
        let apptainer_exists = Path::new(&apptainer_def).exists();
        let mut runtimes = Vec::new();
        if docker_exists {
            runtimes.push("docker".to_string());
        }
        if apptainer_exists {
            runtimes.push("apptainer".to_string());
        }
        if runtimes.is_empty() {
            runtimes = vec!["docker".to_string(), "apptainer".to_string()];
        }
        let is_planned = tool.status == "planned" || tool.default_version == "planned";
        let _ = writeln!(registry_toml, "[[tools]]");
        let _ = writeln!(registry_toml, "id = \"{}\"", tool.id);
        let _ = writeln!(registry_toml, "tool_id = \"{}\"", tool.id);
        let _ = writeln!(registry_toml, "domain = \"{}\"", tool.domain);
        let _ = writeln!(registry_toml, "status = \"{}\"", tool.status);
        let _ = writeln!(registry_toml, "stage_ids = {}", toml_array(&tool.stage_ids));
        let _ = writeln!(registry_toml, "version = \"{}\"", tool.default_version);
        let _ = writeln!(
            registry_toml,
            "default_version = \"{}\"",
            tool.default_version
        );
        let _ = writeln!(registry_toml, "upstream = \"{}\"", tool.upstream);
        registry_toml.push_str("pinned_commit = \"domain-managed\"\n");
        let _ = writeln!(registry_toml, "pin_strategy = \"{}\"", tool.pin_strategy);
        let _ = writeln!(registry_toml, "runtimes = {}", toml_array(&runtimes));
        let _ = writeln!(
            registry_toml,
            "container = {}",
            if is_planned { "false" } else { "true" }
        );
        let _ = writeln!(registry_toml, "version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(registry_toml, "help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(
            registry_toml,
            "smoke_version_cmd = \"{}\"",
            tool.version_cmd
        );
        let _ = writeln!(registry_toml, "smoke_help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(registry_toml, "expected_bin = \"{}\"", tool.id);
        let _ = writeln!(
            registry_toml,
            "expected_artifacts = {}",
            toml_array(&tool.expected_artifacts)
        );
        let _ = writeln!(
            registry_toml,
            "metrics_schema = \"{}\"",
            tool.metrics_schema
        );
        let _ = writeln!(
            registry_toml,
            "comparability_notes = \"{}\"",
            tool.comparability_notes.replace('"', "'")
        );
        let _ = writeln!(registry_toml, "dockerfile = \"{dockerfile}\"");
        let _ = writeln!(registry_toml, "apptainer_def = \"{apptainer_def}\"");
        registry_toml.push_str("require_labels = true\n\n");
    }

    for (stage_id, tools_set) in stage_to_tools {
        let mut all = tools_set.iter().cloned().collect::<Vec<_>>();
        all.sort();
        let mut primary = stage_defaults
            .get(stage_id)
            .cloned()
            .into_iter()
            .collect::<Vec<_>>();
        if primary.is_empty() {
            primary = all.first().cloned().into_iter().collect::<Vec<_>>();
        }
        if primary.is_empty() {
            let stage_domain = stage_id.split('.').next().unwrap_or_default();
            primary.push(if stage_domain == "bam" {
                "samtools".to_string()
            } else {
                "fastp".to_string()
            });
        }
        let optional = all.iter().skip(1).cloned().collect::<Vec<_>>();
        let reporting = if stage_id.contains("qc") {
            vec!["multiqc".to_string()]
        } else {
            Vec::new()
        };
        let _ = writeln!(registry_toml, "[[stages]]");
        let _ = writeln!(registry_toml, "id = \"{stage_id}\"");
        let _ = writeln!(registry_toml, "primary_tools = {}", toml_array(&primary));
        let _ = writeln!(
            registry_toml,
            "optional_alternatives = {}",
            toml_array(&optional)
        );
        registry_toml.push_str("validation_tools = []\n");
        let _ = writeln!(
            registry_toml,
            "reporting_tools = {}",
            toml_array(&reporting)
        );
        let _ = writeln!(
            registry_toml,
            "planned_out_of_scope = {}",
            toml_array(stage_planned.get(stage_id).map_or(&[], Vec::as_slice))
        );
        let rationale = stage_default_rationale
            .get(stage_id)
            .map_or("", std::string::String::as_str)
            .replace('"', "'");
        let _ = writeln!(registry_toml, "default_rationale = \"{rationale}\"");
        registry_toml.push_str("requires_validation = false\n");
        let _ = writeln!(
            registry_toml,
            "requires_reporting = {}",
            if reporting.is_empty() {
                "false"
            } else {
                "true"
            }
        );
        registry_toml.push('\n');
    }
    registry_toml
}

fn build_images_toml(tools: &ToolMap, source_commit: &str) -> String {
    let mut images_toml = generated_header("domain/**", source_commit);
    for tool in tools.values() {
        let _ = writeln!(images_toml, "[{}]", tool.id);
        let _ = writeln!(images_toml, "version = \"{}\"\n", tool.default_version);
    }
    images_toml
}

fn build_stages_toml(
    stage_to_tools: &StageToolMap,
    stage_statuses: &StageStatusMap,
    stage_output_kinds: &StageOutputKindsMap,
    source_commit: &str,
) -> String {
    let mut stages_toml = generated_header("domain/**", source_commit);
    for (stage_id, tools_set) in stage_to_tools {
        let _ = writeln!(stages_toml, "[[stages]]");
        let _ = writeln!(stages_toml, "id = \"{stage_id}\"");
        let status = stage_statuses
            .get(stage_id)
            .map_or("supported", std::string::String::as_str);
        let _ = writeln!(stages_toml, "status = \"{status}\"");
        let mut v = tools_set.iter().cloned().collect::<Vec<_>>();
        v.sort();
        let output_kinds = stage_output_kinds
            .get(stage_id)
            .cloned()
            .unwrap_or_default();
        let _ = writeln!(stages_toml, "output_kinds = {}", toml_array(&output_kinds));
        let _ = writeln!(stages_toml, "tools = {}\n", toml_array(&v));
    }
    stages_toml
}

/// Compile generated config views from authored domain sources.
///
/// # Errors
///
/// Returns an error when domain inputs are invalid, generated outputs cannot be
/// written, or scope invariants are violated.
pub fn compile_domain_configs(options: &CompileOptions) -> Result<()> {
    let (
        tools,
        stage_to_tools,
        stage_planned,
        stage_defaults,
        stage_default_rationale,
        stage_statuses,
        stage_output_kinds,
    ) = collect_domain_data(&options.domain_dir, &options.scope)?;
    if options.scope == "pre_hpc_pre_vcf" {
        if tools.keys().any(|tool_id| tool_id.starts_with("vcf.")) {
            bail!("pre_hpc_pre_vcf scope must not include VCF tools in generated configs");
        }
        if stage_to_tools
            .keys()
            .any(|stage_id| stage_id.starts_with("vcf."))
        {
            bail!("pre_hpc_pre_vcf scope must not include VCF stages in generated configs");
        }
    }
    ensure_dir(&options.configs_dir)
        .with_context(|| format!("create {}", options.configs_dir.display()))?;

    let source_commit = domain_content_hash(&options.domain_dir)
        .ok()
        .or_else(|| git_head_commit(&options.domain_dir))
        .unwrap_or_else(|| "unknown".to_string());

    let tool_registry_path = options.configs_dir.join("tool_registry.toml");
    let registry_toml = build_tool_registry_toml(
        &tools,
        &stage_to_tools,
        &stage_planned,
        &stage_defaults,
        &stage_default_rationale,
        &source_commit,
    );
    write_string(&tool_registry_path, &registry_toml)
        .with_context(|| format!("write {}", tool_registry_path.display()))?;

    let images_path = options.configs_dir.join("images.toml");
    let images_toml = build_images_toml(&tools, &source_commit);
    write_string(&images_path, &images_toml)
        .with_context(|| format!("write {}", images_path.display()))?;

    let stages_path = options.configs_dir.join("stages.toml");
    let stages_toml = build_stages_toml(
        &stage_to_tools,
        &stage_statuses,
        &stage_output_kinds,
        &source_commit,
    );
    write_string(&stages_path, &stages_toml)
        .with_context(|| format!("write {}", stages_path.display()))?;

    println!("generated: {}", tool_registry_path.display());
    println!("generated: {}", images_path.display());
    println!("generated: {}", stages_path.display());
    Ok(())
}

fn require_exists(path: &Path) -> Result<()> {
    if !path.exists() {
        bail!("missing required file: {}", path.display());
    }
    Ok(())
}

/// Validate authored domain files and cross-domain invariants.
///
/// # Errors
///
/// Returns an error when required files are missing, schemas/invariants are
/// violated, or domain catalogs are inconsistent.
#[allow(clippy::too_many_lines)]
pub fn validate_domain(options: &ValidateOptions) -> Result<()> {
    for rel in [
        "fastq/stages/_schema.yaml",
        "bam/stages/_schema.yaml",
        "fastq/tools/_schema.yaml",
        "bam/tools/_schema.yaml",
        "fastq/artifacts.yaml",
        "bam/artifacts.yaml",
        "fastq/metrics.yaml",
        "bam/metrics.yaml",
        "fastq/index.yaml",
        "bam/index.yaml",
    ] {
        require_exists(&options.domain_dir.join(rel))?;
    }

    let mut tool_ids = BTreeMap::<String, String>::new();
    let mut stage_ids = BTreeMap::<String, String>::new();
    let mut artifact_vocab = BTreeMap::<String, BTreeSet<String>>::new();
    let mut metric_vocab = BTreeMap::<String, BTreeSet<String>>::new();

    for dom in ["fastq", "bam"] {
        let artifacts_path = options.domain_dir.join(dom).join("artifacts.yaml");
        let metrics_path = options.domain_dir.join(dom).join("metrics.yaml");
        let artifacts: DomainArtifactVocabulary = read_yaml(&artifacts_path)?;
        let metrics: DomainMetricVocabulary = read_yaml(&metrics_path)?;
        if artifacts.domain != dom {
            bail!(
                "{} domain mismatch: expected {}, got {}",
                artifacts_path.display(),
                dom,
                artifacts.domain
            );
        }
        if metrics.domain != dom {
            bail!(
                "{} domain mismatch: expected {}, got {}",
                metrics_path.display(),
                dom,
                metrics.domain
            );
        }
        if artifacts.artifact_ids.is_empty() {
            bail!("{} missing artifact_ids", artifacts_path.display());
        }
        if metrics.metric_ids.is_empty() {
            bail!("{} missing metric_ids", metrics_path.display());
        }
        artifact_vocab.insert(dom.to_string(), artifacts.artifact_ids.into_iter().collect());
        metric_vocab.insert(dom.to_string(), metrics.metric_ids.into_iter().collect());
    }

    for dom in ["fastq", "bam", "vcf"] {
        let stage_glob = options.domain_dir.join(dom).join("stages");
        if stage_glob.exists() {
            for entry in std::fs::read_dir(&stage_glob)
                .with_context(|| format!("read {}", stage_glob.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                    continue;
                }
                let stage: DomainStage = read_yaml(&path)?;
                let stage_raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if stage.stage_id.is_empty() {
                    bail!("{} missing stage_id", path.display());
                }
                if dom != "vcf" {
                    let artifact_ids = artifact_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing artifact vocab for domain {dom}"))?;
                    let metric_ids = metric_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing metric vocab for domain {dom}"))?;
                    if stage.inputs.is_empty() {
                        bail!("{} missing inputs", path.display());
                    }
                    if stage.outputs.is_empty() {
                        bail!("{} missing outputs", path.display());
                    }
                    if stage.compatible_tools.is_empty() {
                        bail!("{} missing compatible_tools", path.display());
                    }
                    if stage.invariants.is_empty() {
                        bail!("{} missing invariants", path.display());
                    }
                    if stage.assumptions.is_empty() {
                        bail!("{} missing assumptions", path.display());
                    }
                    if stage.metrics.is_empty() {
                        bail!("{} missing metrics", path.display());
                    }
                    if stage.allowed_missingness.is_empty() && stage.status == "supported" {
                        bail!("{} missing allowed_missingness", path.display());
                    }
                    for output in &stage.outputs {
                        if !artifact_ids.contains(&output.name) {
                            bail!(
                                "{} stage output `{}` is outside {} artifact vocabulary",
                                path.display(),
                                output.name,
                                dom
                            );
                        }
                    }
                    for output in &stage.required_outputs {
                        if !artifact_ids.contains(output) {
                            bail!(
                                "{} required_output `{}` is outside {} artifact vocabulary",
                                path.display(),
                                output,
                                dom
                            );
                        }
                    }
                    for metric in &stage.metrics {
                        if !metric_ids.contains(&metric.name) {
                            bail!(
                                "{} metric `{}` is outside {} metric vocabulary",
                                path.display(),
                                metric.name,
                                dom
                            );
                        }
                    }
                }
                let input_names = stage
                    .inputs
                    .iter()
                    .map(|port| port.name.clone())
                    .collect::<BTreeSet<_>>();
                let output_names = stage
                    .outputs
                    .iter()
                    .map(|port| port.name.clone())
                    .collect::<BTreeSet<_>>();
                for port in &stage.inputs {
                    if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                        bail!("{} has input missing data_type/cardinality", path.display());
                    }
                }
                for port in &stage.outputs {
                    if port.data_type.trim().is_empty() || port.cardinality.trim().is_empty() {
                        bail!(
                            "{} has output missing data_type/cardinality",
                            path.display()
                        );
                    }
                }
                for required in &stage.required_inputs {
                    if !input_names.contains(required) {
                        bail!(
                            "{} required_inputs references missing input `{required}`",
                            path.display()
                        );
                    }
                }
                for required in &stage.required_outputs {
                    if !output_names.contains(required) {
                        bail!(
                            "{} required_outputs references missing output `{required}`",
                            path.display()
                        );
                    }
                }
                for metric in &stage.metrics {
                    if metric.name.trim().is_empty() {
                        bail!("{} has metric with empty name", path.display());
                    }
                }
                if has_placeholder_token(&stage_raw) {
                    bail!(
                        "{} contains placeholder token (todo/tbd/placeholder)",
                        path.display()
                    );
                }
                ensure_status(&stage.status, &path)?;
                if dom != "vcf" && stage.scope != "pre_hpc_pre_vcf" {
                    bail!("{} invalid stage scope {}", path.display(), stage.scope);
                }
                if dom != "vcf" && !stage.stage_id.starts_with(&format!("{}.", stage.domain)) {
                    bail!(
                        "{} stage_id {} must be namespaced by domain {}",
                        path.display(),
                        stage.stage_id,
                        stage.domain
                    );
                }
                if let Some(prev) =
                    stage_ids.insert(stage.stage_id.clone(), path.display().to_string())
                {
                    bail!(
                        "duplicate stage_id {} in {} and {}",
                        stage.stage_id,
                        prev,
                        path.display()
                    );
                }
            }
        }

        let tool_glob = options.domain_dir.join(dom).join("tools");
        if tool_glob.exists() {
            for entry in std::fs::read_dir(&tool_glob)
                .with_context(|| format!("read {}", tool_glob.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                    continue;
                }
                if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                    continue;
                }
                let tool: DomainToolLoose = read_yaml(&path)?;
                let tool_raw = std::fs::read_to_string(&path)
                    .with_context(|| format!("read {}", path.display()))?;
                if tool.tool_id.is_empty() {
                    bail!("{} missing tool_id", path.display());
                }
                if has_placeholder_token(&tool_raw) {
                    bail!(
                        "{} contains placeholder token (todo/tbd/placeholder)",
                        path.display()
                    );
                }
                ensure_status(&tool.status, &path)?;
                if dom != "vcf" && tool.scope != "pre_hpc_pre_vcf" {
                    bail!("{} invalid tool scope {}", path.display(), tool.scope);
                }
                if tool.default_version.trim() == "0.0.0" {
                    bail!("{} default_version=0.0.0 is forbidden", path.display());
                }
                if dom != "vcf"
                    && (tool.stage_ids.is_empty()
                        || tool.default_version.is_empty()
                        || tool.upstream.is_empty()
                        || tool.pin_strategy.is_empty()
                        || tool.license.is_empty()
                        || tool.citation.is_empty()
                        || tool.version_cmd.is_empty()
                        || tool.help_cmd.is_empty()
                        || tool.expected_artifacts.is_empty()
                        || tool.metrics_schema_id.is_empty()
                        || tool.comparability_notes.is_empty())
                {
                    bail!("{} missing required tool fields", path.display());
                }
                if dom != "vcf" && tool.status == "supported" {
                    let artifact_ids = artifact_vocab
                        .get(dom)
                        .ok_or_else(|| anyhow!("missing artifact vocab for domain {dom}"))?;
                    for artifact in &tool.expected_artifacts {
                        if !artifact_ids.contains(artifact) {
                            bail!(
                                "{} expected_artifact `{}` is outside {} artifact vocabulary",
                                path.display(),
                                artifact,
                                dom
                            );
                        }
                    }
                    for stage_id in &tool.stage_ids {
                        let stage_domain = stage_id.split('.').next().unwrap_or(dom);
                        let stage_path =
                            options
                                .domain_dir
                                .join(stage_domain)
                                .join("stages")
                                .join(format!(
                                    "{}.yaml",
                                    stage_id
                                        .split_once('.')
                                        .map_or(stage_id.as_str(), |(_, suffix)| suffix)
                                        .replace('.', "_")
                                ));
                        if stage_path.exists() {
                            let stage_yaml_raw = std::fs::read_to_string(&stage_path)
                                .with_context(|| {
                                    format!(
                                        "read stage for output validation {}",
                                        stage_path.display()
                                    )
                                })?;
                            validate_tool_output_subset(
                                &tool_raw,
                                &stage_yaml_raw,
                                &path,
                                stage_id,
                            )?;
                        }
                    }
                    let dockerfile = options
                        .domain_dir
                        .parent()
                        .unwrap_or(&options.domain_dir)
                        .join("containers")
                        .join("docker")
                        .join("arm64")
                        .join(format!("Dockerfile.{}", tool.tool_id));
                    let apptainer = options
                        .domain_dir
                        .parent()
                        .unwrap_or(&options.domain_dir)
                        .join("containers")
                        .join("apptainer")
                        .join(format!("{}.def", tool.tool_id));
                    if !dockerfile.exists() && !apptainer.exists() {
                        bail!(
                            "{} supported tool {} missing container mapping ({} / {})",
                            path.display(),
                            tool.tool_id,
                            dockerfile.display(),
                            apptainer.display()
                        );
                    }
                }
                if let Some(prev) =
                    tool_ids.insert(tool.tool_id.clone(), path.display().to_string())
                {
                    bail!(
                        "duplicate tool_id {} in {} and {}",
                        tool.tool_id,
                        prev,
                        path.display()
                    );
                }
            }
        }
    }

    let fastq_canonical = bijux_dna_domain_fastq::stages::ids::STAGES
        .iter()
        .map(|id| id.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let bam_canonical = bijux_dna_domain_bam::stage_specs::BamStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    for stage_id in stage_ids.keys() {
        if stage_id.starts_with("fastq.") && !fastq_canonical.contains(stage_id) {
            bail!("domain stage_id {stage_id} is not declared in fastq stage catalog");
        }
        if stage_id.starts_with("bam.") && !bam_canonical.contains(stage_id) {
            bail!("domain stage_id {stage_id} is not declared in bam stage catalog");
        }
    }
    for stage_id in &fastq_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("fastq stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }
    for stage_id in &bam_canonical {
        if !stage_ids.contains_key(stage_id) {
            bail!("bam stage catalog contains {stage_id} but domain yaml is missing it");
        }
    }

    for dom in ["fastq", "bam"] {
        let index_path = options.domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        if index.domain != dom {
            bail!(
                "{} has domain {} but expected {}",
                index_path.display(),
                index.domain,
                dom
            );
        }
        if index.stage_ids.is_empty() || index.tool_ids.is_empty() {
            bail!("{} missing stage_ids/tool_ids", index_path.display());
        }
        for stage_id in &index.stage_ids {
            if !stage_ids.contains_key(stage_id) {
                bail!(
                    "{} references unknown stage {}",
                    index_path.display(),
                    stage_id
                );
            }
        }
        for tool_id in &index.tool_ids {
            if !tool_ids.contains_key(tool_id) {
                bail!(
                    "{} references unknown tool {}",
                    index_path.display(),
                    tool_id
                );
            }
        }
        // Enforce index as the single enumerator: every authored file must be listed in index.
        let stage_dir = options.domain_dir.join(dom).join("stages");
        for entry in
            std::fs::read_dir(&stage_dir).with_context(|| format!("read {}", stage_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage: DomainStage = read_yaml(&path)?;
            if !index.stage_ids.contains(&stage.stage_id) {
                bail!(
                    "{} stage {} exists in file system but is not listed in index.yaml",
                    path.display(),
                    stage.stage_id
                );
            }
        }
        let tool_dir = options.domain_dir.join(dom).join("tools");
        for entry in
            std::fs::read_dir(&tool_dir).with_context(|| format!("read {}", tool_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml") {
                continue;
            }
            if path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let tool: DomainToolLoose = read_yaml(&path)?;
            if !index.tool_ids.contains(&tool.tool_id) {
                bail!(
                    "{} tool {} exists in file system but is not listed in index.yaml",
                    path.display(),
                    tool.tool_id
                );
            }
        }
        for (stage_id, tools) in &index.stage_tool_compatibility {
            if !index.stage_ids.contains(stage_id) {
                bail!(
                    "{} matrix references unknown stage {}",
                    index_path.display(),
                    stage_id
                );
            }
            if tools.is_empty() {
                bail!(
                    "{} stage {} has empty compatibility list",
                    index_path.display(),
                    stage_id
                );
            }
            for tool in tools {
                if !index.tool_ids.contains(tool) {
                    bail!(
                        "{} stage {} references unknown tool {}",
                        index_path.display(),
                        stage_id,
                        tool
                    );
                }
                let fixture = options
                    .domain_dir
                    .join(dom)
                    .join("fixtures")
                    .join(stage_id)
                    .join(format!("{tool}.txt"));
                if !fixture.exists() {
                    bail!(
                        "{} stage {} tool {} missing truth fixture at {}",
                        index_path.display(),
                        stage_id,
                        tool,
                        fixture.display()
                    );
                }
            }
        }
        for (stage_id, default_tool) in &index.active_defaults {
            let compatible = index
                .stage_tool_compatibility
                .get(stage_id)
                .is_some_and(|tools| tools.contains(default_tool));
            if !compatible {
                bail!(
                    "{} active default {} for {} is not in compatibility matrix",
                    index_path.display(),
                    default_tool,
                    stage_id
                );
            }
            let rationale = index
                .active_default_rationale
                .get(stage_id)
                .map_or("", std::string::String::as_str);
            if is_unspecified(rationale) {
                bail!(
                    "{} missing non-empty active_default_rationale for {}",
                    index_path.display(),
                    stage_id
                );
            }
            let stage_suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", stage_suffix.replace('.', "_")));
            if stage_path.exists() {
                let stage: DomainStage = read_yaml(&stage_path)?;
                if stage.status != "supported" {
                    bail!(
                        "{} active default stage {} must be supported (found {})",
                        index_path.display(),
                        stage_id,
                        stage.status
                    );
                }
            }
        }
        // Validate that required stage inputs are satisfiable by prior stage outputs in index order.
        let mut available_inputs = if dom == "fastq" {
            BTreeSet::from([
                "reads".to_string(),
                "reads_r1".to_string(),
                "reads_r2".to_string(),
                "reference_fasta".to_string(),
            ])
        } else {
            BTreeSet::from(["bam".to_string(), "reference_fasta".to_string()])
        };
        for stage_id in &index.stage_ids {
            let suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", suffix.replace('.', "_")));
            if !stage_path.exists() {
                continue;
            }
            let stage: DomainStage = read_yaml(&stage_path)?;
            if stage.status != "supported" {
                continue;
            }
            for required in &stage.required_inputs {
                if !available_inputs.contains(required) {
                    bail!(
                        "{} required input `{}` for stage {} is not satisfiable by prior stage outputs",
                        stage_path.display(),
                        required,
                        stage_id
                    );
                }
            }
            for out in &stage.outputs {
                available_inputs.insert(out.name.clone());
            }
        }
    }

    println!("domain-validate: OK");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_tool_output_subset;
    use std::path::Path;

    #[test]
    fn tool_output_validation_rejects_unknown_output_name() {
        let tool = r#"
tool_id: fastp
outputs:
  - name: trimmed_reads
  - name: rogue_output
"#;
        let stage = r#"
stage_id: fastq.trim
outputs:
  - name: trimmed_reads
"#;
        let err = validate_tool_output_subset(tool, stage, Path::new("tool.yaml"), "fastq.trim")
            .expect_err("must reject unknown output");
        assert!(
            err.to_string().contains("rogue_output"),
            "unexpected error: {err}"
        );
    }
}
