use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::detect_adapters::DetectAdaptersEffectiveParams;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;

use crate::{DefaultParams, PipelineProfile};

fn default_params_for<'a>(
    profile: &'a PipelineProfile,
    stage_id: &str,
) -> Option<&'a DefaultParams> {
    let stage = StageId::new(stage_id.to_string());
    profile.defaults.params.get(&stage)
}

pub(super) fn trim_params(profile: &PipelineProfile) -> Option<&TrimEffectiveParams> {
    match default_params_for(profile, bijux_dna_core::prelude::id_catalog::FASTQ_TRIM) {
        Some(DefaultParams::FastqTrim(params)) => Some(params),
        _ => None,
    }
}

pub(super) fn filter_params(profile: &PipelineProfile) -> Option<&FilterEffectiveParams> {
    match default_params_for(profile, bijux_dna_core::prelude::id_catalog::FASTQ_FILTER) {
        Some(DefaultParams::FastqFilter(params)) => Some(params),
        _ => None,
    }
}

pub(super) fn detect_adapters_params(
    profile: &PipelineProfile,
) -> Option<&DetectAdaptersEffectiveParams> {
    match default_params_for(
        profile,
        bijux_dna_core::prelude::id_catalog::FASTQ_DETECT_ADAPTERS,
    ) {
        Some(DefaultParams::FastqDetectAdapters(params)) => Some(params),
        _ => None,
    }
}

pub(super) fn merge_params(profile: &PipelineProfile) -> Option<&MergeEffectiveParams> {
    match default_params_for(profile, bijux_dna_core::prelude::id_catalog::FASTQ_MERGE) {
        Some(DefaultParams::FastqMerge(params)) => Some(params),
        _ => None,
    }
}

pub(super) fn screen_params(profile: &PipelineProfile) -> Option<&ScreenEffectiveParams> {
    match default_params_for(profile, bijux_dna_core::prelude::id_catalog::FASTQ_SCREEN) {
        Some(DefaultParams::FastqScreen(params)) => Some(params),
        _ => None,
    }
}
