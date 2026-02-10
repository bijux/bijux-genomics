use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_infra::{ensure_dir, write_string};
use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(name = "compile_domain_configs")]
struct Args {
    #[arg(long, default_value = "domain")]
    domain_dir: PathBuf,
    #[arg(long, default_value = "configs")]
    configs_dir: PathBuf,
}

#[derive(Debug, Deserialize, Default)]
struct DomainTool {
    tool_id: String,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    default_version: String,
    #[serde(default)]
    upstream: String,
    #[serde(default)]
    versioning_strategy: String,
    #[serde(default)]
    version_cmd: String,
    #[serde(default)]
    help_cmd: String,
    #[serde(default)]
    expected_artifacts: Vec<String>,
    #[serde(default)]
    metrics_schema_id: String,
}

#[derive(Debug, Deserialize, Default)]
struct DomainStage {
    stage_id: String,
    #[serde(default)]
    planned_out_of_scope: Vec<String>,
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
}

type ToolMap = BTreeMap<String, ToolRow>;
type StageToolMap = BTreeMap<String, BTreeSet<String>>;
type StagePlannedMap = BTreeMap<String, Vec<String>>;

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

fn generated_header(source: &str) -> String {
    format!(
        "# GENERATED - DO NOT EDIT - source: {source}\n# Regenerate with: cargo run --bin compile_domain_configs\n\n"
    )
}

fn load_domain_tools(
    domain_dir: &Path,
    domain: &str,
    tools: &mut ToolMap,
    stage_to_tools: &mut StageToolMap,
) -> Result<()> {
    let tools_dir = domain_dir.join(domain).join("tools");
    if !tools_dir.exists() {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(&tools_dir).with_context(|| format!("read {}", tools_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name.starts_with('_') {
            continue;
        }
        let tool: DomainTool = read_yaml(&path)?;
        if tool.tool_id.trim().is_empty() {
            return Err(anyhow!("{} missing tool_id", path.display()));
        }
        if tool.stage_ids.is_empty() {
            return Err(anyhow!("{} missing stage_ids", path.display()));
        }
        for stage in &tool.stage_ids {
            stage_to_tools
                .entry(stage.clone())
                .or_default()
                .insert(tool.tool_id.clone());
        }
        let tool_id = tool.tool_id.clone();
        tools.insert(
            tool_id.clone(),
            ToolRow {
                id: tool_id.clone(),
                domain: domain.to_string(),
                stage_ids: tool.stage_ids,
                default_version: if tool.default_version.is_empty() {
                    "latest-pinned".to_string()
                } else {
                    tool.default_version
                },
                upstream: if tool.upstream.is_empty() {
                    "unknown".to_string()
                } else {
                    tool.upstream
                },
                pin_strategy: if tool.versioning_strategy.is_empty() {
                    "pinned".to_string()
                } else {
                    tool.versioning_strategy
                },
                version_cmd: if tool.version_cmd.is_empty() {
                    format!("{tool_id} --version")
                } else {
                    tool.version_cmd
                },
                help_cmd: if tool.help_cmd.is_empty() {
                    format!("{tool_id} --help")
                } else {
                    tool.help_cmd
                },
                expected_artifacts: tool.expected_artifacts,
                metrics_schema: if tool.metrics_schema_id.is_empty() {
                    "bijux.unknown.v1".to_string()
                } else {
                    tool.metrics_schema_id
                },
            },
        );
    }
    Ok(())
}

fn load_domain_stages(
    domain_dir: &Path,
    domain: &str,
    stage_to_tools: &mut StageToolMap,
    stage_planned: &mut StagePlannedMap,
) -> Result<()> {
    let stages_dir = domain_dir.join(domain).join("stages");
    if !stages_dir.exists() {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(&stages_dir).with_context(|| format!("read {}", stages_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name.starts_with('_') {
            continue;
        }
        let stage: DomainStage = read_yaml(&path)?;
        if stage.stage_id.trim().is_empty() {
            return Err(anyhow!("{} missing stage_id", path.display()));
        }
        stage_to_tools.entry(stage.stage_id.clone()).or_default();
        stage_planned.insert(stage.stage_id, stage.planned_out_of_scope);
    }
    Ok(())
}

fn collect_domain_data(domain_dir: &Path) -> Result<(ToolMap, StageToolMap, StagePlannedMap)> {
    let mut tools: ToolMap = BTreeMap::new();
    let mut stage_to_tools: StageToolMap = BTreeMap::new();
    let mut stage_planned: StagePlannedMap = BTreeMap::new();
    for domain in ["fastq", "bam"] {
        load_domain_tools(domain_dir, domain, &mut tools, &mut stage_to_tools)?;
        load_domain_stages(domain_dir, domain, &mut stage_to_tools, &mut stage_planned)?;
    }
    Ok((tools, stage_to_tools, stage_planned))
}

fn build_tool_registry_toml(
    tools: &ToolMap,
    stage_to_tools: &StageToolMap,
    stage_planned: &StagePlannedMap,
) -> String {
    let mut registry_toml = generated_header("domain/**");
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
        let is_planned = tool.default_version == "planned";
        let _ = writeln!(registry_toml, "[[tools]]");
        let _ = writeln!(registry_toml, "id = \"{}\"", tool.id);
        let _ = writeln!(registry_toml, "tool_id = \"{}\"", tool.id);
        let _ = writeln!(registry_toml, "domain = \"{}\"", tool.domain);
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
        let _ = writeln!(registry_toml, "dockerfile = \"{dockerfile}\"");
        let _ = writeln!(registry_toml, "apptainer_def = \"{apptainer_def}\"");
        registry_toml.push_str("require_labels = true\n\n");
    }

    for (stage_id, tools_set) in stage_to_tools {
        let mut all = tools_set.iter().cloned().collect::<Vec<_>>();
        all.sort();
        let mut primary = all.first().cloned().into_iter().collect::<Vec<_>>();
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

fn build_images_toml(tools: &ToolMap) -> String {
    let mut images_toml = generated_header("domain/**");
    for tool in tools.values() {
        let _ = writeln!(images_toml, "[{}]", tool.id);
        let _ = writeln!(images_toml, "version = \"{}\"\n", tool.default_version);
    }
    images_toml
}

fn build_stages_toml(stage_to_tools: &StageToolMap) -> String {
    let mut stages_toml = generated_header("domain/**");
    for (stage_id, tools_set) in stage_to_tools {
        let _ = writeln!(stages_toml, "[[stages]]");
        let _ = writeln!(stages_toml, "id = \"{stage_id}\"");
        let mut v = tools_set.iter().cloned().collect::<Vec<_>>();
        v.sort();
        let _ = writeln!(stages_toml, "tools = {}\n", toml_array(&v));
    }
    stages_toml
}

fn main() -> Result<()> {
    let args = Args::parse();
    let (tools, stage_to_tools, stage_planned) = collect_domain_data(&args.domain_dir)?;
    ensure_dir(&args.configs_dir)
        .with_context(|| format!("create {}", args.configs_dir.display()))?;

    let tool_registry_path = args.configs_dir.join("tool_registry.toml");
    let registry_toml = build_tool_registry_toml(&tools, &stage_to_tools, &stage_planned);
    write_string(&tool_registry_path, &registry_toml)
        .with_context(|| format!("write {}", tool_registry_path.display()))?;

    let images_path = args.configs_dir.join("images.toml");
    let images_toml = build_images_toml(&tools);
    write_string(&images_path, &images_toml)
        .with_context(|| format!("write {}", images_path.display()))?;

    let stages_path = args.configs_dir.join("stages.toml");
    let stages_toml = build_stages_toml(&stage_to_tools);
    write_string(&stages_path, &stages_toml)
        .with_context(|| format!("write {}", stages_path.display()))?;

    println!("generated: {}", tool_registry_path.display());
    println!("generated: {}", images_path.display());
    println!("generated: {}", stages_path.display());
    Ok(())
}
