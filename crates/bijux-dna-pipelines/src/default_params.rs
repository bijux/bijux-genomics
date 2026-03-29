//! Typed pipeline default parameter envelopes.

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
use serde::de::Deserializer;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmptyParams {}

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
    Bam(BamEffectiveParams),
    Vcf(VcfEffectiveParams),
    Empty(EmptyParams),
}

impl DefaultParams {
    fn encode<T: Serialize>(value: &T, kind: &str) -> serde_json::Value {
        serde_json::to_value(value)
            .unwrap_or_else(|err| panic!("failed to serialize {kind} default params: {err}"))
    }

    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            DefaultParams::FastqValidate(value) => Self::encode(value, "fastq.validate"),
            DefaultParams::FastqStats(value) => Self::encode(value, "fastq.profile_reads"),
            DefaultParams::FastqReadLengthProfile(value) => {
                Self::encode(value, "fastq.profile_read_lengths")
            }
            DefaultParams::FastqCorrect(value) => Self::encode(value, "fastq.correct_errors"),
            DefaultParams::FastqUmi(value) => Self::encode(value, "fastq.extract_umis"),
            DefaultParams::FastqDetectAdapters(value) => {
                Self::encode(value, "fastq.detect_adapters")
            }
            DefaultParams::FastqTrim(value) => Self::encode(value, "fastq.trim_reads"),
            DefaultParams::FastqTrimTerminalDamage(value) => {
                Self::encode(value, "fastq.trim_terminal_damage")
            }
            DefaultParams::FastqTrimPolygTails(value) => {
                Self::encode(value, "fastq.trim_polyg_tails")
            }
            DefaultParams::FastqFilter(value) => Self::encode(value, "fastq.filter_reads"),
            DefaultParams::FastqOverrepresentedProfile(value) => {
                Self::encode(value, "fastq.profile_overrepresented_sequences")
            }
            DefaultParams::FastqQcPost(value) => Self::encode(value, "fastq.report_qc"),
            DefaultParams::FastqPreprocess(value) => Self::encode(value, "fastq.preprocess"),
            DefaultParams::FastqMerge(value) => Self::encode(value, "fastq.merge_pairs"),
            DefaultParams::FastqScreen(value) => Self::encode(value, "fastq.screen_taxonomy"),
            DefaultParams::Bam(value) => match value {
                BamEffectiveParams::Align(inner) => Self::encode(inner, "bam.align"),
                BamEffectiveParams::Validate(inner) => Self::encode(inner, "bam.validate"),
                BamEffectiveParams::QcPre(inner) => Self::encode(inner, "bam.qc_pre"),
                BamEffectiveParams::MappingSummary(inner) => {
                    Self::encode(inner, "bam.mapping_summary")
                }
                BamEffectiveParams::Filter(inner) => Self::encode(inner, "bam.filter"),
                BamEffectiveParams::MapqFilter(inner) => Self::encode(inner, "bam.mapq_filter"),
                BamEffectiveParams::LengthFilter(inner) => Self::encode(inner, "bam.length_filter"),
                BamEffectiveParams::Markdup(inner) => Self::encode(inner, "bam.markdup"),
                BamEffectiveParams::DuplicationMetrics(inner) => {
                    Self::encode(inner, "bam.duplication_metrics")
                }
                BamEffectiveParams::Complexity(inner) => Self::encode(inner, "bam.complexity"),
                BamEffectiveParams::Coverage(inner) => Self::encode(inner, "bam.coverage"),
                BamEffectiveParams::InsertSize(inner) => Self::encode(inner, "bam.insert_size"),
                BamEffectiveParams::GcBias(inner) => Self::encode(inner, "bam.gc_bias"),
                BamEffectiveParams::EndogenousContent(inner) => {
                    Self::encode(inner, "bam.endogenous_content")
                }
                BamEffectiveParams::OverlapCorrection(inner) => {
                    Self::encode(inner, "bam.overlap_correction")
                }
                BamEffectiveParams::Damage(inner) => Self::encode(inner, "bam.damage"),
                BamEffectiveParams::Authenticity(inner) => Self::encode(inner, "bam.authenticity"),
                BamEffectiveParams::Contamination(inner) => {
                    Self::encode(inner, "bam.contamination")
                }
                BamEffectiveParams::Sex(inner) => Self::encode(inner, "bam.sex"),
                BamEffectiveParams::BiasMitigation(inner) => {
                    Self::encode(inner, "bam.bias_mitigation")
                }
                BamEffectiveParams::Recalibration(inner) => {
                    Self::encode(inner, "bam.recalibration")
                }
                BamEffectiveParams::Haplogroups(inner) => Self::encode(inner, "bam.haplogroups"),
                BamEffectiveParams::Genotyping(inner) => Self::encode(inner, "bam.genotyping"),
                BamEffectiveParams::Kinship(inner) => Self::encode(inner, "bam.kinship"),
            },
            DefaultParams::Vcf(value) => match value {
                VcfEffectiveParams::Call(inner) => Self::encode(inner, "vcf.call"),
                VcfEffectiveParams::Filter(inner) => Self::encode(inner, "vcf.filter"),
                VcfEffectiveParams::Stats(inner) => Self::encode(inner, "vcf.stats"),
            },
            DefaultParams::Empty(_) => serde_json::json!({}),
        }
    }
}

impl Serialize for DefaultParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_json().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DefaultParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
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
        if let Ok(parsed) =
            serde_json::from_value::<FastqOverrepresentedProfileParams>(value.clone())
        {
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
        Ok(DefaultParams::Empty(EmptyParams {}))
    }
}
