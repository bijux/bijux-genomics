use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
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

fn main() -> Result<()> {
    let args = Args::parse();
    let mut tools: BTreeMap<String, ToolRow> = BTreeMap::new();
    let mut stage_to_tools: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut stage_planned: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for domain in ["fastq", "bam"] {
        let tools_dir = args.domain_dir.join(domain).join("tools");
        if tools_dir.exists() {
            for entry in std::fs::read_dir(&tools_dir)
                .with_context(|| format!("read {}", tools_dir.display()))?
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
        }

        let stages_dir = args.domain_dir.join(domain).join("stages");
        if stages_dir.exists() {
            for entry in std::fs::read_dir(&stages_dir)
                .with_context(|| format!("read {}", stages_dir.display()))?
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
        }
    }

    std::fs::create_dir_all(&args.configs_dir)
        .with_context(|| format!("create {}", args.configs_dir.display()))?;

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
        registry_toml.push_str("[[tools]]\n");
        registry_toml.push_str(&format!("id = \"{}\"\n", tool.id));
        registry_toml.push_str(&format!("tool_id = \"{}\"\n", tool.id));
        registry_toml.push_str(&format!("domain = \"{}\"\n", tool.domain));
        registry_toml.push_str(&format!("stage_ids = {}\n", toml_array(&tool.stage_ids)));
        registry_toml.push_str(&format!("version = \"{}\"\n", tool.default_version));
        registry_toml.push_str(&format!("default_version = \"{}\"\n", tool.default_version));
        registry_toml.push_str(&format!("upstream = \"{}\"\n", tool.upstream));
        registry_toml.push_str("pinned_commit = \"domain-managed\"\n");
        registry_toml.push_str(&format!("pin_strategy = \"{}\"\n", tool.pin_strategy));
        registry_toml.push_str(&format!("runtimes = {}\n", toml_array(&runtimes)));
        registry_toml.push_str(&format!("container = {}\n", if is_planned { "false" } else { "true" }));
        registry_toml.push_str(&format!("version_cmd = \"{}\"\n", tool.version_cmd));
        registry_toml.push_str(&format!("help_cmd = \"{}\"\n", tool.help_cmd));
        registry_toml.push_str(&format!("smoke_version_cmd = \"{}\"\n", tool.version_cmd));
        registry_toml.push_str(&format!("smoke_help_cmd = \"{}\"\n", tool.help_cmd));
        registry_toml.push_str(&format!("expected_bin = \"{}\"\n", tool.id));
        registry_toml.push_str(&format!(
            "expected_artifacts = {}\n",
            toml_array(&tool.expected_artifacts)
        ));
        registry_toml.push_str(&format!("metrics_schema = \"{}\"\n", tool.metrics_schema));
        registry_toml.push_str(&format!("dockerfile = \"{}\"\n", dockerfile));
        registry_toml.push_str(&format!("apptainer_def = \"{}\"\n", apptainer_def));
        registry_toml.push_str("require_labels = true\n\n");
    }

    for (stage_id, tools_set) in &stage_to_tools {
        let mut all = tools_set.iter().cloned().collect::<Vec<_>>();
        all.sort();
        let mut primary = all.first().cloned().into_iter().collect::<Vec<_>>();
        if primary.is_empty() {
            primary.push(if stage_id.starts_with("bam.") {
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
        registry_toml.push_str("[[stages]]\n");
        registry_toml.push_str(&format!("id = \"{}\"\n", stage_id));
        registry_toml.push_str(&format!("primary_tools = {}\n", toml_array(&primary)));
        registry_toml.push_str(&format!(
            "optional_alternatives = {}\n",
            toml_array(&optional)
        ));
        registry_toml.push_str("validation_tools = []\n");
        registry_toml.push_str(&format!("reporting_tools = {}\n", toml_array(&reporting)));
        registry_toml.push_str(&format!(
            "planned_out_of_scope = {}\n",
            toml_array(
                stage_planned
                    .get(stage_id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[])
            )
        ));
        registry_toml.push_str("requires_validation = false\n");
        registry_toml.push_str(&format!(
            "requires_reporting = {}\n\n",
            if reporting.is_empty() {
                "false"
            } else {
                "true"
            }
        ));
    }

    let tool_registry_path = args.configs_dir.join("tool_registry.toml");
    std::fs::write(&tool_registry_path, registry_toml)
        .with_context(|| format!("write {}", tool_registry_path.display()))?;

    let mut images_toml = generated_header("domain/**");
    for tool in tools.values() {
        images_toml.push_str(&format!("[{}]\n", tool.id));
        images_toml.push_str(&format!("version = \"{}\"\n\n", tool.default_version));
    }
    let images_path = args.configs_dir.join("images.toml");
    std::fs::write(&images_path, images_toml)
        .with_context(|| format!("write {}", images_path.display()))?;

    let mut stages_toml = generated_header("domain/**");
    for (stage_id, tools_set) in &stage_to_tools {
        stages_toml.push_str("[[stages]]\n");
        stages_toml.push_str(&format!("id = \"{}\"\n", stage_id));
        let mut v = tools_set.iter().cloned().collect::<Vec<_>>();
        v.sort();
        stages_toml.push_str(&format!("tools = {}\n\n", toml_array(&v)));
    }
    let stages_path = args.configs_dir.join("stages.toml");
    std::fs::write(&stages_path, stages_toml)
        .with_context(|| format!("write {}", stages_path.display()))?;

    println!("generated: {}", tool_registry_path.display());
    println!("generated: {}", images_path.display());
    println!("generated: {}", stages_path.display());
    Ok(())
}
