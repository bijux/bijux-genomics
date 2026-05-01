use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_bam::stage_specs::BamStage;
use bijux_dna_domain_fastq::params::correct::FastqCorrectParams;
use bijux_dna_domain_fastq::params::defaults::{
    correct_defaults, detect_adapters_defaults, filter_defaults, merge_defaults,
    overrepresented_profile_defaults, preprocess_defaults, qc_post_defaults,
    read_length_profile_defaults, screen_defaults, stats_defaults, trim_defaults,
    trim_polyg_tails_defaults, trim_terminal_damage_defaults, umi_defaults, validate_defaults,
};
use bijux_dna_domain_fastq::params::detect_adapters::DetectAdaptersEffectiveParams;
use bijux_dna_domain_fastq::params::edna::{
    AbundanceNormalizationEffectiveParams, AsvInferenceEffectiveParams,
    ChimeraDetectionEffectiveParams, OtuClusteringEffectiveParams,
    PrimerNormalizationEffectiveParams,
};
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_dna_domain_fastq::params::preprocess::PreprocessEffectiveParams;
use bijux_dna_domain_fastq::params::qc_post::QcPostEffectiveParams;
use bijux_dna_domain_fastq::params::screen::{
    HostDepletionEffectiveParams, ReferenceContaminantEffectiveParams, RrnaEffectiveParams,
    ScreenEffectiveParams,
};
use bijux_dna_domain_fastq::params::stats::{
    FastqOverrepresentedProfileParams, FastqReadLengthProfileParams, FastqStatsParams,
};
use bijux_dna_domain_fastq::params::trim::{
    TrimEffectiveParams, TrimPolygTailsParams, TrimTerminalDamageParams,
};
use bijux_dna_domain_fastq::params::umi::FastqUmiParams;
use bijux_dna_domain_fastq::params::validate::ValidateEffectiveParams;
use bijux_dna_domain_vcf::params::{
    VcfCallParams, VcfEffectiveParams, VcfFilterParams, VcfStatsParams,
};
use bijux_dna_domain_vcf::VcfStage;

use crate::{DefaultParams, EmptyParams};

pub(super) fn from_json(value: serde_json::Value) -> anyhow::Result<DefaultParams> {
    if value.as_object().is_some_and(serde_json::Map::is_empty) {
        return Ok(DefaultParams::Empty(EmptyParams {}));
    }
    if let Ok(parsed) = serde_json::from_value::<BamEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::Bam(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<VcfEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::Vcf(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<ValidateEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqValidate(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<FastqStatsParams>(value.clone()) {
        return Ok(DefaultParams::FastqStats(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<FastqReadLengthProfileParams>(value.clone()) {
        return Ok(DefaultParams::FastqReadLengthProfile(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<FastqCorrectParams>(value.clone()) {
        return Ok(DefaultParams::FastqCorrect(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<FastqUmiParams>(value.clone()) {
        return Ok(DefaultParams::FastqUmi(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<DetectAdaptersEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqDetectAdapters(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<TrimTerminalDamageParams>(value.clone()) {
        return Ok(DefaultParams::FastqTrimTerminalDamage(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<TrimPolygTailsParams>(value.clone()) {
        return Ok(DefaultParams::FastqTrimPolygTails(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<TrimEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqTrim(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<FilterEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqFilter(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<FastqOverrepresentedProfileParams>(value.clone()) {
        return Ok(DefaultParams::FastqOverrepresentedProfile(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<QcPostEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqQcPost(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<PreprocessEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqPreprocess(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<MergeEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqMerge(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<ScreenEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqScreen(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<HostDepletionEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqHostDepletion(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<ReferenceContaminantEffectiveParams>(value.clone())
    {
        return Ok(DefaultParams::FastqReferenceContaminantDepletion(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<RrnaEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqRrna(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<PrimerNormalizationEffectiveParams>(value.clone())
    {
        return Ok(DefaultParams::FastqPrimerNormalization(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<ChimeraDetectionEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqChimeraDetection(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<AsvInferenceEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqAsvInference(parsed));
    }
    if let Ok(parsed) = serde_json::from_value::<OtuClusteringEffectiveParams>(value.clone()) {
        return Ok(DefaultParams::FastqOtuClustering(parsed));
    }
    if let Ok(parsed) =
        serde_json::from_value::<AbundanceNormalizationEffectiveParams>(value.clone())
    {
        return Ok(DefaultParams::FastqAbundanceNormalization(parsed));
    }
    Err(anyhow::anyhow!("unrecognized non-empty default params payload"))
}

pub(super) fn from_stage_json(
    stage_id: &str,
    value: serde_json::Value,
) -> anyhow::Result<DefaultParams> {
    if value.as_object().is_some_and(serde_json::Map::is_empty) {
        return Ok(DefaultParams::Empty(EmptyParams {}));
    }
    if let Ok(stage) = BamStage::try_from(stage_id) {
        return stage.parse_effective_params(&value).map(DefaultParams::Bam);
    }
    if let Ok(stage) = VcfStage::try_from(stage_id) {
        return parse_vcf_stage_params(stage, value).map(DefaultParams::Vcf);
    }
    if let Some(params) = parse_fastq_stage_params(stage_id, &value) {
        return params;
    }
    from_json(value)
}

fn parse_fastq_stage_params(
    stage_id: &str,
    value: &serde_json::Value,
) -> Option<anyhow::Result<DefaultParams>> {
    let paired = legacy_paired_mode(value);
    let default = match stage_id {
        "fastq.validate_reads" => DefaultParams::FastqValidate(validate_defaults(paired)),
        "fastq.profile_reads" => DefaultParams::FastqStats(stats_defaults(paired)),
        "fastq.profile_read_lengths" => {
            DefaultParams::FastqReadLengthProfile(read_length_profile_defaults(paired))
        }
        "fastq.correct_errors" => DefaultParams::FastqCorrect(correct_defaults(paired)),
        "fastq.extract_umis" => DefaultParams::FastqUmi(umi_defaults(paired)),
        "fastq.detect_adapters" => {
            DefaultParams::FastqDetectAdapters(detect_adapters_defaults(paired))
        }
        "fastq.trim_reads" => DefaultParams::FastqTrim(trim_defaults(paired)),
        "fastq.trim_terminal_damage" => {
            DefaultParams::FastqTrimTerminalDamage(trim_terminal_damage_defaults(paired))
        }
        "fastq.trim_polyg_tails" => {
            DefaultParams::FastqTrimPolygTails(trim_polyg_tails_defaults(paired))
        }
        "fastq.filter_reads" => DefaultParams::FastqFilter(filter_defaults(paired)),
        "fastq.profile_overrepresented_sequences" => {
            DefaultParams::FastqOverrepresentedProfile(overrepresented_profile_defaults(paired))
        }
        "fastq.report_qc" => DefaultParams::FastqQcPost(qc_post_defaults(paired)),
        "fastq.preprocess" => DefaultParams::FastqPreprocess(preprocess_defaults(paired)),
        "fastq.merge_pairs" => DefaultParams::FastqMerge(merge_defaults(paired)),
        "fastq.screen_taxonomy" => DefaultParams::FastqScreen(screen_defaults(paired)),
        _ => return None,
    };
    Some(from_json(overlay_known_default_fields(default.to_json(), value.clone())))
}

fn legacy_paired_mode(value: &serde_json::Value) -> bool {
    value
        .get("paired_mode")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|mode| mode == "paired_end")
}

fn overlay_known_default_fields(
    mut defaults: serde_json::Value,
    overrides: serde_json::Value,
) -> serde_json::Value {
    let (Some(defaults), Some(overrides)) = (defaults.as_object_mut(), overrides.as_object())
    else {
        return defaults;
    };
    for (key, value) in overrides {
        if defaults.contains_key(key) {
            defaults.insert(key.clone(), value.clone());
        }
    }
    serde_json::Value::Object(defaults.clone())
}

fn parse_vcf_stage_params(
    stage: VcfStage,
    value: serde_json::Value,
) -> anyhow::Result<VcfEffectiveParams> {
    match stage {
        VcfStage::Call => {
            serde_json::from_value::<VcfCallParams>(value).map(VcfEffectiveParams::Call)
        }
        VcfStage::Filter => {
            serde_json::from_value::<VcfFilterParams>(value).map(VcfEffectiveParams::Filter)
        }
        VcfStage::Stats => {
            serde_json::from_value::<VcfStatsParams>(value).map(VcfEffectiveParams::Stats)
        }
    }
    .map_err(|err| anyhow::anyhow!("failed to parse params for {}: {err}", stage.as_str()))
}
