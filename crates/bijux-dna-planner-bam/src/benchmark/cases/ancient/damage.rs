//! Governed BAM benchmark ancient damage.
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
use bijux_dna_domain_bam::params::DamageEffectiveParams;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_DAMAGE_CONFIG_PATH: &str = "configs/bench/local/bam-damage.toml";

const DEFAULT_LOCAL_DAMAGE_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.damage";

#[derive(Debug, Clone)]
pub struct LocalDamageSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub reference: PathBuf,
    pub expected_terminal_c_to_t_5p: f64,
    pub expected_terminal_g_to_a_3p: f64,
    pub expected_short_fragment_fraction: f64,
    pub expected_damage_signal: String,
    pub expected_strict_profile_upgraded: bool,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalDamageSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalDamageSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalDamageSmokeCase {
    sample_id: String,
    bam: PathBuf,
    reference: PathBuf,
    expected_terminal_c_to_t_5p: f64,
    expected_terminal_g_to_a_3p: f64,
    expected_short_fragment_fraction: f64,
    expected_damage_signal: String,
    expected_strict_profile_upgraded: bool,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.damage` plans cannot be built.
pub fn local_damage_smoke_plans(repo_root: &Path) -> Result<Vec<LocalDamageSmokeCasePlan>> {
    let config = load_local_damage_smoke_config(repo_root)?;
    ensure_unique_damage_sample_ids(&config.cases)?;

    let stage = BamStage::Damage;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.damage tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DAMAGE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_damage_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or one of
/// the admitted `bam.damage` tool plans cannot be built for output-contract proof coverage.
pub fn local_damage_output_contract_plans(
    repo_root: &Path,
) -> Result<Vec<LocalDamageSmokeCasePlan>> {
    let config = load_local_damage_smoke_config(repo_root)?;
    ensure_unique_damage_sample_ids(&config.cases)?;

    let stage = BamStage::Damage;
    let stage_id = StageId::new(stage.as_str().to_string());
    let mut tool_ids = allowed_tools_for_stage(stage);
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_DAMAGE_OUTPUT_DIR));
    let cases = config.cases;

    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        for case in &cases {
            plans.push(build_local_damage_smoke_case(
                repo_root,
                &tool_spec,
                &output_root,
                case.clone(),
            )?);
        }
    }
    Ok(plans)
}

fn build_local_damage_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalDamageSmokeCase,
) -> Result<LocalDamageSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.damage BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_terminal_c_to_t_5p) {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` must keep expected_terminal_c_to_t_5p within [0, 1]",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_terminal_g_to_a_3p) {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` must keep expected_terminal_g_to_a_3p within [0, 1]",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_short_fragment_fraction) {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` must keep expected_short_fragment_fraction within [0, 1]",
            case.sample_id
        ));
    }
    if case.expected_damage_signal.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` must declare a non-empty expected_damage_signal",
            case.sample_id
        ));
    }
    if !matches!(case.expected_damage_signal.as_str(), "low" | "moderate" | "high") {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` must declare expected_damage_signal as one of `low`, `moderate`, or `high`",
            case.sample_id
        ));
    }
    let terminal_damage = case.expected_terminal_c_to_t_5p.max(case.expected_terminal_g_to_a_3p);
    let governed_damage_signal = if terminal_damage >= 0.20 {
        "high"
    } else if terminal_damage >= 0.10 {
        "moderate"
    } else {
        "low"
    };
    if case.expected_damage_signal != governed_damage_signal {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` must keep expected_damage_signal aligned with the governed terminal damage thresholds",
            case.sample_id
        ));
    }
    if case.expected_strict_profile_upgraded {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` must keep expected_strict_profile_upgraded false for the governed evidence-only damage profile",
            case.sample_id
        ));
    }

    let reference_abs = repo_root.join(&case.reference);
    if !reference_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.damage case `{}` reference does not exist: {}",
            case.sample_id,
            reference_abs.display()
        ));
    }
    let params = DamageEffectiveParams {
        udg_model: bijux_dna_domain_bam::params::UdgModel::NonUdg,
        pmd_threshold_5p: 0.3,
        pmd_threshold_3p: 0.3,
        trim_5p: 2,
        trim_3p: 2,
        damage_tool_profile: Some("ancient_dna_evidence".to_string()),
        evidence_only: true,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::damage::plan(
        tool_spec,
        &case.bam,
        Some(&case.reference),
        &out_dir,
        &params,
    )?;

    Ok(LocalDamageSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        reference: case.reference,
        expected_terminal_c_to_t_5p: case.expected_terminal_c_to_t_5p,
        expected_terminal_g_to_a_3p: case.expected_terminal_g_to_a_3p,
        expected_short_fragment_fraction: case.expected_short_fragment_fraction,
        expected_damage_signal: case.expected_damage_signal,
        expected_strict_profile_upgraded: case.expected_strict_profile_upgraded,
        plan,
    })
}

fn ensure_unique_damage_sample_ids(cases: &[LocalDamageSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.damage sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!("duplicate local-smoke bam.damage sample_id `{}`", case.sample_id));
        }
    }
    Ok(())
}

fn load_local_damage_smoke_config(repo_root: &Path) -> Result<LocalDamageSmokeConfig> {
    let path = repo_root.join(LOCAL_DAMAGE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalDamageSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_damage.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.damage schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.damage must declare at least one governed case"));
    }
    Ok(config)
}
