use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_vcf::params::VcfEffectiveParams;
use serde::Serialize;

use crate::DefaultParams;

fn encode<T: Serialize>(value: &T, kind: &str) -> serde_json::Value {
    serde_json::to_value(value)
        .unwrap_or_else(|err| panic!("failed to serialize {kind} default params: {err}"))
}

pub(super) fn to_json(params: &DefaultParams) -> serde_json::Value {
    match params {
        DefaultParams::FastqValidate(value) => encode(value, "fastq.validate"),
        DefaultParams::FastqStats(value) => encode(value, "fastq.profile_reads"),
        DefaultParams::FastqReadLengthProfile(value) => encode(value, "fastq.profile_read_lengths"),
        DefaultParams::FastqCorrect(value) => encode(value, "fastq.correct_errors"),
        DefaultParams::FastqUmi(value) => encode(value, "fastq.extract_umis"),
        DefaultParams::FastqDetectAdapters(value) => encode(value, "fastq.detect_adapters"),
        DefaultParams::FastqTrim(value) => encode(value, "fastq.trim_reads"),
        DefaultParams::FastqTrimTerminalDamage(value) => {
            encode(value, "fastq.trim_terminal_damage")
        }
        DefaultParams::FastqTrimPolygTails(value) => encode(value, "fastq.trim_polyg_tails"),
        DefaultParams::FastqFilter(value) => encode(value, "fastq.filter_reads"),
        DefaultParams::FastqOverrepresentedProfile(value) => {
            encode(value, "fastq.profile_overrepresented_sequences")
        }
        DefaultParams::FastqQcPost(value) => encode(value, "fastq.report_qc"),
        DefaultParams::FastqPreprocess(value) => encode(value, "fastq.preprocess"),
        DefaultParams::FastqMerge(value) => encode(value, "fastq.merge_pairs"),
        DefaultParams::FastqScreen(value) => encode(value, "fastq.screen_taxonomy"),
        DefaultParams::FastqHostDepletion(value) => encode(value, "fastq.deplete_host"),
        DefaultParams::FastqReferenceContaminantDepletion(value) => {
            encode(value, "fastq.deplete_reference_contaminants")
        }
        DefaultParams::FastqRrna(value) => encode(value, "fastq.deplete_rrna"),
        DefaultParams::FastqPrimerNormalization(value) => encode(value, "fastq.normalize_primers"),
        DefaultParams::FastqChimeraDetection(value) => encode(value, "fastq.remove_chimeras"),
        DefaultParams::FastqAsvInference(value) => encode(value, "fastq.infer_asvs"),
        DefaultParams::FastqOtuClustering(value) => encode(value, "fastq.cluster_otus"),
        DefaultParams::FastqAbundanceNormalization(value) => {
            encode(value, "fastq.normalize_abundance")
        }
        DefaultParams::Bam(value) => match value {
            BamEffectiveParams::Align(inner) => encode(inner, "bam.align"),
            BamEffectiveParams::Validate(inner) => encode(inner, "bam.validate"),
            BamEffectiveParams::QcPre(inner) => encode(inner, "bam.qc_pre"),
            BamEffectiveParams::MappingSummary(inner) => encode(inner, "bam.mapping_summary"),
            BamEffectiveParams::Filter(inner) => encode(inner, "bam.filter"),
            BamEffectiveParams::MapqFilter(inner) => encode(inner, "bam.mapq_filter"),
            BamEffectiveParams::LengthFilter(inner) => encode(inner, "bam.length_filter"),
            BamEffectiveParams::Markdup(inner) => encode(inner, "bam.markdup"),
            BamEffectiveParams::DuplicationMetrics(inner) => {
                encode(inner, "bam.duplication_metrics")
            }
            BamEffectiveParams::Complexity(inner) => encode(inner, "bam.complexity"),
            BamEffectiveParams::Coverage(inner) => encode(inner, "bam.coverage"),
            BamEffectiveParams::InsertSize(inner) => encode(inner, "bam.insert_size"),
            BamEffectiveParams::GcBias(inner) => encode(inner, "bam.gc_bias"),
            BamEffectiveParams::EndogenousContent(inner) => encode(inner, "bam.endogenous_content"),
            BamEffectiveParams::OverlapCorrection(inner) => encode(inner, "bam.overlap_correction"),
            BamEffectiveParams::Damage(inner) => encode(inner, "bam.damage"),
            BamEffectiveParams::Authenticity(inner) => encode(inner, "bam.authenticity"),
            BamEffectiveParams::Contamination(inner) => encode(inner, "bam.contamination"),
            BamEffectiveParams::Sex(inner) => encode(inner, "bam.sex"),
            BamEffectiveParams::BiasMitigation(inner) => encode(inner, "bam.bias_mitigation"),
            BamEffectiveParams::Recalibration(inner) => encode(inner, "bam.recalibration"),
            BamEffectiveParams::Haplogroups(inner) => encode(inner, "bam.haplogroups"),
            BamEffectiveParams::Genotyping(inner) => encode(inner, "bam.genotyping"),
            BamEffectiveParams::Kinship(inner) => encode(inner, "bam.kinship"),
        },
        DefaultParams::Vcf(value) => match value {
            VcfEffectiveParams::Call(inner) => encode(inner, "vcf.call"),
            VcfEffectiveParams::Filter(inner) => encode(inner, "vcf.filter"),
            VcfEffectiveParams::Stats(inner) => encode(inner, "vcf.stats"),
        },
        DefaultParams::Empty(_) => serde_json::json!({}),
    }
}
