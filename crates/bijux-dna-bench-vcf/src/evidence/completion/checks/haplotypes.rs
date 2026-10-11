use super::super::contracts::{
    ensure_imputation_family_output, ensure_single_phasing_output, read_governed_json,
    render_and_validate_direct_vcf_ready_output, PublishedAdapterReport,
};
use super::super::ports::DirectReadinessStage;
use super::super::{
    model::VcfAllRetainedToolsCompleteGoalCheck, ports::VcfCompletionPorts, state::CompletionState,
};
use super::record_goal_check;
use anyhow::bail;
use std::path::Path;
use std::path::PathBuf;
pub(super) fn prepare_reference_panel(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        346,
        "vcf.prepare_reference_panel readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::PrepareReferencePanel).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::PrepareReferencePanel,
                "vcf.prepare_reference_panel readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.prepare_reference_panel` row end-to-end"
                .to_string())
        },
    );
}
pub(super) fn phasing(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        347,
        "vcf.phasing retained family coverage",
        Some(crate::adapters::phasing::DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH.to_string()),
        || {
            for (tool_id, output_path) in [
                ("shapeit", crate::adapters::phasing::DEFAULT_VCF_SHAPEIT_ADAPTER_PATH),
                ("shapeit5", crate::adapters::phasing::DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH),
                ("beagle", crate::adapters::phasing::DEFAULT_VCF_BEAGLE_ADAPTER_PATH),
                ("eagle", crate::adapters::phasing::DEFAULT_VCF_EAGLE_ADAPTER_PATH),
            ] {
                (ports.adapters.phasing)(repo_root, tool_id, PathBuf::from(output_path))?;
            }
            ensure_single_phasing_output(
                &read_governed_json(
                    repo_root,
                    crate::adapters::phasing::DEFAULT_VCF_SHAPEIT_ADAPTER_PATH,
                )?,
                "shapeit",
                "benchmark_ready",
                1,
                1,
                "shapeit phasing adapter",
            )?;
            ensure_single_phasing_output(
                &read_governed_json(
                    repo_root,
                    crate::adapters::phasing::DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
                )?,
                "shapeit5",
                "benchmark_ready",
                1,
                1,
                "shapeit5 phasing adapter",
            )?;
            ensure_single_phasing_output(
                &read_governed_json(
                    repo_root,
                    crate::adapters::phasing::DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
                )?,
                "beagle",
                "benchmark_ready",
                3,
                3,
                "beagle phasing adapter",
            )?;
            ensure_single_phasing_output(
                &read_governed_json(
                    repo_root,
                    crate::adapters::phasing::DEFAULT_VCF_EAGLE_ADAPTER_PATH,
                )?,
                "eagle",
                "benchmark_ready",
                1,
                1,
                "eagle phasing adapter",
            )?;
            Ok("validated executable benchmark-ready phasing rows for every supported backend"
                .to_string())
        },
    );
}
pub(super) fn imputation(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        348,
        "vcf.impute retained family coverage",
        Some(crate::adapters::imputation::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH.to_string()),
        || {
            (ports.adapters.imputation)(
                repo_root,
                PathBuf::from(
                    crate::adapters::imputation::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH,
                ),
            )?;
            let payload: PublishedAdapterReport = read_governed_json(
                repo_root,
                crate::adapters::imputation::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH,
            )?;
            if payload.row_count != Some(8)
                || payload.tool_count != Some(4)
                || payload.benchmark_ready_row_count != Some(8)
                || payload.parser_output_row_count != Some(8)
                || payload.missing_input_test_passed_row_count != Some(8)
            {
                bail!(
                    "VCF imputation-family coverage drifted: rows={}, tools={}, benchmark_ready={}, parser_outputs={}, missing_input_passed={}",
                    payload.row_count.unwrap_or_default(),
                    payload.tool_count.unwrap_or_default(),
                    payload.benchmark_ready_row_count.unwrap_or_default(),
                    payload.parser_output_row_count.unwrap_or_default(),
                    payload.missing_input_test_passed_row_count.unwrap_or_default()
                );
            }
            ensure_imputation_family_output(&payload, "beagle", "vcf.impute", "benchmark_ready")?;
            ensure_imputation_family_output(&payload, "glimpse", "vcf.impute", "benchmark_ready")?;
            ensure_imputation_family_output(&payload, "impute5", "vcf.impute", "benchmark_ready")?;
            ensure_imputation_family_output(&payload, "minimac4", "vcf.impute", "benchmark_ready")?;
            Ok("validated executable benchmark-ready imputation rows for every supported backend"
                .to_string())
        },
    );
}
pub(super) fn imputation_metrics(
    repo_root: &Path,
    ports: &VcfCompletionPorts<'_>,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        349,
        "vcf.imputation_metrics readiness",
        Some((ports.direct.output_path)(DirectReadinessStage::ImputationMetrics).to_string()),
        || {
            render_and_validate_direct_vcf_ready_output(
                repo_root,
                &ports.direct,
                DirectReadinessStage::ImputationMetrics,
                "vcf.imputation_metrics readiness",
                4,
            )?;
            Ok("validated governed imputation metrics rows for every supported backend end-to-end"
                .to_string())
        },
    );
}
