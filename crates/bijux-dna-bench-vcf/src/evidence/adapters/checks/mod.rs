use super::ports::VcfAdapterReadinessPorts;
use super::state::ReadinessState;
use crate::adapters::angsd::DEFAULT_VCF_ANGSD_ADAPTER_PATH;
use crate::adapters::bcftools::DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH;
use crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH;
use crate::adapters::eigensoft::DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH;
use crate::adapters::imputation::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH;
use crate::adapters::observations::VcfAdapterObservations;
use crate::adapters::phasing::DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH;
use crate::evidence::commands::DEFAULT_VCF_RENDERED_COMMANDS_PATH;
use crate::evidence::inputs::DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH;
use crate::evidence::output_coverage::DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH;
use crate::evidence::registry::matrix::DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH;
use crate::evidence::registry::orphans::DEFAULT_VCF_ORPHAN_TOOLS_PATH;
use crate::evidence::registry::undercoverage::DEFAULT_VCF_UNDERCOVERED_STAGES_PATH;
use crate::evidence::serving::DEFAULT_VCF_TOOL_SERVING_MAP_PATH;
mod commands;
mod completeness;
mod genotypes;
mod haplotypes;
mod inputs;
mod outputs;
mod population;
mod record;
mod registry;
use super::{model::VcfAdaptersReadyGoalCheck, DEFAULT_VCF_ADAPTERS_READY_PATH};
use record::record_goal_check;
pub(super) fn registry_checks(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
    checks: &mut Vec<VcfAdaptersReadyGoalCheck>,
) {
    record_goal_check(
        checks,
        231,
        "vcf tool-serving map",
        Some(DEFAULT_VCF_TOOL_SERVING_MAP_PATH.to_string()),
        || registry::serving(observations, state),
    );
    record_goal_check(
        checks,
        232,
        "vcf orphan tools",
        Some(DEFAULT_VCF_ORPHAN_TOOLS_PATH.to_string()),
        || registry::orphans(observations),
    );
    record_goal_check(
        checks,
        233,
        "vcf undercovered stages",
        Some(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH.to_string()),
        || registry::undercoverage(observations),
    );
    record_goal_check(
        checks,
        234,
        "vcf matrix-registry consistency",
        Some(DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH.to_string()),
        || registry::consistency(observations, state),
    );
}
pub(super) fn adapter_checks(
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
    checks: &mut Vec<VcfAdaptersReadyGoalCheck>,
) {
    record_goal_check(
        checks,
        235,
        "vcf bcftools adapter",
        Some(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH.to_string()),
        || genotypes::bcftools(observations, state),
    );
    record_goal_check(
        checks,
        236,
        "vcf angsd adapter",
        Some(DEFAULT_VCF_ANGSD_ADAPTER_PATH.to_string()),
        || genotypes::angsd(observations),
    );
    record_goal_check(
        checks,
        237,
        "vcf plink family adapters",
        Some("artifacts/benchmark-smoke/readiness/adapters/plink.vcf.json".to_string()),
        || population::plink(observations, state),
    );
    record_goal_check(
        checks,
        238,
        "vcf eigensoft adapter",
        Some(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH.to_string()),
        || population::eigensoft(observations),
    );
    record_goal_check(
        checks,
        239,
        "vcf phasing family adapters",
        Some(DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH.to_string()),
        || haplotypes::phasing(observations),
    );
    record_goal_check(
        checks,
        240,
        "vcf imputation family adapter",
        Some(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH.to_string()),
        || haplotypes::imputation(observations),
    );
    record_goal_check(
        checks,
        241,
        "vcf descent family adapter",
        Some(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || haplotypes::descent(observations),
    );
}
pub(super) fn evidence_checks(
    run_id: &str,
    observations: &VcfAdapterObservations<'_>,
    state: &mut ReadinessState,
    checks: &mut Vec<VcfAdaptersReadyGoalCheck>,
    ports: &VcfAdapterReadinessPorts,
) {
    record_goal_check(
        checks,
        242,
        "vcf adapter output coverage",
        Some(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH.to_string()),
        || outputs::output_coverage(run_id, observations, state),
    );
    record_goal_check(
        checks,
        243,
        "vcf adapter missing-input tests",
        Some(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH.to_string()),
        || inputs::missing_inputs(observations),
    );
    record_goal_check(
        checks,
        244,
        "vcf rendered commands",
        Some(DEFAULT_VCF_RENDERED_COMMANDS_PATH.to_string()),
        || commands::rendered_commands(observations, ports, state),
    );
    record_goal_check(
        checks,
        245,
        "vcf adapter benchmark-ready pair completeness",
        Some(DEFAULT_VCF_ADAPTERS_READY_PATH.to_string()),
        || completeness::complete_pairs(observations, state),
    );
}
