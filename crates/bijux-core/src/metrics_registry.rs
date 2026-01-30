#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricsSchemaId {
    pub stage_id: &'static str,
    pub schema: &'static str,
    pub version: i32,
}

pub const FASTQ_METRICS_SCHEMAS: &[MetricsSchemaId] = &[
    MetricsSchemaId {
        stage_id: "fastq.trim",
        schema: "fastq_trim_v2",
        version: 2,
    },
    MetricsSchemaId {
        stage_id: "fastq.validate_pre",
        schema: "fastq_validate_pre_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.filter",
        schema: "fastq_filter_v2",
        version: 2,
    },
    MetricsSchemaId {
        stage_id: "fastq.merge",
        schema: "fastq_merge_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.correct",
        schema: "fastq_correct_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.qc_post",
        schema: "fastq_qc_post_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.umi",
        schema: "fastq_umi_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.screen",
        schema: "fastq_screen_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.stats_neutral",
        schema: "fastq_stats_neutral_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "fastq.preprocess",
        schema: "fastq_preprocess_v1",
        version: 1,
    },
];

#[must_use]
pub fn metrics_schema_for_stage(stage_id: &str) -> Option<&'static MetricsSchemaId> {
    FASTQ_METRICS_SCHEMAS
        .iter()
        .find(|schema| schema.stage_id == stage_id)
}
