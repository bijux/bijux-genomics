use super::super::contracts::render_and_validate_direct_vcf_ready_output;
use super::super::ports::DirectReadinessStage;
use super::super::{
    model::VcfAllRetainedToolsCompleteGoalCheck, ports::VcfCompletionPorts, state::CompletionState,
};
use super::record_goal_check;
use std::path::Path;
pub(super) fn damage_filter(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        341,
        "vcf.damage_filter readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::DamageFilter).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::DamageFilter,
                "vcf.damage_filter readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.damage_filter` row end-to-end".to_string())
        },
    );
}
pub(super) fn filter(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        343,
        "vcf.filter readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::Filter).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::Filter,
                "vcf.filter readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.filter` row end-to-end".to_string())
        },
    );
}
pub(super) fn stats(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        344,
        "vcf.stats readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::Stats).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::Stats,
                "vcf.stats readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.stats` row end-to-end".to_string())
        },
    );
}
pub(super) fn qc(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        345,
        "vcf.qc readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::Qc).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::Qc,
                "vcf.qc readiness",
                3,
            )?;
            Ok("validated the governed `vcf.qc` rows for bcftools, plink, and plink2".to_string())
        },
    );
}
