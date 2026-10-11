//! Governed BAM benchmark qc endogenous content.
use crate::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_bam_domain_tool_planning_spec;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_ENDOGENOUS_CONTENT_CONFIG_PATH: &str =
    "configs/bench/local/bam-endogenous-content.toml";

const DEFAULT_LOCAL_ENDOGENOUS_CONTENT_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.endogenous_content";

#[derive(Debug, Clone)]
pub struct LocalEndogenousContentSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub host_reference_scope: String,
    pub expected_total_reads: u64,
    pub expected_mapped_reads: u64,
    pub expected_endogenous_fraction: f64,
    pub expected_method: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalEndogenousContentSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalEndogenousContentSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalEndogenousContentSmokeCase {
    sample_id: String,
    bam: PathBuf,
    host_reference_scope: String,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_endogenous_fraction: f64,
    expected_method: String,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.endogenous_content` plans cannot be built.
pub fn local_endogenous_content_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalEndogenousContentSmokeCasePlan>> {
    let config = load_local_endogenous_content_smoke_config(repo_root)?;
    ensure_unique_endogenous_content_sample_ids(&config.cases)?;

    let stage = BamStage::EndogenousContent;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_ENDOGENOUS_CONTENT_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_endogenous_content_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_endogenous_content_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalEndogenousContentSmokeCase,
) -> Result<LocalEndogenousContentSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.host_reference_scope.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must declare a non-empty host_reference_scope",
            case.sample_id
        ));
    }
    if case.expected_total_reads == 0 {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must declare expected_total_reads greater than zero",
            case.sample_id
        ));
    }
    if case.expected_mapped_reads > case.expected_total_reads {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` cannot declare mapped reads greater than total reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_endogenous_fraction) {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must keep expected_endogenous_fraction within [0, 1]",
            case.sample_id
        ));
    }
    if case.expected_method.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must declare a non-empty expected_method",
            case.sample_id
        ));
    }
    let expected_fraction = case.expected_mapped_reads as f64 / case.expected_total_reads as f64;
    if (case.expected_endogenous_fraction - expected_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content case `{}` must keep expected_endogenous_fraction aligned with expected_mapped_reads and expected_total_reads",
            case.sample_id
        ));
    }

    let params = bijux_dna_domain_bam::params::EndogenousContentEffectiveParams {
        regions: None,
        depth_thresholds: vec![1],
        host_reference_scope: case.host_reference_scope.clone(),
        host_reference_digest: None,
        refuse_without_host_reference: true,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::endogenous_content::plan(
        tool_spec, &case.bam, &out_dir, &params,
    )?;

    Ok(LocalEndogenousContentSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        host_reference_scope: case.host_reference_scope,
        expected_total_reads: case.expected_total_reads,
        expected_mapped_reads: case.expected_mapped_reads,
        expected_endogenous_fraction: case.expected_endogenous_fraction,
        expected_method: case.expected_method,
        plan,
    })
}

fn ensure_unique_endogenous_content_sample_ids(
    cases: &[LocalEndogenousContentSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.endogenous_content sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.endogenous_content sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_endogenous_content_smoke_config(
    repo_root: &Path,
) -> Result<LocalEndogenousContentSmokeConfig> {
    let path = repo_root.join(LOCAL_ENDOGENOUS_CONTENT_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalEndogenousContentSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_endogenous_content.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.endogenous_content schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.endogenous_content must declare at least one governed case"
        ));
    }
    Ok(config)
}
