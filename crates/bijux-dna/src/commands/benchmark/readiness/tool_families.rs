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

pub(crate) const DEFAULT_TOOL_FAMILIES_PATH: &str = "benchmarks/configs/local/tool-families.toml";
pub(crate) const LOCAL_TOOL_FAMILIES_SCHEMA_VERSION: &str = "bijux.bench.local_tool_families.v1";
const TOOL_FAMILIES_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.tool_families_validation.v1";
const TOOL_FAMILY_CLASSIFICATION_SCOPE: &str = "primary_benchmark_function";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolFamilyConfig {
    schema_version: String,
    classification_scope: String,
    families: Vec<ToolFamilyDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ToolFamilyDefinition {
    family_id: String,
    summary: String,
    tool_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ToolFamilyAssignmentRow {
    pub(crate) tool_id: String,
    pub(crate) family_id: String,
    pub(crate) domains: Vec<String>,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) family_summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolFamiliesValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) classification_scope: String,
    pub(crate) family_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) multidomain_tool_count: usize,
    pub(crate) family_counts: BTreeMap<String, usize>,
    pub(crate) valid: bool,
    pub(crate) rows: Vec<ToolFamilyAssignmentRow>,
}

pub(crate) fn run_validate_tool_families(
    args: &parse::BenchReadinessValidateToolFamiliesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let config_path = match &args.config {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_TOOL_FAMILIES_PATH),
    };
    let report = validate_tool_families_path(&repo_root, &config_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn validate_tool_families_path(
    repo_root: &Path,
    config_path: &Path,
) -> Result<ToolFamiliesValidationReport> {
    let config = load_tool_families_config(config_path)?;
    validate_tool_family_config_contract(&config)?;

    let fastq_map = render_fastq_tool_serving_map(
        repo_root,
        PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH),
    )?;
    let bam_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;
    let benchmark_tool_index = build_benchmark_tool_index(&fastq_map.rows, &bam_map.rows);
    let assignment_index = build_assignment_index(&config)?;

    let configured_tool_ids = assignment_index.keys().cloned().collect::<BTreeSet<_>>();
    let benchmark_tool_ids = benchmark_tool_index.keys().cloned().collect::<BTreeSet<_>>();

    let missing_tool_ids =
        benchmark_tool_ids.difference(&configured_tool_ids).cloned().collect::<Vec<_>>();
    if !missing_tool_ids.is_empty() {
        return Err(anyhow!(
            "tool family config is missing {} benchmark tool ids: {}",
            missing_tool_ids.len(),
            missing_tool_ids.join(", "),
        ));
    }

    let extra_tool_ids =
        configured_tool_ids.difference(&benchmark_tool_ids).cloned().collect::<Vec<_>>();
    if !extra_tool_ids.is_empty() {
        return Err(anyhow!(
            "tool family config declares {} tool ids outside the governed benchmark scope: {}",
            extra_tool_ids.len(),
            extra_tool_ids.join(", "),
        ));
    }

    let mut family_counts = BTreeMap::<String, usize>::new();
    let mut rows = Vec::with_capacity(benchmark_tool_index.len());
    for (tool_id, benchmark_entry) in benchmark_tool_index {
        let assignment = assignment_index
            .get(&tool_id)
            .expect("tool family assignment must exist after coverage validation");
        *family_counts.entry(assignment.family_id.clone()).or_default() += 1;
        rows.push(ToolFamilyAssignmentRow {
            tool_id: tool_id.clone(),
            family_id: assignment.family_id.clone(),
            domains: benchmark_entry.domains,
            stage_ids: benchmark_entry.stage_ids,
            family_summary: assignment.family_summary.clone(),
        });
    }
    rows.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));

    let multidomain_tool_count = rows.iter().filter(|row| row.domains.len() > 1).count();

    Ok(ToolFamiliesValidationReport {
        schema_version: TOOL_FAMILIES_VALIDATION_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, config_path),
        classification_scope: config.classification_scope,
        family_count: config.families.len(),
        tool_count: rows.len(),
        multidomain_tool_count,
        family_counts,
        valid: true,
        rows,
    })
}

fn load_tool_families_config(config_path: &Path) -> Result<ToolFamilyConfig> {
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

fn validate_tool_family_config_contract(config: &ToolFamilyConfig) -> Result<()> {
    if config.schema_version != LOCAL_TOOL_FAMILIES_SCHEMA_VERSION {
        return Err(anyhow!("unsupported tool family schema `{}`", config.schema_version));
    }
    if config.classification_scope != TOOL_FAMILY_CLASSIFICATION_SCOPE {
        return Err(anyhow!(
            "unsupported tool family classification scope `{}`",
            config.classification_scope
        ));
    }
    if config.families.is_empty() {
        return Err(anyhow!("tool family config must declare at least one `[[families]]` entry"));
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ToolFamilyAssignment {
    family_id: String,
    family_summary: String,
}

fn build_assignment_index(
    config: &ToolFamilyConfig,
) -> Result<BTreeMap<String, ToolFamilyAssignment>> {
    let mut seen_family_ids = BTreeSet::<String>::new();
    let mut assignments = BTreeMap::<String, ToolFamilyAssignment>::new();

    for family in &config.families {
        validate_family_definition(family)?;
        if !seen_family_ids.insert(family.family_id.clone()) {
            return Err(anyhow!("tool family config repeats family_id `{}`", family.family_id));
        }

        for tool_id in &family.tool_ids {
            if let Some(previous) = assignments.get(tool_id) {
                return Err(anyhow!(
                    "tool `{}` is assigned to both family `{}` and `{}`",
                    tool_id,
                    previous.family_id,
                    family.family_id
                ));
            }
            assignments.insert(
                tool_id.clone(),
                ToolFamilyAssignment {
                    family_id: family.family_id.clone(),
                    family_summary: family.summary.clone(),
                },
            );
        }
    }

    Ok(assignments)
}

fn validate_family_definition(family: &ToolFamilyDefinition) -> Result<()> {
    if family.family_id.trim().is_empty() {
        return Err(anyhow!("tool family entries must declare a non-empty `family_id`"));
    }
    if !is_lower_snake_case(&family.family_id) {
        return Err(anyhow!("tool family `{}` must use lowercase snake_case", family.family_id));
    }
    if family.summary.trim().is_empty() {
        return Err(anyhow!(
            "tool family `{}` must declare a non-empty `summary`",
            family.family_id
        ));
    }
    if family.tool_ids.is_empty() {
        return Err(anyhow!(
            "tool family `{}` must declare at least one `tool_ids` entry",
            family.family_id
        ));
    }

    let mut sorted_tool_ids = family.tool_ids.clone();
    sorted_tool_ids.sort();
    if sorted_tool_ids != family.tool_ids {
        return Err(anyhow!(
            "tool family `{}` must keep `tool_ids` sorted lexically",
            family.family_id
        ));
    }
    let unique_tool_ids = family.tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    if unique_tool_ids.len() != family.tool_ids.len() {
        return Err(anyhow!("tool family `{}` repeats one or more tool ids", family.family_id));
    }
    Ok(())
}

fn is_lower_snake_case(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
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
        validate_tool_families_path, DEFAULT_TOOL_FAMILIES_PATH,
        TOOL_FAMILIES_VALIDATION_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn tool_family_config_covers_governed_benchmark_tools() {
        let root = repo_root();
        let report = validate_tool_families_path(&root, &root.join(DEFAULT_TOOL_FAMILIES_PATH))
            .expect("validate tool families");

        assert_eq!(report.schema_version, TOOL_FAMILIES_VALIDATION_SCHEMA_VERSION);
        assert_eq!(report.classification_scope, "primary_benchmark_function");
        assert_eq!(report.family_count, 25);
        assert_eq!(report.tool_count, 67);
        assert!(report.valid, "governed tool family config must validate cleanly");

        let bowtie2_build = report
            .rows
            .iter()
            .find(|row| row.tool_id == "bowtie2_build")
            .expect("bowtie2_build row");
        assert_eq!(bowtie2_build.family_id, "reference_indexing");

        let kraken2 = report.rows.iter().find(|row| row.tool_id == "kraken2").expect("kraken2 row");
        assert_eq!(kraken2.family_id, "taxonomy_classification");

        let pydamage =
            report.rows.iter().find(|row| row.tool_id == "pydamage").expect("pydamage row");
        assert_eq!(pydamage.family_id, "damage_and_postmortem_bias");

        let addeam = report.rows.iter().find(|row| row.tool_id == "addeam").expect("addeam row");
        assert_eq!(addeam.family_id, "damage_and_postmortem_bias");
    }

    #[test]
    fn tool_family_rows_preserve_domains_and_stage_scope() {
        let root = repo_root();
        let report = validate_tool_families_path(&root, &root.join(DEFAULT_TOOL_FAMILIES_PATH))
            .expect("validate tool families");

        let bowtie2 = report.rows.iter().find(|row| row.tool_id == "bowtie2").expect("bowtie2 row");
        assert_eq!(bowtie2.domains, vec!["bam".to_string(), "fastq".to_string()]);
        assert!(
            bowtie2.stage_ids.contains(&"bam.align".to_string())
                && bowtie2.stage_ids.contains(&"fastq.deplete_host".to_string()),
            "cross-domain rows must preserve their governed benchmark stage scope"
        );

        let multiqc = report.rows.iter().find(|row| row.tool_id == "multiqc").expect("multiqc row");
        assert_eq!(multiqc.family_id, "report_aggregation");
        assert_eq!(multiqc.domains, vec!["bam".to_string(), "fastq".to_string()]);
    }
}
