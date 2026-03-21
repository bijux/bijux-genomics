use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::correct::{
    CorrectionEngine, FastqCorrectParams, QualityEncoding, CORRECT_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::defaults::detect_adapters_defaults;
use bijux_dna_domain_fastq::params::defaults::{
    correct_defaults, remove_duplicates_defaults, stats_defaults, trim_polyg_tails_defaults,
    trim_terminal_damage_defaults, umi_defaults,
};
use bijux_dna_domain_fastq::params::detect_adapters::{
    AdapterInspectionMode, DetectAdaptersEffectiveParams, DETECT_ADAPTERS_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::edna::{
    AbundanceNormalizationEffectiveParams, ChimeraDetectionEffectiveParams,
    OtuClusteringEffectiveParams, DEFAULT_OTU_IDENTITY_THRESHOLD,
};
use bijux_dna_domain_fastq::params::merge::{
    MergeEffectiveParams, MergeEngine, UnmergedReadPolicy, MERGE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::qc_post::{
    QcAggregationEngine, QcAggregationScope, QcPostEffectiveParams, REPORT_QC_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::remove_duplicates::{
    DedupMode, RemoveDuplicatesEffectiveParams, REMOVE_DUPLICATES_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::screen::{
    HostDepletionEffectiveParams, MappingReportFormat, ReadRetentionPolicy, ReferenceDecoyPolicy,
    ReferenceMaskingPolicy, ReferenceScope, RrnaEffectiveParams, RrnaReportFormat,
    RrnaScreeningEngine, ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier,
    TaxonomyDatabaseScope, TaxonomyReportFormat, HOST_DEPLETION_SCHEMA_VERSION,
    RRNA_DEPLETION_SCHEMA_VERSION, SCREEN_TAXONOMY_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::stats::{FastqStatsParams, STATS_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::trim::{
    TrimPolygTailsParams, TrimTerminalDamageParams, TRIM_POLYG_TAILS_SCHEMA_VERSION,
    TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::umi::{FastqUmiParams, UMI_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};
use bijux_dna_domain_fastq::{parse_effective_params, stage_param_descriptor, EffectiveParams};

fn roundtrip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let json = serde_json::to_value(value).unwrap_or_else(|err| panic!("to_value failed: {err}"));
    serde_json::from_value(json).unwrap_or_else(|err| panic!("from_value failed: {err}"))
}

#[test]
fn stats_params_roundtrip_and_schema_version() {
    let params = stats_defaults(true);
    let decoded: FastqStatsParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, STATS_SCHEMA_VERSION);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn correct_params_roundtrip_and_schema_version() {
    let params = correct_defaults(true);
    let decoded: FastqCorrectParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, CORRECT_SCHEMA_VERSION);
    assert_eq!(decoded.correction_engine, CorrectionEngine::Rcorrector);
    assert_eq!(decoded.quality_encoding, QualityEncoding::Phred33);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn umi_params_roundtrip_and_schema_version() {
    let params = umi_defaults(true);
    let decoded: FastqUmiParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, UMI_SCHEMA_VERSION);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn detect_adapters_params_roundtrip_and_remain_inspection_only() {
    let params = detect_adapters_defaults(true);
    let decoded: DetectAdaptersEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, DETECT_ADAPTERS_SCHEMA_VERSION);
    assert_eq!(decoded.inspection_mode, AdapterInspectionMode::EvidenceOnly);
    assert!(decoded.report_only);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn trim_terminal_damage_params_roundtrip_with_stage_specific_schema() {
    let params = trim_terminal_damage_defaults(true);
    let decoded: TrimTerminalDamageParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION);
    assert_eq!(decoded.damage_mode, DamageMode::Ancient);
    assert_eq!(decoded.trim_5p_bases, 2);
    assert_eq!(decoded.trim_3p_bases, 2);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn trim_polyg_tails_params_roundtrip_with_stage_specific_schema() {
    let params = trim_polyg_tails_defaults(true);
    let decoded: TrimPolygTailsParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, TRIM_POLYG_TAILS_SCHEMA_VERSION);
    assert!(decoded.trim_polyg);
    assert_eq!(decoded.min_polyg_run, 10);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn remove_duplicates_params_roundtrip_with_stage_specific_schema() {
    let params = remove_duplicates_defaults(true);
    let decoded: RemoveDuplicatesEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, REMOVE_DUPLICATES_SCHEMA_VERSION);
    assert_eq!(decoded.dedup_mode, DedupMode::Exact);
    assert!(decoded.keep_order);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn remove_duplicates_descriptor_and_parser_stay_in_sync() {
    let stage_id = StageId::from_static("fastq.remove_duplicates");
    let descriptor = stage_param_descriptor(&stage_id).expect("remove_duplicates descriptor");
    assert_eq!(descriptor.param_type_id, "fastq.remove_duplicates");
    assert_eq!(descriptor.schema_version, REMOVE_DUPLICATES_SCHEMA_VERSION);

    let params = remove_duplicates_defaults(true);
    let value = serde_json::to_value(&params).expect("serialize remove_duplicates params");
    let parsed = parse_effective_params(&stage_id, &value)
        .expect("parse remove_duplicates effective params");
    match parsed {
        EffectiveParams::RemoveDuplicates(parsed) => assert_eq!(parsed, params),
        other => panic!("unexpected effective params variant: {other:?}"),
    }
}

#[test]
fn merge_params_roundtrip_with_engine_specific_output_policy() {
    let params = MergeEffectiveParams {
        schema_version: MERGE_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 2,
        merge_overlap: Some(10),
        min_len: Some(30),
        merge_engine: MergeEngine::Pear,
        unmerged_read_policy: UnmergedReadPolicy::EmitUnmergedPairs,
    };
    let decoded: MergeEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, MERGE_SCHEMA_VERSION);
    assert_eq!(decoded.merge_engine, MergeEngine::Pear);
    assert_eq!(
        decoded.unmerged_read_policy,
        UnmergedReadPolicy::EmitUnmergedPairs,
    );
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn chimera_params_roundtrip_with_sequence_artifact_contract() {
    let params = ChimeraDetectionEffectiveParams {
        method: "vsearch_uchime_denovo".to_string(),
        detection_scope: "denovo".to_string(),
        chimera_sequence_artifact: "chimeras_fasta".to_string(),
        chimera_removed_definition:
            "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                .to_string(),
    };
    let decoded: ChimeraDetectionEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.method, "vsearch_uchime_denovo");
    assert_eq!(decoded.detection_scope, "denovo");
    assert_eq!(decoded.chimera_sequence_artifact, "chimeras_fasta");
}

#[test]
fn otu_clustering_params_roundtrip_with_domain_default_threshold() {
    let params = OtuClusteringEffectiveParams {
        identity_threshold: DEFAULT_OTU_IDENTITY_THRESHOLD,
        output_table_kind: "otu_abundance_table".to_string(),
    };
    let decoded: OtuClusteringEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.identity_threshold, DEFAULT_OTU_IDENTITY_THRESHOLD,);
    assert_eq!(decoded.output_table_kind, "otu_abundance_table");
}

#[test]
fn abundance_normalization_params_roundtrip_with_output_semantics() {
    let params = AbundanceNormalizationEffectiveParams {
        method: "relative_abundance".to_string(),
        expected_columns: vec![
            "sample_id".to_string(),
            "feature_id".to_string(),
            "abundance".to_string(),
        ],
        normalized_value_column: "normalized_abundance".to_string(),
        compositional_rule: "per_sample_sum_to_one".to_string(),
    };
    let decoded: AbundanceNormalizationEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.method, "relative_abundance");
    assert_eq!(decoded.expected_columns.len(), 3);
    assert_eq!(decoded.normalized_value_column, "normalized_abundance");
    assert_eq!(decoded.compositional_rule, "per_sample_sum_to_one");
}

#[test]
fn host_depletion_params_roundtrip_with_reference_provenance_fields() {
    let params = HostDepletionEffectiveParams {
        schema_version: HOST_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 8,
        reference_scope: ReferenceScope::Host,
        reference_catalog_id: "host_reference".to_string(),
        reference_index_artifact_id: "reference_index".to_string(),
        reference_index_backend: "bowtie2_build".to_string(),
        reference_build_id: Some("grch38_no_alt".to_string()),
        reference_digest: Some("sha256:host-ref".to_string()),
        masking_policy: ReferenceMaskingPolicy::HardMasked,
        decoy_policy: ReferenceDecoyPolicy::Included,
        decoy_catalog_id: Some("host_decoys".to_string()),
        identity_threshold: 0.95,
        retained_read_policy: ReadRetentionPolicy::KeepNonHostReads,
        emit_removed_reads: true,
        report_format: MappingReportFormat::Bowtie2MetricsFile,
        retain_unmapped_pairs: true,
    };
    let decoded: HostDepletionEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, HOST_DEPLETION_SCHEMA_VERSION);
    assert_eq!(decoded.reference_scope, ReferenceScope::Host);
    assert_eq!(decoded.reference_catalog_id, "host_reference");
    assert_eq!(decoded.reference_index_artifact_id, "reference_index");
    assert_eq!(decoded.reference_index_backend, "bowtie2_build");
    assert_eq!(decoded.reference_build_id.as_deref(), Some("grch38_no_alt"));
    assert_eq!(decoded.reference_digest.as_deref(), Some("sha256:host-ref"));
    assert_eq!(decoded.masking_policy, ReferenceMaskingPolicy::HardMasked);
    assert_eq!(decoded.decoy_policy, ReferenceDecoyPolicy::Included);
    assert_eq!(decoded.decoy_catalog_id.as_deref(), Some("host_decoys"));
    assert_eq!(decoded.identity_threshold, 0.95);
    assert_eq!(
        decoded.retained_read_policy,
        ReadRetentionPolicy::KeepNonHostReads,
    );
    assert!(decoded.emit_removed_reads);
    assert_eq!(
        decoded.report_format,
        MappingReportFormat::Bowtie2MetricsFile,
    );
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn screen_taxonomy_params_roundtrip_with_classifier_contract() {
    let params = ScreenEffectiveParams {
        schema_version: SCREEN_TAXONOMY_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 4,
        contaminant_db: None,
        database_catalog_id: "taxonomy_reference".to_string(),
        database_artifact_id: "taxonomy_db".to_string(),
        database_build_id: Some("kraken2-standard-2025-01".to_string()),
        database_digest: Some("sha256:taxonomy-db".to_string()),
        database_namespace: Some("read_screening".to_string()),
        database_scope: TaxonomyDatabaseScope::ReadScreening,
        classifier: TaxonomyClassifier::Kraken2,
        report_format: TaxonomyReportFormat::KrakenReport,
        assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
        minimum_confidence: Some(0.2),
        emit_unclassified: true,
    };
    let decoded: ScreenEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, SCREEN_TAXONOMY_SCHEMA_VERSION);
    assert_eq!(decoded.database_catalog_id, "taxonomy_reference");
    assert_eq!(decoded.database_artifact_id, "taxonomy_db");
    assert_eq!(
        decoded.database_build_id.as_deref(),
        Some("kraken2-standard-2025-01"),
    );
    assert_eq!(
        decoded.database_digest.as_deref(),
        Some("sha256:taxonomy-db")
    );
    assert_eq!(
        decoded.database_namespace.as_deref(),
        Some("read_screening")
    );
    assert_eq!(decoded.database_scope, TaxonomyDatabaseScope::ReadScreening);
    assert_eq!(decoded.classifier, TaxonomyClassifier::Kraken2);
    assert_eq!(decoded.report_format, TaxonomyReportFormat::KrakenReport);
    assert_eq!(
        decoded.assignment_format,
        TaxonomyAssignmentFormat::KrakenAssignments,
    );
    assert_eq!(decoded.minimum_confidence, Some(0.2));
    assert!(decoded.emit_unclassified);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn rrna_params_roundtrip_with_database_contract() {
    let params = RrnaEffectiveParams {
        schema_version: RRNA_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::SingleEnd,
        threads: 4,
        contaminant_db: Some("silva-138".to_string()),
        database_artifact_id: "rrna_reference".to_string(),
        database_build_id: Some("silva-138.1".to_string()),
        screening_engine: RrnaScreeningEngine::Sortmerna,
        report_format: RrnaReportFormat::SummaryTsvAndJson,
        emit_removed_reads: false,
    };
    let decoded: RrnaEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, RRNA_DEPLETION_SCHEMA_VERSION);
    assert_eq!(decoded.database_artifact_id, "rrna_reference");
    assert_eq!(decoded.database_build_id.as_deref(), Some("silva-138.1"));
    assert_eq!(decoded.screening_engine, RrnaScreeningEngine::Sortmerna);
    assert_eq!(decoded.report_format, RrnaReportFormat::SummaryTsvAndJson);
    assert!(!decoded.emit_removed_reads);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn report_qc_params_roundtrip_with_multiqc_contract() {
    let params = QcPostEffectiveParams {
        schema_version: REPORT_QC_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 2,
        aggregation_engine: QcAggregationEngine::Multiqc,
        aggregation_scope: QcAggregationScope::FastqQcInputs,
    };
    let decoded: QcPostEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, REPORT_QC_SCHEMA_VERSION);
    assert_eq!(decoded.aggregation_engine, QcAggregationEngine::Multiqc);
    assert_eq!(decoded.aggregation_scope, QcAggregationScope::FastqQcInputs);
    assert!(decoded.missing_required_fields().is_empty());
}
