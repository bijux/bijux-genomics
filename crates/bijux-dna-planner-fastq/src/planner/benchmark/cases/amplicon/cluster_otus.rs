//! Governed FASTQ benchmark amplicon cluster otus.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_cluster_otus_tools;
use crate::tool_adapters::fastq::cluster_otus::plan_with_options as plan_cluster_otus_with_options;
use crate::tool_adapters::fastq::cluster_otus::ClusterOtusPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_CLUSTER_OTUS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_CLUSTER_OTUS_CONFIG_PATH: &str = "configs/bench/local/fastq-cluster-otus.toml";

const DEFAULT_LOCAL_CLUSTER_OTUS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.cluster_otus";

#[derive(Debug, Clone)]
pub struct LocalClusterOtusSmokeCasePlan {
    pub sample_id: String,
    pub reads: PathBuf,
    pub otu_identity: f64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalClusterOtusSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    otu_identity: Option<f64>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalClusterOtusSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalClusterOtusSmokeCase {
    sample_id: String,
    reads: PathBuf,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_cluster_otus_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalClusterOtusSmokeCasePlan>> {
    let config = load_local_cluster_otus_smoke_config(repo_root)?;
    ensure_unique_cluster_otus_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_CLUSTER_OTUS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_cluster_otus_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.cluster_otus tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }
    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let mut plan_options = ClusterOtusPlanOptions::baseline();
    plan_options.threads = Some(tool_spec.resources.threads.max(1));
    if let Some(otu_identity) = config.otu_identity {
        plan_options.otu_identity = otu_identity;
    }
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_CLUSTER_OTUS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_cluster_otus_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_cluster_otus_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &ClusterOtusPlanOptions,
    output_root: &Path,
    case: LocalClusterOtusSmokeCase,
) -> Result<LocalClusterOtusSmokeCasePlan> {
    let reads_abs = repo_root.join(&case.reads);
    if !reads_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.cluster_otus reads fixture is missing: {}",
            reads_abs.display()
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = plan_cluster_otus_with_options(tool_spec, &case.reads, None, &out_dir, options)?;

    Ok(LocalClusterOtusSmokeCasePlan {
        sample_id: case.sample_id,
        reads: case.reads,
        otu_identity: options.otu_identity,
        plan,
    })
}

fn ensure_unique_cluster_otus_sample_ids(cases: &[LocalClusterOtusSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.cluster_otus sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.cluster_otus sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_cluster_otus_smoke_config(repo_root: &Path) -> Result<LocalClusterOtusSmokeConfig> {
    let path = repo_root.join(LOCAL_CLUSTER_OTUS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalClusterOtusSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_cluster_otus.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.cluster_otus schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.cluster_otus must declare at least one governed case"
        ));
    }
    Ok(config)
}
