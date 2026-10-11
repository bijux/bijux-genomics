use super::super::contracts::render_and_validate_direct_vcf_ready_output;
use super::super::ports::DirectReadinessStage;
use super::super::{
    model::VcfAllRetainedToolsCompleteGoalCheck, ports::VcfCompletionPorts, state::CompletionState,
};
use super::record_goal_check;
use std::path::Path;
pub(super) fn call(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        337,
        "vcf.call readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::Call).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::Call,
                "vcf.call readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.call` row end-to-end".to_string())
        },
    );
}
pub(super) fn call_diploid(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        338,
        "vcf.call_diploid readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::CallDiploid).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::CallDiploid,
                "vcf.call_diploid readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.call_diploid` row end-to-end".to_string())
        },
    );
}
pub(super) fn call_pseudohaploid(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        339,
        "vcf.call_pseudohaploid readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::CallPseudohaploid).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::CallPseudohaploid,
                "vcf.call_pseudohaploid readiness",
                2,
            )?;
            Ok("validated governed bcftools and ANGSD `vcf.call_pseudohaploid` rows end-to-end"
                .to_string())
        },
    );
}
pub(super) fn call_gl(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        340,
        "vcf.call_gl readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::CallGl).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::CallGl,
                "vcf.call_gl readiness",
                2,
            )?;
            Ok("validated governed bcftools and ANGSD `vcf.call_gl` rows end-to-end".to_string())
        },
    );
}
pub(super) fn gl_propagation(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        342,
        "vcf.gl_propagation readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::GlPropagation).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::GlPropagation,
                "vcf.gl_propagation readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.gl_propagation` row end-to-end".to_string())
        },
    );
}
