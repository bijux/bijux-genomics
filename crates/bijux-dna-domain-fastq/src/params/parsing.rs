use bijux_dna_core::ids::StageId;

use crate::stages::ids::{
    STAGE_CLUSTER_OTUS, STAGE_CORRECT_ERRORS, STAGE_DEPLETE_HOST,
    STAGE_DEPLETE_REFERENCE_CONTAMINANTS, STAGE_DEPLETE_RRNA, STAGE_DETECT_ADAPTERS,
    STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN, STAGE_EXTRACT_UMIS, STAGE_FILTER_LOW_COMPLEXITY,
    STAGE_FILTER_READS, STAGE_INDEX_REFERENCE, STAGE_INFER_ASVS, STAGE_MERGE_PAIRS,
    STAGE_NORMALIZE_ABUNDANCE, STAGE_NORMALIZE_PRIMERS, STAGE_PROFILE_OVERREPRESENTED_SEQUENCES,
    STAGE_PROFILE_READS, STAGE_PROFILE_READ_LENGTHS, STAGE_REMOVE_CHIMERAS,
    STAGE_REMOVE_DUPLICATES, STAGE_REPORT_QC, STAGE_SCREEN_TAXONOMY, STAGE_TRIM_POLYG_TAILS,
    STAGE_TRIM_READS, STAGE_TRIM_TERMINAL_DAMAGE, STAGE_VALIDATE_READS,
};

use super::{edna, processing, quality, EffectiveParams};

#[must_use]
fn parse_preprocess_effective_params(
    stage_id: &StageId,
    value: &serde_json::Value,
) -> Option<EffectiveParams> {
    if stage_id == &STAGE_VALIDATE_READS {
        return serde_json::from_value::<quality::validate::ValidateEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Validate);
    }
    if stage_id == &STAGE_PROFILE_READS {
        return serde_json::from_value::<quality::stats::FastqStatsParams>(value.clone())
            .ok()
            .map(EffectiveParams::Stats);
    }
    if stage_id == &STAGE_PROFILE_READ_LENGTHS {
        return serde_json::from_value::<quality::stats::FastqReadLengthProfileParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::ReadLengthProfile);
    }
    if stage_id == &STAGE_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN {
        return serde_json::from_value::<
            quality::stats::FastqEstimateLibraryComplexityPrealignParams,
        >(value.clone())
        .ok()
        .map(EffectiveParams::EstimateLibraryComplexityPrealign);
    }
    if stage_id == &STAGE_CORRECT_ERRORS {
        return serde_json::from_value::<processing::correct::FastqCorrectParams>(value.clone())
            .ok()
            .map(EffectiveParams::Correct);
    }
    if stage_id == &STAGE_EXTRACT_UMIS {
        return serde_json::from_value::<processing::umi::FastqUmiParams>(value.clone())
            .ok()
            .map(EffectiveParams::Umi);
    }
    if stage_id == &STAGE_DETECT_ADAPTERS {
        return serde_json::from_value::<quality::detect_adapters::DetectAdaptersEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::DetectAdapters);
    }
    if stage_id == &STAGE_TRIM_READS {
        return serde_json::from_value::<quality::trim::TrimEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Trim);
    }
    if stage_id == &STAGE_TRIM_TERMINAL_DAMAGE {
        return serde_json::from_value::<quality::trim::TrimTerminalDamageParams>(value.clone())
            .ok()
            .map(EffectiveParams::TrimTerminalDamage);
    }
    if stage_id == &STAGE_TRIM_POLYG_TAILS {
        return serde_json::from_value::<quality::trim::TrimPolygTailsParams>(value.clone())
            .ok()
            .map(EffectiveParams::TrimPolygTails);
    }
    if stage_id == &STAGE_REMOVE_DUPLICATES {
        return serde_json::from_value::<
            processing::remove_duplicates::RemoveDuplicatesEffectiveParams,
        >(value.clone())
        .ok()
        .map(EffectiveParams::RemoveDuplicates);
    }
    if stage_id == &STAGE_FILTER_READS || stage_id == &STAGE_FILTER_LOW_COMPLEXITY {
        return serde_json::from_value::<quality::filter::FilterEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Filter);
    }
    if stage_id == &STAGE_MERGE_PAIRS {
        return serde_json::from_value::<processing::merge::MergeEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Merge);
    }
    if stage_id == &STAGE_INDEX_REFERENCE {
        return serde_json::from_value::<processing::reference_index::ReferenceIndexEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::ReferenceIndex);
    }
    None
}

#[must_use]
fn parse_screen_effective_params(
    stage_id: &StageId,
    value: &serde_json::Value,
) -> Option<EffectiveParams> {
    if stage_id == &STAGE_DEPLETE_HOST {
        return serde_json::from_value::<quality::screen::HostDepletionEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::HostDepletion);
    }
    if stage_id == &STAGE_DEPLETE_REFERENCE_CONTAMINANTS {
        return serde_json::from_value::<quality::screen::ReferenceContaminantEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::ReferenceContaminantDepletion);
    }
    if stage_id == &STAGE_DEPLETE_RRNA {
        return serde_json::from_value::<quality::screen::RrnaEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Rrna);
    }
    if stage_id == &STAGE_SCREEN_TAXONOMY {
        return serde_json::from_value::<quality::screen::ScreenEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::Screen);
    }
    if stage_id == &STAGE_REPORT_QC {
        return serde_json::from_value::<quality::qc_post::QcPostEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::ReportQc);
    }
    if stage_id == &STAGE_PROFILE_OVERREPRESENTED_SEQUENCES {
        return serde_json::from_value::<quality::stats::FastqOverrepresentedProfileParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::OverrepresentedProfile);
    }
    None
}

#[must_use]
fn parse_amplicon_effective_params(
    stage_id: &StageId,
    value: &serde_json::Value,
) -> Option<EffectiveParams> {
    if stage_id == &STAGE_NORMALIZE_PRIMERS {
        return serde_json::from_value::<edna::PrimerNormalizationEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::PrimerNormalization);
    }
    if stage_id == &STAGE_REMOVE_CHIMERAS {
        return serde_json::from_value::<edna::ChimeraDetectionEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::ChimeraDetection);
    }
    if stage_id == &STAGE_INFER_ASVS {
        return serde_json::from_value::<edna::AsvInferenceEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::AsvInference);
    }
    if stage_id == &STAGE_CLUSTER_OTUS {
        return serde_json::from_value::<edna::OtuClusteringEffectiveParams>(value.clone())
            .ok()
            .map(EffectiveParams::OtuClustering);
    }
    if stage_id == &STAGE_NORMALIZE_ABUNDANCE {
        return serde_json::from_value::<edna::AbundanceNormalizationEffectiveParams>(
            value.clone(),
        )
        .ok()
        .map(EffectiveParams::AbundanceNormalization);
    }
    None
}

#[must_use]
pub fn parse_effective_params(
    stage_id: &StageId,
    value: &serde_json::Value,
) -> Option<EffectiveParams> {
    parse_preprocess_effective_params(stage_id, value)
        .or_else(|| parse_screen_effective_params(stage_id, value))
        .or_else(|| parse_amplicon_effective_params(stage_id, value))
}
