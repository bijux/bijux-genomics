//! Damage filtering admits scientific policy independently of process environment.
mod environment;
mod execution;
mod policy;

use super::*;
pub use execution::run_damage_filter_stage_with_policy;
pub use policy::{DamageFilterPolicy, DamageLibraryType, DamageMaskMode};

#[derive(Debug, Clone)]
pub struct DamageFilterStageParams {
    pub udg_regime: DamageUdgRegime,
    pub strict_regime: bool,
    pub min_qual: f64,
    pub max_damage_ratio: f64,
}

impl DamageFilterStageParams {
    #[must_use]
    pub fn recommended() -> Self {
        Self {
            udg_regime: DamageUdgRegime::Unknown,
            strict_regime: true,
            min_qual: 30.0,
            max_damage_ratio: 0.35,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DamageFilterOutputs {
    pub filtered_vcf: PathBuf,
    pub filtered_tbi: PathBuf,
    pub damage_filter_summary_json: PathBuf,
    pub damage_filter_counts_json: PathBuf,
    pub warnings_json: PathBuf,
    pub damage_genotype_manifest_json: PathBuf,
}

/// Capture legacy environment configuration once and execute damage filtering.
/// Concurrent services should use [`run_damage_filter_stage_with_policy`].
///
/// # Errors
/// Returns an error for an unknown strict regime, invalid policy or failed stage execution.
pub fn run_damage_filter_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &DamageFilterStageParams,
) -> Result<DamageFilterOutputs> {
    let policy = environment::capture(params.udg_regime);
    run_damage_filter_stage_with_policy(input_vcf, out_dir, params, &policy)
}
