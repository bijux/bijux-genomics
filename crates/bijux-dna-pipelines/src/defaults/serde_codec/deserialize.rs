use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_fastq::params::correct::FastqCorrectParams;
use bijux_dna_domain_fastq::params::detect_adapters::DetectAdaptersEffectiveParams;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_dna_domain_fastq::params::preprocess::PreprocessEffectiveParams;
use bijux_dna_domain_fastq::params::qc_post::QcPostEffectiveParams;
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::params::stats::{
    FastqOverrepresentedProfileParams, FastqReadLengthProfileParams, FastqStatsParams,
};
use bijux_dna_domain_fastq::params::trim::{
    TrimEffectiveParams, TrimPolygTailsParams, TrimTerminalDamageParams,
};
use bijux_dna_domain_fastq::params::umi::FastqUmiParams;
use bijux_dna_domain_fastq::params::validate::ValidateEffectiveParams;
use bijux_dna_domain_vcf::params::VcfEffectiveParams;

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
    Err(anyhow::anyhow!("unrecognized non-empty default params payload"))
}
