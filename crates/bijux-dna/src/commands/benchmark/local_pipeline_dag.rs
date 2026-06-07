use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_stage_compatibility::{
    validate_corpus_stage_compatibility_path, LocalCorpusStageCompatibilityEntryReport,
    DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain, LocalStageReadinessKind,
};
use crate::commands::benchmark::local_vcf_stage_matrix::{
    LocalVcfStageMatrixConfig, VcfStageMatrixRow,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const BENCHMARK_LOCAL_PIPELINE_CONFIG_ROOT: &str = "benchmarks/configs/pipelines/local";
pub(crate) const DEFAULT_FASTQ_CORE_PREPROCESS_PIPELINE_CONFIG_PATH: &str =
    "benchmarks/configs/pipelines/local/fastq-core-preprocess.toml";
const LOCAL_PIPELINE_DAG_SCHEMA_VERSION: &str = "bijux.bench.local_pipeline_dag.v1";
const LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.local_pipeline_dag_validation.v1";

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LocalPipelineDagDomain {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

impl LocalPipelineDagDomain {
    fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
            Self::Vcf => "vcf",
            Self::Cross => "cross",
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
pub(crate) struct LocalPipelineDagValidationProfileReport {
    pub(crate) profile_id: String,
    pub(crate) check_count: usize,
    pub(crate) checks: Vec<String>,
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
    pub(crate) validation_profiles: Vec<LocalPipelineDagValidationProfileReport>,
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

pub(crate) fn benchmark_local_pipeline_config_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(BENCHMARK_LOCAL_PIPELINE_CONFIG_ROOT)
}

pub(crate) fn benchmark_local_pipeline_config_path(repo_root: &Path, pipeline_id: &str) -> PathBuf {
    benchmark_local_pipeline_config_dir(repo_root).join(format!("{pipeline_id}.toml"))
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

    let inventory_index = load_pipeline_inventory_index(repo_root, config.domain)?;

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

    let validation_profiles = build_validation_profiles(repo_root, config)?;

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
        validation_profiles,
        nodes,
    })
}

fn build_validation_profiles(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<Vec<LocalPipelineDagValidationProfileReport>> {
    let mut profiles = Vec::new();
    let stage_ids = config.nodes.iter().map(|node| node.stage_id.as_str()).collect::<BTreeSet<_>>();

    if stage_ids.contains("vcf.call_pseudohaploid") || stage_ids.contains("vcf.damage_filter") {
        profiles.push(validate_ancient_dna_pseudohaploid_profile(repo_root, config)?);
    }
    if stage_ids.contains("vcf.call_gl") || stage_ids.contains("vcf.gl_propagation") {
        profiles.push(validate_ancient_dna_gl_profile(repo_root, config)?);
    }
    if stage_ids.contains("vcf.call_diploid") && stage_ids.contains("bam.recalibration") {
        profiles.push(validate_diploid_small_sample_profile(repo_root, config)?);
    }
    if stage_ids.contains("vcf.prepare_reference_panel") && stage_ids.contains("vcf.impute") {
        profiles.push(validate_reference_panel_imputation_profile(repo_root, config)?);
    }
    if stage_ids.contains("vcf.population_structure") && stage_ids.contains("vcf.admixture") {
        profiles.push(validate_population_structure_vcf_profile(repo_root, config)?);
    }
    if stage_ids.contains("vcf.ibd")
        && stage_ids.contains("vcf.roh")
        && stage_ids.contains("vcf.demography")
    {
        profiles.push(validate_relatedness_segments_vcf_profile(repo_root, config)?);
    }
    if stage_ids.contains("bam.genotyping")
        && stage_ids.contains("vcf.filter")
        && stage_ids.contains("vcf.stats")
        && stage_ids.contains("vcf.qc")
        && stage_ids.contains("vcf.pca")
        && stage_ids.contains("vcf.roh")
    {
        profiles.push(validate_bam_genotyping_vcf_downstream_profile(repo_root, config)?);
    }
    if config.pipeline_id == "edna-taxonomy-no-vcf" {
        profiles.push(validate_edna_taxonomy_no_vcf_profile(repo_root, config)?);
    }
    if config.pipeline_id == "amplicon-asv-otu-no-vcf" {
        profiles.push(validate_amplicon_asv_otu_no_vcf_profile(repo_root, config)?);
    }

    Ok(profiles)
}

fn validate_ancient_dna_pseudohaploid_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "ancient_dna_pseudohaploid";

    if config.default_corpus_id != "corpus-01-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed FASTQ corpus `corpus-01-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let trim_terminal_damage =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.trim_terminal_damage")?;
    let remove_duplicates =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.remove_duplicates")?;
    let bam_align = require_profile_stage(&stage_index, PROFILE_ID, "bam.align")?;
    let bam_damage = require_profile_stage(&stage_index, PROFILE_ID, "bam.damage")?;
    let bam_contamination = require_profile_stage(&stage_index, PROFILE_ID, "bam.contamination")?;
    let bam_authenticity = require_profile_stage(&stage_index, PROFILE_ID, "bam.authenticity")?;
    let vcf_call = require_profile_stage(&stage_index, PROFILE_ID, "vcf.call_pseudohaploid")?;
    let vcf_damage_filter = require_profile_stage(&stage_index, PROFILE_ID, "vcf.damage_filter")?;
    let vcf_stats = require_profile_stage(&stage_index, PROFILE_ID, "vcf.stats")?;

    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r1_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R1 handoff",
    )?;
    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r2_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R2 handoff",
    )?;
    require_list_contains(
        &bam_align.depends_on,
        "fastq.filter_reads",
        PROFILE_ID,
        "bam.align must remain downstream of the duplicate-aware FASTQ preprocessing branch",
    )?;
    require_list_contains(
        &bam_damage.external_inputs,
        "expected_damage_contract",
        PROFILE_ID,
        "bam.damage must declare the governed expected-damage asset contract",
    )?;
    require_list_contains(
        &bam_contamination.external_inputs,
        "adna_contamination_panel_contract",
        PROFILE_ID,
        "bam.contamination must declare the governed ancient-DNA contamination panel contract",
    )?;
    require_list_contains(
        &bam_authenticity.external_inputs,
        "adna_authenticity_policy_contract",
        PROFILE_ID,
        "bam.authenticity must declare the governed ancient-DNA authenticity policy contract",
    )?;
    require_list_contains(
        &vcf_call.upstream_inputs,
        "damage_report_json",
        PROFILE_ID,
        "vcf.call_pseudohaploid must consume the BAM damage evidence handoff",
    )?;
    require_list_contains(
        &vcf_call.upstream_inputs,
        "authenticity_report_json",
        PROFILE_ID,
        "vcf.call_pseudohaploid must consume the BAM authenticity evidence handoff",
    )?;
    require_list_contains(
        &vcf_call.external_inputs,
        "adna_reference_fasta_contract",
        PROFILE_ID,
        "vcf.call_pseudohaploid must declare the governed ancient-DNA reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_call.external_inputs,
        "adna_reference_fai_contract",
        PROFILE_ID,
        "vcf.call_pseudohaploid must declare the governed ancient-DNA reference index contract",
    )?;
    require_list_contains(
        &vcf_damage_filter.depends_on,
        "bam.damage",
        PROFILE_ID,
        "vcf.damage_filter must remain downstream of BAM damage evidence",
    )?;
    require_list_contains(
        &vcf_damage_filter.upstream_inputs,
        "pseudohaploid_vcf",
        PROFILE_ID,
        "vcf.damage_filter must consume the pseudohaploid VCF handoff",
    )?;
    require_list_contains(
        &vcf_damage_filter.upstream_inputs,
        "damage_report_json",
        PROFILE_ID,
        "vcf.damage_filter must consume the BAM damage evidence handoff",
    )?;
    require_list_contains(
        &vcf_damage_filter.external_inputs,
        "damage_filter_policy_contract",
        PROFILE_ID,
        "vcf.damage_filter must declare the governed damage-filter policy contract",
    )?;
    require_list_contains(
        &vcf_stats.depends_on,
        "vcf.damage_filter",
        PROFILE_ID,
        "vcf.stats must remain downstream of damage-aware filtering",
    )?;
    require_list_contains(
        &trim_terminal_damage.external_inputs,
        "terminal_damage_policy_contract",
        PROFILE_ID,
        "fastq.trim_terminal_damage must declare the governed terminal-damage trimming policy contract",
    )?;

    let compatibility_report = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.trim_terminal_damage",
        "corpus-01",
        "corpus-01-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.damage",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.authenticity",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-mini for governed aDNA-like FASTQ inputs".to_string(),
        "fastq.trim_terminal_damage is fixture-backed by corpus-01-mini".to_string(),
        "fastq.remove_duplicates stays downstream of terminal-damage trimming".to_string(),
        "bam.damage and bam.authenticity stay fixture-backed by corpus-01-adna-damage-mini".to_string(),
        "bam.contamination declares the ancient-DNA contamination panel contract".to_string(),
        "vcf.call_pseudohaploid consumes BAM damage and authenticity evidence with ancient-DNA reference contracts".to_string(),
        "vcf.damage_filter stays downstream of BAM damage evidence and pseudohaploid calling".to_string(),
        "vcf.stats stays downstream of damage-aware filtering".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_ancient_dna_gl_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "ancient_dna_gl";

    if config.default_corpus_id != "corpus-01-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed FASTQ corpus `corpus-01-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let trim_terminal_damage =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.trim_terminal_damage")?;
    let remove_duplicates =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.remove_duplicates")?;
    let bam_align = require_profile_stage(&stage_index, PROFILE_ID, "bam.align")?;
    let bam_damage = require_profile_stage(&stage_index, PROFILE_ID, "bam.damage")?;
    let bam_contamination = require_profile_stage(&stage_index, PROFILE_ID, "bam.contamination")?;
    let bam_authenticity = require_profile_stage(&stage_index, PROFILE_ID, "bam.authenticity")?;
    let vcf_call_gl = require_profile_stage(&stage_index, PROFILE_ID, "vcf.call_gl")?;
    let vcf_gl_propagation = require_profile_stage(&stage_index, PROFILE_ID, "vcf.gl_propagation")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;

    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r1_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R1 handoff",
    )?;
    require_list_contains(
        &remove_duplicates.upstream_inputs,
        "terminal_damage_trimmed_reads_r2_path",
        PROFILE_ID,
        "fastq.remove_duplicates must consume the terminal-damage-trimmed R2 handoff",
    )?;
    require_list_contains(
        &bam_align.depends_on,
        "fastq.filter_reads",
        PROFILE_ID,
        "bam.align must remain downstream of the duplicate-aware FASTQ preprocessing branch",
    )?;
    require_list_contains(
        &bam_damage.external_inputs,
        "expected_damage_contract",
        PROFILE_ID,
        "bam.damage must declare the governed expected-damage asset contract",
    )?;
    require_list_contains(
        &bam_contamination.external_inputs,
        "adna_contamination_panel_contract",
        PROFILE_ID,
        "bam.contamination must declare the governed ancient-DNA contamination panel contract",
    )?;
    require_list_contains(
        &bam_authenticity.external_inputs,
        "adna_authenticity_policy_contract",
        PROFILE_ID,
        "bam.authenticity must declare the governed ancient-DNA authenticity policy contract",
    )?;
    require_list_contains(
        &vcf_call_gl.upstream_inputs,
        "damage_report_json",
        PROFILE_ID,
        "vcf.call_gl must consume the BAM damage evidence handoff",
    )?;
    require_list_contains(
        &vcf_call_gl.upstream_inputs,
        "authenticity_report_json",
        PROFILE_ID,
        "vcf.call_gl must consume the BAM authenticity evidence handoff",
    )?;
    require_list_contains(
        &vcf_call_gl.external_inputs,
        "adna_reference_fasta_contract",
        PROFILE_ID,
        "vcf.call_gl must declare the governed ancient-DNA reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_call_gl.external_inputs,
        "adna_reference_fai_contract",
        PROFILE_ID,
        "vcf.call_gl must declare the governed ancient-DNA reference index contract",
    )?;
    require_list_contains(
        &vcf_call_gl.outputs,
        "gl_sites_vcf",
        PROFILE_ID,
        "vcf.call_gl must produce the likelihood-bearing VCF handoff",
    )?;
    require_list_contains(
        &vcf_gl_propagation.depends_on,
        "vcf.call_gl",
        PROFILE_ID,
        "vcf.gl_propagation must remain downstream of genotype-likelihood calling",
    )?;
    require_list_contains(
        &vcf_gl_propagation.upstream_inputs,
        "gl_sites_vcf",
        PROFILE_ID,
        "vcf.gl_propagation must consume the genotype-likelihood VCF handoff",
    )?;
    require_list_contains(
        &vcf_gl_propagation.outputs,
        "gl_propagated_vcf",
        PROFILE_ID,
        "vcf.gl_propagation must keep a propagated likelihood-bearing VCF handoff explicit",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.gl_propagation",
        PROFILE_ID,
        "vcf.qc must remain downstream of genotype-likelihood propagation",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "gl_propagated_vcf",
        PROFILE_ID,
        "vcf.qc must consume the propagated genotype-likelihood VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "gl_propagation_report_json",
        PROFILE_ID,
        "vcf.qc must consume explicit genotype-likelihood propagation evidence",
    )?;
    require_list_contains(
        &trim_terminal_damage.external_inputs,
        "terminal_damage_policy_contract",
        PROFILE_ID,
        "fastq.trim_terminal_damage must declare the governed terminal-damage trimming policy contract",
    )?;

    let compatibility_report = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.trim_terminal_damage",
        "corpus-01",
        "corpus-01-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.damage",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.authenticity",
        "corpus-01-adna-bam",
        "corpus-01-adna-damage-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-mini for governed aDNA-like FASTQ inputs".to_string(),
        "fastq.trim_terminal_damage is fixture-backed by corpus-01-mini".to_string(),
        "fastq.remove_duplicates stays downstream of terminal-damage trimming".to_string(),
        "bam.damage and bam.authenticity stay fixture-backed by corpus-01-adna-damage-mini".to_string(),
        "bam.contamination declares the ancient-DNA contamination panel contract".to_string(),
        "vcf.call_gl consumes BAM damage and authenticity evidence with ancient-DNA reference contracts".to_string(),
        "vcf.gl_propagation stays downstream of genotype-likelihood calling with explicit GL handoff coverage".to_string(),
        "vcf.qc stays downstream of propagated genotype-likelihood outputs".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_diploid_small_sample_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "diploid_small_sample";

    if config.default_corpus_id != "corpus-01-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed FASTQ corpus `corpus-01-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let bam_filter = require_profile_stage(&stage_index, PROFILE_ID, "bam.filter")?;
    let bam_recalibration = require_profile_stage(&stage_index, PROFILE_ID, "bam.recalibration")?;
    let vcf_call_diploid = require_profile_stage(&stage_index, PROFILE_ID, "vcf.call_diploid")?;
    let vcf_filter = require_profile_stage(&stage_index, PROFILE_ID, "vcf.filter")?;
    let vcf_stats = require_profile_stage(&stage_index, PROFILE_ID, "vcf.stats")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_phasing = require_profile_stage(&stage_index, PROFILE_ID, "vcf.phasing")?;

    require_list_contains(
        &bam_filter.outputs,
        "filtered_bam",
        PROFILE_ID,
        "bam.filter must keep the filtered-BAM fallback output explicit",
    )?;
    require_list_contains(
        &bam_filter.outputs,
        "filtered_bai",
        PROFILE_ID,
        "bam.filter must keep the filtered-BAM index fallback output explicit",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_reference_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration reference contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_known_sites_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed known-sites recalibration contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_coverage_gate_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed low-coverage recalibration gate contract",
    )?;
    require_list_contains(
        &vcf_call_diploid.depends_on,
        "bam.filter",
        PROFILE_ID,
        "vcf.call_diploid must remain downstream of the filtered-BAM fallback branch",
    )?;
    require_list_contains(
        &vcf_call_diploid.depends_on,
        "bam.recalibration",
        PROFILE_ID,
        "vcf.call_diploid must remain downstream of the recalibrated-BAM run branch",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "filtered_bam",
        PROFILE_ID,
        "vcf.call_diploid must consume the filtered-BAM fallback handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "filtered_bai",
        PROFILE_ID,
        "vcf.call_diploid must consume the filtered-BAM index fallback handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "recalibrated_bam",
        PROFILE_ID,
        "vcf.call_diploid must consume the recalibrated-BAM run-path handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "recalibrated_bai",
        PROFILE_ID,
        "vcf.call_diploid must consume the recalibrated-BAM index run-path handoff",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "recalibration_summary_json",
        PROFILE_ID,
        "vcf.call_diploid must consume the recalibration decision summary",
    )?;
    require_list_contains(
        &vcf_call_diploid.upstream_inputs,
        "coverage_report_json",
        PROFILE_ID,
        "vcf.call_diploid must consume explicit coverage evidence for the low-coverage skip path",
    )?;
    require_list_contains(
        &vcf_call_diploid.external_inputs,
        "reference_fasta_contract",
        PROFILE_ID,
        "vcf.call_diploid must declare the governed reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_call_diploid.external_inputs,
        "reference_fai_contract",
        PROFILE_ID,
        "vcf.call_diploid must declare the governed reference FASTA index contract",
    )?;
    require_list_contains(
        &vcf_filter.depends_on,
        "vcf.call_diploid",
        PROFILE_ID,
        "vcf.filter must remain downstream of diploid calling",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "diploid_vcf",
        PROFILE_ID,
        "vcf.filter must consume the diploid VCF handoff",
    )?;
    require_list_contains(
        &vcf_stats.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.stats must remain downstream of variant filtering",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.qc must remain downstream of filtered VCF outputs",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.stats",
        PROFILE_ID,
        "vcf.qc must remain downstream of explicit VCF stats evidence",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.qc must consume the filtered VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "stats_json",
        PROFILE_ID,
        "vcf.qc must consume explicit VCF stats evidence",
    )?;
    if vcf_qc.depends_on.iter().any(|value| value == "vcf.phasing") {
        return Err(anyhow!("{PROFILE_ID}: vcf.qc must not depend on optional phasing output"));
    }
    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.phasing must remain downstream of the filtered VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.phasing must remain downstream of VCF QC instead of blocking it",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.phasing must consume the filtered VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "qc_report",
        PROFILE_ID,
        "vcf.phasing must consume the QC report from the completed small-sample branch",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed reference-panel lock contract",
    )?;

    let compatibility_report = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.filter",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.recalibration",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-mini for governed small-sample FASTQ inputs"
            .to_string(),
        "bam.filter and bam.recalibration stay fixture-backed by corpus-01-bam-mini"
            .to_string(),
        "bam.recalibration keeps the low-coverage gate and known-sites contracts explicit"
            .to_string(),
        "vcf.call_diploid consumes both the filtered-BAM fallback and recalibrated-BAM run-path handoffs"
            .to_string(),
        "vcf.call_diploid keeps explicit coverage and recalibration decision evidence"
            .to_string(),
        "vcf.filter and vcf.stats stay downstream of diploid calling".to_string(),
        "vcf.qc stays downstream of filtered VCF and stats without depending on phasing"
            .to_string(),
        "vcf.phasing remains an optional downstream branch with explicit map and panel contracts"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_reference_panel_imputation_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "reference_panel_imputation";

    if config.default_corpus_id != "vcf_production_regression" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed VCF corpus `vcf_production_regression`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let prepare_reference_panel =
        require_profile_stage(&stage_index, PROFILE_ID, "vcf.prepare_reference_panel")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_phasing = require_profile_stage(&stage_index, PROFILE_ID, "vcf.phasing")?;
    let vcf_impute = require_profile_stage(&stage_index, PROFILE_ID, "vcf.impute")?;
    let vcf_imputation = require_profile_stage(&stage_index, PROFILE_ID, "vcf.imputation")?;

    let matrix_index = load_local_vcf_stage_matrix_index(repo_root)?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.prepare_reference_panel",
        "vcf_reference_panel",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.qc",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.phasing",
        "vcf_cohort_with_panel",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.impute",
        "vcf_cohort_with_panel",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.imputation",
        "vcf_cohort_with_panel",
        "vcf_production_regression",
    )?;

    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed panel-id contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_fasta_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference FASTA contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_fai_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference FASTA index contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.external_inputs,
        "reference_dict_contract",
        PROFILE_ID,
        "vcf.prepare_reference_panel must declare the governed reference dictionary contract",
    )?;
    require_list_contains(
        &prepare_reference_panel.outputs,
        "prepared_panel_vcf",
        PROFILE_ID,
        "vcf.prepare_reference_panel must produce the prepared panel VCF handoff",
    )?;
    require_list_contains(
        &prepare_reference_panel.outputs,
        "prepared_panel_vcf_tbi",
        PROFILE_ID,
        "vcf.prepare_reference_panel must produce the prepared panel tabix handoff",
    )?;
    require_list_contains(
        &prepare_reference_panel.outputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.prepare_reference_panel must surface explicit panel identity for downstream phasing and imputation",
    )?;

    require_list_contains(
        &vcf_qc.outputs,
        "qc_target_vcf",
        PROFILE_ID,
        "vcf.qc must keep the target-cohort VCF handoff explicit for optional prephasing",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_target_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must keep the target-cohort tabix handoff explicit for optional prephasing",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "phasing_requirement_decision_json",
        PROFILE_ID,
        "vcf.qc must emit the governed phasing-requirement decision handoff",
    )?;

    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.prepare_reference_panel",
        PROFILE_ID,
        "vcf.phasing must remain downstream of prepared-panel materialization",
    )?;
    require_list_contains(
        &vcf_phasing.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.phasing must remain downstream of target-cohort QC",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed panel-id contract",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &vcf_phasing.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.phasing must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "qc_target_vcf",
        PROFILE_ID,
        "vcf.phasing must consume the QC-qualified target VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "phasing_requirement_decision_json",
        PROFILE_ID,
        "vcf.phasing must consume the explicit phasing decision handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "prepared_panel_vcf",
        PROFILE_ID,
        "vcf.phasing must consume the prepared reference panel VCF handoff",
    )?;
    require_list_contains(
        &vcf_phasing.upstream_inputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.phasing must consume explicit prepared-panel identity",
    )?;

    require_list_contains(
        &vcf_impute.depends_on,
        "vcf.prepare_reference_panel",
        PROFILE_ID,
        "vcf.impute must remain downstream of prepared-panel materialization",
    )?;
    require_list_contains(
        &vcf_impute.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.impute must remain downstream of target-cohort QC",
    )?;
    require_list_contains(
        &vcf_impute.depends_on,
        "vcf.phasing",
        PROFILE_ID,
        "vcf.impute must remain downstream of the optional prephasing branch",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.impute must not run without the governed panel-id contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_fasta_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed imputation reference FASTA contract",
    )?;
    require_list_contains(
        &vcf_impute.external_inputs,
        "reference_fai_contract",
        PROFILE_ID,
        "vcf.impute must declare the governed imputation reference FASTA index contract",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "qc_target_vcf",
        PROFILE_ID,
        "vcf.impute must consume the QC-qualified target VCF fallback handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "phased_vcf",
        PROFILE_ID,
        "vcf.impute must consume the phased VCF run-path handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "phasing_requirement_decision_json",
        PROFILE_ID,
        "vcf.impute must consume the explicit phasing-decision handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "prepared_panel_vcf",
        PROFILE_ID,
        "vcf.impute must consume the prepared panel VCF handoff",
    )?;
    require_list_contains(
        &vcf_impute.upstream_inputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.impute must consume explicit panel identity from prepared-panel outputs",
    )?;

    require_list_contains(
        &vcf_imputation.depends_on,
        "vcf.prepare_reference_panel",
        PROFILE_ID,
        "vcf.imputation must remain downstream of prepared-panel provenance",
    )?;
    require_list_contains(
        &vcf_imputation.depends_on,
        "vcf.impute",
        PROFILE_ID,
        "vcf.imputation must remain downstream of the imputation execution stage",
    )?;
    require_list_contains(
        &vcf_imputation.external_inputs,
        "reference_panel_id_contract",
        PROFILE_ID,
        "vcf.imputation must declare the governed panel-id contract",
    )?;
    require_list_contains(
        &vcf_imputation.external_inputs,
        "reference_panel_lock_contract",
        PROFILE_ID,
        "vcf.imputation must declare the governed reference-panel lock contract",
    )?;
    require_list_contains(
        &vcf_imputation.external_inputs,
        "genetic_map_contract",
        PROFILE_ID,
        "vcf.imputation must declare the governed genetic map contract",
    )?;
    require_list_contains(
        &vcf_imputation.upstream_inputs,
        "prepared_panel_panel_id",
        PROFILE_ID,
        "vcf.imputation must retain explicit panel identity in the downstream metrics branch",
    )?;
    require_list_contains(
        &vcf_imputation.upstream_inputs,
        "imputation_manifest_json",
        PROFILE_ID,
        "vcf.imputation must consume explicit imputation manifest evidence",
    )?;
    require_list_contains(
        &vcf_imputation.upstream_inputs,
        "imputation_qc_json",
        PROFILE_ID,
        "vcf.imputation must consume explicit imputation QC evidence",
    )?;

    let checks = vec![
        "default corpus anchored to vcf_production_regression for governed target-cohort VCF inputs"
            .to_string(),
        "vcf.prepare_reference_panel stays bound to the vcf_reference_panel asset profile"
            .to_string(),
        "vcf.qc stays bound to the vcf_cohort asset profile".to_string(),
        "vcf.phasing, vcf.impute, and vcf.imputation stay bound to the vcf_cohort_with_panel asset profile"
            .to_string(),
        "vcf.prepare_reference_panel keeps panel, map, and reference contracts explicit"
            .to_string(),
        "vcf.phasing consumes the qc-qualified target VCF plus prepared-panel identity for the optional prephasing branch"
            .to_string(),
        "vcf.impute consumes both the qc-target fallback path and phased run path with explicit panel identity"
            .to_string(),
        "vcf.imputation stays downstream of imputation execution with panel-provenance and imputation-qc evidence"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_population_structure_vcf_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "population_structure_vcf";

    if config.default_corpus_id != "vcf_production_regression" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed VCF corpus `vcf_production_regression`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_pca = require_profile_stage(&stage_index, PROFILE_ID, "vcf.pca")?;
    let vcf_admixture = require_profile_stage(&stage_index, PROFILE_ID, "vcf.admixture")?;
    let vcf_population_structure =
        require_profile_stage(&stage_index, PROFILE_ID, "vcf.population_structure")?;

    let matrix_index = load_local_vcf_stage_matrix_index(repo_root)?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.qc",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.pca",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.admixture",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.population_structure",
        "vcf_cohort",
        "vcf_production_regression",
    )?;

    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort VCF input",
    )?;
    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort tabix input",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.qc must export the governed pruning/filtering handoff",
    )?;

    for (stage_id, node) in [("vcf.pca", vcf_pca), ("vcf.admixture", vcf_admixture)] {
        require_list_contains(
            &node.external_inputs,
            "sample_metadata_manifest_contract",
            PROFILE_ID,
            &format!("{stage_id} must declare the governed sample-metadata contract"),
        )?;
        require_list_contains(
            &node.external_inputs,
            "population_labels_contract",
            PROFILE_ID,
            &format!("{stage_id} must declare the governed population-label contract"),
        )?;
        require_list_contains(
            &node.external_inputs,
            "population_metadata_manifest_contract",
            PROFILE_ID,
            &format!("{stage_id} must declare the governed population-metadata contract"),
        )?;
        require_list_contains(
            &node.depends_on,
            "vcf.qc",
            PROFILE_ID,
            &format!("{stage_id} must remain downstream of cohort QC and pruning/filtering"),
        )?;
        require_list_contains(
            &node.upstream_inputs,
            "qc_cohort_vcf",
            PROFILE_ID,
            &format!("{stage_id} must consume the QC-qualified cohort VCF handoff"),
        )?;
        require_list_contains(
            &node.upstream_inputs,
            "pruned_variants_tsv",
            PROFILE_ID,
            &format!("{stage_id} must consume the governed pruning/filtering handoff"),
        )?;
    }

    require_list_contains(
        &vcf_pca.outputs,
        "pca_report",
        PROFILE_ID,
        "vcf.pca must export the normalized PCA report",
    )?;
    require_list_contains(
        &vcf_pca.outputs,
        "pca_metadata_join_tsv",
        PROFILE_ID,
        "vcf.pca must export a sample-metadata join handoff",
    )?;

    require_list_contains(
        &vcf_admixture.depends_on,
        "vcf.pca",
        PROFILE_ID,
        "vcf.admixture must remain downstream of PCA in the governed population-structure branch",
    )?;
    require_list_contains(
        &vcf_admixture.upstream_inputs,
        "pca_report",
        PROFILE_ID,
        "vcf.admixture must consume explicit PCA evidence",
    )?;
    require_list_contains(
        &vcf_admixture.outputs,
        "admixture_report",
        PROFILE_ID,
        "vcf.admixture must export the normalized admixture report",
    )?;
    require_list_contains(
        &vcf_admixture.outputs,
        "admixture_metadata_join_tsv",
        PROFILE_ID,
        "vcf.admixture must export a sample-metadata join handoff",
    )?;

    require_list_contains(
        &vcf_population_structure.external_inputs,
        "sample_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.population_structure must declare the governed sample-metadata contract",
    )?;
    require_list_contains(
        &vcf_population_structure.external_inputs,
        "population_labels_contract",
        PROFILE_ID,
        "vcf.population_structure must declare the governed population-label contract",
    )?;
    require_list_contains(
        &vcf_population_structure.external_inputs,
        "population_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.population_structure must declare the governed population-metadata contract",
    )?;
    require_list_contains(
        &vcf_population_structure.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.population_structure must remain downstream of cohort QC",
    )?;
    require_list_contains(
        &vcf_population_structure.depends_on,
        "vcf.pca",
        PROFILE_ID,
        "vcf.population_structure must remain downstream of PCA",
    )?;
    require_list_contains(
        &vcf_population_structure.depends_on,
        "vcf.admixture",
        PROFILE_ID,
        "vcf.population_structure must remain downstream of admixture",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "pca_report",
        PROFILE_ID,
        "vcf.population_structure must consume PCA evidence",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "admixture_report",
        PROFILE_ID,
        "vcf.population_structure must consume admixture evidence",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "pca_metadata_join_tsv",
        PROFILE_ID,
        "vcf.population_structure must consume the PCA sample-metadata join handoff",
    )?;
    require_list_contains(
        &vcf_population_structure.upstream_inputs,
        "admixture_metadata_join_tsv",
        PROFILE_ID,
        "vcf.population_structure must consume the admixture sample-metadata join handoff",
    )?;

    let checks = vec![
        "default corpus anchored to vcf_production_regression for governed cohort-vcf population-structure inputs".to_string(),
        "vcf.qc, vcf.pca, vcf.admixture, and vcf.population_structure stay bound to the vcf_cohort asset profile".to_string(),
        "vcf.qc exports the governed pruning/filtering handoff for downstream cohort analysis".to_string(),
        "vcf.pca requires sample metadata, population metadata, and population labels as explicit inputs".to_string(),
        "vcf.admixture requires sample metadata, population metadata, and population labels as explicit inputs".to_string(),
        "vcf.admixture stays downstream of pca with explicit pca evidence".to_string(),
        "vcf.pca and vcf.admixture both export sample-metadata join handoffs".to_string(),
        "vcf.population_structure consumes the pca and admixture metadata-join handoffs with mandatory labels".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_relatedness_segments_vcf_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "relatedness_segments_vcf";

    if config.default_corpus_id != "vcf_production_regression" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed VCF corpus `vcf_production_regression`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_ibd = require_profile_stage(&stage_index, PROFILE_ID, "vcf.ibd")?;
    let vcf_roh = require_profile_stage(&stage_index, PROFILE_ID, "vcf.roh")?;
    let vcf_demography = require_profile_stage(&stage_index, PROFILE_ID, "vcf.demography")?;

    let matrix_index = load_local_vcf_stage_matrix_index(repo_root)?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.qc",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.ibd",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.roh",
        "vcf_cohort",
        "vcf_production_regression",
    )?;
    require_vcf_matrix_binding(
        &matrix_index,
        PROFILE_ID,
        "vcf.demography",
        "json_ibd_segments",
        "vcf_production_regression",
    )?;

    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort VCF input",
    )?;
    require_list_contains(
        &vcf_qc.external_inputs,
        "corpus.target_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must declare the governed target-cohort tabix input",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must export the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_report",
        PROFILE_ID,
        "vcf.qc must export explicit QC evidence for downstream segment analysis",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.qc must export the governed pruning and filtering handoff",
    )?;

    require_list_contains(
        &vcf_ibd.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.ibd must remain downstream of cohort QC",
    )?;
    require_list_contains(
        &vcf_ibd.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.ibd must consume the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_ibd.upstream_inputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.ibd must consume the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_ibd.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.ibd must consume the governed pruning and filtering handoff",
    )?;
    require_list_contains(
        &vcf_ibd.outputs,
        "ibd_segments",
        PROFILE_ID,
        "vcf.ibd must export the normalized IBD segment handoff",
    )?;
    require_list_contains(
        &vcf_ibd.outputs,
        "ibd_report",
        PROFILE_ID,
        "vcf.ibd must export the normalized IBD report",
    )?;
    require_list_contains(
        &vcf_ibd.outputs,
        "ibd_insufficient_data_json",
        PROFILE_ID,
        "vcf.ibd must export explicit insufficiency evidence for downstream demography",
    )?;

    require_list_contains(
        &vcf_roh.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.roh must remain downstream of cohort QC",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.roh must consume the QC-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.roh must consume the QC-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.roh must consume the governed pruning and filtering handoff",
    )?;
    require_list_contains(
        &vcf_roh.outputs,
        "roh_report",
        PROFILE_ID,
        "vcf.roh must export the normalized ROH report",
    )?;
    require_list_contains(
        &vcf_roh.outputs,
        "roh_segments_tsv",
        PROFILE_ID,
        "vcf.roh must export explicit ROH segment rows",
    )?;
    if vcf_roh.depends_on.iter().any(|value| value == "vcf.ibd")
        || vcf_roh.upstream_inputs.iter().any(|value| value == "ibd_segments")
    {
        return Err(anyhow!(
            "{PROFILE_ID}: vcf.roh must stay independent of the IBD insufficiency branch"
        ));
    }

    require_list_contains(
        &vcf_demography.depends_on,
        "vcf.ibd",
        PROFILE_ID,
        "vcf.demography must remain downstream of IBD inference",
    )?;
    require_list_contains(
        &vcf_demography.upstream_inputs,
        "ibd_segments",
        PROFILE_ID,
        "vcf.demography must consume the normalized IBD segment handoff",
    )?;
    require_list_contains(
        &vcf_demography.upstream_inputs,
        "ibd_report",
        PROFILE_ID,
        "vcf.demography must consume explicit IBD report evidence",
    )?;
    require_list_contains(
        &vcf_demography.upstream_inputs,
        "ibd_insufficient_data_json",
        PROFILE_ID,
        "vcf.demography must consume explicit IBD insufficiency evidence",
    )?;
    require_list_contains(
        &vcf_demography.outputs,
        "demography_report",
        PROFILE_ID,
        "vcf.demography must export the normalized demography report",
    )?;
    require_list_contains(
        &vcf_demography.outputs,
        "demography_insufficient_data_json",
        PROFILE_ID,
        "vcf.demography must export explicit downstream insufficiency evidence",
    )?;
    if vcf_demography.depends_on.iter().any(|value| value == "vcf.roh")
        || vcf_demography.upstream_inputs.iter().any(|value| value == "roh_report")
    {
        return Err(anyhow!(
            "{PROFILE_ID}: vcf.demography must stay local to the IBD branch and must not consume ROH outputs"
        ));
    }

    let checks = vec![
        "default corpus anchored to vcf_production_regression for governed cohort-vcf relatedness inputs".to_string(),
        "vcf.qc, vcf.ibd, and vcf.roh stay bound to the vcf_cohort asset profile".to_string(),
        "vcf.demography stays bound to the json_ibd_segments asset profile".to_string(),
        "vcf.qc exports the qc-qualified cohort and pruning handoffs for downstream segment analysis".to_string(),
        "vcf.ibd stays downstream of qc with explicit insufficiency evidence".to_string(),
        "vcf.roh stays downstream of qc without depending on ibd outputs".to_string(),
        "vcf.demography stays downstream only of ibd with explicit insufficiency propagation".to_string(),
        "ibd insufficiency remains local to demography and does not block roh or qc outputs".to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_bam_genotyping_vcf_downstream_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "bam_genotyping_vcf_downstream";

    if config.default_corpus_id != "corpus-01-bam-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed BAM corpus `corpus-01-bam-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let bam_filter = require_profile_stage(&stage_index, PROFILE_ID, "bam.filter")?;
    let bam_coverage = require_profile_stage(&stage_index, PROFILE_ID, "bam.coverage")?;
    let bam_recalibration = require_profile_stage(&stage_index, PROFILE_ID, "bam.recalibration")?;
    let bam_genotyping = require_profile_stage(&stage_index, PROFILE_ID, "bam.genotyping")?;
    let vcf_filter = require_profile_stage(&stage_index, PROFILE_ID, "vcf.filter")?;
    let vcf_stats = require_profile_stage(&stage_index, PROFILE_ID, "vcf.stats")?;
    let vcf_qc = require_profile_stage(&stage_index, PROFILE_ID, "vcf.qc")?;
    let vcf_pca = require_profile_stage(&stage_index, PROFILE_ID, "vcf.pca")?;
    let vcf_roh = require_profile_stage(&stage_index, PROFILE_ID, "vcf.roh")?;

    let compatibility_report = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.filter",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.coverage",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.recalibration",
        "corpus-01-bam",
        "corpus-01-bam-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "bam.genotyping",
        "corpus-01-genotyping",
        "corpus-01-genotyping-mini",
    )?;

    require_list_contains(
        &bam_filter.outputs,
        "filtered_bam",
        PROFILE_ID,
        "bam.filter must export the governed filtered BAM handoff",
    )?;
    require_list_contains(
        &bam_filter.outputs,
        "filtered_bai",
        PROFILE_ID,
        "bam.filter must export the governed filtered BAM index handoff",
    )?;
    require_list_contains(
        &bam_coverage.depends_on,
        "bam.filter",
        PROFILE_ID,
        "bam.coverage must remain downstream of bam.filter",
    )?;
    require_list_contains(
        &bam_coverage.external_inputs,
        "coverage_region_contract",
        PROFILE_ID,
        "bam.coverage must declare the governed coverage-region contract",
    )?;
    require_list_contains(
        &bam_recalibration.depends_on,
        "bam.filter",
        PROFILE_ID,
        "bam.recalibration must remain downstream of bam.filter",
    )?;
    require_list_contains(
        &bam_recalibration.depends_on,
        "bam.coverage",
        PROFILE_ID,
        "bam.recalibration must remain downstream of bam.coverage",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_reference_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration reference contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_known_sites_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration known-sites contract",
    )?;
    require_list_contains(
        &bam_recalibration.external_inputs,
        "recalibration_coverage_gate_contract",
        PROFILE_ID,
        "bam.recalibration must declare the governed recalibration coverage gate contract",
    )?;

    require_list_contains(
        &bam_genotyping.depends_on,
        "bam.filter",
        PROFILE_ID,
        "bam.genotyping must remain downstream of bam.filter",
    )?;
    require_list_contains(
        &bam_genotyping.depends_on,
        "bam.recalibration",
        PROFILE_ID,
        "bam.genotyping must remain downstream of bam.recalibration",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "filtered_bam",
        PROFILE_ID,
        "bam.genotyping must consume the governed filtered BAM fallback handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "filtered_bai",
        PROFILE_ID,
        "bam.genotyping must consume the governed filtered BAM index fallback handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "recalibrated_bam",
        PROFILE_ID,
        "bam.genotyping must consume the governed recalibrated BAM run-path handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "recalibrated_bai",
        PROFILE_ID,
        "bam.genotyping must consume the governed recalibrated BAM index run-path handoff",
    )?;
    require_list_contains(
        &bam_genotyping.upstream_inputs,
        "recalibration_summary_json",
        PROFILE_ID,
        "bam.genotyping must consume the governed recalibration decision summary",
    )?;
    require_list_contains(
        &bam_genotyping.external_inputs,
        "genotyping_reference_contract",
        PROFILE_ID,
        "bam.genotyping must declare the governed genotyping reference contract",
    )?;
    require_list_contains(
        &bam_genotyping.external_inputs,
        "candidate_sites_vcf_contract",
        PROFILE_ID,
        "bam.genotyping must declare the governed candidate-sites VCF contract",
    )?;
    require_list_contains(
        &bam_genotyping.external_inputs,
        "target_regions_contract",
        PROFILE_ID,
        "bam.genotyping must declare the governed target-regions contract",
    )?;
    require_list_contains(
        &bam_genotyping.outputs,
        "genotyping_bcf",
        PROFILE_ID,
        "bam.genotyping must export the governed BCF bridge artifact",
    )?;
    require_list_contains(
        &bam_genotyping.outputs,
        "genotyping_vcf_gz",
        PROFILE_ID,
        "bam.genotyping must export the governed gzipped VCF bridge artifact",
    )?;
    require_list_contains(
        &bam_genotyping.outputs,
        "genotyping_vcf_tbi",
        PROFILE_ID,
        "bam.genotyping must export the governed tabix bridge artifact",
    )?;

    require_list_contains(
        &vcf_filter.depends_on,
        "bam.genotyping",
        PROFILE_ID,
        "vcf.filter must remain downstream of bam.genotyping",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "genotyping_vcf_gz",
        PROFILE_ID,
        "vcf.filter must consume the exact bam.genotyping VCF handoff",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "genotyping_vcf_tbi",
        PROFILE_ID,
        "vcf.filter must consume the exact bam.genotyping tabix handoff",
    )?;
    require_list_contains(
        &vcf_filter.upstream_inputs,
        "genotyping_report_json",
        PROFILE_ID,
        "vcf.filter must consume explicit bam.genotyping bridge evidence",
    )?;
    if vcf_filter
        .external_inputs
        .iter()
        .any(|value| value == "corpus.target_cohort_vcf" || value == "corpus.target_cohort_vcf_tbi")
    {
        return Err(anyhow!(
            "{PROFILE_ID}: vcf.filter must bridge from bam.genotyping outputs instead of a conceptual external cohort-vcf placeholder"
        ));
    }

    require_list_contains(
        &vcf_stats.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.stats must remain downstream of the filtered VCF bridge",
    )?;
    require_list_contains(
        &vcf_stats.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.stats must consume the filtered VCF bridge handoff",
    )?;
    require_list_contains(
        &vcf_stats.upstream_inputs,
        "filtered_vcf_tbi",
        PROFILE_ID,
        "vcf.stats must consume the filtered VCF bridge tabix handoff",
    )?;

    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.filter",
        PROFILE_ID,
        "vcf.qc must remain downstream of filtered VCF outputs",
    )?;
    require_list_contains(
        &vcf_qc.depends_on,
        "vcf.stats",
        PROFILE_ID,
        "vcf.qc must remain downstream of explicit VCF stats evidence",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "filtered_vcf",
        PROFILE_ID,
        "vcf.qc must consume the filtered VCF bridge handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "filtered_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must consume the filtered VCF bridge tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.upstream_inputs,
        "stats_json",
        PROFILE_ID,
        "vcf.qc must consume explicit VCF stats evidence",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.qc must export the qc-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "qc_cohort_vcf_tbi",
        PROFILE_ID,
        "vcf.qc must export the qc-qualified cohort tabix handoff",
    )?;
    require_list_contains(
        &vcf_qc.outputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.qc must export the governed pruning handoff for optional downstream branches",
    )?;

    require_list_contains(
        &vcf_pca.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.pca must remain downstream of vcf.qc",
    )?;
    require_list_contains(
        &vcf_pca.external_inputs,
        "sample_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.pca must declare the governed sample-metadata contract",
    )?;
    require_list_contains(
        &vcf_pca.external_inputs,
        "population_metadata_manifest_contract",
        PROFILE_ID,
        "vcf.pca must declare the governed population-metadata contract",
    )?;
    require_list_contains(
        &vcf_pca.external_inputs,
        "population_labels_contract",
        PROFILE_ID,
        "vcf.pca must declare the governed population-label contract",
    )?;
    require_list_contains(
        &vcf_pca.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.pca must consume the qc-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_pca.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.pca must consume the governed pruning handoff",
    )?;

    require_list_contains(
        &vcf_roh.depends_on,
        "vcf.qc",
        PROFILE_ID,
        "vcf.roh must remain downstream of vcf.qc",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "qc_cohort_vcf",
        PROFILE_ID,
        "vcf.roh must consume the qc-qualified cohort VCF handoff",
    )?;
    require_list_contains(
        &vcf_roh.upstream_inputs,
        "pruned_variants_tsv",
        PROFILE_ID,
        "vcf.roh must consume the governed pruning handoff",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-01-bam-mini for governed bam-genotyping bridge inputs"
            .to_string(),
        "bam.filter, bam.coverage, and bam.recalibration stay fixture-backed by corpus-01-bam-mini while bam.genotyping stays fixture-backed by corpus-01-genotyping-mini"
            .to_string(),
        "bam.genotyping keeps the filtered fallback path, recalibrated run path, and governed genotyping contracts explicit"
            .to_string(),
        "vcf.filter consumes exact bam.genotyping vcf handoffs instead of a conceptual cohort-vcf placeholder"
            .to_string(),
        "vcf.stats stays downstream of the filtered vcf bridge".to_string(),
        "vcf.qc stays downstream of filtered vcf and stats while exporting qc-qualified cohort outputs"
            .to_string(),
        "optional vcf.pca stays downstream of qc with metadata contracts and pruned cohort handoffs"
            .to_string(),
        "optional vcf.roh stays downstream of qc with the same qc-qualified cohort handoff"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_edna_taxonomy_no_vcf_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "edna_taxonomy_no_vcf";

    if config
        .nodes
        .iter()
        .any(|node| node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf."))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: taxonomy-only pipeline must not admit BAM or VCF germline stages"
        ));
    }
    if !matches!(config.domain, LocalPipelineDagDomain::Fastq) {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must stay in the governed `fastq` domain, found `{}`",
            config.domain.as_str()
        ));
    }
    if config.default_corpus_id != "corpus-02-edna-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed eDNA corpus `corpus-02-edna-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let validate_reads = require_profile_stage(&stage_index, PROFILE_ID, "fastq.validate_reads")?;
    let _filter_reads = require_profile_stage(&stage_index, PROFILE_ID, "fastq.filter_reads")?;
    let screen_taxonomy = require_profile_stage(&stage_index, PROFILE_ID, "fastq.screen_taxonomy")?;
    let report_qc = require_profile_stage(&stage_index, PROFILE_ID, "fastq.report_qc")?;

    require_list_contains(
        &validate_reads.external_inputs,
        "corpus.raw_fastq_reads",
        PROFILE_ID,
        "fastq.validate_reads must declare the governed eDNA FASTQ corpus input",
    )?;
    require_list_contains(
        &screen_taxonomy.depends_on,
        "fastq.filter_reads",
        PROFILE_ID,
        "fastq.screen_taxonomy must remain downstream of FASTQ filtering",
    )?;
    require_list_contains(
        &screen_taxonomy.upstream_inputs,
        "filtered_reads",
        PROFILE_ID,
        "fastq.screen_taxonomy must consume the filtered FASTQ handoff",
    )?;
    require_list_contains(
        &screen_taxonomy.external_inputs,
        "taxonomy_database.root",
        PROFILE_ID,
        "fastq.screen_taxonomy must declare the governed taxonomy database root",
    )?;
    require_list_contains(
        &screen_taxonomy.external_inputs,
        "taxonomy_expected_truth_table",
        PROFILE_ID,
        "fastq.screen_taxonomy must declare the governed expected-taxa contract",
    )?;
    require_list_contains(
        &screen_taxonomy.outputs,
        "taxonomy_classification",
        PROFILE_ID,
        "fastq.screen_taxonomy must keep the classified taxonomy output explicit",
    )?;
    require_list_contains(
        &screen_taxonomy.outputs,
        "unclassified_reads",
        PROFILE_ID,
        "fastq.screen_taxonomy must keep the unclassified-read output explicit",
    )?;
    require_list_contains(
        &screen_taxonomy.outputs,
        "taxonomy_summary",
        PROFILE_ID,
        "fastq.screen_taxonomy must keep the taxonomy summary output explicit",
    )?;
    require_list_contains(
        &report_qc.depends_on,
        "fastq.screen_taxonomy",
        PROFILE_ID,
        "fastq.report_qc must remain downstream of taxonomy screening",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "taxonomy_summary",
        PROFILE_ID,
        "fastq.report_qc must consume the taxonomy summary handoff",
    )?;

    if report_qc.upstream_inputs.iter().any(|value| value.contains("vcf") || value.contains("bam"))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: taxonomy reporting must not mix BAM or VCF germline outputs into the final report"
        ));
    }

    for node in &config.nodes {
        if node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf.") {
            if node.upstream_inputs.iter().any(|value| {
                value == "taxonomy_classification"
                    || value == "unclassified_reads"
                    || value == "taxonomy_summary"
            }) {
                return Err(anyhow!(
                    "{PROFILE_ID}: taxonomy outputs must not bridge into BAM or VCF germline stages"
                ));
            }
        }
    }

    let compatibility_report = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.screen_taxonomy",
        "corpus-02",
        "corpus-02-edna-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-02-edna-mini for governed edna taxonomy inputs"
            .to_string(),
        "pipeline remains fastq-only and rejects bam or vcf germline stages".to_string(),
        "fastq.validate_reads keeps the governed raw-fastq corpus handoff explicit".to_string(),
        "fastq.screen_taxonomy stays fixture-backed by corpus-02-edna-mini".to_string(),
        "fastq.screen_taxonomy remains downstream of filtered reads".to_string(),
        "fastq.screen_taxonomy keeps taxonomy database and expected-truth contracts explicit"
            .to_string(),
        "fastq.screen_taxonomy keeps classification, unclassified, and summary outputs explicit"
            .to_string(),
        "fastq.report_qc consumes taxonomy summary without mixing bam or vcf germline outputs"
            .to_string(),
    ];
    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn validate_amplicon_asv_otu_no_vcf_profile(
    repo_root: &Path,
    config: &LocalPipelineDagConfig,
) -> Result<LocalPipelineDagValidationProfileReport> {
    const PROFILE_ID: &str = "amplicon_asv_otu_no_vcf";

    if config
        .nodes
        .iter()
        .any(|node| node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf."))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: amplicon-only pipeline must not admit BAM or VCF germline stages"
        ));
    }
    if !matches!(config.domain, LocalPipelineDagDomain::Fastq) {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must stay in the governed `fastq` domain, found `{}`",
            config.domain.as_str()
        ));
    }
    if config.default_corpus_id != "corpus-03-amplicon-mini" {
        return Err(anyhow!(
            "{PROFILE_ID} local pipeline DAG must start from governed amplicon corpus `corpus-03-amplicon-mini`, found `{}`",
            config.default_corpus_id
        ));
    }

    let stage_index =
        config.nodes.iter().map(|node| (node.stage_id.clone(), node)).collect::<BTreeMap<_, _>>();

    let validate_reads = require_profile_stage(&stage_index, PROFILE_ID, "fastq.validate_reads")?;
    let normalize_primers =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.normalize_primers")?;
    let infer_asvs = require_profile_stage(&stage_index, PROFILE_ID, "fastq.infer_asvs")?;
    let remove_chimeras = require_profile_stage(&stage_index, PROFILE_ID, "fastq.remove_chimeras")?;
    let cluster_otus = require_profile_stage(&stage_index, PROFILE_ID, "fastq.cluster_otus")?;
    let normalize_abundance =
        require_profile_stage(&stage_index, PROFILE_ID, "fastq.normalize_abundance")?;
    let report_qc = require_profile_stage(&stage_index, PROFILE_ID, "fastq.report_qc")?;

    require_list_contains(
        &validate_reads.external_inputs,
        "corpus.raw_amplicon_fastq_reads",
        PROFILE_ID,
        "fastq.validate_reads must declare the governed amplicon FASTQ corpus input",
    )?;
    require_list_contains(
        &normalize_primers.external_inputs,
        "amplicon_primer_contract",
        PROFILE_ID,
        "fastq.normalize_primers must declare the governed primer contract",
    )?;
    require_list_contains(
        &normalize_primers.upstream_inputs,
        "validated_amplicon_reads",
        PROFILE_ID,
        "fastq.normalize_primers must consume validated amplicon reads",
    )?;
    require_list_contains(
        &infer_asvs.depends_on,
        "fastq.normalize_primers",
        PROFILE_ID,
        "fastq.infer_asvs must remain downstream of primer normalization",
    )?;
    require_list_contains(
        &infer_asvs.upstream_inputs,
        "normalized_amplicon_reads",
        PROFILE_ID,
        "fastq.infer_asvs must consume normalized amplicon reads",
    )?;
    require_list_contains(
        &infer_asvs.outputs,
        "asv_table",
        PROFILE_ID,
        "fastq.infer_asvs must keep the ASV table output explicit",
    )?;
    require_list_contains(
        &infer_asvs.outputs,
        "asv_representatives",
        PROFILE_ID,
        "fastq.infer_asvs must keep the ASV representatives output explicit",
    )?;
    require_list_contains(
        &remove_chimeras.depends_on,
        "fastq.infer_asvs",
        PROFILE_ID,
        "fastq.remove_chimeras must remain downstream of ASV inference",
    )?;
    require_list_contains(
        &remove_chimeras.external_inputs,
        "chimera_control_contract",
        PROFILE_ID,
        "fastq.remove_chimeras must declare the governed chimera-control contract",
    )?;
    require_list_contains(
        &remove_chimeras.upstream_inputs,
        "asv_representatives",
        PROFILE_ID,
        "fastq.remove_chimeras must consume ASV representatives",
    )?;
    require_list_contains(
        &remove_chimeras.outputs,
        "non_chimeric_representatives",
        PROFILE_ID,
        "fastq.remove_chimeras must keep non-chimeric representatives explicit",
    )?;
    require_list_contains(
        &cluster_otus.depends_on,
        "fastq.normalize_primers",
        PROFILE_ID,
        "fastq.cluster_otus must remain downstream of primer normalization",
    )?;
    require_list_contains(
        &cluster_otus.depends_on,
        "fastq.remove_chimeras",
        PROFILE_ID,
        "fastq.cluster_otus must remain downstream of chimera removal",
    )?;
    require_list_contains(
        &cluster_otus.upstream_inputs,
        "normalized_amplicon_reads",
        PROFILE_ID,
        "fastq.cluster_otus must consume normalized amplicon reads",
    )?;
    require_list_contains(
        &cluster_otus.upstream_inputs,
        "non_chimeric_representatives",
        PROFILE_ID,
        "fastq.cluster_otus must consume non-chimeric representatives",
    )?;
    require_list_contains(
        &cluster_otus.outputs,
        "otu_table",
        PROFILE_ID,
        "fastq.cluster_otus must keep the OTU table output explicit",
    )?;
    require_list_contains(
        &normalize_abundance.depends_on,
        "fastq.cluster_otus",
        PROFILE_ID,
        "fastq.normalize_abundance must remain downstream of OTU clustering",
    )?;
    require_list_contains(
        &normalize_abundance.upstream_inputs,
        "otu_table",
        PROFILE_ID,
        "fastq.normalize_abundance must consume the OTU table handoff",
    )?;
    require_list_contains(
        &normalize_abundance.outputs,
        "normalized_abundance_table",
        PROFILE_ID,
        "fastq.normalize_abundance must keep the normalized abundance output explicit",
    )?;
    require_list_contains(
        &report_qc.depends_on,
        "fastq.normalize_abundance",
        PROFILE_ID,
        "fastq.report_qc must remain downstream of abundance normalization",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "asv_metrics",
        PROFILE_ID,
        "fastq.report_qc must consume ASV metrics",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "otu_table",
        PROFILE_ID,
        "fastq.report_qc must consume OTU table evidence",
    )?;
    require_list_contains(
        &report_qc.upstream_inputs,
        "normalized_abundance_table",
        PROFILE_ID,
        "fastq.report_qc must consume normalized abundance evidence",
    )?;

    if report_qc.upstream_inputs.iter().any(|value| value.contains("vcf") || value.contains("bam"))
    {
        return Err(anyhow!(
            "{PROFILE_ID}: amplicon reporting must not mix BAM or VCF germline outputs into the final report"
        ));
    }

    for node in &config.nodes {
        if node.stage_id.starts_with("bam.") || node.stage_id.starts_with("vcf.") {
            if node.upstream_inputs.iter().any(|value| {
                value == "asv_table"
                    || value == "asv_representatives"
                    || value == "otu_table"
                    || value == "otu_representatives"
                    || value == "normalized_abundance_table"
            }) {
                return Err(anyhow!(
                    "{PROFILE_ID}: ASV or OTU outputs must not bridge into BAM or VCF germline stages"
                ));
            }
        }
    }

    let compatibility_report = validate_corpus_stage_compatibility_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_STAGE_COMPATIBILITY_PATH),
    )?;
    let compatibility_index = compatibility_report
        .stages
        .iter()
        .map(|entry| (entry.stage_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.normalize_primers",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.infer_asvs",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.cluster_otus",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;
    require_compatibility_fixture(
        &compatibility_index,
        PROFILE_ID,
        "fastq.normalize_abundance",
        "corpus-03",
        "corpus-03-amplicon-mini",
    )?;

    let checks = vec![
        "default corpus anchored to corpus-03-amplicon-mini for governed amplicon inputs"
            .to_string(),
        "pipeline remains fastq-only and rejects bam or vcf germline stages".to_string(),
        "fastq.normalize_primers, fastq.infer_asvs, fastq.cluster_otus, and fastq.normalize_abundance stay fixture-backed by corpus-03-amplicon-mini"
            .to_string(),
        "fastq.normalize_primers keeps the governed primer contract explicit".to_string(),
        "fastq.infer_asvs and fastq.remove_chimeras keep ASV and chimera handoffs explicit"
            .to_string(),
        "fastq.cluster_otus consumes normalized reads plus non-chimeric representatives".to_string(),
        "fastq.normalize_abundance stays downstream of OTU clustering with an explicit abundance table output"
            .to_string(),
        "fastq.report_qc consumes ASV, OTU, and abundance evidence without mixing bam or vcf germline outputs"
            .to_string(),
    ];

    Ok(LocalPipelineDagValidationProfileReport {
        profile_id: PROFILE_ID.to_string(),
        check_count: checks.len(),
        checks,
    })
}

fn require_profile_stage<'a>(
    stage_index: &'a BTreeMap<String, &'a LocalPipelineDagNode>,
    profile_id: &str,
    stage_id: &str,
) -> Result<&'a LocalPipelineDagNode> {
    stage_index
        .get(stage_id)
        .copied()
        .ok_or_else(|| anyhow!("{profile_id} local pipeline DAG must declare stage `{stage_id}`"))
}

fn require_list_contains(
    values: &[String],
    expected: &str,
    profile_id: &str,
    message: &str,
) -> Result<()> {
    if values.iter().any(|value| value == expected) {
        Ok(())
    } else {
        Err(anyhow!("{profile_id}: {message}"))
    }
}

fn require_compatibility_fixture(
    compatibility_index: &BTreeMap<&str, &LocalCorpusStageCompatibilityEntryReport>,
    profile_id: &str,
    stage_id: &str,
    expected_corpus_family_id: &str,
    expected_fixture_id: &str,
) -> Result<()> {
    let Some(entry) = compatibility_index.get(stage_id).copied() else {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG cannot confirm corpus compatibility for missing stage `{stage_id}`"
        ));
    };
    if entry.corpus_family_id.as_deref() != Some(expected_corpus_family_id)
        || entry.fixture_id.as_deref() != Some(expected_fixture_id)
    {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG expected `{stage_id}` to remain governed by `{expected_corpus_family_id}` / `{expected_fixture_id}`, found {:?} / {:?}",
            entry.corpus_family_id,
            entry.fixture_id
        ));
    }
    Ok(())
}

fn load_pipeline_inventory_index(
    repo_root: &Path,
    domain: LocalPipelineDagDomain,
) -> Result<BTreeMap<String, LocalStageReadinessKind>> {
    let mut inventory_index = BTreeMap::new();
    let inventory_domains = match domain {
        LocalPipelineDagDomain::Fastq => vec![BenchLocalDomain::Fastq],
        LocalPipelineDagDomain::Bam => vec![BenchLocalDomain::Bam],
        LocalPipelineDagDomain::Vcf => Vec::new(),
        LocalPipelineDagDomain::Cross => vec![BenchLocalDomain::Fastq, BenchLocalDomain::Bam],
    };

    for inventory_domain in inventory_domains {
        let inventory = load_local_stage_inventory(repo_root, inventory_domain)?;
        for stage in inventory.stages {
            inventory_index.insert(stage.stage_id, stage.readiness_kind);
        }
    }

    if matches!(domain, LocalPipelineDagDomain::Vcf | LocalPipelineDagDomain::Cross) {
        inventory_index.extend(load_local_vcf_pipeline_inventory_index(repo_root)?);
    }

    Ok(inventory_index)
}

fn load_local_vcf_pipeline_inventory_index(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalStageReadinessKind>> {
    let matrix_path = repo_root.join("benchmarks/configs/local/vcf-stage-matrix.toml");
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != "bijux.bench.vcf.local_stage_matrix.v1" {
        return Err(anyhow!(
            "{} declares `{}` but `bijux.bench.vcf.local_stage_matrix.v1` is required for `vcf` local pipeline inventory",
            matrix_path.display(),
            matrix.schema_version
        ));
    }

    let mut inventory_index = BTreeMap::new();
    for row in matrix.rows {
        inventory_index.entry(row.stage_id).or_insert(LocalStageReadinessKind::Smoke);
    }
    Ok(inventory_index)
}

fn load_local_vcf_stage_matrix_index(
    repo_root: &Path,
) -> Result<BTreeMap<String, VcfStageMatrixRow>> {
    let matrix_path = repo_root.join("benchmarks/configs/local/vcf-stage-matrix.toml");
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != "bijux.bench.vcf.local_stage_matrix.v1" {
        return Err(anyhow!(
            "{} declares `{}` but `bijux.bench.vcf.local_stage_matrix.v1` is required for VCF stage-matrix validation",
            matrix_path.display(),
            matrix.schema_version
        ));
    }

    Ok(matrix.rows.into_iter().map(|row| (row.stage_id.clone(), row)).collect::<BTreeMap<_, _>>())
}

fn require_vcf_matrix_binding(
    matrix_index: &BTreeMap<String, VcfStageMatrixRow>,
    profile_id: &str,
    stage_id: &str,
    expected_asset_profile_id: &str,
    expected_corpus_id: &str,
) -> Result<()> {
    let Some(row) = matrix_index.get(stage_id) else {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG cannot confirm governed VCF stage-matrix coverage for missing stage `{stage_id}`"
        ));
    };
    if row.asset_profile_id != expected_asset_profile_id || row.corpus_id != expected_corpus_id {
        return Err(anyhow!(
            "{profile_id} local pipeline DAG expected `{stage_id}` to remain governed by asset profile `{expected_asset_profile_id}` and corpus `{expected_corpus_id}`, found `{}` / `{}`",
            row.asset_profile_id,
            row.corpus_id
        ));
    }
    Ok(())
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
    use std::fs;

    fn repo_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn cross_pipeline_dag_can_mix_fastq_and_bam_inventory_nodes() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("fastq-to-bam-cross.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "fastq-to-bam-cross"
domain = "cross"
summary = "Cross-domain proof that local pipeline validation can mix governed FASTQ and BAM stages."
default_corpus_id = "corpus-01-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed FASTQ inputs before alignment."
depends_on = []
external_inputs = ["corpus.raw_fastq_reads"]
upstream_inputs = []
outputs = ["validated_reads_r1_path", "validated_reads_r2_path", "validation_report"]

[[nodes]]
node_id = "bam.align"
stage_id = "bam.align"
readiness_kind = "dry_or_smoke"
summary = "Align validated FASTQ paths against the governed BAM alignment contracts."
depends_on = ["fastq.validate_reads"]
external_inputs = [
  "alignment_reference_fasta_contract",
  "alignment_reference_index_contract",
  "alignment_read_group_contract",
]
upstream_inputs = ["validated_reads_r1_path", "validated_reads_r2_path"]
outputs = ["align_bam", "align_bai", "align_metrics"]
"#,
        )
        .expect("write cross config");

        let output_path = tempdir.path().join("fastq-to-bam-cross.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate cross local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-to-bam-cross");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.node_count, 2);
        assert_eq!(report.edge_count, 1);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.align"
                    && node.upstream_inputs
                        == vec!["validated_reads_r1_path", "validated_reads_r2_path"]
            }),
            "cross DAG validation must admit BAM alignment consumers of governed FASTQ path outputs"
        );
    }

    #[test]
    fn cross_pipeline_dag_can_mix_fastq_bam_and_vcf_inventory_nodes() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("fastq-bam-vcf-cross.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "fastq-bam-vcf-cross"
domain = "cross"
summary = "Cross-domain proof that local pipeline validation can mix governed FASTQ, BAM, and VCF stages."
default_corpus_id = "corpus-01-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed FASTQ inputs before downstream alignment."
depends_on = []
external_inputs = ["corpus.raw_fastq_reads"]
upstream_inputs = []
outputs = ["validated_reads_r1_path", "validated_reads_r2_path", "validation_report"]

[[nodes]]
node_id = "bam.align"
stage_id = "bam.align"
readiness_kind = "dry_or_smoke"
summary = "Align governed FASTQ path outputs into BAM."
depends_on = ["fastq.validate_reads"]
external_inputs = [
  "alignment_reference_fasta_contract",
  "alignment_reference_index_contract",
  "alignment_read_group_contract",
]
upstream_inputs = ["validated_reads_r1_path", "validated_reads_r2_path"]
outputs = ["aligned_bam", "aligned_bai", "align_metrics"]

[[nodes]]
node_id = "vcf.call"
stage_id = "vcf.call"
readiness_kind = "smoke"
summary = "Call variants from the governed aligned BAM handoff."
depends_on = ["bam.align"]
external_inputs = ["reference_fasta_contract", "reference_fai_contract"]
upstream_inputs = ["aligned_bam", "aligned_bai"]
outputs = ["called_vcf", "called_vcf_tbi", "call_stage_metrics"]
"#,
        )
        .expect("write cross VCF config");

        let output_path = tempdir.path().join("fastq-bam-vcf-cross.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate cross local pipeline dag with VCF");

        assert_eq!(report.pipeline_id, "fastq-bam-vcf-cross");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.node_count, 3);
        assert_eq!(report.edge_count, 2);
        assert!(report.acyclic);
        assert_eq!(report.topological_order, vec!["fastq.validate_reads", "bam.align", "vcf.call"]);
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
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/fastq-paired-merge.toml");
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

    #[test]
    fn fastq_edna_taxonomy_pipeline_dag_tracks_taxonomy_assets_and_outputs() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/fastq-edna-taxonomy.toml");
        let output_path =
            repo_root.join("target/local-ready/pipeline-dag/fastq-edna-taxonomy.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate edna taxonomy local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-edna-taxonomy");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-02-edna-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.screen_taxonomy"
                    && node.external_inputs
                        == vec!["taxonomy_database.root", "taxonomy_expected_truth_table"]
                    && node.outputs
                        == vec!["taxonomy_classification", "unclassified_reads", "taxonomy_summary"]
            }),
            "screen_taxonomy must carry governed taxonomy assets and explicit classified plus unclassified outputs"
        );
    }

    #[test]
    fn edna_taxonomy_no_vcf_pipeline_dag_keeps_taxonomy_reporting_local_to_fastq() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/edna-taxonomy-no-vcf.toml");
        let output_path =
            repo_root.join("target/local-ready/pipeline-dag/edna-taxonomy-no-vcf.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate edna taxonomy no-vcf local pipeline dag");

        assert_eq!(report.pipeline_id, "edna-taxonomy-no-vcf");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-02-edna-mini");
        assert_eq!(report.node_count, 6);
        assert_eq!(report.edge_count, 10);
        assert!(report.acyclic);
        assert!(
            report
                .validation_profiles
                .iter()
                .any(|profile| profile.profile_id == "edna_taxonomy_no_vcf"
                    && profile.check_count == 8),
            "edna taxonomy no-vcf pipeline must surface the governed separation profile"
        );
        assert!(
            report.nodes.iter().all(|node| node.stage_id.starts_with("fastq.")),
            "taxonomy-only pipeline must keep every declared stage in the fastq domain"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.screen_taxonomy"
                    && node.external_inputs
                        == vec!["taxonomy_database.root", "taxonomy_expected_truth_table"]
                    && node.outputs
                        == vec!["taxonomy_classification", "unclassified_reads", "taxonomy_summary"]
            }),
            "screen_taxonomy must keep the governed taxonomy assets and outputs explicit"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.depends_on
                        == vec![
                            "fastq.validate_reads",
                            "fastq.detect_adapters",
                            "fastq.trim_reads",
                            "fastq.filter_reads",
                            "fastq.screen_taxonomy",
                        ]
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "adapter_report",
                            "trim_metrics",
                            "filter_metrics",
                            "taxonomy_summary",
                        ]
            }),
            "report_qc must stay downstream of taxonomy screening without any bam or vcf handoffs"
        );
    }

    #[test]
    fn edna_taxonomy_no_vcf_profile_rejects_vcf_bridge_from_taxonomy_outputs() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("edna-taxonomy-no-vcf.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "edna-taxonomy-no-vcf"
domain = "cross"
summary = "Invalid proof that taxonomy outputs must not bridge into VCF stages."
default_corpus_id = "corpus-02-edna-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed eDNA FASTQ inputs."
depends_on = []
external_inputs = ["corpus.raw_fastq_reads"]
upstream_inputs = []
outputs = ["validated_reads", "validation_report"]

[[nodes]]
node_id = "fastq.filter_reads"
stage_id = "fastq.filter_reads"
readiness_kind = "smoke"
summary = "Filter validated reads."
depends_on = ["fastq.validate_reads"]
external_inputs = []
upstream_inputs = ["validated_reads"]
outputs = ["filtered_reads", "filter_metrics"]

[[nodes]]
node_id = "fastq.screen_taxonomy"
stage_id = "fastq.screen_taxonomy"
readiness_kind = "dry_or_smoke"
summary = "Screen taxonomy from filtered reads."
depends_on = ["fastq.filter_reads"]
external_inputs = ["taxonomy_database.root", "taxonomy_expected_truth_table"]
upstream_inputs = ["filtered_reads"]
outputs = ["taxonomy_classification", "unclassified_reads", "taxonomy_summary"]

[[nodes]]
node_id = "vcf.call"
stage_id = "vcf.call"
readiness_kind = "smoke"
summary = "Invalid germline bridge from taxonomy-only outputs."
depends_on = ["fastq.screen_taxonomy"]
external_inputs = ["reference_fasta_contract", "reference_fai_contract"]
upstream_inputs = ["taxonomy_summary"]
outputs = ["called_vcf", "called_vcf_tbi", "call_stage_metrics"]
"#,
        )
        .expect("write invalid edna taxonomy config");

        let output_path = tempdir.path().join("edna-taxonomy-no-vcf.json");
        let error = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect_err("taxonomy-only profile must reject a vcf bridge");
        assert!(
            error
                .to_string()
                .contains("taxonomy-only pipeline must not admit BAM or VCF germline stages"),
            "profile must fail closed when taxonomy outputs bridge into germline stages: {error:#}"
        );
    }

    #[test]
    fn fastq_amplicon_pipeline_dag_tracks_amplicon_handoffs() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/fastq-amplicon.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/fastq-amplicon.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate amplicon local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-amplicon");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-03-amplicon-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.cluster_otus"
                    && node.upstream_inputs
                        == vec!["normalized_amplicon_reads", "non_chimeric_representatives"]
                    && node.outputs == vec!["otu_table", "otu_representatives"]
            }),
            "cluster_otus must consume normalized reads plus non-chimeric representatives"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.normalize_abundance"
                    && node.upstream_inputs == vec!["otu_table"]
                    && node.outputs == vec!["normalized_abundance_table"]
            }),
            "normalize_abundance must consume the clustered OTU table"
        );
    }

    #[test]
    fn amplicon_asv_otu_no_vcf_pipeline_dag_keeps_amplicon_reporting_local_to_fastq() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/amplicon-asv-otu-no-vcf.toml");
        let output_path =
            repo_root.join("target/local-ready/pipeline-dag/amplicon-asv-otu-no-vcf.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate amplicon asv otu no-vcf local pipeline dag");

        assert_eq!(report.pipeline_id, "amplicon-asv-otu-no-vcf");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-03-amplicon-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 12);
        assert!(report.acyclic);
        assert!(
            report
                .validation_profiles
                .iter()
                .any(|profile| profile.profile_id == "amplicon_asv_otu_no_vcf"
                    && profile.check_count == 8),
            "amplicon asv otu no-vcf pipeline must surface the governed separation profile"
        );
        assert!(
            report.nodes.iter().all(|node| node.stage_id.starts_with("fastq.")),
            "amplicon-only pipeline must keep every declared stage in the fastq domain"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.cluster_otus"
                    && node.upstream_inputs
                        == vec!["normalized_amplicon_reads", "non_chimeric_representatives"]
                    && node.outputs == vec!["otu_table", "otu_representatives"]
            }),
            "cluster_otus must keep the governed normalized-read and non-chimeric representative handoffs explicit"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "primer_normalization_metrics",
                            "asv_metrics",
                            "chimera_report",
                            "otu_table",
                            "normalized_abundance_table",
                        ]
            }),
            "report_qc must stay downstream of asv, otu, and abundance evidence without bam or vcf handoffs"
        );
    }

    #[test]
    fn amplicon_asv_otu_no_vcf_profile_rejects_vcf_bridge_from_asv_or_otu_outputs() {
        let repo_root = repo_root();
        let tempdir = tempfile::tempdir().expect("tempdir");
        let config_path = tempdir.path().join("amplicon-asv-otu-no-vcf.toml");
        fs::write(
            &config_path,
            r#"
schema_version = "bijux.bench.local_pipeline_dag.v1"
pipeline_id = "amplicon-asv-otu-no-vcf"
domain = "cross"
summary = "Invalid proof that amplicon outputs must not bridge into VCF stages."
default_corpus_id = "corpus-03-amplicon-mini"

[[nodes]]
node_id = "fastq.validate_reads"
stage_id = "fastq.validate_reads"
readiness_kind = "smoke"
summary = "Validate governed amplicon FASTQ inputs."
depends_on = []
external_inputs = ["corpus.raw_amplicon_fastq_reads"]
upstream_inputs = []
outputs = ["validated_amplicon_reads", "validation_report"]

[[nodes]]
node_id = "fastq.normalize_primers"
stage_id = "fastq.normalize_primers"
readiness_kind = "smoke"
summary = "Normalize primers before ASV inference."
depends_on = ["fastq.validate_reads"]
external_inputs = ["amplicon_primer_contract"]
upstream_inputs = ["validated_amplicon_reads"]
outputs = ["normalized_amplicon_reads", "primer_normalization_metrics"]

[[nodes]]
node_id = "fastq.infer_asvs"
stage_id = "fastq.infer_asvs"
readiness_kind = "smoke"
summary = "Infer ASVs from normalized reads."
depends_on = ["fastq.normalize_primers"]
external_inputs = []
upstream_inputs = ["normalized_amplicon_reads"]
outputs = ["asv_table", "asv_representatives", "asv_metrics"]

[[nodes]]
node_id = "vcf.call"
stage_id = "vcf.call"
readiness_kind = "smoke"
summary = "Invalid germline bridge from amplicon-only outputs."
depends_on = ["fastq.infer_asvs"]
external_inputs = ["reference_fasta_contract", "reference_fai_contract"]
upstream_inputs = ["asv_table"]
outputs = ["called_vcf", "called_vcf_tbi", "call_stage_metrics"]
"#,
        )
        .expect("write invalid amplicon config");

        let output_path = tempdir.path().join("amplicon-asv-otu-no-vcf.json");
        let error = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect_err("amplicon-only profile must reject a vcf bridge");
        assert!(
            error
                .to_string()
                .contains("amplicon-only pipeline must not admit BAM or VCF germline stages"),
            "profile must fail closed when asv or otu outputs bridge into germline stages: {error:#}"
        );
    }

    #[test]
    fn fastq_umi_pipeline_dag_tracks_downstream_umi_consumers() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/fastq-umi.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/fastq-umi.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate umi local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-umi");
        assert_eq!(report.domain, "fastq");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 8);
        assert_eq!(report.edge_count, 13);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.extract_umis"
                    && node.outputs
                        == vec![
                            "umi_tagged_reads_r1",
                            "umi_tagged_reads_r2",
                            "umi_extraction_report",
                        ]
            }),
            "extract_umis must emit UMI-tagged reads and the governed extraction report"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.remove_duplicates"
                    && node.upstream_inputs
                        == vec!["filtered_umi_reads_r1", "filtered_umi_reads_r2"]
                    && node.outputs
                        == vec![
                            "deduplicated_umi_reads_r1",
                            "deduplicated_umi_reads_r2",
                            "duplicate_classes_tsv",
                            "duplicate_provenance_json",
                            "deduplication_report",
                        ]
            }),
            "remove_duplicates must consume filtered UMI-tagged reads and expose duplicate-aware outputs"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.report_qc"
                    && node.upstream_inputs
                        == vec![
                            "validation_report",
                            "umi_extraction_report",
                            "duplicate_signal_report",
                            "trim_metrics",
                            "filter_metrics",
                            "deduplication_report",
                            "deduplicated_profile",
                        ]
            }),
            "report_qc must collate UMI extraction and duplicate-aware preprocessing metrics"
        );
    }

    #[test]
    fn bam_core_qc_pipeline_dag_tracks_qc_summary_filter_and_coverage_handoffs() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/bam-core-qc.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/bam-core-qc.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam core qc local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-core-qc");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-bam-mini");
        assert_eq!(report.node_count, 5);
        assert_eq!(report.edge_count, 5);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.qc_pre"
                    && node.upstream_inputs == vec!["validation_report"]
                    && node.outputs
                        == vec![
                            "qc_pre_flagstat",
                            "qc_pre_idxstats",
                            "qc_pre_stats",
                            "qc_pre_stage_metrics",
                        ]
            }),
            "bam.qc_pre must consume validation output and expose governed pre-QC metric artifacts"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.filter"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.upstream_inputs
                        == vec!["qc_pre_flagstat", "qc_pre_idxstats", "qc_pre_stats"]
                    && node.outputs
                        == vec![
                            "filtered_bam",
                            "filtered_bai",
                            "filter_report_json",
                            "filter_stage_metrics",
                        ]
            }),
            "bam.filter must consume pre-QC outputs and expose the filtered BAM handoff"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.coverage"
                    && node.upstream_inputs
                        == vec!["mapping_summary_report_json", "filtered_bam", "filtered_bai"]
                    && node.external_inputs == vec!["coverage_region_contract"]
            }),
            "bam.coverage must consume both mapping summary context and filtered BAM outputs"
        );
    }

    #[test]
    fn bam_authenticity_pipeline_dag_tracks_only_required_authenticity_evidence() {
        let repo_root = repo_root();
        let config_path =
            repo_root.join("benchmarks/configs/pipelines/local/bam-authenticity.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/bam-authenticity.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam authenticity local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-authenticity");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-adna-damage-mini");
        assert_eq!(report.node_count, 7);
        assert_eq!(report.edge_count, 7);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.sex"
                    && node.upstream_inputs == vec!["coverage_report_json"]
                    && node.external_inputs == vec!["corpus.aligned_bam", "sex_reference_contract"]
            }),
            "bam.sex must stay explicit as a coverage-driven diagnostic branch"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.authenticity"
                    && node.upstream_inputs
                        == vec![
                            "mapping_summary_report_json",
                            "coverage_report_json",
                            "damage_report_json",
                            "contamination_report_json",
                            "complexity_report_json",
                        ]
                    && node.depends_on
                        == vec![
                            "bam.mapping_summary",
                            "bam.coverage",
                            "bam.damage",
                            "bam.contamination",
                            "bam.complexity",
                        ]
            }),
            "bam.authenticity must depend only on the governed metrics it currently consumes"
        );
        assert!(
            report.nodes.iter().all(|node| {
                node.stage_id != "bam.authenticity"
                    || !node.upstream_inputs.iter().any(|input| input == "sex_report_json")
            }),
            "bam.authenticity must not claim bam.sex as a required upstream metric"
        );
    }

    #[test]
    fn bam_genotyping_pipeline_dag_tracks_recalibration_skip_and_run_handoffs() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/bam-genotyping.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/bam-genotyping.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam genotyping local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-genotyping");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-bam-mini");
        assert_eq!(report.node_count, 4);
        assert_eq!(report.edge_count, 5);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.recalibration"
                    && node.upstream_inputs == vec!["filtered_bam", "coverage_report_json"]
                    && node.outputs
                        == vec![
                            "recalibrated_bam",
                            "recalibrated_bai",
                            "recalibration_report_text",
                            "recalibration_summary_json",
                            "recalibration_stage_metrics",
                        ]
            }),
            "bam.recalibration must expose the coverage-gated decision summary plus recalibrated BAM outputs"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.genotyping"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.depends_on == vec!["bam.filter", "bam.recalibration"]
                    && node.upstream_inputs
                        == vec![
                            "filtered_bam",
                            "filtered_bai",
                            "recalibrated_bam",
                            "recalibrated_bai",
                            "recalibration_summary_json",
                        ]
                    && node.external_inputs
                        == vec![
                            "genotyping_reference_contract",
                            "candidate_sites_vcf_contract",
                            "target_regions_contract",
                        ]
            }),
            "bam.genotyping must expose both the filtered-BAM skip path and recalibrated-BAM run path"
        );
    }

    #[test]
    fn bam_kinship_pipeline_dag_keeps_overlap_insufficiency_local_to_kinship() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/bam-kinship.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/bam-kinship.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate bam kinship local pipeline dag");

        assert_eq!(report.pipeline_id, "bam-kinship");
        assert_eq!(report.domain, "bam");
        assert_eq!(report.default_corpus_id, "corpus-01-bam-mini");
        assert_eq!(report.node_count, 4);
        assert_eq!(report.edge_count, 4);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.overlap_correction"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.upstream_inputs == vec!["filtered_bam", "filtered_bai"]
                    && node.outputs
                        == vec![
                            "overlap_corrected_bam",
                            "overlap_corrected_bai",
                            "overlap_correction_summary_json",
                            "overlap_correction_stage_metrics",
                        ]
            }),
            "bam.overlap_correction must expose the overlap-sufficiency handoff needed only by kinship"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.kinship"
                    && node.depends_on == vec!["bam.overlap_correction", "bam.genotyping"]
                    && node.upstream_inputs
                        == vec![
                            "overlap_corrected_bam",
                            "overlap_corrected_bai",
                            "overlap_correction_summary_json",
                            "genotyping_report_json",
                        ]
                    && node.external_inputs
                        == vec![
                            "kinship_reference_panel_contract",
                            "kinship_population_scope_contract",
                            "kinship_pairing_contract",
                        ]
            }),
            "bam.kinship must consume overlap and genotype-readiness handoffs before pairwise inference"
        );
        assert!(
            report.nodes.iter().all(|node| {
                node.stage_id == "bam.kinship"
                    || !node
                        .upstream_inputs
                        .iter()
                        .any(|input| input == "overlap_correction_summary_json")
            }),
            "overlap insufficiency must stay local to bam.kinship instead of blocking unrelated BAM stages"
        );
    }

    #[test]
    fn fastq_to_bam_pipeline_dag_maps_trimmed_fastq_paths_into_bam_alignment() {
        let repo_root = repo_root();
        let config_path = repo_root.join("benchmarks/configs/pipelines/local/fastq-to-bam.toml");
        let output_path = repo_root.join("target/local-ready/pipeline-dag/fastq-to-bam.json");
        let report = validate_pipeline_dag_path(&repo_root, &config_path, &output_path)
            .expect("validate fastq-to-bam local pipeline dag");

        assert_eq!(report.pipeline_id, "fastq-to-bam");
        assert_eq!(report.domain, "cross");
        assert_eq!(report.default_corpus_id, "corpus-01-mini");
        assert_eq!(report.node_count, 6);
        assert_eq!(report.edge_count, 7);
        assert!(report.acyclic);
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "fastq.trim_reads"
                    && node.outputs
                        == vec!["trimmed_reads_r1_path", "trimmed_reads_r2_path", "trim_metrics"]
            }),
            "fastq.trim_reads must emit explicit R1 and R2 path outputs for cross-domain alignment"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.align"
                    && node.readiness_kind == "dry_or_smoke"
                    && node.depends_on == vec!["fastq.trim_reads"]
                    && node.upstream_inputs
                        == vec!["trimmed_reads_r1_path", "trimmed_reads_r2_path"]
                    && node.external_inputs
                        == vec![
                            "alignment_reference_fasta_contract",
                            "alignment_reference_index_contract",
                            "alignment_read_group_contract",
                        ]
            }),
            "bam.align must consume the governed trimmed FASTQ path handoff and alignment contracts"
        );
        assert!(
            report.nodes.iter().any(|node| {
                node.stage_id == "bam.mapping_summary"
                    && node.depends_on == vec!["bam.align", "bam.qc_pre"]
                    && node.upstream_inputs
                        == vec!["align_bam", "qc_pre_flagstat", "qc_pre_idxstats", "qc_pre_stats"]
            }),
            "bam.mapping_summary must consume both alignment output and BAM pre-QC context"
        );
    }
}
