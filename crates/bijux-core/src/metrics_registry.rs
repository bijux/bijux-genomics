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
        stage_id: "fastq.detect_adapters",
        schema: "fastq_detect_adapters_v1",
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

pub const BAM_METRICS_SCHEMAS: &[MetricsSchemaId] = &[
    MetricsSchemaId {
        stage_id: "bam.align",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.validate",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.qc_pre",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.filter",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.markdup",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.complexity",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.coverage",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.damage",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.contamination",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.sex",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.bias_mitigation",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.recalibration",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.haplogroups",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.genotyping",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.kinship",
        schema: "bam_metrics_v1",
        version: 1,
    },
    MetricsSchemaId {
        stage_id: "bam.authenticity",
        schema: "bam_metrics_v1",
        version: 1,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricDirection {
    HigherBetter,
    LowerBetter,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricSemantics {
    pub metric_id: &'static str,
    pub direction: MetricDirection,
    pub units: &'static str,
    pub range: &'static str,
    pub missing_data_policy: &'static str,
}

pub const COMPARE_METRIC_SEMANTICS: &[MetricSemantics] = &[
    MetricSemantics {
        metric_id: "runtime_s",
        direction: MetricDirection::LowerBetter,
        units: "seconds",
        range: ">= 0",
        missing_data_policy: "treat_as_infinite",
    },
    MetricSemantics {
        metric_id: "memory_mb",
        direction: MetricDirection::LowerBetter,
        units: "MB",
        range: ">= 0",
        missing_data_policy: "treat_as_infinite",
    },
    MetricSemantics {
        metric_id: "read_retention",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
    },
    MetricSemantics {
        metric_id: "base_retention",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
    },
    MetricSemantics {
        metric_id: "merge_rate",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
    },
    MetricSemantics {
        metric_id: "error_reduction_proxy",
        direction: MetricDirection::HigherBetter,
        units: "mean_q_delta",
        range: "[0, 45]",
        missing_data_policy: "treat_as_0.0",
    },
];

#[must_use]
pub fn metrics_schema_for_stage(stage_id: &str) -> Option<&'static MetricsSchemaId> {
    FASTQ_METRICS_SCHEMAS
        .iter()
        .chain(BAM_METRICS_SCHEMAS.iter())
        .find(|schema| schema.stage_id == stage_id)
}

#[must_use]
pub fn metric_semantics(metric_id: &str) -> Option<&'static MetricSemantics> {
    COMPARE_METRIC_SEMANTICS
        .iter()
        .find(|spec| spec.metric_id == metric_id)
}
