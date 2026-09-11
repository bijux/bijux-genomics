use super::super::{
    fastq_command_adapter_coverage::{
        render_fastq_command_adapter_coverage, DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH,
    },
    fastq_normalized_metrics_schema::render_fastq_normalized_metrics_schema,
};
use crate::commands::benchmark::schema_paths::DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH;
use anyhow::Result;
use bijux_dna_bench_fastq::{
    proofs::inputs::{observations::PublishedSchema, ports::SchemaEvidence},
    readiness::adapters::FastqCommandAdapterCoverageRow,
};
use std::path::Path;
pub(super) struct SchemaAdapter;
impl SchemaEvidence for SchemaAdapter {
    fn commands(&self, root: &Path) -> Result<Vec<FastqCommandAdapterCoverageRow>> {
        Ok(render_fastq_command_adapter_coverage(
            root,
            DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH.into(),
        )?
        .rows)
    }
    fn publish(&self, root: &Path) -> Result<PublishedSchema> {
        let report = render_fastq_normalized_metrics_schema(
            root,
            DEFAULT_FASTQ_NORMALIZED_METRICS_SCHEMA_PATH.into(),
        )?;
        Ok(PublishedSchema { output_path: report.output_path })
    }
    fn document(&self) -> serde_json::Value {
        bijux_dna_api::v1::api::bench::render_fastq_normalized_metrics_schema()
    }
}
