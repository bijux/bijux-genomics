use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::correct::{
    CorrectionEngine, FastqCorrectParams, QualityEncoding, CORRECT_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::defaults::detect_adapters_defaults;
use bijux_dna_domain_fastq::params::defaults::{
    correct_defaults, merge_defaults, remove_duplicates_defaults, stats_defaults, trim_defaults,
    trim_polyg_tails_defaults, trim_terminal_damage_defaults, umi_defaults,
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
    TaxonomyDatabaseScope, TaxonomyInterpretationBoundary, TaxonomyReportFormat,
    HOST_DEPLETION_SCHEMA_VERSION, RRNA_DEPLETION_SCHEMA_VERSION, SCREEN_TAXONOMY_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::stats::{FastqStatsParams, STATS_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::trim::{
    default_terminal_damage_execution_policy, parse_terminal_damage_execution_policy,
    resolve_terminal_damage_policy, resolve_terminal_damage_policy_with_override,
    TerminalDamageExecutionPolicy, TrimPolygTailsParams, TrimTerminalDamageParams,
    TRIM_POLYG_TAILS_SCHEMA_VERSION, TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::umi::{
    FastqUmiParams, UmiDedupPolicy, UmiGroupingPolicy, UMI_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::validate::{PairSyncPolicy, ValidationMode};
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
    assert_eq!(decoded.threads, 2);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn correct_params_roundtrip_and_schema_version() {
    let params = correct_defaults(true);
    let decoded: FastqCorrectParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, CORRECT_SCHEMA_VERSION);
    assert_eq!(decoded.correction_engine, CorrectionEngine::Rcorrector);
    assert_eq!(decoded.quality_encoding, QualityEncoding::Phred33);
    assert_eq!(decoded.kmer_size, None);
    assert_eq!(decoded.musket_kmer_budget, None);
    assert_eq!(decoded.genome_size, None);
    assert_eq!(decoded.max_memory_gb, None);
    assert_eq!(decoded.trusted_kmer_artifact, None);
    assert!(!decoded.conservative_mode);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn umi_params_roundtrip_and_schema_version() {
    let params = umi_defaults(true);
    let decoded: FastqUmiParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, UMI_SCHEMA_VERSION);
    assert_eq!(decoded.grouping_policy, UmiGroupingPolicy::PairAware);
    assert_eq!(decoded.downstream_dedup_policy, UmiDedupPolicy::SequenceIdentityRecommended);
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
fn validate_params_roundtrip_supports_explicit_policy_variants() {
    let params = bijux_dna_domain_fastq::params::defaults::validate_defaults(true);
    let mut report_only = params.clone();
    report_only.validation_mode = ValidationMode::ReportOnly;
    report_only.pair_sync_policy = PairSyncPolicy::SkipHeaderSync;

    let decoded: bijux_dna_domain_fastq::params::validate::ValidateEffectiveParams =
        roundtrip(&report_only);
    assert_eq!(
        decoded.schema_version,
        bijux_dna_domain_fastq::params::validate::VALIDATE_SCHEMA_VERSION
    );
    assert_eq!(decoded.threads, 4);
    assert_eq!(decoded.validation_mode, ValidationMode::ReportOnly);
    assert_eq!(decoded.pair_sync_policy, PairSyncPolicy::SkipHeaderSync);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn screen_defaults_roundtrip_with_declared_thread_count() {
    let params = bijux_dna_domain_fastq::params::defaults::screen_defaults(true);
    let decoded: ScreenEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, SCREEN_TAXONOMY_SCHEMA_VERSION);
    assert_eq!(decoded.threads, 4);
    assert_eq!(decoded.classifier, TaxonomyClassifier::Kraken2);
    assert_eq!(decoded.report_format, TaxonomyReportFormat::KrakenReport);
    assert_eq!(decoded.assignment_format, TaxonomyAssignmentFormat::KrakenAssignments);
    assert_eq!(
        decoded.interpretation_boundary,
        bijux_dna_domain_fastq::params::screen::TaxonomyInterpretationBoundary::ScreeningOnly
    );
    assert!(decoded.truth_conditions.is_empty());
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn trim_terminal_damage_params_roundtrip_with_stage_specific_schema() {
    let params = trim_terminal_damage_defaults(true);
    let decoded: TrimTerminalDamageParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION);
    assert_eq!(decoded.damage_mode, DamageMode::Ancient);
    assert_eq!(decoded.execution_policy, default_terminal_damage_execution_policy());
    assert_eq!(decoded.trim_5p_bases, 2);
    assert_eq!(decoded.trim_3p_bases, 2);
    assert_eq!(decoded.requested_trim_5p_bases, None);
    assert_eq!(decoded.requested_trim_3p_bases, None);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn trim_defaults_keep_declared_minimum_read_length() {
    let decoded = roundtrip(&trim_defaults(true));
    assert_eq!(decoded.min_len, 30);
    assert_eq!(decoded.adapter_policy, "none");
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn merge_defaults_roundtrip_with_declared_thread_count() {
    let decoded: MergeEffectiveParams = roundtrip(&merge_defaults(true));
    assert_eq!(decoded.schema_version, MERGE_SCHEMA_VERSION);
    assert_eq!(decoded.threads, 6);
    assert_eq!(decoded.merge_engine, MergeEngine::Pear);
    assert_eq!(decoded.unmerged_read_policy, UnmergedReadPolicy::EmitUnmergedPairs);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn udg_trimmed_default_policy_preserves_terminal_ends() {
    let policy = resolve_terminal_damage_policy(DamageMode::UdgTrimmed, 2, 2);
    assert_eq!(policy.execution_policy, TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds);
    assert_eq!(policy.effective_trim_5p_bases, 0);
    assert_eq!(policy.effective_trim_3p_bases, 0);
    assert_eq!(policy.requested_trim_5p_bases, 2);
    assert_eq!(policy.requested_trim_3p_bases, 2);
}

#[test]
fn explicit_terminal_damage_policy_override_forces_trim_execution() {
    let policy = resolve_terminal_damage_policy_with_override(
        DamageMode::UdgTrimmed,
        2,
        2,
        Some(TerminalDamageExecutionPolicy::ExplicitTerminalTrim),
    )
    .unwrap_or_else(|err| panic!("explicit override must resolve: {err}"));
    assert_eq!(policy.execution_policy, TerminalDamageExecutionPolicy::ExplicitTerminalTrim);
    assert_eq!(policy.effective_trim_5p_bases, 2);
    assert_eq!(policy.effective_trim_3p_bases, 2);
}

#[test]
fn preserve_udg_policy_rejects_non_udg_damage_mode() {
    let Err(error) = resolve_terminal_damage_policy_with_override(
        DamageMode::Ancient,
        2,
        2,
        Some(TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds),
    ) else {
        panic!("preserve policy must reject ancient damage mode");
    };
    assert!(error
        .to_string()
        .contains("preserve_udg_trimmed_ends requires damage_mode=udg_trimmed"));
}

#[test]
fn trim_terminal_damage_policy_parser_accepts_policy_derived() {
    assert_eq!(parse_terminal_damage_execution_policy("policy_derived"), Some(None));
    assert_eq!(parse_terminal_damage_execution_policy("auto"), Some(None));
}

#[test]
fn trim_polyg_tails_params_roundtrip_with_stage_specific_schema() {
    let params = trim_polyg_tails_defaults(true);
    let decoded: TrimPolygTailsParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, TRIM_POLYG_TAILS_SCHEMA_VERSION);
    assert_eq!(decoded.threads, 4);
    assert!(decoded.trim_polyg);
    assert_eq!(decoded.min_polyg_run, 10);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn remove_duplicates_params_roundtrip_with_stage_specific_schema() {
    let params = remove_duplicates_defaults(true);
    let decoded: RemoveDuplicatesEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, REMOVE_DUPLICATES_SCHEMA_VERSION);
    assert_eq!(decoded.threads, 4);
    assert_eq!(decoded.dedup_mode, DedupMode::Exact);
    assert!(decoded.keep_order);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn remove_duplicates_parser_matches_public_stage_descriptor() {
    let stage_id = StageId::from_static("fastq.remove_duplicates");
    let descriptor = stage_param_descriptor(&stage_id).unwrap_or_else(|| {
        panic!("remove_duplicates must publish its governed parameter descriptor")
    });
    assert_eq!(descriptor.param_type_id, "fastq.remove_duplicates");
    let params = remove_duplicates_defaults(true);
    let value = serde_json::to_value(&params)
        .unwrap_or_else(|err| panic!("serialize remove_duplicates params: {err}"));
    let parsed = parse_effective_params(&stage_id, &value)
        .unwrap_or_else(|| panic!("parse remove_duplicates effective params"));
    match parsed {
        EffectiveParams::RemoveDuplicates(parsed) => assert_eq!(parsed, params),
        other => panic!("unexpected effective params variant: {other:?}"),
    }
}

#[test]
fn governed_stage_descriptors_cover_manifest_declared_fastq_knobs() {
    for (stage, expected_param_type_id) in [
        ("fastq.profile_read_lengths", "fastq.profile_read_lengths"),
        ("fastq.profile_overrepresented_sequences", "fastq.profile_overrepresented_sequences"),
        ("fastq.profile_reads", "fastq.profile_reads"),
        ("fastq.validate_reads", "fastq.validate_reads"),
        ("fastq.remove_chimeras", "fastq.remove_chimeras"),
        ("fastq.remove_duplicates", "fastq.remove_duplicates"),
        ("fastq.report_qc", "fastq.report_qc"),
    ] {
        let descriptor =
            stage_param_descriptor(&StageId::from_static(stage)).unwrap_or_else(|| {
                panic!("{stage} must publish a governed stage parameter descriptor")
            });
        assert_eq!(descriptor.param_type_id, expected_param_type_id);
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
    assert_eq!(decoded.unmerged_read_policy, UnmergedReadPolicy::EmitUnmergedPairs,);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn merge_params_roundtrip_with_adapterremoval_engine() {
    let params = MergeEffectiveParams {
        schema_version: MERGE_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 8,
        merge_overlap: Some(17),
        min_len: Some(45),
        merge_engine: MergeEngine::AdapterRemoval,
        unmerged_read_policy: UnmergedReadPolicy::OmitUnmergedPairs,
    };
    let decoded: MergeEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.merge_engine, MergeEngine::AdapterRemoval);
    assert_eq!(decoded.merge_overlap, Some(17));
    assert_eq!(decoded.min_len, Some(45));
    assert_eq!(decoded.unmerged_read_policy, UnmergedReadPolicy::OmitUnmergedPairs,);
    assert!(decoded.missing_required_fields().is_empty());
}

#[test]
fn chimera_params_roundtrip_with_sequence_artifact_contract() {
    let params = ChimeraDetectionEffectiveParams {
        threads: 4,
        method: "vsearch_uchime_denovo".to_string(),
        detection_scope: "denovo".to_string(),
        input_layout: "single_stream".to_string(),
        report_artifact: "report_json".to_string(),
        metrics_artifact: "chimera_metrics_json".to_string(),
        chimera_sequence_artifact: "chimeras_fasta".to_string(),
        raw_backend_report_artifact: "uchime_report_tsv".to_string(),
        raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
        chimera_removed_definition:
            "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                .to_string(),
        fallback_behavior: "copy_input_reads_and_mark_report".to_string(),
    };
    let decoded: ChimeraDetectionEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.threads, 4);
    assert_eq!(decoded.method, "vsearch_uchime_denovo");
    assert_eq!(decoded.detection_scope, "denovo");
    assert_eq!(decoded.input_layout, "single_stream");
    assert_eq!(decoded.report_artifact, "report_json");
    assert_eq!(decoded.metrics_artifact, "chimera_metrics_json");
    assert_eq!(decoded.chimera_sequence_artifact, "chimeras_fasta");
    assert_eq!(decoded.raw_backend_report_artifact, "uchime_report_tsv");
    assert_eq!(decoded.raw_backend_report_format, "vsearch_uchime_tsv");
    assert_eq!(decoded.fallback_behavior, "copy_input_reads_and_mark_report");
}

#[test]
fn otu_clustering_params_roundtrip_with_domain_default_threshold() {
    let params = OtuClusteringEffectiveParams {
        schema_version: "bijux.params.edna.v1".to_string(),
        identity_threshold: DEFAULT_OTU_IDENTITY_THRESHOLD,
        threads: 4,
        output_table_kind: "otu_abundance_table".to_string(),
        report_artifact: "report_json".to_string(),
        raw_backend_report_artifact: Some("otu_clusters_uc".to_string()),
        raw_backend_report_format: Some("vsearch_uc".to_string()),
    };
    let decoded: OtuClusteringEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, "bijux.params.edna.v1");
    assert!((decoded.identity_threshold - DEFAULT_OTU_IDENTITY_THRESHOLD).abs() < f64::EPSILON);
    assert_eq!(decoded.threads, 4);
    assert_eq!(decoded.output_table_kind, "otu_abundance_table");
    assert_eq!(decoded.report_artifact, "report_json");
    assert_eq!(decoded.raw_backend_report_artifact.as_deref(), Some("otu_clusters_uc"));
    assert_eq!(decoded.raw_backend_report_format.as_deref(), Some("vsearch_uc"));
}

#[test]
fn abundance_normalization_params_roundtrip_with_output_semantics() {
    let params = AbundanceNormalizationEffectiveParams {
        schema_version: "bijux.params.edna.v1".to_string(),
        method: "relative_abundance".to_string(),
        expected_columns: vec![
            "sample_id".to_string(),
            "feature_id".to_string(),
            "abundance".to_string(),
        ],
        input_value_column: "abundance".to_string(),
        normalized_value_column: "normalized_abundance".to_string(),
        compositional_rule: "per_sample_sum_to_one".to_string(),
        scale_factor: None,
        report_artifact: "report_json".to_string(),
    };
    let decoded: AbundanceNormalizationEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, "bijux.params.edna.v1");
    assert_eq!(decoded.method, "relative_abundance");
    assert_eq!(decoded.expected_columns.len(), 3);
    assert_eq!(decoded.input_value_column, "abundance");
    assert_eq!(decoded.normalized_value_column, "normalized_abundance");
    assert_eq!(decoded.compositional_rule, "per_sample_sum_to_one");
    assert_eq!(decoded.scale_factor, None);
    assert_eq!(decoded.report_artifact, "report_json");
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
    assert!((decoded.identity_threshold - 0.95).abs() < f64::EPSILON);
    assert_eq!(decoded.retained_read_policy, ReadRetentionPolicy::KeepNonHostReads,);
    assert!(decoded.emit_removed_reads);
    assert_eq!(decoded.report_format, MappingReportFormat::Bowtie2MetricsFile,);
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
        interpretation_boundary: TaxonomyInterpretationBoundary::ScreeningOnly,
        truth_conditions: Vec::new(),
    };
    let decoded: ScreenEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, SCREEN_TAXONOMY_SCHEMA_VERSION);
    assert_eq!(decoded.database_catalog_id, "taxonomy_reference");
    assert_eq!(decoded.database_artifact_id, "taxonomy_db");
    assert_eq!(decoded.database_build_id.as_deref(), Some("kraken2-standard-2025-01"),);
    assert_eq!(decoded.database_digest.as_deref(), Some("sha256:taxonomy-db"));
    assert_eq!(decoded.database_namespace.as_deref(), Some("read_screening"));
    assert_eq!(decoded.database_scope, TaxonomyDatabaseScope::ReadScreening);
    assert_eq!(decoded.classifier, TaxonomyClassifier::Kraken2);
    assert_eq!(decoded.report_format, TaxonomyReportFormat::KrakenReport);
    assert_eq!(decoded.assignment_format, TaxonomyAssignmentFormat::KrakenAssignments,);
    assert_eq!(decoded.minimum_confidence, Some(0.2));
    assert!(decoded.emit_unclassified);
    assert_eq!(decoded.interpretation_boundary, TaxonomyInterpretationBoundary::ScreeningOnly);
    assert!(decoded.truth_conditions.is_empty());
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
    let params = bijux_dna_domain_fastq::params::defaults::qc_post_defaults(true);
    let decoded: QcPostEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, REPORT_QC_SCHEMA_VERSION);
    assert_eq!(decoded.aggregation_engine, QcAggregationEngine::Multiqc);
    assert_eq!(decoded.aggregation_scope, QcAggregationScope::GovernedQcArtifacts);
    assert!(decoded.missing_required_fields().is_empty());
}
