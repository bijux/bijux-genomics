//! Typed pipeline default parameter envelopes.

use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_fastq::params::correct::FastqCorrectParams;
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
use bijux_dna_domain_vcf::params::VcfEffectiveParams;

use super::EmptyParams;

#[derive(Debug, Clone)]
pub enum DefaultParams {
    FastqValidate(ValidateEffectiveParams),
    FastqStats(FastqStatsParams),
    FastqReadLengthProfile(FastqReadLengthProfileParams),
    FastqCorrect(FastqCorrectParams),
    FastqUmi(FastqUmiParams),
    FastqDetectAdapters(DetectAdaptersEffectiveParams),
    FastqTrim(TrimEffectiveParams),
    FastqTrimTerminalDamage(TrimTerminalDamageParams),
    FastqTrimPolygTails(TrimPolygTailsParams),
    FastqFilter(FilterEffectiveParams),
    FastqOverrepresentedProfile(FastqOverrepresentedProfileParams),
    FastqQcPost(QcPostEffectiveParams),
    FastqPreprocess(PreprocessEffectiveParams),
    FastqMerge(MergeEffectiveParams),
    FastqScreen(ScreenEffectiveParams),
    FastqHostDepletion(HostDepletionEffectiveParams),
    FastqReferenceContaminantDepletion(ReferenceContaminantEffectiveParams),
    FastqRrna(RrnaEffectiveParams),
    FastqPrimerNormalization(PrimerNormalizationEffectiveParams),
    FastqChimeraDetection(ChimeraDetectionEffectiveParams),
    FastqAsvInference(AsvInferenceEffectiveParams),
    FastqOtuClustering(OtuClusteringEffectiveParams),
    FastqAbundanceNormalization(AbundanceNormalizationEffectiveParams),
    Bam(BamEffectiveParams),
    Vcf(VcfEffectiveParams),
    Empty(EmptyParams),
}
