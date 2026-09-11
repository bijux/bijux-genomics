use super::{NO_VALUE, REQUIRED_METRIC_NAMES};
use crate::qc::{
    LocalVcfQcSmokeHeterozygosity, LocalVcfQcSmokeHweSummary, LocalVcfQcSmokeMafSummary,
};
use bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBinding;
use std::collections::BTreeMap;

pub(super) fn expected_result_id(
    row: &bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow,
) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        "vcf",
        &row.corpus_id,
        &row.stage_id,
        &row.asset_profile_id,
        &row.tool_id,
    )
}
pub(super) fn retained_result_id(binding: &VcfStageReadinessBinding) -> String {
    bijux_dna_core::ids::build_asset_profile_benchmark_result_id(
        "vcf",
        &binding.retained_row.corpus_id,
        &binding.retained_row.stage_id,
        &binding.retained_row.asset_profile_id,
        &binding.retained_row.tool_id,
    )
}
pub(super) fn contains_artifact_id(entries: &[String], expected_id: &str) -> bool {
    entries.iter().any(|entry| artifact_id(entry) == expected_id)
}
pub(super) fn artifact_id(entry: &str) -> &str {
    entry.split_once('=').map_or(entry, |(id, _)| id)
}
pub(super) fn required_metric_names() -> Vec<String> {
    REQUIRED_METRIC_NAMES.iter().map(|value| (*value).to_string()).collect()
}
pub(super) fn no_value_string() -> String {
    NO_VALUE.to_string()
}
pub(super) fn empty_maf_summary() -> LocalVcfQcSmokeMafSummary {
    LocalVcfQcSmokeMafSummary {
        allele_frequency_mean: 0.0,
        maf_bin_counts: BTreeMap::new(),
        observed_variant_count: 0,
    }
}
pub(super) fn empty_heterozygosity() -> LocalVcfQcSmokeHeterozygosity {
    LocalVcfQcSmokeHeterozygosity {
        heterozygous_call_count: 0,
        homozygous_alt_call_count: 0,
        het_hom_ratio: None,
    }
}
pub(super) fn empty_hwe_summary() -> LocalVcfQcSmokeHweSummary {
    LocalVcfQcSmokeHweSummary {
        tested_variant_count: 0,
        pvalue_mean: None,
        status: NO_VALUE.to_string(),
    }
}
