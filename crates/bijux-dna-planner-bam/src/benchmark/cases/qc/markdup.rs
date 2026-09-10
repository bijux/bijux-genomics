//! Governed BAM benchmark qc markdup.
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

const LOCAL_MARKDUP_CONFIG_PATH: &str = "configs/bench/local/bam-markdup.toml";

const DEFAULT_LOCAL_MARKDUP_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.markdup";

#[derive(Debug, Clone)]
pub struct LocalMarkdupSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_input_reads: u64,
    pub expected_output_reads: u64,
    pub expected_removed_reads: u64,
    pub expected_duplicate_reads_before: u64,
    pub expected_duplicate_reads_after: u64,
    pub expected_duplicate_fraction: f64,
    pub expected_newly_marked_reads: u64,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalMarkdupSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMarkdupSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMarkdupSmokeCase {
    sample_id: String,
    bam: PathBuf,
    duplicate_action: DuplicateAction,
    optical_duplicates: OpticalDuplicatePolicy,
    umi_policy: UmiPolicy,
    expected_input_reads: u64,
    expected_output_reads: u64,
    expected_removed_reads: u64,
    expected_duplicate_reads_before: u64,
    expected_duplicate_reads_after: u64,
    expected_duplicate_fraction: f64,
    expected_newly_marked_reads: u64,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.markdup` plans cannot be built.
pub fn local_markdup_smoke_plans(repo_root: &Path) -> Result<Vec<LocalMarkdupSmokeCasePlan>> {
    let config = load_local_markdup_smoke_config(repo_root)?;
    ensure_unique_markdup_sample_ids(&config.cases)?;

    let stage = BamStage::Markdup;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.markdup tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MARKDUP_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_markdup_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

fn build_local_markdup_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalMarkdupSmokeCase,
) -> Result<LocalMarkdupSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.markdup BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_output_reads > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare output reads greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_removed_reads
        != case.expected_input_reads.saturating_sub(case.expected_output_reads)
    {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` must keep expected removed reads aligned with input and output reads",
            case.sample_id
        ));
    }
    if case.expected_duplicate_reads_before > case.expected_input_reads {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare duplicate reads before greater than input reads",
            case.sample_id
        ));
    }
    if case.expected_duplicate_reads_after > case.expected_output_reads {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare duplicate reads after greater than output reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_duplicate_fraction) {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` must declare duplicate fraction within [0, 1]",
            case.sample_id
        ));
    }
    let derived_fraction = if case.expected_output_reads == 0 {
        0.0
    } else {
        case.expected_duplicate_reads_after as f64 / case.expected_output_reads as f64
    };
    if (derived_fraction - case.expected_duplicate_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` must keep duplicate fraction aligned with output and duplicate reads",
            case.sample_id
        ));
    }
    if case.expected_newly_marked_reads > case.expected_duplicate_reads_after {
        return Err(anyhow!(
            "local-smoke bam.markdup case `{}` cannot declare newly marked reads greater than duplicate reads after processing",
            case.sample_id
        ));
    }
    match case.duplicate_action {
        DuplicateAction::Mark => {
            if case.expected_removed_reads != 0 {
                return Err(anyhow!(
                    "local-smoke bam.markdup case `{}` must not remove reads when duplicate_action is mark",
                    case.sample_id
                ));
            }
        }
        DuplicateAction::Remove => {
            if case.expected_newly_marked_reads != 0 {
                return Err(anyhow!(
                    "local-smoke bam.markdup case `{}` must not declare newly marked reads when duplicate_action is remove",
                    case.sample_id
                ));
            }
        }
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
    let plan = crate::tool_adapters::bam::markdup::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalMarkdupSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_input_reads: case.expected_input_reads,
        expected_output_reads: case.expected_output_reads,
        expected_removed_reads: case.expected_removed_reads,
        expected_duplicate_reads_before: case.expected_duplicate_reads_before,
        expected_duplicate_reads_after: case.expected_duplicate_reads_after,
        expected_duplicate_fraction: case.expected_duplicate_fraction,
        expected_newly_marked_reads: case.expected_newly_marked_reads,
        plan,
    })
}

fn ensure_unique_markdup_sample_ids(cases: &[LocalMarkdupSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.markdup sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.markdup sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_markdup_smoke_config(repo_root: &Path) -> Result<LocalMarkdupSmokeConfig> {
    let path = repo_root.join(LOCAL_MARKDUP_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMarkdupSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_markdup.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.markdup schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.markdup must declare at least one governed case"));
    }
    Ok(config)
}
