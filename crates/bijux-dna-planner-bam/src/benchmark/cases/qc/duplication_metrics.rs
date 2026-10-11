//! Governed BAM benchmark qc duplication metrics.
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
use bijux_dna_domain_bam::params::DuplicateAction;
use bijux_dna_domain_bam::params::MarkDupEffectiveParams;
use bijux_dna_domain_bam::params::OpticalDuplicatePolicy;
use bijux_dna_domain_bam::params::UmiPolicy;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_DUPLICATION_METRICS_CONFIG_PATH: &str =
    "configs/bench/local/bam-duplication-metrics.toml";

const DEFAULT_LOCAL_DUPLICATION_METRICS_OUTPUT_DIR: &str =
    "artifacts/runs/bench/local-smoke/bam.duplication_metrics";

#[derive(Debug, Clone)]
pub struct LocalDuplicationMetricsSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_examined_reads: u64,
    pub expected_duplicate_reads: u64,
    pub expected_duplicate_fraction: f64,
    pub expected_estimated_library_size: Option<u64>,
    pub expected_insufficient_library_size_reason: Option<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalDuplicationMetricsSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalDuplicationMetricsSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalDuplicationMetricsSmokeCase {
    sample_id: String,
    bam: PathBuf,
    optical_duplicates: OpticalDuplicatePolicy,
    umi_policy: UmiPolicy,
    duplicate_action: DuplicateAction,
    expected_examined_reads: u64,
    expected_duplicate_reads: u64,
    expected_duplicate_fraction: f64,
    #[serde(default)]
    expected_estimated_library_size: Option<u64>,
    #[serde(default)]
    expected_insufficient_library_size_reason: Option<String>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.duplication_metrics` plans cannot be built.
pub fn local_duplication_metrics_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalDuplicationMetricsSmokeCasePlan>> {
    let config = load_local_duplication_metrics_smoke_config(repo_root)?;
    ensure_unique_duplication_metrics_sample_ids(&config.cases)?;

    let stage = BamStage::DuplicationMetrics;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DUPLICATION_METRICS_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_duplication_metrics_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_duplication_metrics_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalDuplicationMetricsSmokeCase,
) -> Result<LocalDuplicationMetricsSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_duplicate_reads > case.expected_examined_reads {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` cannot declare duplicate reads greater than examined reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_duplicate_fraction) {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must declare duplicate fraction within [0, 1]",
            case.sample_id
        ));
    }
    let derived_fraction = if case.expected_examined_reads == 0 {
        0.0
    } else {
        case.expected_duplicate_reads as f64 / case.expected_examined_reads as f64
    };
    if (derived_fraction - case.expected_duplicate_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must keep duplicate fraction aligned with examined and duplicate reads",
            case.sample_id
        ));
    }
    if case.expected_estimated_library_size.is_some()
        == case.expected_insufficient_library_size_reason.is_some()
    {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must declare exactly one of expected_estimated_library_size or expected_insufficient_library_size_reason",
            case.sample_id
        ));
    }
    if case.expected_insufficient_library_size_reason.as_deref().is_some_and(str::is_empty) {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics case `{}` must not declare an empty insufficiency reason",
            case.sample_id
        ));
    }

    let params = MarkDupEffectiveParams {
        optical_duplicates: case.optical_duplicates,
        umi_policy: case.umi_policy,
        duplicate_action: case.duplicate_action,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::duplication_metrics::plan(
        tool_spec, &case.bam, &out_dir, &params,
    )?;

    Ok(LocalDuplicationMetricsSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_examined_reads: case.expected_examined_reads,
        expected_duplicate_reads: case.expected_duplicate_reads,
        expected_duplicate_fraction: case.expected_duplicate_fraction,
        expected_estimated_library_size: case.expected_estimated_library_size,
        expected_insufficient_library_size_reason: case.expected_insufficient_library_size_reason,
        plan,
    })
}

fn ensure_unique_duplication_metrics_sample_ids(
    cases: &[LocalDuplicationMetricsSmokeCase],
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.duplication_metrics sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.duplication_metrics sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_duplication_metrics_smoke_config(
    repo_root: &Path,
) -> Result<LocalDuplicationMetricsSmokeConfig> {
    let path = repo_root.join(LOCAL_DUPLICATION_METRICS_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDuplicationMetricsSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_duplication_metrics.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.duplication_metrics schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.duplication_metrics must declare at least one governed case"
        ));
    }
    Ok(config)
}
