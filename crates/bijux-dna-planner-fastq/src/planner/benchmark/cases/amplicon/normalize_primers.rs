//! Governed FASTQ benchmark amplicon normalize primers.
use crate::planner::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::planner::benchmark::cases::hydrate_smoke_threads;
use crate::selection::load_fastq_domain_tool_execution_spec;
use crate::selection::select_normalize_primers_tools;
use crate::tool_adapters::fastq::normalize_primers::plan_with_options as plan_normalize_primers;
use crate::tool_adapters::fastq::normalize_primers::NormalizePrimersPlanOptions;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_fastq::stages::ids::STAGE_NORMALIZE_PRIMERS;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_NORMALIZE_PRIMERS_CONFIG_PATH: &str =
    "configs/bench/local/fastq-normalize-primers.toml";

const DEFAULT_LOCAL_NORMALIZE_PRIMERS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/fastq.normalize_primers";

#[derive(Debug, Clone)]
pub struct LocalNormalizePrimersSmokeCasePlan {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalNormalizePrimersSmokeConfig {
    schema_version: String,
    tool_id: String,
    primer_set_id: String,
    marker_id: String,
    primer_fasta: PathBuf,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    orientation_policy: Option<String>,
    #[serde(default)]
    max_mismatch_rate: Option<f64>,
    #[serde(default)]
    min_overlap_bp: Option<u32>,
    #[serde(default)]
    strict_5p_anchor: Option<bool>,
    #[serde(default)]
    allow_iupac_codes: Option<bool>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalNormalizePrimersSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalNormalizePrimersSmokeCase {
    sample_id: String,
    r1: PathBuf,
    #[serde(default)]
    r2: Option<PathBuf>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, the fixture inputs do not
/// exist, or stage plans cannot be built for the governed smoke cases.
pub fn local_normalize_primers_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalNormalizePrimersSmokeCasePlan>> {
    let config = load_local_normalize_primers_smoke_config(repo_root)?;
    ensure_unique_normalize_primers_sample_ids(&config.cases)?;

    let stage_id = StageId::new(STAGE_NORMALIZE_PRIMERS.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    let normalized_tools = select_normalize_primers_tools(std::slice::from_ref(&config.tool_id))?;
    if normalized_tools.len() != 1 || normalized_tools[0] != tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers tool selection normalized unexpectedly: {normalized_tools:?}"
        ));
    }

    let primer_fasta = repo_root.join(&config.primer_fasta);
    if !primer_fasta.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers primer_fasta is missing: {}",
            primer_fasta.display()
        ));
    }

    let mut tool_spec = load_fastq_domain_tool_execution_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_NORMALIZE_PRIMERS_OUTPUT_DIR));
    let plan_options = NormalizePrimersPlanOptions {
        primer_set_id: config.primer_set_id,
        marker_id: Some(config.marker_id),
        primer_fasta: Some(config.primer_fasta),
        orientation_policy: config
            .orientation_policy
            .unwrap_or_else(|| "normalize_to_forward_primer".to_string()),
        max_mismatch_rate: config.max_mismatch_rate.unwrap_or(0.10),
        min_overlap_bp: config.min_overlap_bp.unwrap_or(10).max(1),
        strict_5p_anchor: config.strict_5p_anchor.unwrap_or(true),
        allow_iupac_codes: config.allow_iupac_codes.unwrap_or(true),
    };

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_normalize_primers_smoke_case(
                repo_root,
                &tool_spec,
                &plan_options,
                &output_root,
                case,
            )
        })
        .collect()
}

fn build_local_normalize_primers_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    options: &NormalizePrimersPlanOptions,
    output_root: &Path,
    case: LocalNormalizePrimersSmokeCase,
) -> Result<LocalNormalizePrimersSmokeCasePlan> {
    let r1_abs = repo_root.join(&case.r1);
    if !r1_abs.is_file() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers r1 fixture is missing: {}",
            r1_abs.display()
        ));
    }
    if let Some(r2) = case.r2.as_ref() {
        let r2_abs = repo_root.join(r2);
        if !r2_abs.is_file() {
            return Err(anyhow!(
                "local-smoke fastq.normalize_primers r2 fixture is missing: {}",
                r2_abs.display()
            ));
        }
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = plan_normalize_primers(tool_spec, &case.r1, case.r2.as_deref(), &out_dir, options)?;

    Ok(LocalNormalizePrimersSmokeCasePlan {
        sample_id: case.sample_id,
        r1: case.r1,
        r2: case.r2,
        plan,
    })
}

fn ensure_unique_normalize_primers_sample_ids(
    cases: &[LocalNormalizePrimersSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke fastq.normalize_primers sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke fastq.normalize_primers sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_normalize_primers_smoke_config(
    repo_root: &Path,
) -> Result<LocalNormalizePrimersSmokeConfig> {
    let path = repo_root.join(LOCAL_NORMALIZE_PRIMERS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalNormalizePrimersSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.fastq.local_normalize_primers.v1" {
        return Err(anyhow!(
            "unsupported local-smoke fastq.normalize_primers schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke fastq.normalize_primers must declare at least one governed case"
        ));
    }
    Ok(config)
}
