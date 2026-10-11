//! Governed BAM benchmark downstream kinship.
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

#[cfg(feature = "bam_downstream")]
const LOCAL_KINSHIP_CONFIG_PATH: &str = "configs/bench/local/bam-kinship.toml";

#[cfg(feature = "bam_downstream")]
const DEFAULT_LOCAL_KINSHIP_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.kinship";

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalKinshipSmokeExpectedPair {
    pub sample_a: String,
    pub sample_b: String,
    pub overlap_snps: u32,
    pub matching_sites: u32,
    pub mismatch_sites: u32,
    pub concordance: f64,
    pub kinship_coefficient: f64,
    pub relationship_label: String,
}

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Clone)]
pub struct LocalKinshipSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub reference_panel: String,
    pub reference_build: String,
    pub population_scope: String,
    pub min_overlap_snps: u32,
    pub requires_cohort_context: bool,
    pub expected_status: String,
    pub expected_observed_max_overlap_snps: u32,
    pub expected_insufficiency_reason: Option<String>,
    pub expected_pairwise_results: Vec<LocalKinshipSmokeExpectedPair>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Deserialize)]
struct LocalKinshipSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalKinshipSmokeCase>,
}

#[cfg(feature = "bam_downstream")]
#[derive(Debug, Clone, Deserialize)]
struct LocalKinshipSmokeCase {
    sample_id: String,
    bam: PathBuf,
    reference_panel: String,
    reference_build: String,
    population_scope: String,
    min_overlap_snps: u32,
    requires_cohort_context: bool,
    expected_status: String,
    expected_observed_max_overlap_snps: u32,
    #[serde(default)]
    expected_insufficiency_reason: Option<String>,
    #[serde(default)]
    expected_pairwise_results: Vec<LocalKinshipSmokeExpectedPair>,
}

#[cfg(feature = "bam_downstream")]
/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.kinship` plans cannot be built.
pub fn local_kinship_smoke_plans(repo_root: &Path) -> Result<Vec<LocalKinshipSmokeCasePlan>> {
    let config = load_local_kinship_smoke_config(repo_root)?;
    ensure_unique_kinship_sample_ids(&config.cases)?;

    let stage = BamStage::Kinship;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.kinship tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_KINSHIP_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_kinship_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

#[cfg(feature = "bam_downstream")]
/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or one of
/// the admitted `bam.kinship` tool plans cannot be built for output-contract proof coverage.
pub fn local_kinship_output_contract_plans(
    repo_root: &Path,
) -> Result<Vec<LocalKinshipSmokeCasePlan>> {
    let config = load_local_kinship_smoke_config(repo_root)?;
    ensure_unique_kinship_sample_ids(&config.cases)?;

    let stage = BamStage::Kinship;
    let stage_id = StageId::new(stage.as_str().to_string());
    let mut tool_ids = allowed_tools_for_stage(stage);
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let output_root = config
        .output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_KINSHIP_OUTPUT_DIR));
    let cases = config.cases;

    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        for case in &cases {
            plans.push(build_local_kinship_smoke_case(
                repo_root,
                &tool_spec,
                &output_root,
                case.clone(),
            )?);
        }
    }
    Ok(plans)
}

#[cfg(feature = "bam_downstream")]
fn build_local_kinship_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalKinshipSmokeCase,
) -> Result<LocalKinshipSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.kinship BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.reference_panel.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.kinship case `{}` must declare a non-empty reference_panel",
            case.sample_id
        ));
    }
    if case.reference_build.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.kinship case `{}` must declare a non-empty reference_build",
            case.sample_id
        ));
    }
    if case.population_scope.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.kinship case `{}` must declare a non-empty population_scope",
            case.sample_id
        ));
    }
    if case.min_overlap_snps == 0 {
        return Err(anyhow!(
            "local-smoke bam.kinship case `{}` must declare min_overlap_snps greater than zero",
            case.sample_id
        ));
    }
    match case.expected_status.as_str() {
        "ok" => {
            if case.expected_pairwise_results.is_empty() {
                return Err(anyhow!(
                    "local-smoke bam.kinship case `{}` must declare at least one expected pairwise result when expected_status is ok",
                    case.sample_id
                ));
            }
            if case.expected_insufficiency_reason.is_some() {
                return Err(anyhow!(
                    "local-smoke bam.kinship case `{}` must not declare expected_insufficiency_reason when expected_status is ok",
                    case.sample_id
                ));
            }
            if case.expected_observed_max_overlap_snps < case.min_overlap_snps {
                return Err(anyhow!(
                    "local-smoke bam.kinship case `{}` must keep expected_observed_max_overlap_snps at or above min_overlap_snps when expected_status is ok",
                    case.sample_id
                ));
            }
        }
        "insufficient" => {
            if !case.expected_pairwise_results.is_empty() {
                return Err(anyhow!(
                    "local-smoke bam.kinship case `{}` must not declare pairwise results when expected_status is insufficient",
                    case.sample_id
                ));
            }
            if case
                .expected_insufficiency_reason
                .as_deref()
                .is_none_or(|reason| reason.trim().is_empty())
            {
                return Err(anyhow!(
                    "local-smoke bam.kinship case `{}` must declare a non-empty expected_insufficiency_reason when expected_status is insufficient",
                    case.sample_id
                ));
            }
        }
        _ => {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must declare expected_status as `ok` or `insufficient`",
                case.sample_id
            ));
        }
    }

    let mut seen_pairs = BTreeSet::new();
    for pair in &case.expected_pairwise_results {
        if pair.sample_a.trim().is_empty() || pair.sample_b.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must keep pairwise sample names non-empty",
                case.sample_id
            ));
        }
        if pair.sample_a == pair.sample_b {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must keep pairwise sample names distinct",
                case.sample_id
            ));
        }
        if pair.overlap_snps == 0 {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must keep pairwise overlap_snps greater than zero",
                case.sample_id
            ));
        }
        if pair.matching_sites > pair.overlap_snps {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` cannot declare matching_sites greater than overlap_snps",
                case.sample_id
            ));
        }
        if pair.mismatch_sites != pair.overlap_snps.saturating_sub(pair.matching_sites) {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must keep mismatch_sites aligned with overlap_snps and matching_sites",
                case.sample_id
            ));
        }
        if !(0.0..=1.0).contains(&pair.concordance) {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must keep concordance within [0, 1]",
                case.sample_id
            ));
        }
        if pair.kinship_coefficient < 0.0 {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must keep kinship_coefficient non-negative",
                case.sample_id
            ));
        }
        if pair.relationship_label.trim().is_empty() {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` must declare a non-empty relationship_label",
                case.sample_id
            ));
        }
        let pair_key = if pair.sample_a < pair.sample_b {
            (pair.sample_a.clone(), pair.sample_b.clone())
        } else {
            (pair.sample_b.clone(), pair.sample_a.clone())
        };
        if !seen_pairs.insert(pair_key) {
            return Err(anyhow!(
                "local-smoke bam.kinship case `{}` declared a duplicate pairwise sample combination",
                case.sample_id
            ));
        }
    }

    let params = bijux_dna_domain_bam::params::KinshipEffectiveParams {
        reference_panel: case.reference_panel.clone(),
        reference_build: case.reference_build.clone(),
        population_scope: case.population_scope.clone(),
        min_overlap_snps: case.min_overlap_snps,
        requires_cohort_context: case.requires_cohort_context,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::kinship::plan(tool_spec, &case.bam, &out_dir, &params)?;

    Ok(LocalKinshipSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        reference_panel: case.reference_panel,
        reference_build: case.reference_build,
        population_scope: case.population_scope,
        min_overlap_snps: case.min_overlap_snps,
        requires_cohort_context: case.requires_cohort_context,
        expected_status: case.expected_status,
        expected_observed_max_overlap_snps: case.expected_observed_max_overlap_snps,
        expected_insufficiency_reason: case.expected_insufficiency_reason,
        expected_pairwise_results: case.expected_pairwise_results,
        plan,
    })
}

#[cfg(feature = "bam_downstream")]
fn ensure_unique_kinship_sample_ids(cases: &[LocalKinshipSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.kinship sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "local-smoke bam.kinship sample_id `{}` must be unique",
                case.sample_id
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "bam_downstream")]
fn load_local_kinship_smoke_config(repo_root: &Path) -> Result<LocalKinshipSmokeConfig> {
    let path = repo_root.join(LOCAL_KINSHIP_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalKinshipSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_kinship.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.kinship schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.kinship must declare at least one governed case"));
    }
    Ok(config)
}
