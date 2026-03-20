use bijux_dna_domain_fastq::params::correct::{FastqCorrectParams, CORRECT_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::defaults::detect_adapters_defaults;
use bijux_dna_domain_fastq::params::defaults::{correct_defaults, stats_defaults, umi_defaults};
use bijux_dna_domain_fastq::params::detect_adapters::{
    AdapterInspectionMode, DetectAdaptersEffectiveParams, DETECT_ADAPTERS_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::merge::{
    MergeEffectiveParams, MergeEngine, UnmergedReadPolicy, MERGE_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::screen::{
    HostDepletionEffectiveParams, MappingReportFormat, ReadRetentionPolicy, ReferenceScope,
    ScreenEffectiveParams, TaxonomyAssignmentFormat, TaxonomyClassifier, TaxonomyReportFormat,
    HOST_DEPLETION_SCHEMA_VERSION, SCREEN_TAXONOMY_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::stats::{FastqStatsParams, STATS_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::umi::{FastqUmiParams, UMI_SCHEMA_VERSION};
use bijux_dna_domain_fastq::params::PairedMode;

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
fn host_depletion_params_roundtrip_with_reference_provenance_fields() {
    let params = HostDepletionEffectiveParams {
        schema_version: HOST_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: PairedMode::PairedEnd,
        threads: 8,
        reference_scope: ReferenceScope::Host,
        reference_index_artifact_id: "reference_index".to_string(),
        retained_read_policy: ReadRetentionPolicy::KeepNonHostReads,
        emit_removed_reads: true,
        report_format: MappingReportFormat::Bowtie2MetricsFile,
        retain_unmapped_pairs: true,
    };
    let decoded: HostDepletionEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, HOST_DEPLETION_SCHEMA_VERSION);
    assert_eq!(decoded.reference_scope, ReferenceScope::Host);
    assert_eq!(decoded.reference_index_artifact_id, "reference_index");
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
        classifier: TaxonomyClassifier::Kraken2,
        report_format: TaxonomyReportFormat::KrakenReport,
        assignment_format: TaxonomyAssignmentFormat::KrakenAssignments,
    };
    let decoded: ScreenEffectiveParams = roundtrip(&params);
    assert_eq!(decoded.schema_version, SCREEN_TAXONOMY_SCHEMA_VERSION);
    assert_eq!(decoded.classifier, TaxonomyClassifier::Kraken2);
    assert_eq!(decoded.report_format, TaxonomyReportFormat::KrakenReport);
    assert_eq!(
        decoded.assignment_format,
        TaxonomyAssignmentFormat::KrakenAssignments,
    );
    assert!(decoded.missing_required_fields().is_empty());
}
