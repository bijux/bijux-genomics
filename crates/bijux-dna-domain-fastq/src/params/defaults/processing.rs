use super::super::correct::{
    CorrectionEngine, FastqCorrectParams, QualityEncoding, CORRECT_SCHEMA_VERSION,
};
use super::super::merge::{
    MergeEffectiveParams, MergeEngine, UnmergedReadPolicy, MERGE_SCHEMA_VERSION,
};
use super::super::preprocess::{LibraryDamageTreatment, PreprocessEffectiveParams};
use super::super::remove_duplicates::{
    DedupMode, RemoveDuplicatesEffectiveParams, REMOVE_DUPLICATES_SCHEMA_VERSION,
};
use super::super::umi::{
    FastqUmiParams, UmiDownstreamPropagation, UmiExtractionLocation, UmiFailedExtractionPolicy,
    UmiReadNameTransform, UMI_SCHEMA_VERSION,
};
use super::shared::paired_mode;
use crate::pipeline_contract::FastqPipelineMode;

#[must_use]
pub fn correct_defaults(paired: bool) -> FastqCorrectParams {
    FastqCorrectParams {
        schema_version: CORRECT_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        correction_engine: CorrectionEngine::Rcorrector,
        quality_encoding: QualityEncoding::Phred33,
        kmer_size: None,
        musket_kmer_budget: None,
        genome_size: None,
        max_memory_gb: None,
        trusted_kmer_artifact: None,
        conservative_mode: false,
    }
}

#[must_use]
pub fn umi_defaults(paired: bool) -> FastqUmiParams {
    FastqUmiParams {
        schema_version: UMI_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        umi_pattern: None,
        extraction_location: UmiExtractionLocation::Read1Prefix,
        read_name_transform: UmiReadNameTransform::AppendToHeader,
        failed_extraction_policy: UmiFailedExtractionPolicy::RefuseStage,
        downstream_propagation: UmiDownstreamPropagation::HeaderAndReport,
    }
}

#[must_use]
pub fn preprocess_defaults(paired: bool) -> PreprocessEffectiveParams {
    PreprocessEffectiveParams {
        pipeline_mode: FastqPipelineMode::Shotgun,
        paired_mode: paired_mode(paired),
        library_declared_paired: paired,
        library_damage_treatment: LibraryDamageTreatment::NoUdg,
        threads: 1,
        stages: vec![
            "fastq.validate_reads".to_string(),
            "fastq.detect_adapters".to_string(),
            "fastq.trim_reads".to_string(),
            "fastq.filter_reads".to_string(),
            "fastq.profile_reads".to_string(),
            "fastq.report_qc".to_string(),
        ],
        enable_contaminant_removal: false,
    }
}

#[must_use]
pub fn merge_defaults(paired: bool) -> MergeEffectiveParams {
    MergeEffectiveParams {
        schema_version: MERGE_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 6,
        merge_overlap: None,
        min_len: None,
        merge_engine: MergeEngine::Pear,
        unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
    }
}

#[must_use]
pub fn remove_duplicates_defaults(paired: bool) -> RemoveDuplicatesEffectiveParams {
    RemoveDuplicatesEffectiveParams {
        schema_version: REMOVE_DUPLICATES_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 4,
        dedup_mode: DedupMode::Exact,
        keep_order: true,
    }
}
