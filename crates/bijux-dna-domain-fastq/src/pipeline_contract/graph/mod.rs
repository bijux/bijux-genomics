mod assembly;
mod dependencies;
mod edges;

pub use assembly::{
    preprocess_pipeline, preprocess_pipeline_for_mode, preprocess_pipeline_graph_for_stage_order,
};

#[cfg(test)]
mod tests {
    use super::preprocess_pipeline_graph_for_stage_order;
    use bijux_dna_core::ids::StageId;

    #[test]
    fn report_qc_edges_bind_governed_artifacts_instead_of_generic_stage_edges() {
        let pipeline = preprocess_pipeline_graph_for_stage_order(&[
            StageId::from_static("fastq.validate_reads"),
            StageId::from_static("fastq.report_qc"),
        ]);

        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("validation_report")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("validated_reads_manifest")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
        assert!(!pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.validate_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.is_none()
                && edge.to_input_id.is_none()
        }));
    }

    #[test]
    fn report_qc_keeps_parallel_artifact_joins_for_multiple_contributors() {
        let pipeline = preprocess_pipeline_graph_for_stage_order(&[
            StageId::from_static("fastq.detect_adapters"),
            StageId::from_static("fastq.profile_reads"),
            StageId::from_static("fastq.report_qc"),
        ]);

        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.detect_adapters"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("adapter_evidence_dir")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
        assert!(pipeline.edges.iter().any(|edge| {
            edge.from == "fastq.profile_reads"
                && edge.to == "fastq.report_qc"
                && edge.from_output_id.as_deref() == Some("qc_json")
                && edge.to_input_id.as_deref() == Some("qc_artifacts")
        }));
    }
}
