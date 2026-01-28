mod hash;
mod seqkit;

pub use hash::hash_file_sha256;
pub use seqkit::{
    input_fastq_stats, length_histogram, output_fastq_stats, parse_fastqvalidator_count,
    SeqkitMetrics,
};

use anyhow::Result;

use crate::types::{MetricSet, StageResult};

pub fn observe_stage(result: &StageResult) -> Result<MetricSet> {
    if crate::types::trace_enabled() {
        println!(
            "[engine][observer] stage={} tool={}",
            result.invocation.stage_id, result.invocation.tool_id
        );
    }
    Ok(MetricSet {
        schema: "engine.metric.v1".to_string(),
        metrics: serde_json::json!({}),
    })
}
