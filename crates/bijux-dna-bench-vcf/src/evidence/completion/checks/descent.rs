use super::super::contracts::{
    ensure_descent_family_output, ensure_descent_family_output_contract, read_governed_json,
    PublishedAdapterReport,
};
use super::super::{
    model::VcfAllRetainedToolsCompleteGoalCheck, ports::VcfCompletionPorts, state::CompletionState,
};
use super::record_goal_check;
use std::path::Path;
use std::path::PathBuf;
pub(super) fn runs_of_homozygosity(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        353,
        "vcf.roh retained family coverage",
        Some(crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || {
            (ports.adapters.descent)(
                repo_root,
                PathBuf::from(crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
            )?;
            let payload: PublishedAdapterReport = read_governed_json(
                repo_root,
                crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
            )?;
            ensure_descent_family_output_contract(&payload)?;
            ensure_descent_family_output(&payload, "plink2", "vcf.roh", "benchmark_ready")?;
            Ok(
                "validated the governed plink2 `vcf.roh` retained-family row with benchmark-ready command coverage"
                    .to_string(),
            )
        },
    );
}
pub(super) fn identity_by_descent(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        354,
        "vcf.ibd retained family coverage",
        Some(crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || {
            (ports.adapters.descent)(
                repo_root,
                PathBuf::from(crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
            )?;
            let payload: PublishedAdapterReport = read_governed_json(
                repo_root,
                crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
            )?;
            ensure_descent_family_output_contract(&payload)?;
            ensure_descent_family_output(&payload, "germline", "vcf.ibd", "benchmark_ready")?;
            ensure_descent_family_output(&payload, "ibdhap", "vcf.ibd", "benchmark_ready")?;
            ensure_descent_family_output(&payload, "ibdseq", "vcf.ibd", "benchmark_ready")?;
            Ok("validated benchmark-ready segment-call rows for GERMLINE, IBDHap, and IBDseq"
                .to_string())
        },
    );
}
pub(super) fn demography(
    repo_root: &Path,
    ports: &VcfCompletionPorts,
    _state: &mut CompletionState,
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
) {
    record_goal_check(
        checks,
        355,
        "vcf.demography retained family coverage",
        Some(crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || {
            (ports.adapters.descent)(
                repo_root,
                PathBuf::from(crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
            )?;
            let payload: PublishedAdapterReport = read_governed_json(
                repo_root,
                crate::adapters::descent::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
            )?;
            ensure_descent_family_output_contract(&payload)?;
            ensure_descent_family_output(&payload, "ibdne", "vcf.demography", "benchmark_ready")?;
            Ok(
                "validated the governed ibdne `vcf.demography` retained-family row with benchmark-ready command coverage"
                    .to_string(),
            )
        },
    );
}
