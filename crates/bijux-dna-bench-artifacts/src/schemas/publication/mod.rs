//! Atomic schema files use canonical model definitions.
mod execution;
mod normalized;
pub use execution::{
    render_benchmark_attempt_archive_schema, render_benchmark_attempt_schema,
    render_benchmark_cycle_status_schema, render_benchmark_pipeline_parameters_schema,
    render_benchmark_report_admission_schema, render_benchmark_stage_interfaces_schema,
    render_benchmark_stage_report_schema, write_benchmark_attempt_archive_schema,
    write_benchmark_attempt_schema, write_benchmark_cycle_status_schema,
    write_benchmark_pipeline_parameters_schema, write_benchmark_report_admission_schema,
    write_benchmark_stage_interfaces_schema, write_benchmark_stage_report_schema,
};
pub use normalized::{
    render_bam_normalized_metrics_schema, render_fastq_normalized_metrics_schema,
    render_vcf_normalized_metrics_schema, render_vcf_normalized_metrics_stage_schema,
    validate_vcf_normalized_metrics, vcf_normalized_metrics_stage_schema_descriptors,
    write_bam_normalized_metrics_schema, write_fastq_normalized_metrics_schema,
    write_vcf_normalized_metrics_schema, write_vcf_normalized_metrics_stage_schemas,
};
