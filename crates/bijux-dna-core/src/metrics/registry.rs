#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsSchemaId {
    pub stage_id: &'static str,
    pub schema: &'static str,
    pub version: i32,
}

pub const FASTQ_METRICS_SCHEMAS: &[MetricsSchemaId] = &[
    MetricsSchemaId { stage_id: "fastq.trim_reads", schema: "fastq_trim_reads_v2", version: 2 },
    MetricsSchemaId {
        stage_id: "fastq.validate_reads",
        schema: "fastq_validate_reads_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.detect_adapters",
        schema: "fastq_detect_adapters_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.profile_read_lengths",
        schema: "fastq_profile_read_lengths_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.profile_overrepresented_sequences",
        schema: "fastq_profile_overrepresented_sequences_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.index_reference",
        schema: "fastq_index_reference_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.trim_terminal_damage",
        schema: "fastq_trim_terminal_damage_v2",
        version: 2,
    },
    MetricsSchemaId {
        stage_id: "fastq.trim_polyg_tails",
        schema: "fastq_trim_polyg_tails_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.filter_reads", schema: "fastq_filter_reads_v2", version: 2 },
    MetricsSchemaId {
        stage_id: "fastq.remove_duplicates",
        schema: "fastq_deduplicate_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.filter_low_complexity",
        schema: "fastq_low_complexity_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.deplete_host", schema: "fastq_deplete_host_v1", version: 1 },
    MetricsSchemaId {
        stage_id: "fastq.deplete_reference_contaminants",
        schema: "fastq_deplete_reference_contaminants_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.merge_pairs", schema: "fastq_merge_pairs_v1", version: 1 },
    MetricsSchemaId {
        stage_id: "fastq.correct_errors",
        schema: "fastq_correct_errors_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.report_qc", schema: "fastq_report_qc_v1", version: 1 },
    MetricsSchemaId {
        stage_id: "fastq.normalize_primers",
        schema: "fastq_normalize_primers_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.remove_chimeras",
        schema: "fastq_remove_chimeras_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.cluster_otus", schema: "fastq_cluster_otus_v1", version: 1 },
    MetricsSchemaId {
        stage_id: "fastq.normalize_abundance",
        schema: "fastq_normalize_abundance_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.extract_umis", schema: "fastq_extract_umis_v1", version: 1 },
    MetricsSchemaId {
        stage_id: "fastq.screen_taxonomy",
        schema: "fastq_screen_taxonomy_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.deplete_rrna", schema: "fastq_deplete_rrna_v2", version: 2 },
    MetricsSchemaId {
        stage_id: "fastq.profile_reads",
        schema: "fastq_profile_reads_v1",
        version: 1,
    },
    MetricsSchemaId { stage_id: "fastq.preprocess", schema: "fastq_preprocess_v1", version: 1 },
];

pub const BAM_METRICS_SCHEMAS: &[MetricsSchemaId] = &[
    MetricsSchemaId { stage_id: "bam.align", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.validate", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.qc_pre", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.mapping_summary", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.filter", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.mapq_filter", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.length_filter", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.markdup", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.duplication_metrics", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.complexity", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.coverage", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.insert_size", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.gc_bias", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.endogenous_content", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.overlap_correction", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.damage", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.contamination", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.sex", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.bias_mitigation", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.recalibration", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.haplogroups", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.genotyping", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.kinship", schema: "bam_metrics_v1", version: 1 },
    MetricsSchemaId { stage_id: "bam.authenticity", schema: "bam_metrics_v1", version: 1 },
];

#[allow(dead_code)]
#[must_use]
pub fn metrics_schema_for_stage(stage_id: &str) -> Option<&'static MetricsSchemaId> {
    FASTQ_METRICS_SCHEMAS
        .iter()
        .chain(BAM_METRICS_SCHEMAS.iter())
        .find(|schema| schema.stage_id == stage_id)
}
