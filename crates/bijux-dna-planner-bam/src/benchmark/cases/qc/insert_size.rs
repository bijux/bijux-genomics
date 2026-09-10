//! Governed BAM benchmark qc insert size.
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
use bijux_dna_domain_bam::params::CoverageEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_INSERT_SIZE_CONFIG_PATH: &str = "configs/bench/local/bam-insert-size.toml";

const DEFAULT_LOCAL_INSERT_SIZE_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.insert_size";

#[derive(Debug, Clone)]
pub struct LocalInsertSizeSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_read_pairs: u64,
    pub expected_median_insert_size: f64,
    pub expected_mean_insert_size: f64,
    pub expected_min_insert_size: u64,
    pub expected_max_insert_size: u64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalInsertSizeSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalInsertSizeSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalInsertSizeSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_read_pairs: u64,
    expected_median_insert_size: f64,
    expected_mean_insert_size: f64,
    expected_min_insert_size: u64,
    expected_max_insert_size: u64,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.insert_size` plans cannot be built.
pub fn local_insert_size_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalInsertSizeSmokeCasePlan>> {
    let config = load_local_insert_size_smoke_config(repo_root)?;
    ensure_unique_insert_size_sample_ids(&config.cases)?;

    let stage = BamStage::InsertSize;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.insert_size tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_INSERT_SIZE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_insert_size_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_insert_size_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalInsertSizeSmokeCase,
) -> Result<LocalInsertSizeSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.insert_size BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_read_pairs == 0 {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must declare expected_read_pairs greater than zero",
            case.sample_id
        ));
    }
    if case.expected_min_insert_size == 0 || case.expected_max_insert_size == 0 {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected insert-size bounds greater than zero",
            case.sample_id
        ));
    }
    if case.expected_min_insert_size > case.expected_max_insert_size {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected min insert size less than or equal to expected max insert size",
            case.sample_id
        ));
    }
    if case.expected_mean_insert_size < case.expected_min_insert_size as f64
        || case.expected_mean_insert_size > case.expected_max_insert_size as f64
    {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected mean insert size within the declared bounds",
            case.sample_id
        ));
    }
    if case.expected_median_insert_size < case.expected_min_insert_size as f64
        || case.expected_median_insert_size > case.expected_max_insert_size as f64
    {
        return Err(anyhow!(
            "local-smoke bam.insert_size case `{}` must keep expected median insert size within the declared bounds",
            case.sample_id
        ));
    }

    let params = CoverageEffectiveParams {
        regions: None,
        depth_thresholds: vec![1],
        regime_mode: "advisory_and_enforced".to_string(),
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan =
        crate::tool_adapters::bam::insert_size::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalInsertSizeSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_read_pairs: case.expected_read_pairs,
        expected_median_insert_size: case.expected_median_insert_size,
        expected_mean_insert_size: case.expected_mean_insert_size,
        expected_min_insert_size: case.expected_min_insert_size,
        expected_max_insert_size: case.expected_max_insert_size,
        plan,
    })
}

fn ensure_unique_insert_size_sample_ids(cases: &[LocalInsertSizeSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.insert_size sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.insert_size sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_insert_size_smoke_config(repo_root: &Path) -> Result<LocalInsertSizeSmokeConfig> {
    let path = repo_root.join(LOCAL_INSERT_SIZE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalInsertSizeSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_insert_size.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.insert_size schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.insert_size must declare at least one governed case"));
    }
    Ok(config)
}
