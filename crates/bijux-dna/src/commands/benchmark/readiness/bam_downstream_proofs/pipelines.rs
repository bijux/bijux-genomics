use crate::commands::benchmark::local_pipeline_dag::{
    benchmark_local_pipeline_config_path, validate_pipeline_dag_path,
};
use anyhow::Result;
use bijux_dna_bench_bam_proofs::downstream::{
    observations::{PipelineNode, PipelineObservation},
    ports::PipelineEvidence,
};
use std::path::Path;
pub(super) struct PipelineAdapter;
impl PipelineEvidence for PipelineAdapter {
    fn validate(
        &self,
        root: &Path,
        pipeline_id: &str,
        output_path: &Path,
    ) -> Result<PipelineObservation> {
        let report = validate_pipeline_dag_path(
            root,
            &benchmark_local_pipeline_config_path(root, pipeline_id),
            output_path,
        )?;
        Ok(PipelineObservation {
            output_path: report.output_path,
            pipeline_id: report.pipeline_id,
            valid: report.valid,
            nodes: report
                .nodes
                .into_iter()
                .map(|node| PipelineNode {
                    stage_id: node.stage_id,
                    depends_on: node.depends_on,
                    external_inputs: node.external_inputs,
                    upstream_inputs: node.upstream_inputs,
                    outputs: node.outputs,
                })
                .collect(),
        })
    }
}
