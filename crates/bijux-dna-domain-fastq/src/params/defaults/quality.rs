use super::super::detect_adapters::{
    AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode,
    DetectAdaptersEffectiveParams, DETECT_ADAPTERS_SCHEMA_VERSION,
};
use super::super::filter::FilterEffectiveParams;
use super::super::qc_post::{
    QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams, REPORT_QC_SCHEMA_VERSION,
};
use super::super::screen::{
    ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyDatabaseScope,
    TaxonomyInterpretationBoundary, TaxonomyReportFormat, SCREEN_TAXONOMY_SCHEMA_VERSION,
};
use super::super::trim::{
    default_terminal_damage_execution_policy, TrimEffectiveParams, TrimPolygTailsParams,
    TrimTerminalDamageParams, DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES,
    DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES, TRIM_POLYG_TAILS_SCHEMA_VERSION,
    TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
};
use super::super::validate::{
    PairSyncPolicy, ValidateEffectiveParams, ValidationMode, VALIDATE_SCHEMA_VERSION,
};
use super::shared::paired_mode;
use crate::params::DamageMode;

#[must_use]
pub fn validate_defaults(paired: bool) -> ValidateEffectiveParams {
    ValidateEffectiveParams {
        schema_version: VALIDATE_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 4,
        validation_mode: ValidationMode::Strict,
        pair_sync_policy: if paired {
            PairSyncPolicy::RequireHeaderSync
        } else {
            PairSyncPolicy::NotApplicable
        },
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
        evidence_scope: AdapterEvidenceScope::FullInput,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "report_json".to_string(),
    }
}

#[must_use]
pub fn trim_defaults(paired: bool) -> TrimEffectiveParams {
    TrimEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        min_len: 30,
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
        damage_mode: DamageMode::Ancient,
        execution_policy: default_terminal_damage_execution_policy(),
        trim_5p_bases: DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES,
        trim_3p_bases: DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES,
        requested_trim_5p_bases: None,
        requested_trim_3p_bases: None,
    }
}

#[must_use]
pub fn trim_polyg_tails_defaults(paired: bool) -> TrimPolygTailsParams {
    TrimPolygTailsParams {
        schema_version: TRIM_POLYG_TAILS_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 4,
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
        aggregation_engine: QcAggregationEngine::Multiqc,
        aggregation_scope: QcAggregationScope::GovernedQcArtifacts,
    }
}

#[must_use]
pub fn screen_defaults(paired: bool) -> ScreenEffectiveParams {
    ScreenEffectiveParams {
        schema_version: SCREEN_TAXONOMY_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 4,
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
        interpretation_boundary: TaxonomyInterpretationBoundary::ScreeningOnly,
        truth_conditions: Vec::new(),
    }
}
