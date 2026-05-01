use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::defaults::{
    overrepresented_profile_defaults, qc_post_defaults, read_length_profile_defaults,
    screen_defaults, stats_defaults,
};
use bijux_dna_domain_fastq::params::edna::{
    AbundanceNormalizationEffectiveParams, AsvInferenceEffectiveParams,
    ChimeraDetectionEffectiveParams, OtuClusteringEffectiveParams,
    PrimerNormalizationEffectiveParams, DEFAULT_OTU_IDENTITY_THRESHOLD, EDNA_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::screen::{
    HostDepletionEffectiveParams, MappingReportFormat, ReadRetentionPolicy,
    ReferenceContaminantEffectiveParams, ReferenceDecoyPolicy, ReferenceMaskingPolicy,
    ReferenceScope, RrnaEffectiveParams, RrnaReportFormat, RrnaScreeningEngine,
    HOST_DEPLETION_SCHEMA_VERSION, REFERENCE_DEPLETION_SCHEMA_VERSION,
    RRNA_DEPLETION_SCHEMA_VERSION,
};
use bijux_dna_domain_fastq::params::PairedMode;

use crate::DefaultParams;

pub(super) fn fastq_analysis_params(paired: bool) -> BTreeMap<StageId, DefaultParams> {
    BTreeMap::from([
        (
            StageId::from_static("fastq.profile_reads"),
            DefaultParams::FastqStats(stats_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.profile_read_lengths"),
            DefaultParams::FastqReadLengthProfile(read_length_profile_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.profile_overrepresented_sequences"),
            DefaultParams::FastqOverrepresentedProfile(overrepresented_profile_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.report_qc"),
            DefaultParams::FastqQcPost(qc_post_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            DefaultParams::FastqScreen(screen_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.deplete_rrna"),
            DefaultParams::FastqRrna(rrna_depletion_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.deplete_host"),
            DefaultParams::FastqHostDepletion(host_depletion_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.deplete_reference_contaminants"),
            DefaultParams::FastqReferenceContaminantDepletion(reference_depletion_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.normalize_primers"),
            DefaultParams::FastqPrimerNormalization(primer_normalization_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.remove_chimeras"),
            DefaultParams::FastqChimeraDetection(chimera_detection_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.infer_asvs"),
            DefaultParams::FastqAsvInference(asv_inference_defaults(paired)),
        ),
        (
            StageId::from_static("fastq.cluster_otus"),
            DefaultParams::FastqOtuClustering(otu_clustering_defaults()),
        ),
        (
            StageId::from_static("fastq.normalize_abundance"),
            DefaultParams::FastqAbundanceNormalization(abundance_normalization_defaults()),
        ),
    ])
}

fn paired_mode(paired: bool) -> PairedMode {
    if paired {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    }
}

fn rrna_depletion_defaults(paired: bool) -> RrnaEffectiveParams {
    RrnaEffectiveParams {
        schema_version: RRNA_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        contaminant_db: Some("rrna_reference".to_string()),
        database_artifact_id: "rrna_db".to_string(),
        database_build_id: None,
        screening_engine: RrnaScreeningEngine::Sortmerna,
        report_format: RrnaReportFormat::SummaryTsvAndJson,
        emit_removed_reads: false,
    }
}

fn host_depletion_defaults(paired: bool) -> HostDepletionEffectiveParams {
    HostDepletionEffectiveParams {
        schema_version: HOST_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        reference_scope: ReferenceScope::Host,
        reference_catalog_id: "host_reference".to_string(),
        reference_index_artifact_id: "host_reference_index".to_string(),
        reference_index_backend: "bowtie2".to_string(),
        reference_build_id: None,
        reference_digest: None,
        masking_policy: ReferenceMaskingPolicy::Unmasked,
        decoy_policy: ReferenceDecoyPolicy::None,
        decoy_catalog_id: None,
        identity_threshold: 0.95,
        retained_read_policy: ReadRetentionPolicy::KeepNonHostReads,
        emit_removed_reads: false,
        report_format: MappingReportFormat::Bowtie2MetricsFile,
        retain_unmapped_pairs: true,
    }
}

fn reference_depletion_defaults(paired: bool) -> ReferenceContaminantEffectiveParams {
    ReferenceContaminantEffectiveParams {
        schema_version: REFERENCE_DEPLETION_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: 1,
        reference_catalog_id: "reference_contaminants".to_string(),
        contaminant_reference: "contaminants.fasta".to_string(),
        index_artifact: "contaminants_index".to_string(),
        reference_index_backend: "bowtie2".to_string(),
        reference_build_id: None,
        reference_digest: None,
        retain_unmapped_pairs: true,
    }
}

fn primer_normalization_defaults(paired: bool) -> PrimerNormalizationEffectiveParams {
    PrimerNormalizationEffectiveParams {
        schema_version: EDNA_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        threads: Some(1),
        orientation_policy: "forward_reverse".to_string(),
        primer_set_id: "metabarcoding_default".to_string(),
        marker_id: None,
        primer_fasta: Some("assets/reference/primers/COI.fasta".to_string()),
        max_mismatch_rate: 0.1,
        min_overlap_bp: 18,
        strict_5p_anchor: true,
        allow_iupac_codes: true,
    }
}

fn chimera_detection_defaults(paired: bool) -> ChimeraDetectionEffectiveParams {
    ChimeraDetectionEffectiveParams {
        method: "uchime_denovo".to_string(),
        detection_scope: "sample".to_string(),
        input_layout: if paired { "paired_end" } else { "single_end" }.to_string(),
        threads: 1,
        report_artifact: "report_json".to_string(),
        metrics_artifact: "chimera_metrics_json".to_string(),
        chimera_sequence_artifact: "chimera_sequences_fasta".to_string(),
        raw_backend_report_artifact: "uchime_report_tsv".to_string(),
        raw_backend_report_format: "uchime_tsv".to_string(),
        chimera_removed_definition: "reads flagged as chimeric by governed UCHIME policy"
            .to_string(),
        fallback_behavior: "refuse_stage".to_string(),
    }
}

fn asv_inference_defaults(paired: bool) -> AsvInferenceEffectiveParams {
    AsvInferenceEffectiveParams {
        schema_version: EDNA_SCHEMA_VERSION.to_string(),
        paired_mode: paired_mode(paired),
        denoising_method: "dada2".to_string(),
        pooling_mode: "independent".to_string(),
        chimera_policy: "report_only".to_string(),
        threads: Some(1),
        requires_r_runtime: true,
        output_table_kind: "feature_table".to_string(),
        report_artifact: "report_json".to_string(),
        raw_backend_report_artifact: Some("infer_asvs_report.json".to_string()),
        raw_backend_report_format: Some("dada2_json".to_string()),
    }
}

fn otu_clustering_defaults() -> OtuClusteringEffectiveParams {
    OtuClusteringEffectiveParams {
        schema_version: EDNA_SCHEMA_VERSION.to_string(),
        identity_threshold: DEFAULT_OTU_IDENTITY_THRESHOLD,
        threads: 1,
        output_table_kind: "otu_table".to_string(),
        report_artifact: "report_json".to_string(),
        raw_backend_report_artifact: Some("cluster_otus_report.json".to_string()),
        raw_backend_report_format: Some("vsearch_json".to_string()),
    }
}

fn abundance_normalization_defaults() -> AbundanceNormalizationEffectiveParams {
    AbundanceNormalizationEffectiveParams {
        schema_version: EDNA_SCHEMA_VERSION.to_string(),
        method: "relative_abundance".to_string(),
        expected_columns: vec!["feature_id".to_string(), "count".to_string()],
        input_value_column: "count".to_string(),
        normalized_value_column: "relative_abundance".to_string(),
        compositional_rule: "sum_to_one".to_string(),
        scale_factor: Some(1.0),
        report_artifact: "report_json".to_string(),
    }
}
