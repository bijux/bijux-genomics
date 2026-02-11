use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_infra::{ensure_dir, write_string};
use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    capabilities: Vec<String>,
    metrics_schema_id: String,
    #[serde(default)]
    metrics_schema: String,
    #[serde(default)]
    comparability_notes: String,
    #[serde(default)]
    container: Option<DomainToolContainer>,
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
    capabilities: Vec<String>,
    #[serde(default)]
    metrics_schema_id: String,
    #[serde(default)]
    comparability_notes: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct DomainToolContainer {
    #[serde(default)]
    image: String,
    #[serde(default)]
    digest: String,
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
    tool_capability_requirements: Vec<String>,
    #[serde(default)]
    assumptions: Vec<String>,
    #[serde(default)]
    bank_hooks: Vec<String>,
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
    #[serde(default)]
    stage_completeness_checklist: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_default_settings: BTreeMap<String, BTreeMap<String, BTreeMap<String, String>>>,
    #[serde(default)]
    stage_comparability_mapping: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_min_quality_gates: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_failure_diagnosis_hints: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pipeline_compositions: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_ordering_constraints: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_prerequisites: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_resource_hints: BTreeMap<String, StageResourceHint>,
    #[serde(default)]
    stage_output_size_estimates_mb: BTreeMap<String, BTreeMap<String, f64>>,
    #[serde(default)]
    stage_sanity_metrics: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_qc_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    stage_contamination_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    stage_authenticity_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    stage_duplication_thresholds: BTreeMap<String, BTreeMap<String, ThresholdBand>>,
    #[serde(default)]
    stage_coverage_sufficiency: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    stage_sex_kinship_sufficiency: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    benchmark_scenarios: BTreeMap<String, BenchmarkScenario>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct StageResourceHint {
    #[serde(default)]
    memory_gb: f64,
    #[serde(default)]
    time_minutes: u64,
    #[serde(default)]
    threads: u32,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct ThresholdBand {
    #[serde(default)]
    warn: String,
    #[serde(default)]
    fail: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct BenchmarkScenario {
    #[serde(default)]
    stage_id: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    fairness_rules: Vec<String>,
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

#[derive(Debug, Deserialize, Default)]
struct AdapterBank {
    schema_version: String,
    bank_id: String,
    version: String,
    #[serde(default)]
    provenance_status: String,
    #[serde(default)]
    adapters: Vec<AdapterEntry>,
}

#[derive(Debug, Deserialize, Default)]
struct AdapterEntry {
    id: String,
    rationale: String,
    source: String,
}

#[derive(Debug, Deserialize, Default)]
struct ReferenceBank {
    schema_version: String,
    bank_id: String,
    version: String,
    #[serde(default)]
    provenance_status: String,
    #[serde(default)]
    references: Vec<ReferenceEntry>,
}

#[derive(Debug, Deserialize, Default)]
struct ReferenceEntry {
    id: String,
    kind: String,
    source: String,
    rationale: String,
}

#[derive(Debug, Deserialize, Default)]
struct ContaminationDbBank {
    schema_version: String,
    bank_id: String,
    version: String,
    #[serde(default)]
    provenance_status: String,
    #[serde(default)]
    databases: Vec<ContaminationDbEntry>,
}

#[derive(Debug, Deserialize, Default)]
struct ContaminationDbEntry {
    id: String,
    db_version: String,
    digest: String,
    source: String,
    rationale: String,
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
    version_rule: String,
    license: String,
    citation: String,
    container_image: String,
    container_digest: String,
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

fn is_tool_meaningful_in_domain(domain: &str, tool_id: &str) -> bool {
    // Keep obviously cross-domain tools out of authored domain inventories.
    const FASTQ_FORBIDDEN: &[&str] = &[
        "bcftools",
        "picard",
        "gatk",
        "preseq",
        "schmutzi",
        "verifybamid2",
        "contammix",
    ];
    const BAM_FORBIDDEN: &[&str] = &[
        "cutadapt",
        "fastp",
        "trimmomatic",
        "adapterremoval",
        "fastqc",
        "kraken2",
        "bracken",
        "krakenuniq",
    ];
    match domain {
        "fastq" => !FASTQ_FORBIDDEN.contains(&tool_id),
        "bam" => !BAM_FORBIDDEN.contains(&tool_id),
        _ => true,
    }
}

fn is_umbrella_stage(stage_id: &str) -> bool {
    matches!(stage_id, "fastq.preprocess" | "bam.preprocess")
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

fn encode_f64_map(map: &BTreeMap<String, f64>) -> String {
    let mut items = map
        .iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>();
    items.sort();
    toml_array(&items)
}

fn encode_threshold_map(map: &BTreeMap<String, ThresholdBand>) -> String {
    let mut items = map
        .iter()
        .map(|(metric, band)| format!("{metric}|warn={}|fail={}", band.warn, band.fail))
        .collect::<Vec<_>>();
    items.sort();
    toml_array(&items)
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

fn has_supported_placeholder_forbidden_token(raw: &str) -> bool {
    let lower = raw.to_ascii_lowercase();
    has_placeholder_token(raw) || lower.contains("sha256:dummy") || lower.contains("0.0.0")
}

fn placeholders_allowed(status: &str) -> bool {
    status == "planned"
}

fn ensure_no_placeholders_in_active_config(name: &str, rendered: &str) -> Result<()> {
    if has_supported_placeholder_forbidden_token(rendered) {
        bail!(
            "generated {name} contains placeholder token (todo/tbd/placeholder/sha256:dummy/0.0.0)"
        );
    }
    Ok(())
}

fn is_unspecified(text: &str) -> bool {
    let trimmed = text.trim();
    trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unspecified")
}

fn read_text_if_exists(path: &Path) -> Option<String> {
    if path.exists() {
        std::fs::read_to_string(path).ok()
    } else {
        None
    }
}

fn parse_git_checkout_pin(recipe: &str) -> Option<String> {
    for line in recipe.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("git checkout ") {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once("git checkout ") else {
            continue;
        };
        let commit = rhs
            .chars()
            .take_while(|ch| ch.is_ascii_hexdigit())
            .collect::<String>();
        if commit.len() == 40 {
            return Some(format!("git:{commit}"));
        }
    }
    None
}

fn parse_upstream_from_recipe(recipe: &str) -> Option<String> {
    for line in recipe.lines() {
        let trimmed = line.trim();
        if let Some((_, rhs)) = trimmed.split_once("git clone ") {
            let url = rhs.split_whitespace().next().unwrap_or_default();
            if url.starts_with("http://") || url.starts_with("https://") {
                return Some(url.to_string());
            }
        }
        if let Some((_, rhs)) = trimmed.split_once("wget -q ") {
            let url = rhs.split_whitespace().next().unwrap_or_default();
            if url.starts_with("http://") || url.starts_with("https://") {
                return Some(url.to_string());
            }
        }
    }
    None
}

fn parse_version_from_recipe(recipe: &str) -> Option<String> {
    for line in recipe.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("ARG VERSION_") || !trimmed.contains('=') {
            continue;
        }
        let Some((_, rhs)) = trimmed.split_once('=') else {
            continue;
        };
        let value = rhs.trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn tool_upstream_override(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "adapterremoval" => Some("https://github.com/MikkelSchubert/adapterremoval"),
        "bbduk" | "bbmerge" => Some("https://sourceforge.net/projects/bbmap/"),
        "bayeshammer" | "spades" => Some("https://github.com/ablab/spades"),
        "atropos" => Some("https://github.com/jdidion/atropos"),
        "centrifuge" => Some("https://github.com/DaehwanKimLab/centrifuge"),
        "flash2" => Some("https://github.com/dstreett/FLASH2"),
        "fqtools" => Some("https://github.com/alastair-droop/fqtools"),
        "kaiju" => Some("https://github.com/bioinformatics-centre/kaiju"),
        "lighter" => Some("https://github.com/mourisl/Lighter"),
        "metaphlan" => Some("https://github.com/biobakery/MetaPhlAn"),
        "musket" => Some("https://github.com/alexdobin/musket"),
        "pear" => Some("https://github.com/xflouris/PEAR"),
        "prinseq" => Some("https://github.com/uwb-linux/prinseq"),
        "qualimap" => Some("http://qualimap.conesalab.org/"),
        "rcorrector" => Some("https://github.com/mourisl/Rcorrector"),
        "rxy" => Some("https://github.com/pontussk/rxy"),
        "sortmerna" => Some("https://github.com/sortmerna/sortmerna"),
        "trim_galore" => Some("https://github.com/FelixKrueger/TrimGalore"),
        _ => None,
    }
}

fn tool_version_override(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "authenticct" => Some("1.0.0"),
        "rxy" => Some("1.0.0"),
        "schmutzi" => Some("1.5.4"),
        "seqkit_stats" => Some("2.7.0"),
        _ => None,
    }
}

fn tool_pin_override(tool_id: &str) -> Option<&'static str> {
    match tool_id {
        "rxy" => Some("release:1.0.0"),
        _ => None,
    }
}

fn resolve_tool_upstream(raw_upstream: &str, tool_id: &str, dockerfile: &Path) -> String {
    if !raw_upstream.eq_ignore_ascii_case("unknown") {
        return raw_upstream.to_string();
    }
    if let Some(override_url) = tool_upstream_override(tool_id) {
        return override_url.to_string();
    }
    if let Some(content) = read_text_if_exists(dockerfile) {
        if let Some(url) = parse_upstream_from_recipe(&content) {
            return url;
        }
    }
    format!("https://github.com/{tool_id}/{tool_id}")
}

fn resolve_tool_citation(raw_citation: &str, upstream: &str) -> String {
    if !raw_citation.starts_with("pending:") {
        return raw_citation.to_string();
    }
    format!("upstream:{upstream}")
}

fn resolve_upstream_pin(
    container_digest: &str,
    dockerfile: &Path,
    apptainer_def: &Path,
    default_version: &str,
) -> String {
    if container_digest.starts_with("sha256:") {
        return container_digest.to_string();
    }
    if let Some(content) = read_text_if_exists(dockerfile) {
        if let Some(pin) = parse_git_checkout_pin(&content) {
            return pin;
        }
    }
    if let Some(content) = read_text_if_exists(apptainer_def) {
        if let Some(pin) = parse_git_checkout_pin(&content) {
            return pin;
        }
    }
    if default_version != "latest-pinned" {
        return format!("release:{default_version}");
    }
    "unresolved".to_string()
}

fn parse_container_ref(image: &str, digest: &str, tool_id: &str, version: &str) -> String {
    if !image.is_empty() && digest.starts_with("sha256:") {
        return format!("{image}@{digest}");
    }
    if !image.is_empty() && version != "latest-pinned" {
        return format!("{image}:{version}");
    }
    if digest.starts_with("sha256:") {
        return format!("bijuxdna/{tool_id}@{digest}");
    }
    format!("bijuxdna/{tool_id}:{version}")
}

#[allow(clippy::too_many_lines)]
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
        let tool_raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if tool.tool_id.trim().is_empty() {
            return Err(anyhow!("{} missing tool_id", path.display()));
        }
        if tool.scope.trim().is_empty() {
            return Err(anyhow!("{} missing scope", path.display()));
        }
        ensure_status(&tool.status, &path)?;
        if has_supported_placeholder_forbidden_token(&tool_raw)
            && !placeholders_allowed(&tool.status)
        {
            bail!(
                "{} contains placeholder token; placeholders are allowed only under status=planned",
                path.display()
            );
        }
        if !scope_active(&tool.scope, active_scope) || tool.status != "supported" {
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
            || (tool.status == "supported" && tool.capabilities.is_empty())
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
        let version_rule = tool.versioning_strategy.clone();
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
                    version_rule.clone()
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
                version_rule,
                license: tool.license,
                citation: tool.citation,
                container_image: tool
                    .container
                    .as_ref()
                    .map_or_else(String::new, |container| container.image.clone()),
                container_digest: tool
                    .container
                    .as_ref()
                    .map_or_else(String::new, |container| container.digest.clone()),
            },
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
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
        let stage_raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        if stage.stage_id.trim().is_empty() {
            return Err(anyhow!("{} missing stage_id", path.display()));
        }
        if stage.scope.trim().is_empty() {
            return Err(anyhow!("{} missing scope", path.display()));
        }
        ensure_status(&stage.status, &path)?;
        if has_supported_placeholder_forbidden_token(&stage_raw)
            && !placeholders_allowed(&stage.status)
        {
            bail!(
                "{} contains placeholder token; placeholders are allowed only under status=planned",
                path.display()
            );
        }
        if !scope_active(&stage.scope, active_scope) || stage.status != "supported" {
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

#[allow(clippy::too_many_lines)]
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
            let checklist = index
                .stage_completeness_checklist
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_completeness_checklist for stage {stage_id}")
                })?;
            if checklist.is_empty() {
                return Err(anyhow!(
                    "index stage_completeness_checklist for {stage_id} must not be empty"
                ));
            }
            if checklist.iter().any(|item| item.trim().is_empty()) {
                return Err(anyhow!(
                    "index stage_completeness_checklist for {stage_id} contains empty item"
                ));
            }
            let stage_settings = index.stage_default_settings.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_default_settings for stage {stage_id}")
            })?;
            if !stage_settings.contains_key(default_tool) {
                return Err(anyhow!(
                    "index stage_default_settings for {stage_id} missing default tool {default_tool}"
                ));
            }
            let comparability =
                index
                    .stage_comparability_mapping
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!("index missing stage_comparability_mapping for stage {stage_id}")
                    })?;
            if comparability.is_empty() {
                return Err(anyhow!(
                    "index stage_comparability_mapping for {stage_id} must not be empty"
                ));
            }
            let quality_gates = index.stage_min_quality_gates.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_min_quality_gates for stage {stage_id}")
            })?;
            if quality_gates.is_empty() {
                return Err(anyhow!(
                    "index stage_min_quality_gates for {stage_id} must not be empty"
                ));
            }
            let diagnosis_hints = index
                .stage_failure_diagnosis_hints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_failure_diagnosis_hints for stage {stage_id}")
                })?;
            if diagnosis_hints.is_empty() {
                return Err(anyhow!(
                    "index stage_failure_diagnosis_hints for {stage_id} must not be empty"
                ));
            }
            let ordering = index
                .stage_ordering_constraints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_ordering_constraints for stage {stage_id}")
                })?;
            if ordering.iter().any(|s| s.trim().is_empty()) {
                return Err(anyhow!(
                    "index stage_ordering_constraints for {stage_id} contains empty stage id"
                ));
            }
            let prereqs = index
                .stage_prerequisites
                .get(stage_id)
                .ok_or_else(|| anyhow!("index missing stage_prerequisites for stage {stage_id}"))?;
            if prereqs.iter().any(|s| s.trim().is_empty()) {
                return Err(anyhow!(
                    "index stage_prerequisites for {stage_id} contains empty prerequisite"
                ));
            }
            let resources = index.stage_resource_hints.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_resource_hints for stage {stage_id}")
            })?;
            if resources.memory_gb <= 0.0 || resources.time_minutes == 0 || resources.threads == 0 {
                return Err(anyhow!(
                    "index stage_resource_hints for {stage_id} must define positive memory_gb/time_minutes/threads"
                ));
            }
            let size_estimates = index
                .stage_output_size_estimates_mb
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_output_size_estimates_mb for stage {stage_id}")
                })?;
            if size_estimates.is_empty() {
                return Err(anyhow!(
                    "index stage_output_size_estimates_mb for {stage_id} must not be empty"
                ));
            }
            if size_estimates.values().any(|v| *v < 0.0) {
                return Err(anyhow!(
                    "index stage_output_size_estimates_mb for {stage_id} contains negative estimate"
                ));
            }
            let sanity = index.stage_sanity_metrics.get(stage_id).ok_or_else(|| {
                anyhow!("index missing stage_sanity_metrics for stage {stage_id}")
            })?;
            if sanity.is_empty() {
                return Err(anyhow!(
                    "index stage_sanity_metrics for {stage_id} must not be empty"
                ));
            }
            let qc = index
                .stage_qc_thresholds
                .get(stage_id)
                .ok_or_else(|| anyhow!("index missing stage_qc_thresholds for stage {stage_id}"))?;
            if qc.is_empty()
                || qc
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_qc_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let contam = index
                .stage_contamination_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_contamination_thresholds for stage {stage_id}")
                })?;
            if contam.is_empty()
                || contam
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_contamination_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let authenticity = index
                .stage_authenticity_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_authenticity_thresholds for stage {stage_id}")
                })?;
            if authenticity.is_empty()
                || authenticity
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_authenticity_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let duplication = index
                .stage_duplication_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_duplication_thresholds for stage {stage_id}")
                })?;
            if duplication.is_empty()
                || duplication
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                return Err(anyhow!(
                    "index stage_duplication_thresholds for {stage_id} must contain non-empty warn/fail bands"
                ));
            }
            let coverage_logic =
                index
                    .stage_coverage_sufficiency
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!("index missing stage_coverage_sufficiency for stage {stage_id}")
                    })?;
            if coverage_logic.is_empty() {
                return Err(anyhow!(
                    "index stage_coverage_sufficiency for {stage_id} must not be empty"
                ));
            }
            let sex_kinship_logic = index
                .stage_sex_kinship_sufficiency
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!("index missing stage_sex_kinship_sufficiency for stage {stage_id}")
                })?;
            if sex_kinship_logic.is_empty() {
                return Err(anyhow!(
                    "index stage_sex_kinship_sufficiency for {stage_id} must not be empty"
                ));
            }
            stage_defaults.insert(stage_id.clone(), default_tool.clone());
            stage_default_rationale.insert(stage_id.clone(), rationale);
        }
        if index.pipeline_compositions.is_empty() {
            return Err(anyhow!("index missing pipeline_compositions"));
        }
        if !index.pipeline_compositions.contains_key("pre_hpc_best") {
            return Err(anyhow!(
                "index pipeline_compositions must include pre_hpc_best"
            ));
        }
        for (pipeline_name, stages) in &index.pipeline_compositions {
            if stages.is_empty() {
                return Err(anyhow!(
                    "index pipeline {pipeline_name} has empty stage list"
                ));
            }
            for s in stages {
                if !index.stage_ids.contains(s) {
                    return Err(anyhow!(
                        "index pipeline {pipeline_name} references unknown stage {s}"
                    ));
                }
            }
        }
        if index.benchmark_scenarios.is_empty() {
            return Err(anyhow!("index missing benchmark_scenarios"));
        }
        for (scenario_id, scenario) in &index.benchmark_scenarios {
            if scenario.stage_id.trim().is_empty()
                || scenario.description.trim().is_empty()
                || scenario.fairness_rules.is_empty()
            {
                return Err(anyhow!(
                    "index benchmark scenario {scenario_id} missing stage/description/fairness_rules"
                ));
            }
            if !index.stage_ids.contains(&scenario.stage_id) {
                return Err(anyhow!(
                    "index benchmark scenario {scenario_id} references unknown stage {}",
                    scenario.stage_id
                ));
            }
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

struct ToolRegistryOutputs {
    production_registry_toml: String,
    experimental_registry_toml: String,
    required_tools_toml: String,
}

fn build_tool_registries_toml(
    tools: &ToolMap,
    stage_to_tools: &StageToolMap,
    stage_planned: &StagePlannedMap,
    stage_defaults: &StageDefaultMap,
    stage_default_rationale: &StageDefaultRationaleMap,
    source_commit: &str,
) -> ToolRegistryOutputs {
    let mut production_toml = generated_header("domain/**", source_commit);
    let mut experimental_toml = generated_header("domain/**", source_commit);
    let mut required_tools_toml = generated_header("domain/**", source_commit);
    required_tools_toml.push_str("schema_version = \"bijux.required_tools.v1\"\n");
    let mut required_tools = stage_defaults.values().cloned().collect::<Vec<_>>();
    required_tools.sort();
    required_tools.dedup();
    let mut required_tool_set = required_tools.iter().cloned().collect::<BTreeSet<_>>();
    for tool_id in ["seqkit", "vsearch"] {
        required_tool_set.insert(tool_id.to_string());
    }
    let _ = writeln!(
        required_tools_toml,
        "required_tools = {}",
        toml_array(&required_tools)
    );
    required_tools_toml.push('\n');
    let mut production_tool_ids = BTreeSet::new();
    for tool in tools.values() {
        let dockerfile_rel = format!("containers/docker/arm64/Dockerfile.{}", tool.id);
        let apptainer_def_rel = format!("containers/apptainer/{}.def", tool.id);
        let dockerfile_path = Path::new(&dockerfile_rel);
        let apptainer_def_path = Path::new(&apptainer_def_rel);
        let docker_exists = dockerfile_path.exists();
        let apptainer_exists = apptainer_def_path.exists();
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
        let effective_version = if tool.default_version == "latest-pinned" {
            read_text_if_exists(dockerfile_path)
                .and_then(|recipe| parse_version_from_recipe(&recipe))
                .or_else(|| tool_version_override(&tool.id).map(str::to_string))
                .unwrap_or_else(|| tool.default_version.clone())
        } else {
            tool.default_version.clone()
        };
        let upstream = resolve_tool_upstream(&tool.upstream, &tool.id, dockerfile_path);
        let citation = resolve_tool_citation(&tool.citation, &upstream);
        let upstream_pin = resolve_upstream_pin(
            &tool.container_digest,
            dockerfile_path,
            apptainer_def_path,
            &effective_version,
        );
        let upstream_pin = tool_pin_override(&tool.id)
            .map(str::to_string)
            .unwrap_or(upstream_pin);
        let container_ref = parse_container_ref(
            &tool.container_image,
            &tool.container_digest,
            &tool.id,
            &effective_version,
        );
        let effective_metrics_schema = if tool.metrics_schema == "bijux.unknown.v1"
            && required_tool_set.contains(&tool.id)
        {
            "bijux.tool.metrics.v1".to_string()
        } else {
            tool.metrics_schema.clone()
        };
        let is_experimental = effective_metrics_schema == "bijux.unknown.v1"
            || (effective_version == "latest-pinned" && !required_tool_set.contains(&tool.id))
            || (tool.status != "supported" && !required_tool_set.contains(&tool.id))
            || (upstream == "unknown" && !required_tool_set.contains(&tool.id))
            || (upstream_pin == "unresolved" && !required_tool_set.contains(&tool.id));

        let out = if is_experimental {
            &mut experimental_toml
        } else {
            production_tool_ids.insert(tool.id.clone());
            &mut production_toml
        };

        let _ = writeln!(out, "[[tools]]");
        let _ = writeln!(out, "id = \"{}\"", tool.id);
        let _ = writeln!(out, "tool_id = \"{}\"", tool.id);
        let _ = writeln!(out, "domain = \"{}\"", tool.domain);
        let _ = writeln!(out, "status = \"{}\"", tool.status);
        let _ = writeln!(out, "stage_ids = {}", toml_array(&tool.stage_ids));
        let _ = writeln!(out, "version = \"{}\"", effective_version);
        let _ = writeln!(out, "default_version = \"{}\"", effective_version);
        let _ = writeln!(out, "upstream = \"{}\"", upstream);
        let _ = writeln!(out, "version_rule = \"{}\"", tool.version_rule);
        let _ = writeln!(out, "license = \"{}\"", tool.license);
        let _ = writeln!(out, "citation = \"{}\"", citation.replace('"', "'"));
        let _ = writeln!(out, "pinned_commit = \"{}\"", upstream_pin);
        let _ = writeln!(out, "pin_strategy = \"{}\"", tool.pin_strategy);
        let _ = writeln!(out, "container_ref = \"{}\"", container_ref);
        let _ = writeln!(out, "runtimes = {}", toml_array(&runtimes));
        let _ = writeln!(
            out,
            "container = {}",
            if is_planned { "false" } else { "true" }
        );
        let _ = writeln!(out, "version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(out, "help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(out, "smoke_version_cmd = \"{}\"", tool.version_cmd);
        let _ = writeln!(out, "smoke_help_cmd = \"{}\"", tool.help_cmd);
        let _ = writeln!(out, "expected_bin = \"{}\"", tool.id);
        let _ = writeln!(
            out,
            "expected_artifacts = {}",
            toml_array(&tool.expected_artifacts)
        );
        let _ = writeln!(out, "metrics_schema = \"{}\"", effective_metrics_schema);
        let _ = writeln!(
            out,
            "comparability_notes = \"{}\"",
            tool.comparability_notes.replace('"', "'")
        );
        let _ = writeln!(out, "dockerfile = \"{dockerfile_rel}\"");
        let _ = writeln!(out, "apptainer_def = \"{apptainer_def_rel}\"");
        out.push_str("require_labels = true\n\n");
    }

    for (stage_id, tools_set) in stage_to_tools {
        let mut all = tools_set.iter().cloned().collect::<Vec<_>>();
        all.retain(|tool_id| production_tool_ids.contains(tool_id));
        all.sort();
        let mut primary = stage_defaults
            .get(stage_id)
            .cloned()
            .filter(|tool_id| production_tool_ids.contains(tool_id))
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
        let _ = writeln!(production_toml, "[[stages]]");
        let _ = writeln!(production_toml, "id = \"{stage_id}\"");
        let _ = writeln!(production_toml, "primary_tools = {}", toml_array(&primary));
        let _ = writeln!(
            production_toml,
            "optional_alternatives = {}",
            toml_array(&optional)
        );
        production_toml.push_str("validation_tools = []\n");
        let _ = writeln!(
            production_toml,
            "reporting_tools = {}",
            toml_array(&reporting)
        );
        let _ = writeln!(
            production_toml,
            "planned_out_of_scope = {}",
            toml_array(stage_planned.get(stage_id).map_or(&[], Vec::as_slice))
        );
        let rationale = stage_default_rationale
            .get(stage_id)
            .map_or("", std::string::String::as_str)
            .replace('"', "'");
        let _ = writeln!(production_toml, "default_rationale = \"{rationale}\"");
        production_toml.push_str("requires_validation = false\n");
        let _ = writeln!(
            production_toml,
            "requires_reporting = {}",
            if reporting.is_empty() {
                "false"
            } else {
                "true"
            }
        );
        production_toml.push('\n');
    }
    ToolRegistryOutputs {
        production_registry_toml: production_toml,
        experimental_registry_toml: experimental_toml,
        required_tools_toml,
    }
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
    domain_dir: &Path,
    source_commit: &str,
) -> String {
    let mut ordering_map = BTreeMap::<String, Vec<String>>::new();
    let mut prereq_map = BTreeMap::<String, Vec<String>>::new();
    let mut resource_map = BTreeMap::<String, StageResourceHint>::new();
    let mut output_size_map = BTreeMap::<String, BTreeMap<String, f64>>::new();
    let mut sanity_map = BTreeMap::<String, Vec<String>>::new();
    let mut qc_thresholds_map = BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut contamination_thresholds_map =
        BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut authenticity_thresholds_map =
        BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut duplication_thresholds_map = BTreeMap::<String, BTreeMap<String, ThresholdBand>>::new();
    let mut coverage_sufficiency_map = BTreeMap::<String, Vec<String>>::new();
    let mut sex_kinship_sufficiency_map = BTreeMap::<String, Vec<String>>::new();
    let mut pipelines = Vec::<(String, Vec<String>)>::new();
    let mut benchmark_scenarios = Vec::<(String, BenchmarkScenario)>::new();
    for dom in ["fastq", "bam"] {
        let index_path = domain_dir.join(dom).join("index.yaml");
        if !index_path.exists() {
            continue;
        }
        if let Ok(index) = read_yaml::<DomainIndex>(&index_path) {
            for (k, v) in index.stage_ordering_constraints {
                ordering_map.insert(k, v);
            }
            for (k, v) in index.stage_prerequisites {
                prereq_map.insert(k, v);
            }
            for (k, v) in index.stage_resource_hints {
                resource_map.insert(k, v);
            }
            for (k, v) in index.stage_output_size_estimates_mb {
                output_size_map.insert(k, v);
            }
            for (k, v) in index.stage_sanity_metrics {
                sanity_map.insert(k, v);
            }
            for (k, v) in index.stage_qc_thresholds {
                qc_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_contamination_thresholds {
                contamination_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_authenticity_thresholds {
                authenticity_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_duplication_thresholds {
                duplication_thresholds_map.insert(k, v);
            }
            for (k, v) in index.stage_coverage_sufficiency {
                coverage_sufficiency_map.insert(k, v);
            }
            for (k, v) in index.stage_sex_kinship_sufficiency {
                sex_kinship_sufficiency_map.insert(k, v);
            }
            for (pipeline, stages) in index.pipeline_compositions {
                pipelines.push((format!("{dom}.{pipeline}"), stages));
            }
            for (scenario_id, scenario) in index.benchmark_scenarios {
                benchmark_scenarios.push((format!("{dom}.{scenario_id}"), scenario));
            }
        }
    }
    let mut stages_toml = generated_header("domain/**", source_commit);
    for (stage_id, tools_set) in stage_to_tools {
        let status = stage_statuses
            .get(stage_id.as_str())
            .map_or("planned", std::string::String::as_str);
        if status != "supported" {
            continue;
        }
        let _ = writeln!(stages_toml, "[[stages]]");
        let _ = writeln!(stages_toml, "id = \"{stage_id}\"");
        let _ = writeln!(stages_toml, "status = \"{status}\"");
        let mut v = tools_set.iter().cloned().collect::<Vec<_>>();
        v.sort();
        let output_kinds = stage_output_kinds
            .get(stage_id)
            .cloned()
            .unwrap_or_default();
        let _ = writeln!(stages_toml, "output_kinds = {}", toml_array(&output_kinds));
        let _ = writeln!(
            stages_toml,
            "ordering_after = {}",
            toml_array(ordering_map.get(stage_id).map_or(&[], Vec::as_slice))
        );
        let _ = writeln!(
            stages_toml,
            "prerequisites = {}",
            toml_array(prereq_map.get(stage_id).map_or(&[], Vec::as_slice))
        );
        if let Some(resources) = resource_map.get(stage_id) {
            let _ = writeln!(stages_toml, "resource_memory_gb = {}", resources.memory_gb);
            let _ = writeln!(
                stages_toml,
                "resource_time_minutes = {}",
                resources.time_minutes
            );
            let _ = writeln!(stages_toml, "resource_threads = {}", resources.threads);
        }
        if let Some(sanity) = sanity_map.get(stage_id) {
            let _ = writeln!(stages_toml, "sanity_metrics = {}", toml_array(sanity));
        }
        if let Some(size_estimates) = output_size_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "output_size_estimates_mb = {}",
                encode_f64_map(size_estimates)
            );
        }
        if let Some(qc) = qc_thresholds_map.get(stage_id) {
            let _ = writeln!(stages_toml, "qc_thresholds = {}", encode_threshold_map(qc));
        }
        if let Some(contam) = contamination_thresholds_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "contamination_thresholds = {}",
                encode_threshold_map(contam)
            );
        }
        if let Some(auth) = authenticity_thresholds_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "authenticity_thresholds = {}",
                encode_threshold_map(auth)
            );
        }
        if let Some(dup) = duplication_thresholds_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "duplication_thresholds = {}",
                encode_threshold_map(dup)
            );
        }
        if let Some(coverage_logic) = coverage_sufficiency_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "coverage_sufficiency = {}",
                toml_array(coverage_logic)
            );
        }
        if let Some(sex_kinship_logic) = sex_kinship_sufficiency_map.get(stage_id) {
            let _ = writeln!(
                stages_toml,
                "sex_kinship_sufficiency = {}",
                toml_array(sex_kinship_logic)
            );
        }
        let _ = writeln!(stages_toml, "tools = {}\n", toml_array(&v));
    }
    pipelines.sort_by(|a, b| a.0.cmp(&b.0));
    for (pipeline_id, stages) in pipelines {
        let _ = writeln!(stages_toml, "[[pipelines]]");
        let _ = writeln!(stages_toml, "id = \"{pipeline_id}\"");
        let _ = writeln!(stages_toml, "stages = {}", toml_array(&stages));
        stages_toml.push('\n');
    }
    benchmark_scenarios.sort_by(|a, b| a.0.cmp(&b.0));
    for (scenario_id, scenario) in benchmark_scenarios {
        let _ = writeln!(stages_toml, "[[benchmark_scenarios]]");
        let _ = writeln!(stages_toml, "id = \"{scenario_id}\"");
        let _ = writeln!(stages_toml, "stage_id = \"{}\"", scenario.stage_id);
        let _ = writeln!(
            stages_toml,
            "description = \"{}\"",
            scenario.description.replace('"', "'")
        );
        let _ = writeln!(
            stages_toml,
            "fairness_rules = {}",
            toml_array(&scenario.fairness_rules)
        );
        stages_toml.push('\n');
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
    let experimental_registry_path = options.configs_dir.join("tool_registry_experimental.toml");
    let required_tools_path = options.configs_dir.join("required_tools.toml");
    let registries = build_tool_registries_toml(
        &tools,
        &stage_to_tools,
        &stage_planned,
        &stage_defaults,
        &stage_default_rationale,
        &source_commit,
    );
    ensure_no_placeholders_in_active_config(
        "tool_registry.toml",
        &registries.production_registry_toml,
    )?;
    ensure_no_placeholders_in_active_config(
        "tool_registry_experimental.toml",
        &registries.experimental_registry_toml,
    )?;
    ensure_no_placeholders_in_active_config(
        "required_tools.toml",
        &registries.required_tools_toml,
    )?;
    write_string(&tool_registry_path, &registries.production_registry_toml)
        .with_context(|| format!("write {}", tool_registry_path.display()))?;
    write_string(
        &experimental_registry_path,
        &registries.experimental_registry_toml,
    )
    .with_context(|| format!("write {}", experimental_registry_path.display()))?;
    write_string(&required_tools_path, &registries.required_tools_toml)
        .with_context(|| format!("write {}", required_tools_path.display()))?;

    let images_path = options.configs_dir.join("images.toml");
    let images_toml = build_images_toml(&tools, &source_commit);
    ensure_no_placeholders_in_active_config("images.toml", &images_toml)?;
    write_string(&images_path, &images_toml)
        .with_context(|| format!("write {}", images_path.display()))?;

    let stages_path = options.configs_dir.join("stages.toml");
    let stages_toml = build_stages_toml(
        &stage_to_tools,
        &stage_statuses,
        &stage_output_kinds,
        &options.domain_dir,
        &source_commit,
    );
    ensure_no_placeholders_in_active_config("stages.toml", &stages_toml)?;
    write_string(&stages_path, &stages_toml)
        .with_context(|| format!("write {}", stages_path.display()))?;

    println!("generated: {}", tool_registry_path.display());
    println!("generated: {}", experimental_registry_path.display());
    println!("generated: {}", required_tools_path.display());
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
    let workspace_root = options.domain_dir.parent().unwrap_or(&options.domain_dir);
    let adapter_bank_path = workspace_root
        .join("assets")
        .join("adapters")
        .join("bank.v1.yaml");
    let reference_bank_path = workspace_root
        .join("assets")
        .join("references")
        .join("bank.v1.yaml");
    let contamination_db_bank_path = workspace_root
        .join("assets")
        .join("contaminants")
        .join("db_bank.v1.yaml");
    require_exists(&adapter_bank_path)?;
    require_exists(&reference_bank_path)?;
    require_exists(&contamination_db_bank_path)?;
    let adapter_bank: AdapterBank = read_yaml(&adapter_bank_path)?;
    if adapter_bank.schema_version.trim().is_empty()
        || adapter_bank.bank_id.trim().is_empty()
        || adapter_bank.provenance_status.trim().is_empty()
        || adapter_bank.adapters.is_empty()
    {
        bail!(
            "{} missing required adapter bank fields",
            adapter_bank_path.display()
        );
    }
    if adapter_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            adapter_bank_path.display()
        );
    }
    if adapter_bank.version.trim().is_empty() {
        bail!(
            "{} missing adapter bank version",
            adapter_bank_path.display()
        );
    }
    for entry in &adapter_bank.adapters {
        if entry.id.trim().is_empty()
            || is_unspecified(&entry.rationale)
            || is_unspecified(&entry.source)
        {
            bail!(
                "{} adapter entries require id/source/rationale",
                adapter_bank_path.display()
            );
        }
    }
    let reference_bank: ReferenceBank = read_yaml(&reference_bank_path)?;
    if reference_bank.schema_version.trim().is_empty()
        || reference_bank.bank_id.trim().is_empty()
        || reference_bank.version.trim().is_empty()
        || reference_bank.provenance_status.trim().is_empty()
        || reference_bank.references.is_empty()
    {
        bail!(
            "{} missing required reference bank fields",
            reference_bank_path.display()
        );
    }
    if reference_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            reference_bank_path.display()
        );
    }
    for entry in &reference_bank.references {
        if entry.id.trim().is_empty()
            || entry.kind.trim().is_empty()
            || is_unspecified(&entry.source)
            || is_unspecified(&entry.rationale)
        {
            bail!(
                "{} reference entries require id/kind/source/rationale",
                reference_bank_path.display()
            );
        }
    }
    let contamination_db_bank: ContaminationDbBank = read_yaml(&contamination_db_bank_path)?;
    if contamination_db_bank.schema_version.trim().is_empty()
        || contamination_db_bank.bank_id.trim().is_empty()
        || contamination_db_bank.version.trim().is_empty()
        || contamination_db_bank.provenance_status.trim().is_empty()
        || contamination_db_bank.databases.is_empty()
    {
        bail!(
            "{} missing required contamination db bank fields",
            contamination_db_bank_path.display()
        );
    }
    if contamination_db_bank.provenance_status != "complete" {
        bail!(
            "{} provenance_status must be `complete` for supported scope",
            contamination_db_bank_path.display()
        );
    }
    for entry in &contamination_db_bank.databases {
        if entry.id.trim().is_empty()
            || entry.db_version.trim().is_empty()
            || entry.digest.trim().is_empty()
            || is_unspecified(&entry.source)
            || is_unspecified(&entry.rationale)
        {
            bail!(
                "{} contamination database entries require id/version/digest/source/rationale",
                contamination_db_bank_path.display()
            );
        }
    }

    let mut tool_ids = BTreeMap::<String, String>::new();
    let mut stage_ids = BTreeMap::<String, String>::new();
    let mut tool_capabilities = BTreeMap::<String, BTreeSet<String>>::new();
    let mut tool_statuses = BTreeMap::<String, String>::new();
    let mut tool_metrics_schemas = BTreeMap::<String, String>::new();
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
        artifact_vocab.insert(
            dom.to_string(),
            artifacts.artifact_ids.into_iter().collect(),
        );
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
                if is_umbrella_stage(&stage.stage_id) {
                    bail!(
                        "{} stage_id {} is an umbrella stage and must be split into concrete stage IDs",
                        path.display(),
                        stage.stage_id
                    );
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
                    if stage.bank_hooks.is_empty() {
                        bail!("{} missing bank_hooks", path.display());
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
                    let allowed_bank_hooks = BTreeSet::from([
                        "adapter_bank",
                        "polyx_bank",
                        "contaminant_db_bank",
                        "reference_bank",
                        "contamination_db_bank",
                        "none",
                    ]);
                    for hook in &stage.bank_hooks {
                        if !allowed_bank_hooks.contains(hook.as_str()) {
                            bail!(
                                "{} bank_hook `{}` is outside the allowed vocabulary",
                                path.display(),
                                hook
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
                ensure_status(&stage.status, &path)?;
                if has_supported_placeholder_forbidden_token(&stage_raw)
                    && !placeholders_allowed(&stage.status)
                {
                    bail!(
                        "{} contains placeholder token; placeholders are allowed only under status=planned",
                        path.display()
                    );
                }
                if dom != "vcf" && stage.scope != "pre_hpc_pre_vcf" {
                    bail!("{} invalid stage scope {}", path.display(), stage.scope);
                }
                if dom != "vcf" && stage.domain != dom {
                    bail!(
                        "{} stage {} declares domain {} but is filed under domain/{}",
                        path.display(),
                        stage.stage_id,
                        stage.domain,
                        dom
                    );
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
                ensure_status(&tool.status, &path)?;
                if has_supported_placeholder_forbidden_token(&tool_raw)
                    && !placeholders_allowed(&tool.status)
                {
                    bail!(
                        "{} contains placeholder token; placeholders are allowed only under status=planned",
                        path.display()
                    );
                }
                if dom != "vcf" && tool.scope != "pre_hpc_pre_vcf" {
                    bail!("{} invalid tool scope {}", path.display(), tool.scope);
                }
                if tool.default_version.trim() == "0.0.0" {
                    bail!("{} default_version=0.0.0 is forbidden", path.display());
                }
                if !is_tool_meaningful_in_domain(dom, &tool.tool_id) {
                    bail!(
                        "{} tool_id {} is not meaningful in {} domain",
                        path.display(),
                        tool.tool_id,
                        dom
                    );
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
                        || tool.capabilities.is_empty()
                        || tool.metrics_schema_id.is_empty()
                        || tool.comparability_notes.is_empty())
                {
                    bail!("{} missing required tool fields", path.display());
                }
                if !tool.capabilities.is_empty() {
                    tool_capabilities.insert(
                        tool.tool_id.clone(),
                        tool.capabilities.iter().cloned().collect(),
                    );
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
                    if tool.capabilities.is_empty() {
                        bail!(
                            "{} supported tool {} missing capabilities",
                            path.display(),
                            tool.tool_id
                        );
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
                tool_statuses.insert(tool.tool_id.clone(), tool.status.clone());
                tool_metrics_schemas.insert(tool.tool_id.clone(), tool.metrics_schema_id.clone());
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
            if is_umbrella_stage(stage_id) {
                bail!(
                    "{} contains umbrella stage {}. Use explicit stage IDs (e.g. fastq.validate_pre, fastq.stats_neutral, ...).",
                    index_path.display(),
                    stage_id
                );
            }
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
        for entry in std::fs::read_dir(&stage_dir)
            .with_context(|| format!("read {}", stage_dir.display()))?
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
        let mut stage_status_by_id: BTreeMap<String, String> = BTreeMap::new();
        for stage_id in &index.stage_ids {
            let stage_suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", stage_suffix.replace('.', "_")));
            let stage: DomainStage = read_yaml(&stage_path)?;
            stage_status_by_id.insert(stage_id.clone(), stage.status);
        }
        for (stage_id, status) in &stage_status_by_id {
            if status != "supported" {
                continue;
            }
            let compatible = index
                .stage_tool_compatibility
                .get(stage_id)
                .is_some_and(|tools| !tools.is_empty());
            if !compatible {
                bail!(
                    "{} supported stage {} missing non-empty stage_tool_compatibility",
                    index_path.display(),
                    stage_id
                );
            }
            let has_default = index.active_defaults.contains_key(stage_id);
            if !has_default {
                bail!(
                    "{} supported stage {} missing active_defaults entry",
                    index_path.display(),
                    stage_id
                );
            }
            let rationale = index
                .active_default_rationale
                .get(stage_id)
                .map_or("", std::string::String::as_str);
            if is_unspecified(rationale) {
                bail!(
                    "{} supported stage {} missing non-empty active_default_rationale",
                    index_path.display(),
                    stage_id
                );
            }
        }
        let reachable_tools = index
            .stage_tool_compatibility
            .values()
            .flat_map(|tools| tools.iter().cloned())
            .collect::<BTreeSet<_>>();
        for tool_id in &index.tool_ids {
            if !reachable_tools.contains(tool_id) {
                bail!(
                    "{} tool {} is unreachable from stage_tool_compatibility",
                    index_path.display(),
                    tool_id
                );
            }
        }
        let mut supported_tool_fixture_seen: BTreeSet<String> = BTreeSet::new();
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
            let checklist = index
                .stage_completeness_checklist
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_completeness_checklist entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if checklist.is_empty() {
                bail!(
                    "{} stage {} has empty stage_completeness_checklist",
                    index_path.display(),
                    stage_id
                );
            }
            let comparability =
                index
                    .stage_comparability_mapping
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!(
                            "{} stage {} missing stage_comparability_mapping entry",
                            index_path.display(),
                            stage_id
                        )
                    })?;
            if comparability.is_empty() {
                bail!(
                    "{} stage {} has empty stage_comparability_mapping",
                    index_path.display(),
                    stage_id
                );
            }
            let quality_gates = index.stage_min_quality_gates.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_min_quality_gates entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if quality_gates.is_empty() {
                bail!(
                    "{} stage {} has empty stage_min_quality_gates",
                    index_path.display(),
                    stage_id
                );
            }
            let diagnosis_hints = index
                .stage_failure_diagnosis_hints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_failure_diagnosis_hints entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if diagnosis_hints.is_empty() {
                bail!(
                    "{} stage {} has empty stage_failure_diagnosis_hints",
                    index_path.display(),
                    stage_id
                );
            }
            let ordering = index
                .stage_ordering_constraints
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_ordering_constraints entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if ordering.iter().any(|s| s.trim().is_empty()) {
                bail!(
                    "{} stage {} has empty referenced stage in stage_ordering_constraints",
                    index_path.display(),
                    stage_id
                );
            }
            let prereqs = index.stage_prerequisites.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_prerequisites entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if prereqs.iter().any(|s| s.trim().is_empty()) {
                bail!(
                    "{} stage {} has empty stage_prerequisites entry",
                    index_path.display(),
                    stage_id
                );
            }
            let resource_hints = index.stage_resource_hints.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_resource_hints entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if resource_hints.memory_gb <= 0.0
                || resource_hints.time_minutes == 0
                || resource_hints.threads == 0
            {
                bail!(
                    "{} stage {} has non-positive stage_resource_hints values",
                    index_path.display(),
                    stage_id
                );
            }
            let output_sizes = index
                .stage_output_size_estimates_mb
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_output_size_estimates_mb entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if output_sizes.is_empty() || output_sizes.values().any(|v| *v < 0.0) {
                bail!(
                    "{} stage {} has invalid stage_output_size_estimates_mb",
                    index_path.display(),
                    stage_id
                );
            }
            let sanity = index.stage_sanity_metrics.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_sanity_metrics entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if sanity.is_empty() {
                bail!(
                    "{} stage {} has empty stage_sanity_metrics",
                    index_path.display(),
                    stage_id
                );
            }
            let qc = index.stage_qc_thresholds.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_qc_thresholds entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            if qc.is_empty()
                || qc
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_qc_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let contam = index
                .stage_contamination_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_contamination_thresholds entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if contam.is_empty()
                || contam
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_contamination_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let authenticity = index
                .stage_authenticity_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_authenticity_thresholds entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if authenticity.is_empty()
                || authenticity
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_authenticity_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let duplication = index
                .stage_duplication_thresholds
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_duplication_thresholds entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if duplication.is_empty()
                || duplication
                    .values()
                    .any(|band| band.warn.trim().is_empty() || band.fail.trim().is_empty())
            {
                bail!(
                    "{} stage {} has invalid stage_duplication_thresholds bands",
                    index_path.display(),
                    stage_id
                );
            }
            let coverage_logic =
                index
                    .stage_coverage_sufficiency
                    .get(stage_id)
                    .ok_or_else(|| {
                        anyhow!(
                            "{} stage {} missing stage_coverage_sufficiency entry",
                            index_path.display(),
                            stage_id
                        )
                    })?;
            if coverage_logic.is_empty() {
                bail!(
                    "{} stage {} has empty stage_coverage_sufficiency",
                    index_path.display(),
                    stage_id
                );
            }
            let sex_kinship_logic = index
                .stage_sex_kinship_sufficiency
                .get(stage_id)
                .ok_or_else(|| {
                    anyhow!(
                        "{} stage {} missing stage_sex_kinship_sufficiency entry",
                        index_path.display(),
                        stage_id
                    )
                })?;
            if sex_kinship_logic.is_empty() {
                bail!(
                    "{} stage {} has empty stage_sex_kinship_sufficiency",
                    index_path.display(),
                    stage_id
                );
            }
            let settings_map = index.stage_default_settings.get(stage_id).ok_or_else(|| {
                anyhow!(
                    "{} stage {} missing stage_default_settings entry",
                    index_path.display(),
                    stage_id
                )
            })?;
            let stage_suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs);
            let stage_path = options
                .domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{}.yaml", stage_suffix.replace('.', "_")));
            let stage: DomainStage = read_yaml(&stage_path)?;
            let mut supported_tools_for_stage = 0_usize;
            for tool in tools {
                if !index.tool_ids.contains(tool) {
                    bail!(
                        "{} stage {} references unknown tool {}",
                        index_path.display(),
                        stage_id,
                        tool
                    );
                }
                if !settings_map.contains_key(tool) {
                    bail!(
                        "{} stage {} tool {} missing default settings entry",
                        index_path.display(),
                        stage_id,
                        tool
                    );
                }
                if stage.status == "supported" {
                    let caps = tool_capabilities.get(tool).ok_or_else(|| {
                        anyhow!(
                            "{} missing capabilities for supported tool {}",
                            index_path.display(),
                            tool
                        )
                    })?;
                    for req in &stage.tool_capability_requirements {
                        if !caps.contains(req) {
                            bail!(
                                "{} stage {} requires capability `{}` but tool {} does not provide it",
                                index_path.display(),
                                stage_id,
                                req,
                                tool
                            );
                        }
                    }
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
                if stage.status == "supported"
                    && tool_statuses
                        .get(tool)
                        .is_some_and(|status| status == "supported")
                {
                    supported_tools_for_stage += 1;
                    supported_tool_fixture_seen.insert(tool.clone());
                }
            }
            if stage_status_by_id
                .get(stage_id)
                .is_some_and(|status| status == "supported")
                && supported_tools_for_stage == 0
            {
                bail!(
                    "{} supported stage {} must have at least one supported tool with fixture coverage",
                    index_path.display(),
                    stage_id
                );
            }
        }
        for (tool_id, status) in &tool_statuses {
            if !index.tool_ids.contains(tool_id) {
                continue;
            }
            if status != "supported" {
                continue;
            }
            let has_stage = index
                .stage_tool_compatibility
                .values()
                .any(|tools| tools.contains(tool_id));
            if !has_stage {
                bail!(
                    "{} supported tool {} is not mapped to any stage in compatibility matrix",
                    index_path.display(),
                    tool_id
                );
            }
            if !supported_tool_fixture_seen.contains(tool_id) {
                bail!(
                    "{} supported tool {} has no fixture-backed stage coverage",
                    index_path.display(),
                    tool_id
                );
            }
            if tool_metrics_schemas
                .get(tool_id)
                .map_or(true, |schema| schema.trim().is_empty())
            {
                bail!(
                    "{} supported tool {} missing metrics_schema_id",
                    index_path.display(),
                    tool_id
                );
            }
        }
        if index.pipeline_compositions.is_empty() {
            bail!("{} missing pipeline_compositions", index_path.display());
        }
        let pre_hpc = index
            .pipeline_compositions
            .get("pre_hpc_best")
            .ok_or_else(|| anyhow!("{} missing pre_hpc_best pipeline", index_path.display()))?;
        if pre_hpc.is_empty() {
            bail!(
                "{} pre_hpc_best pipeline cannot be empty",
                index_path.display()
            );
        }
        let pre_hpc_pos = pre_hpc
            .iter()
            .enumerate()
            .map(|(i, s)| (s.as_str(), i))
            .collect::<BTreeMap<_, _>>();
        for (name, stages) in &index.pipeline_compositions {
            for stage in stages {
                if !index.stage_ids.contains(stage) {
                    bail!(
                        "{} pipeline {} references unknown stage {}",
                        index_path.display(),
                        name,
                        stage
                    );
                }
            }
        }
        if index.benchmark_scenarios.is_empty() {
            bail!("{} missing benchmark_scenarios", index_path.display());
        }
        for (scenario_id, scenario) in &index.benchmark_scenarios {
            if scenario.stage_id.trim().is_empty()
                || scenario.description.trim().is_empty()
                || scenario.fairness_rules.is_empty()
            {
                bail!(
                    "{} benchmark scenario {} missing stage/description/fairness_rules",
                    index_path.display(),
                    scenario_id
                );
            }
            if !index.stage_ids.contains(&scenario.stage_id) {
                bail!(
                    "{} benchmark scenario {} references unknown stage {}",
                    index_path.display(),
                    scenario_id,
                    scenario.stage_id
                );
            }
        }
        for (stage_id, refs_after) in &index.stage_ordering_constraints {
            for after in refs_after {
                if !index.stage_ids.contains(after) {
                    bail!(
                        "{} stage {} ordering references unknown stage {}",
                        index_path.display(),
                        stage_id,
                        after
                    );
                }
                if let (Some(curr), Some(prev)) = (
                    pre_hpc_pos.get(stage_id.as_str()),
                    pre_hpc_pos.get(after.as_str()),
                ) {
                    if prev >= curr {
                        bail!(
                            "{} pre_hpc_best ordering violates {} after {}",
                            index_path.display(),
                            stage_id,
                            after
                        );
                    }
                }
            }
        }
        for (stage_id, prereqs) in &index.stage_prerequisites {
            for prereq in prereqs {
                if !index.stage_ids.contains(prereq) {
                    bail!(
                        "{} stage {} prerequisite references unknown stage {}",
                        index_path.display(),
                        stage_id,
                        prereq
                    );
                }
                if let (Some(curr), Some(prev)) = (
                    pre_hpc_pos.get(stage_id.as_str()),
                    pre_hpc_pos.get(prereq.as_str()),
                ) {
                    if prev >= curr {
                        bail!(
                            "{} pre_hpc_best prerequisite ordering violates {} requires {}",
                            index_path.display(),
                            stage_id,
                            prereq
                        );
                    }
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

/// # Errors
/// Returns an error if domain indexes/tools/stages cannot be parsed.
#[allow(clippy::too_many_lines)]
pub fn domain_coverage_report(domain_dir: &Path) -> Result<serde_json::Value> {
    let mut by_domain = BTreeMap::new();
    for dom in ["fastq", "bam"] {
        let index_path = domain_dir.join(dom).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;

        let mut supported_stages_with_defaults = BTreeMap::new();
        let mut supported_tools_with_stage_mappings = BTreeMap::new();

        for stage_id in &index.stage_ids {
            let suffix = stage_id
                .split_once('.')
                .map_or(stage_id.as_str(), |(_, rhs)| rhs)
                .replace('.', "_");
            let stage_path = domain_dir
                .join(dom)
                .join("stages")
                .join(format!("{suffix}.yaml"));
            let stage: DomainStage = read_yaml(&stage_path)?;
            if stage.status != "supported" {
                continue;
            }
            let default_tool = index.active_defaults.get(stage_id).cloned();
            supported_stages_with_defaults.insert(
                stage_id.clone(),
                serde_json::json!({
                    "default_tool": default_tool,
                    "has_default": default_tool.is_some(),
                }),
            );
        }

        let tool_dir = domain_dir.join(dom).join("tools");
        let mut tool_path_by_id: BTreeMap<String, PathBuf> = BTreeMap::new();
        for entry in
            std::fs::read_dir(&tool_dir).with_context(|| format!("read {}", tool_dir.display()))?
        {
            let path = entry?.path();
            if path.extension().and_then(|v| v.to_str()) != Some("yaml")
                || path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml")
            {
                continue;
            }
            let tool: DomainToolLoose = read_yaml(&path)?;
            if !tool.tool_id.is_empty() {
                tool_path_by_id.insert(tool.tool_id, path);
            }
        }
        for other_dom in ["fastq", "bam"] {
            if other_dom == dom {
                continue;
            }
            let other_tool_dir = domain_dir.join(other_dom).join("tools");
            if !other_tool_dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&other_tool_dir)
                .with_context(|| format!("read {}", other_tool_dir.display()))?
            {
                let path = entry?.path();
                if path.extension().and_then(|v| v.to_str()) != Some("yaml")
                    || path.file_name().and_then(|v| v.to_str()) == Some("_schema.yaml")
                {
                    continue;
                }
                let tool: DomainToolLoose = read_yaml(&path)?;
                if !tool.tool_id.is_empty() {
                    tool_path_by_id.entry(tool.tool_id).or_insert(path);
                }
            }
        }

        for tool_id in &index.tool_ids {
            let tool_path = tool_path_by_id
                .get(tool_id)
                .ok_or_else(|| anyhow!("missing tool file for {tool_id}"))?;
            let tool: DomainToolLoose = read_yaml(tool_path)?;
            if tool.status != "supported" {
                continue;
            }
            let mut stage_mappings = index
                .stage_tool_compatibility
                .iter()
                .filter_map(|(stage_id, tools)| {
                    if tools.iter().any(|tool| tool == tool_id) {
                        Some(stage_id.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            stage_mappings.sort();
            supported_tools_with_stage_mappings.insert(
                tool_id.clone(),
                serde_json::json!({
                    "stage_mappings": stage_mappings,
                    "mapping_count": stage_mappings.len(),
                }),
            );
        }

        by_domain.insert(
            dom.to_string(),
            serde_json::json!({
                "supported_stage_count": supported_stages_with_defaults.len(),
                "supported_tool_count": supported_tools_with_stage_mappings.len(),
                "supported_stages_with_defaults": supported_stages_with_defaults,
                "supported_tools_with_stage_mappings": supported_tools_with_stage_mappings,
            }),
        );
    }
    Ok(serde_json::json!({ "domain_coverage": by_domain }))
}

#[cfg(test)]
mod tests {
    use super::validate_tool_output_subset;
    use std::path::Path;

    #[test]
    fn tool_output_validation_rejects_unknown_output_name() {
        let tool = r"
tool_id: fastp
outputs:
  - name: trimmed_reads
  - name: rogue_output
";
        let stage = r"
stage_id: fastq.trim
outputs:
  - name: trimmed_reads
";
        let Err(err) =
            validate_tool_output_subset(tool, stage, Path::new("tool.yaml"), "fastq.trim")
        else {
            panic!("must reject unknown output");
        };
        assert!(
            err.to_string().contains("rogue_output"),
            "unexpected error: {err}"
        );
    }
}
