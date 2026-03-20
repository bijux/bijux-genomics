use super::correct::{
    CorrectionEngine, FastqCorrectParams, QualityEncoding, CORRECT_SCHEMA_VERSION,
};
use super::detect_adapters::{
    AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode,
    DetectAdaptersEffectiveParams, DETECT_ADAPTERS_SCHEMA_VERSION,
};
use super::filter::FilterEffectiveParams;
use super::merge::{MergeEffectiveParams, MergeEngine, UnmergedReadPolicy, MERGE_SCHEMA_VERSION};
use super::preprocess::LibraryDamageTreatment;
use super::preprocess::PreprocessEffectiveParams;
use super::qc_post::{
    QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams, REPORT_QC_SCHEMA_VERSION,
};
use super::screen::{
    ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyDatabaseScope,
    TaxonomyReportFormat, SCREEN_TAXONOMY_SCHEMA_VERSION,
};
use super::stats::{FastqStatsParams, STATS_SCHEMA_VERSION};
use super::trim::{
    TrimEffectiveParams, TrimPolygTailsParams, TrimTerminalDamageParams,
    TRIM_POLYG_TAILS_SCHEMA_VERSION, TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
};
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
        correction_engine: CorrectionEngine::Rcorrector,
        quality_encoding: QualityEncoding::Phred33,
        kmer_size: None,
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
        evidence_scope: AdapterEvidenceScope::SampledReads,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "adapter_report".to_string(),
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
pub fn trim_terminal_damage_defaults(paired: bool) -> TrimTerminalDamageParams {
    TrimTerminalDamageParams {
        schema_version: TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        damage_mode: super::DamageMode::Ancient,
        trim_5p_bases: 2,
        trim_3p_bases: 2,
    }
}

#[must_use]
pub fn trim_polyg_tails_defaults(paired: bool) -> TrimPolygTailsParams {
    TrimPolygTailsParams {
        schema_version: TRIM_POLYG_TAILS_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        trim_polyg: true,
        min_polyg_run: 10,
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
        schema_version: REPORT_QC_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        aggregation_engine: QcAggregationEngine::Multiqc,
        aggregation_scope: QcAggregationScope::FastqQcInputs,
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
        threads: 1,
        merge_overlap: None,
        min_len: None,
        merge_engine: MergeEngine::Pear,
        unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
    }
}

#[must_use]
pub fn screen_defaults(paired: bool) -> ScreenEffectiveParams {
    ScreenEffectiveParams {
        schema_version: SCREEN_TAXONOMY_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        contaminant_db: None,
        database_catalog_id: "taxonomy_reference".to_string(),
        database_artifact_id: "taxonomy_db".to_string(),
        database_build_id: None,
        database_digest: None,
        database_namespace: Some("read_screening".to_string()),
        database_scope: TaxonomyDatabaseScope::ReadScreening,
        classifier: TaxonomyClassifier::Kraken2,
        report_format: TaxonomyReportFormat::KrakenReport,
        assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
        minimum_confidence: None,
        emit_unclassified: true,
    }
}
