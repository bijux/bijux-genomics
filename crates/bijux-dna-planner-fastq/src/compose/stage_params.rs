use anyhow::{anyhow, Result};
use bijux_dna_domain_fastq::params::defaults::{screen_defaults, validate_defaults};
use bijux_dna_domain_fastq::params::{
    edna::ChimeraDetectionEffectiveParams,
    qc_post::{
        QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams, REPORT_QC_SCHEMA_VERSION,
    },
    remove_duplicates::RemoveDuplicatesEffectiveParams,
    screen::ScreenEffectiveParams,
    stats::FastqStatsParams,
    trim::{TrimEffectiveParams, TrimPolygTailsParams},
    validate::ValidateEffectiveParams,
    PairedMode,
};
use bijux_dna_domain_fastq::{FastqOverrepresentedProfileParams, FastqReadLengthProfileParams};

use crate::{
    ClusterOtusStageParams, CorrectErrorsStageParams, DepleteHostStageParams,
    DepleteReferenceContaminantsStageParams, DepleteRrnaStageParams, DetectAdaptersStageParams,
    ExtractUmisStageParams, FastqStageBinding, FastqStageParameters,
    FilterLowComplexityStageParams, FilterReadsStageParams, IndexReferenceStageParams,
    InferAsvsStageParams, MergePairsStageParams, NormalizeAbundanceStageParams,
    NormalizePrimersStageParams, TrimTerminalDamageStageParams,
};

pub(super) fn ensure_stage_params_match(binding: &FastqStageBinding) -> Result<()> {
    let Some(params) = binding.params.as_ref() else {
        return Ok(());
    };
    let (expected_stage_id, params_name) = stage_params_contract(params);
    if binding.stage_id == expected_stage_id {
        return Ok(());
    }
    Err(anyhow!(
        "FASTQ stage {} received {params_name} parameters for {}; use the matching FastqStageParameters variant",
        binding.stage_id,
        expected_stage_id
    ))
}

fn stage_params_contract(params: &FastqStageParameters) -> (&'static str, &'static str) {
    quality_params_contract(params)
        .or_else(|| processing_params_contract(params))
        .or_else(|| amplicon_and_reference_params_contract(params))
        .unwrap_or_else(|| unreachable!("all FASTQ stage parameter variants must map to a stage"))
}

fn quality_params_contract(params: &FastqStageParameters) -> Option<(&'static str, &'static str)> {
    match params {
        FastqStageParameters::Validate(_) => Some(("fastq.validate_reads", "Validate")),
        FastqStageParameters::DetectAdapters(_) => {
            Some(("fastq.detect_adapters", "DetectAdapters"))
        }
        FastqStageParameters::FilterReads(_) => Some(("fastq.filter_reads", "FilterReads")),
        FastqStageParameters::FilterLowComplexity(_) => {
            Some(("fastq.filter_low_complexity", "FilterLowComplexity"))
        }
        FastqStageParameters::ProfileReadLengths(_) => {
            Some(("fastq.profile_read_lengths", "ProfileReadLengths"))
        }
        FastqStageParameters::ProfileOverrepresented(_) => {
            Some(("fastq.profile_overrepresented_sequences", "ProfileOverrepresented"))
        }
        FastqStageParameters::ProfileReads(_) => Some(("fastq.profile_reads", "ProfileReads")),
        FastqStageParameters::ReportQc(_) => Some(("fastq.report_qc", "ReportQc")),
        FastqStageParameters::Screen(_) => Some(("fastq.screen_taxonomy", "Screen")),
        FastqStageParameters::TrimTerminalDamage(_) => {
            Some(("fastq.trim_terminal_damage", "TrimTerminalDamage"))
        }
        FastqStageParameters::DepleteRrna(_) => Some(("fastq.deplete_rrna", "DepleteRrna")),
        FastqStageParameters::DepleteHost(_) => Some(("fastq.deplete_host", "DepleteHost")),
        FastqStageParameters::DepleteReferenceContaminants(_) => {
            Some(("fastq.deplete_reference_contaminants", "DepleteReferenceContaminants"))
        }
        _ => None,
    }
}

fn processing_params_contract(
    params: &FastqStageParameters,
) -> Option<(&'static str, &'static str)> {
    match params {
        FastqStageParameters::ExtractUmis(_) => Some(("fastq.extract_umis", "ExtractUmis")),
        FastqStageParameters::RemoveDuplicates(_) => {
            Some(("fastq.remove_duplicates", "RemoveDuplicates"))
        }
        FastqStageParameters::Trim(_) => Some(("fastq.trim_reads", "Trim")),
        FastqStageParameters::TrimPolygTails(_) => {
            Some(("fastq.trim_polyg_tails", "TrimPolygTails"))
        }
        FastqStageParameters::MergePairs(_) => Some(("fastq.merge_pairs", "MergePairs")),
        FastqStageParameters::CorrectErrors(_) => Some(("fastq.correct_errors", "CorrectErrors")),
        _ => None,
    }
}

fn amplicon_and_reference_params_contract(
    params: &FastqStageParameters,
) -> Option<(&'static str, &'static str)> {
    match params {
        FastqStageParameters::RemoveChimeras(_) => {
            Some(("fastq.remove_chimeras", "RemoveChimeras"))
        }
        FastqStageParameters::NormalizePrimers(_) => {
            Some(("fastq.normalize_primers", "NormalizePrimers"))
        }
        FastqStageParameters::NormalizeAbundance(_) => {
            Some(("fastq.normalize_abundance", "NormalizeAbundance"))
        }
        FastqStageParameters::IndexReference(_) => {
            Some(("fastq.index_reference", "IndexReference"))
        }
        FastqStageParameters::InferAsvs(_) => Some(("fastq.infer_asvs", "InferAsvs")),
        FastqStageParameters::ClusterOtus(_) => Some(("fastq.cluster_otus", "ClusterOtus")),
        _ => None,
    }
}

pub(super) fn trim_terminal_damage_params(
    binding: &FastqStageBinding,
) -> TrimTerminalDamageStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::TrimTerminalDamage(params)) => params.clone(),
        _ => TrimTerminalDamageStageParams::baseline(),
    }
}

pub(super) fn validate_reads_params(
    binding: &FastqStageBinding,
    paired: bool,
) -> ValidateEffectiveParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::Validate(params)) => params.clone(),
        _ => validate_defaults(paired),
    }
}

pub(super) fn filter_reads_plan_options(
    binding: &FastqStageBinding,
) -> crate::tool_adapters::fastq::filter_reads::FilterPlanOptions {
    let params = match binding.params.as_ref() {
        Some(FastqStageParameters::FilterReads(params)) => params.clone(),
        _ => FilterReadsStageParams::default(),
    };
    crate::tool_adapters::fastq::filter_reads::FilterPlanOptions {
        threads: params.threads,
        max_n: params.max_n,
        max_n_fraction: params.max_n_fraction,
        max_n_count: params.max_n_count,
        low_complexity_threshold: params.low_complexity_threshold,
        entropy_threshold: params.entropy_threshold,
        kmer_ref: params.kmer_ref,
        redundant_filters: Vec::new(),
        polyx_policy: params.polyx_policy,
    }
}

pub(super) fn filter_low_complexity_plan_options(
    binding: &FastqStageBinding,
) -> crate::tool_adapters::fastq::filter_low_complexity::LowComplexityPlanOptions {
    let params = match binding.params.as_ref() {
        Some(FastqStageParameters::FilterLowComplexity(params)) => params.clone(),
        _ => FilterLowComplexityStageParams::default(),
    };
    crate::tool_adapters::fastq::filter_low_complexity::LowComplexityPlanOptions {
        entropy_threshold: params.entropy_threshold,
        polyx_threshold: params.polyx_threshold,
    }
}

pub(super) fn extract_umis_params(binding: &FastqStageBinding) -> ExtractUmisStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::ExtractUmis(params)) => params.clone(),
        _ => ExtractUmisStageParams::default(),
    }
}

pub(super) fn detect_adapters_params(binding: &FastqStageBinding) -> DetectAdaptersStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::DetectAdapters(params)) => params.clone(),
        _ => DetectAdaptersStageParams::default(),
    }
}

pub(super) fn profile_read_lengths_params(
    binding: &FastqStageBinding,
) -> Option<FastqReadLengthProfileParams> {
    match binding.params.as_ref() {
        Some(FastqStageParameters::ProfileReadLengths(params)) => Some(params.clone()),
        _ => None,
    }
}

pub(super) fn profile_overrepresented_params(
    binding: &FastqStageBinding,
) -> Option<FastqOverrepresentedProfileParams> {
    match binding.params.as_ref() {
        Some(FastqStageParameters::ProfileOverrepresented(params)) => Some(params.clone()),
        _ => None,
    }
}

pub(super) fn profile_reads_params(binding: &FastqStageBinding) -> Option<FastqStatsParams> {
    match binding.params.as_ref() {
        Some(FastqStageParameters::ProfileReads(params)) => Some(params.clone()),
        _ => None,
    }
}

pub(super) fn remove_duplicates_params(
    binding: &FastqStageBinding,
) -> Option<RemoveDuplicatesEffectiveParams> {
    match binding.params.as_ref() {
        Some(FastqStageParameters::RemoveDuplicates(params)) => Some(params.clone()),
        _ => None,
    }
}

pub(super) fn remove_chimeras_params(
    binding: &FastqStageBinding,
) -> Option<ChimeraDetectionEffectiveParams> {
    match binding.params.as_ref() {
        Some(FastqStageParameters::RemoveChimeras(params)) => Some(params.clone()),
        _ => None,
    }
}

pub(super) fn report_qc_params(
    binding: &FastqStageBinding,
    paired_mode: PairedMode,
) -> QcPostEffectiveParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::ReportQc(params)) => params.clone(),
        _ => QcPostEffectiveParams {
            schema_version: REPORT_QC_SCHEMA_VERSION.to_string(),
            paired_mode,
            aggregation_engine: QcAggregationEngine::Multiqc,
            aggregation_scope: QcAggregationScope::GovernedQcArtifacts,
        },
    }
}

pub(super) fn trim_reads_options(
    binding: &FastqStageBinding,
) -> Option<crate::tool_adapters::fastq::trim_reads::TrimPlanOptions> {
    match binding.params.as_ref() {
        Some(FastqStageParameters::Trim(params)) => Some(trim_plan_options(params)),
        _ => None,
    }
}

pub(super) fn trim_polyg_options(
    binding: &FastqStageBinding,
) -> Option<crate::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions> {
    match binding.params.as_ref() {
        Some(FastqStageParameters::TrimPolygTails(params)) => Some(trim_polyg_plan_options(params)),
        _ => None,
    }
}

pub(super) fn screen_params(binding: &FastqStageBinding, paired: bool) -> ScreenEffectiveParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::Screen(params)) => params.clone(),
        _ => screen_defaults(paired),
    }
}

fn trim_polyg_plan_options(
    params: &TrimPolygTailsParams,
) -> crate::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions {
    crate::tool_adapters::fastq::trim_polyg_tails::TrimPolygPlanOptions {
        threads: Some(params.threads),
        trim_polyg: params.trim_polyg,
        min_polyg_run: params.min_polyg_run,
    }
}

fn trim_plan_options(
    params: &TrimEffectiveParams,
) -> crate::tool_adapters::fastq::trim_reads::TrimPlanOptions {
    crate::tool_adapters::fastq::trim_reads::TrimPlanOptions {
        threads: Some(params.threads),
        min_length: Some(params.min_len),
        quality_cutoff: params.q_cutoff,
        n_policy: params.n_policy.clone(),
        adapter_policy: Some(params.adapter_policy.clone()),
        polyx_policy: params.polyx_policy.clone(),
        contaminant_policy: params.contaminant_policy.clone(),
    }
}

pub(super) fn merge_pairs_plan_options(
    binding: &FastqStageBinding,
) -> crate::tool_adapters::fastq::merge_pairs::MergePlanOptions {
    let params = match binding.params.as_ref() {
        Some(FastqStageParameters::MergePairs(params)) => params.clone(),
        _ => MergePairsStageParams::baseline(),
    };
    crate::tool_adapters::fastq::merge_pairs::MergePlanOptions {
        threads: params.threads,
        merge_overlap: params.merge_overlap,
        min_length: params.min_len,
        unmerged_read_policy: params.unmerged_read_policy,
    }
}

pub(super) fn normalize_abundance_plan_options(
    binding: &FastqStageBinding,
) -> crate::tool_adapters::fastq::normalize_abundance::NormalizeAbundancePlanOptions {
    let params = match binding.params.as_ref() {
        Some(FastqStageParameters::NormalizeAbundance(params)) => params.clone(),
        _ => NormalizeAbundanceStageParams::baseline(),
    };
    crate::tool_adapters::fastq::normalize_abundance::NormalizeAbundancePlanOptions {
        method: params.method,
    }
}

pub(super) fn normalize_primers_plan_options(
    binding: &FastqStageBinding,
) -> crate::tool_adapters::fastq::normalize_primers::NormalizePrimersPlanOptions {
    let params = match binding.params.as_ref() {
        Some(FastqStageParameters::NormalizePrimers(params)) => params.clone(),
        _ => NormalizePrimersStageParams::baseline(),
    };
    crate::tool_adapters::fastq::normalize_primers::NormalizePrimersPlanOptions {
        primer_set_id: params.primer_set_id,
        marker_id: params.marker_id,
        primer_fasta: params.primer_fasta,
        orientation_policy: params.orientation_policy,
        max_mismatch_rate: params.max_mismatch_rate,
        min_overlap_bp: params.min_overlap_bp,
        strict_5p_anchor: params.strict_5p_anchor,
        allow_iupac_codes: params.allow_iupac_codes,
    }
}

pub(super) fn index_reference_params(binding: &FastqStageBinding) -> IndexReferenceStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::IndexReference(params)) => params.clone(),
        _ => IndexReferenceStageParams::default(),
    }
}

pub(super) fn correct_errors_params(binding: &FastqStageBinding) -> CorrectErrorsStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::CorrectErrors(params)) => params.clone(),
        _ => CorrectErrorsStageParams::baseline(),
    }
}

pub(super) fn infer_asvs_params(binding: &FastqStageBinding) -> InferAsvsStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::InferAsvs(params)) => params.clone(),
        _ => InferAsvsStageParams::baseline(),
    }
}

pub(super) fn cluster_otus_params(binding: &FastqStageBinding) -> ClusterOtusStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::ClusterOtus(params)) => params.clone(),
        _ => ClusterOtusStageParams::baseline(),
    }
}

pub(super) fn deplete_rrna_params(binding: &FastqStageBinding) -> DepleteRrnaStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::DepleteRrna(params)) => params.clone(),
        _ => DepleteRrnaStageParams::baseline(),
    }
}

pub(super) fn deplete_host_params(binding: &FastqStageBinding) -> DepleteHostStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::DepleteHost(params)) => params.clone(),
        _ => DepleteHostStageParams::baseline(),
    }
}

pub(super) fn deplete_reference_contaminants_params(
    binding: &FastqStageBinding,
) -> DepleteReferenceContaminantsStageParams {
    match binding.params.as_ref() {
        Some(FastqStageParameters::DepleteReferenceContaminants(params)) => params.clone(),
        _ => DepleteReferenceContaminantsStageParams::baseline(),
    }
}
