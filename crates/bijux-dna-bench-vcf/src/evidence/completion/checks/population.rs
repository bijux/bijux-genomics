use super::super::contracts::render_and_validate_direct_vcf_ready_output;
use super::super::ports::DirectReadinessStage;
use super::super::{
    model::VcfAllRetainedToolsCompleteGoalCheck, ports::VcfCompletionPorts, state::CompletionState,
};
use super::record_goal_check;
use std::path::Path;
pub(super) fn pca(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        350,
        "vcf.pca readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::Pca).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::Pca,
                "vcf.pca readiness",
                2,
            )?;
            Ok("validated the governed eigensoft and plink2 `vcf.pca` rows end-to-end".to_string())
        },
    );
}
pub(super) fn admixture(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        351,
        "vcf.admixture readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::Admixture).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::Admixture,
                "vcf.admixture readiness",
                2,
            )?;
            Ok("validated the governed PLINK and PLINK2 `vcf.admixture` backend contracts"
                .to_string())
        },
    );
}
pub(super) fn population_structure(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        352,
        "vcf.population_structure readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::PopulationStructure).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::PopulationStructure,
                "vcf.population_structure readiness",
                3,
            )?;
            Ok("validated the governed EIGENSOFT, PLINK, and PLINK2 `vcf.population_structure` backend contracts".to_string())
        },
    );
}
