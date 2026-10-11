//! Governed FASTQ benchmark qc report qc.
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::tool_adapters::fastq::report_qc::plan_qc_post_with_qc_inputs;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::contract::ArtifactRef;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::params::qc_post::QcAggregationEngine;
use bijux_dna_domain_fastq::params::qc_post::QcAggregationScope;
use bijux_dna_domain_fastq::params::PairedMode;
use bijux_dna_domain_fastq::GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_REPORT_QC_CONFIG_PATH: &str = "configs/bench/local/fastq-report-qc.toml";

const DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/fastq.report_qc";

#[derive(Debug, Deserialize)]
struct LocalReportQcSmokePlanConfig {
    schema_version: String,
    tool_id: String,
    fixture_root: PathBuf,
    manifest_template: PathBuf,
    #[serde(default)]
    aggregation_engine: Option<String>,
    #[serde(default)]
    aggregation_scope: Option<String>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the governed QC manifest cannot
/// be resolved against the fixture bundle, or the stage plan cannot be built.
pub fn local_report_qc_smoke_plan(
    repo_root: &Path,
) -> Result<bijux_dna_stage_contract::StagePlanV1> {
    let config = load_local_report_qc_smoke_plan_config(repo_root)?;
    let stage_id = StageId::from_static("fastq.report_qc");
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = crate::tool_adapters::fastq::report_qc::normalize_qc_post_tool_list(
        std::slice::from_ref(&config.tool_id),
    )?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.report_qc tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let manifest_path = repo_root.join(&config.manifest_template);
    let raw_manifest = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    let mut manifest: bijux_dna_domain_fastq::GovernedQcInputsManifestV1 =
        serde_json::from_str(&raw_manifest)
            .with_context(|| format!("parse {}", manifest_path.display()))?;
    if manifest.schema_version != GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required",
            manifest_path.display(),
            manifest.schema_version,
            GOVERNED_QC_INPUTS_MANIFEST_SCHEMA_VERSION
        ));
    }

    let fixture_root = repo_root.join(&config.fixture_root);
    if !fixture_root.is_dir() {
        return Err(anyhow!(
            "local-smoke fastq.report_qc fixture_root is missing: {}",
            fixture_root.display()
        ));
    }

    rebase_governed_qc_inputs(&mut manifest.qc_inputs, &fixture_root)?;
    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, None);
    let aggregation_engine =
        resolve_local_report_qc_aggregation_engine(config.aggregation_engine.as_deref())?;
    let aggregation_scope =
        resolve_local_report_qc_aggregation_scope(config.aggregation_scope.as_deref())?;
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_REPORT_QC_OUTPUT_DIR));

    plan_qc_post_with_qc_inputs(
        &tool_spec,
        &manifest.qc_inputs,
        &output_root,
        std::collections::BTreeMap::new(),
        PairedMode::SingleEnd,
        aggregation_engine,
        aggregation_scope,
        None,
        None,
    )
}

fn resolve_local_report_qc_aggregation_engine(value: Option<&str>) -> Result<QcAggregationEngine> {
    match value.unwrap_or("multiqc") {
        "auto" | "multiqc" => Ok(QcAggregationEngine::Multiqc),
        other => {
            Err(anyhow!("unsupported local-smoke fastq.report_qc aggregation_engine `{other}`"))
        }
    }
}

fn resolve_local_report_qc_aggregation_scope(value: Option<&str>) -> Result<QcAggregationScope> {
    match value.unwrap_or("governed_qc_artifacts") {
        "governed_qc_artifacts" => Ok(QcAggregationScope::GovernedQcArtifacts),
        other => {
            Err(anyhow!("unsupported local-smoke fastq.report_qc aggregation_scope `{other}`"))
        }
    }
}

fn rebase_governed_qc_inputs(qc_inputs: &mut [ArtifactRef], fixture_root: &Path) -> Result<()> {
    for qc_input in qc_inputs {
        let source = fixture_root.join(&qc_input.path);
        if !source.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.report_qc governed QC input is missing: {}",
                source.display()
            ));
        }
        qc_input.path = source;
    }
    Ok(())
}

fn load_local_report_qc_smoke_plan_config(
    repo_root: &Path,
) -> Result<LocalReportQcSmokePlanConfig> {
    let config_path = repo_root.join(LOCAL_REPORT_QC_CONFIG_PATH);
    let raw = std::fs::read_to_string(&config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: LocalReportQcSmokePlanConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_report_qc.v1" {
        return Err(anyhow!(
            "{} declares unexpected schema_version `{}`",
            config_path.display(),
            config.schema_version
        ));
    }
    Ok(config)
}
