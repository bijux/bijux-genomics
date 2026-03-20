use super::correct::{FastqCorrectParams, CORRECT_SCHEMA_VERSION};
use super::detect_adapters::{
    AdapterInspectionMode, DetectAdaptersEffectiveParams, DETECT_ADAPTERS_SCHEMA_VERSION,
};
use super::filter::FilterEffectiveParams;
use super::merge::MergeEffectiveParams;
use super::preprocess::LibraryDamageTreatment;
use super::preprocess::PreprocessEffectiveParams;
use super::qc_post::QcPostEffectiveParams;
use super::screen::{
    ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyReportFormat,
    SCREEN_TAXONOMY_SCHEMA_VERSION,
};
use super::stats::{FastqStatsParams, STATS_SCHEMA_VERSION};
use super::trim::TrimEffectiveParams;
use super::umi::{FastqUmiParams, UMI_SCHEMA_VERSION};
use super::validate::ValidateEffectiveParams;
use super::PairedMode;
use crate::pipeline_contract::FastqPipelineMode;

fn paired_mode(paired: bool) -> PairedMode {
    if paired {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    }
}

#[must_use]
pub fn validate_defaults(paired: bool) -> ValidateEffectiveParams {
    ValidateEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        q_cutoff: None,
    }
}

#[must_use]
pub fn stats_defaults(paired: bool) -> FastqStatsParams {
    FastqStatsParams {
        schema_version: STATS_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
    }
}

#[must_use]
pub fn correct_defaults(paired: bool) -> FastqCorrectParams {
    FastqCorrectParams {
        schema_version: CORRECT_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        kmer_size: None,
    }
}

#[must_use]
pub fn umi_defaults(paired: bool) -> FastqUmiParams {
    FastqUmiParams {
        schema_version: UMI_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        umi_pattern: None,
    }
}

#[must_use]
pub fn detect_adapters_defaults(paired: bool) -> DetectAdaptersEffectiveParams {
    DetectAdaptersEffectiveParams {
        schema_version: DETECT_ADAPTERS_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        sample_reads: None,
        inspection_mode: AdapterInspectionMode::EvidenceOnly,
        report_only: true,
        evidence_engine: "fastqc".to_string(),
    }
}

#[must_use]
pub fn trim_defaults(paired: bool) -> TrimEffectiveParams {
    TrimEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        min_len: 0,
        q_cutoff: None,
        adapter_policy: "none".to_string(),
        damage_mode: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
    }
}

#[must_use]
pub fn filter_defaults(paired: bool) -> FilterEffectiveParams {
    FilterEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        max_n: None,
        max_n_fraction: None,
        max_n_count: None,
        low_complexity_threshold: None,
        entropy_threshold: None,
        contaminant_db: None,
        n_policy: None,
        polyx_policy: None,
        damage_mode: None,
    }
}

#[must_use]
pub fn qc_post_defaults(paired: bool) -> QcPostEffectiveParams {
    QcPostEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
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
        paired_mode: paired_mode(paired),
        threads: 1,
        merge_overlap: None,
        min_len: None,
    }
}

#[must_use]
pub fn screen_defaults(paired: bool) -> ScreenEffectiveParams {
    ScreenEffectiveParams {
        schema_version: SCREEN_TAXONOMY_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        contaminant_db: None,
        classifier: TaxonomyClassifier::Kraken2,
        report_format: TaxonomyReportFormat::KrakenReport,
        assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
    }
}
