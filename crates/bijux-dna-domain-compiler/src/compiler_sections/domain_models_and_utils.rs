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
    #[serde(default)]
    container: Option<DomainToolContainer>,
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
    domain_version: String,
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

#[derive(Debug, Clone)]
struct ToolRow {
    id: String,
    domain: String,
    domains: Vec<String>,
    stage_ids: Vec<String>,
    bindings: Vec<String>,
    tool_role: String,
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
    expected_version_regex: String,
    healthcheck_cmd: String,
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
        "# GENERATED - DO NOT EDIT - source: {source}\n# source_commit: {source_commit}\n# domain_schema_version: bijux.domain.v1\n# Regenerate with: cargo run -p bijux-dna-domain-compiler --bin compile_domain_configs -- --domain-dir domain --configs-dir configs\n# schema_version = 1\n# owner = bijux-dna-domain-compiler\n# purpose = Contract config generated from domain/** sources\n# authority = bijux-dna-domain-compiler\n# stability = stable\n# last_updated = 2026-02-14\n\n"
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
            .take_while(char::is_ascii_hexdigit)
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
        "authenticct" | "rxy" => Some("1.0.0"),
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

fn default_version_regex(tool_id: &str) -> &'static str {
    match tool_id {
        "authenticct" => "authentic|v?[0-9]+[.][0-9]+",
        "fastqvalidator" => "fastqvalidator|v?[0-9]+[.][0-9]+",
        _ => "v?[0-9]+[.][0-9]+([.-][0-9A-Za-z]+)?",
    }
}

fn default_healthcheck_cmd(tool_id: &str, help_cmd: &str) -> String {
    if help_cmd.trim().is_empty() {
        return format!("{tool_id} --help");
    }
    help_cmd.to_string()
}

fn tool_role_from_stage_id(stage_id: &str) -> &'static str {
    if stage_id.contains(".align") || stage_id.contains("host_depletion") {
        "aligner"
    } else if stage_id.contains("screen") || stage_id.contains("contaminant") {
        "screen"
    } else if stage_id.contains("trim") || stage_id.contains("adapter") {
        "trimmer"
    } else if stage_id.contains("qc") || stage_id.contains("stats") || stage_id.contains("report") {
        "qc"
    } else if stage_id.contains("filter") {
        "filter"
    } else if stage_id.contains("validate") {
        "validator"
    } else if stage_id.contains("merge") {
        "merger"
    } else if stage_id.contains("correct") {
        "corrector"
    } else {
        "transform"
    }
}

fn infer_tool_role(stage_ids: &[String]) -> String {
    stage_ids.first().map_or_else(
        || "transform".to_string(),
        |stage_id| tool_role_from_stage_id(stage_id).to_string(),
    )
}

fn required_tool_roles_for_stage(stage_id: &str) -> Vec<String> {
    vec![tool_role_from_stage_id(stage_id).to_string()]
}
