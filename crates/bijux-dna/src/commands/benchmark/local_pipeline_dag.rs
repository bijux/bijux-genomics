use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH: &str =
    "configs/pipelines/local/fastq-core-preprocess.toml";
const LOCAL_PIPELINE_DAG_SCHEMA_VERSION: &str = "bijux.bench.local_pipeline_dag.v1";
const LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_pipeline_dag_validation.v1";

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalPipelineDagDomain {
    Fastq,
    Bam,
}

impl LocalPipelineDagDomain {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
        }
    }

    fn inventory_domain(self) -> BenchLocalDomain {
        match self {
            Self::Fastq => BenchLocalDomain::Fastq,
            Self::Bam => BenchLocalDomain::Bam,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalPipelineDagConfig {
    schema_version: String,
    pipeline_id: String,
    domain: LocalPipelineDagDomain,
    summary: String,
    default_corpus_id: String,
    nodes: Vec<LocalPipelineDagNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalPipelineDagNode {
    node_id: String,
    stage_id: String,
    readiness_kind: LocalStageReadinessKind,
    summary: String,
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    external_inputs: Vec<String>,
    #[serde(default)]
    upstream_inputs: Vec<String>,
    #[serde(default)]
    outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalPipelineDagValidationNodeReport {
    pub(crate) node_id: String,
    pub(crate) stage_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) dependency_count: usize,
    pub(crate) external_input_count: usize,
    pub(crate) upstream_input_count: usize,
    pub(crate) output_count: usize,
    pub(crate) depends_on: Vec<String>,
    pub(crate) external_inputs: Vec<String>,
    pub(crate) upstream_inputs: Vec<String>,
    pub(crate) outputs: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LocalPipelineDagValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) output_path: String,
    pub(crate) pipeline_id: String,
    pub(crate) domain: String,
    pub(crate) default_corpus_id: String,
    pub(crate) summary: String,
    pub(crate) node_count: usize,
    pub(crate) edge_count: usize,
    pub(crate) acyclic: bool,
    pub(crate) valid: bool,
    pub(crate) topological_order: Vec<String>,
    pub(crate) nodes: Vec<LocalPipelineDagValidationNodeReport>,
}

pub(crate) fn run_validate_pipeline_dag(
    args: &parse::BenchLocalValidatePipelineDagArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let config_path = match &args.config {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH),
    };
    let config = load_local_pipeline_dag_config(&config_path)?;
    let output_path = match &args.output {
        Some(path) if path.is_absolute() => path.clone(),
        Some(path) => repo_root.join(path),
        None => repo_root.join(default_pipeline_report_relative_path(&config.pipeline_id)),
    };

    let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_pipeline_dag_path(
    repo_root: &Path,
    config_path: &Path,
    output_path: &Path,
) -> Result<LocalPipelineDagValidationReport> {
    let config = load_local_pipeline_dag_config(config_path)?;
    let report = validate_pipeline_dag(repo_root, config_path, output_path, &config)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn load_local_pipeline_dag_config(config_path: &Path) -> Result<LocalPipelineDagConfig> {
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))
}

fn validate_pipeline_dag(
    repo_root: &Path,
    config_path: &Path,
    output_path: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationReport> {
    validate_pipeline_contract(config)?;

    let inventory = load_local_stage_inventory(repo_root, config.domain.inventory_domain())?;
    let inventory_index = inventory
        .stages
        .into_iter()
        .map(|stage| (stage.stage_id, stage.readiness_kind))
        .collect::<BTreeMap<_, _>>();

    let mut seen_node_ids = BTreeSet::new();
    let mut seen_stage_ids = BTreeSet::new();
    let mut seen_outputs = BTreeSet::new();
    let mut node_index = BTreeMap::<String, usize>::new();
    let mut output_producers = BTreeMap::<String, String>::new();
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    let mut indegree = BTreeMap::<String, usize>::new();

    for (index, node) in config.nodes.iter().enumerate() {
        require_non_empty(
            &node.node_id,
            "local pipeline DAG nodes must declare a non-empty `node_id`",
        )?;
        require_non_empty(
            &node.stage_id,
            "local pipeline DAG nodes must declare a non-empty `stage_id`",
        )?;
        require_non_empty(
            &node.summary,
            "local pipeline DAG nodes must declare a non-empty `summary`",
        )?;
        if !seen_node_ids.insert(node.node_id.clone()) {
            return Err(anyhow!("local pipeline DAG repeats node_id `{}`", node.node_id));
        }
        if !seen_stage_ids.insert(node.stage_id.clone()) {
            return Err(anyhow!("local pipeline DAG repeats stage_id `{}`", node.stage_id));
        }
        let Some(expected_readiness_kind) = inventory_index.get(&node.stage_id) else {
            return Err(anyhow!(
                "local pipeline DAG stage `{}` is not present in the governed `{}` local stage inventory",
                node.stage_id,
                config.domain.as_str()
            ));
        };
        if node.readiness_kind != *expected_readiness_kind {
            return Err(anyhow!(
                "local pipeline DAG stage `{}` declares readiness `{}` but governed `{}` local inventory requires `{}`",
                node.stage_id,
                node.readiness_kind.as_str(),
                config.domain.as_str(),
                expected_readiness_kind.as_str()
            ));
        }
        validate_symbol_list(
            &node.external_inputs,
            &format!("node `{}` external_inputs", node.node_id),
        )?;
        validate_symbol_list(
            &node.upstream_inputs,
            &format!("node `{}` upstream_inputs", node.node_id),
        )?;
        validate_symbol_list(&node.outputs, &format!("node `{}` outputs", node.node_id))?;
        validate_symbol_list(&node.depends_on, &format!("node `{}` depends_on", node.node_id))?;
        if node.external_inputs.is_empty() && node.upstream_inputs.is_empty() {
            return Err(anyhow!(
                "local pipeline DAG node `{}` must declare at least one external or upstream input",
                node.node_id
            ));
        }
        if node.outputs.is_empty() {
            return Err(anyhow!(
                "local pipeline DAG node `{}` must declare at least one output",
                node.node_id
            ));
        }
        for output in &node.outputs {
            if !seen_outputs.insert(output.clone()) {
                return Err(anyhow!(
                    "local pipeline DAG output `{output}` is produced more than once"
                ));
            }
            output_producers.insert(output.clone(), node.node_id.clone());
        }
        node_index.insert(node.node_id.clone(), index);
        indegree.insert(node.node_id.clone(), node.depends_on.len());
        adjacency.entry(node.node_id.clone()).or_default();
    }

    for node in &config.nodes {
        let mut required_dependencies = BTreeSet::new();
        for dependency in &node.depends_on {
            if !node_index.contains_key(dependency) {
                return Err(anyhow!(
                    "local pipeline DAG node `{}` depends on unknown node `{dependency}`",
                    node.node_id
                ));
            }
            adjacency.entry(dependency.clone()).or_default().push(node.node_id.clone());
        }
        for upstream_input in &node.upstream_inputs {
            let Some(producer) = output_producers.get(upstream_input) else {
                return Err(anyhow!(
                    "local pipeline DAG node `{}` consumes upstream input `{upstream_input}` that no node produces",
                    node.node_id
                ));
            };
            if producer == &node.node_id {
                return Err(anyhow!(
                    "local pipeline DAG node `{}` cannot consume its own output `{upstream_input}` as an upstream input",
                    node.node_id
                ));
            }
            required_dependencies.insert(producer.clone());
        }
        let declared_dependencies = node.depends_on.iter().cloned().collect::<BTreeSet<_>>();
        if declared_dependencies != required_dependencies {
            return Err(anyhow!(
                "local pipeline DAG node `{}` declares dependencies {:?} but upstream inputs require {:?}",
                node.node_id,
                node.depends_on,
                required_dependencies.into_iter().collect::<Vec<_>>()
            ));
        }
    }

    let mut ready = VecDeque::new();
    for node in &config.nodes {
        if indegree.get(&node.node_id).copied().unwrap_or_default() == 0 {
            ready.push_back(node.node_id.clone());
        }
    }
    let mut topological_order = Vec::with_capacity(config.nodes.len());
    while let Some(node_id) = ready.pop_front() {
        topological_order.push(node_id.clone());
        for successor in adjacency.get(&node_id).into_iter().flatten() {
            let Some(entry) = indegree.get_mut(successor) else {
                continue;
            };
            *entry -= 1;
            if *entry == 0 {
                ready.push_back(successor.clone());
            }
        }
    }
    if topological_order.len() != config.nodes.len() {
        return Err(anyhow!("local pipeline DAG `{}` contains a cycle", config.pipeline_id));
    }

    let nodes = config
        .nodes
        .iter()
        .map(|node| LocalPipelineDagValidationNodeReport {
            node_id: node.node_id.clone(),
            stage_id: node.stage_id.clone(),
            readiness_kind: node.readiness_kind.as_str().to_string(),
            dependency_count: node.depends_on.len(),
            external_input_count: node.external_inputs.len(),
            upstream_input_count: node.upstream_inputs.len(),
            output_count: node.outputs.len(),
            depends_on: node.depends_on.clone(),
            external_inputs: node.external_inputs.clone(),
            upstream_inputs: node.upstream_inputs.clone(),
            outputs: node.outputs.clone(),
        })
        .collect::<Vec<_>>();

    Ok(LocalPipelineDagValidationReport {
        schema_version: LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, config_path),
        output_path: path_relative_to_repo(repo_root, output_path),
        pipeline_id: config.pipeline_id.clone(),
        domain: config.domain.as_str().to_string(),
        default_corpus_id: config.default_corpus_id.clone(),
        summary: config.summary.clone(),
        node_count: nodes.len(),
        edge_count: config.nodes.iter().map(|node| node.depends_on.len()).sum(),
        acyclic: true,
        valid: true,
        topological_order,
        nodes,
    })
}

fn validate_pipeline_contract(config: &LocalPipelineDagConfig) -> Result<()> {
    if config.schema_version != LOCAL_PIPELINE_DAG_SCHEMA_VERSION {
        return Err(anyhow!("unsupported local pipeline DAG schema `{}`", config.schema_version));
    }
    require_non_empty(
        &config.pipeline_id,
        "local pipeline DAG must declare a non-empty `pipeline_id`",
    )?;
    require_non_empty(&config.summary, "local pipeline DAG must declare a non-empty `summary`")?;
    require_non_empty(
        &config.default_corpus_id,
        "local pipeline DAG must declare a non-empty `default_corpus_id`",
    )?;
    if config.nodes.is_empty() {
        return Err(anyhow!("local pipeline DAG must declare at least one `[[nodes]]` entry"));
    }
    Ok(())
}

fn require_non_empty(value: &str, message: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!(message.to_string()));
    }
    Ok(())
}

fn validate_symbol_list(values: &[String], label: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(anyhow!("{label} contains an empty symbol"));
        }
        if !seen.insert(value.clone()) {
            return Err(anyhow!("{label} repeats symbol `{value}`"));
        }
    }
    Ok(())
}

fn default_pipeline_report_relative_path(pipeline_id: &str) -> PathBuf {
    PathBuf::from("target/local-ready/pipeline-dag").join(format!("{pipeline_id}.json"))
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{validate_pipeline_dag_path, DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH};

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn fastq_core_preprocess_pipeline_dag_is_acyclic_and_inventory_aligned() {
        let repo_root = repo_root();
        let config_path = repo_root.join(DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH);
        let output_path =
            repo_root.join("target/local-ready/pipeline-dag/fastq-core-preprocess.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-core-preprocess");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert_eq!(
            report.topological_order.first().map(String::as_str),
            Some("fastq.validate_reads")
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "read_length_metrics",
                            "adapter_report",
                            "trim_metrics",
                            "filter_metrics",
                            "filtered_profile",
                        ]
            }),
            "report_qc must consume the governed upstream preprocessing metrics"
        );
    }

    #[test]
    fn fastq_paired_merge_pipeline_dag_tracks_merged_and_unmerged_handoffs() {
        let repo_root = repo_root();
        let config_path = repo_root.join("configs/pipelines/local/fastq-paired-merge.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/fastq-paired-merge.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate paired merge local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-paired-merge");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.merge_pairs"
                    && node.outputs
                        == vec![
                            "merged_reads",
                            "unmerged_r1_reads",
                            "unmerged_r2_reads",
                            "merge_metrics",
                        ]
            }),
            "merge_pairs must declare merged and unmerged outputs explicitly"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.filter_reads"
                    && node.upstream_inputs
                        == vec!["merged_reads", "unmerged_r1_reads", "unmerged_r2_reads"]
            }),
            "filter_reads must consume the merged and unmerged merge-pairs outputs"
        );
    }
}
