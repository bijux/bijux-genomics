use bijux_dna_core::contract::PipelineEdgeSpec;

pub fn pipeline_edge(from: &str, to: &str) -> PipelineEdgeSpec {
    PipelineEdgeSpec {
        from: from.to_string(),
        to: to.to_string(),
        from_output_id: None,
        to_input_id: None,
    }
}

pub fn pipeline_artifact_edge(
    from: &str,
    to: &str,
    from_output_id: &str,
    to_input_id: &str,
) -> PipelineEdgeSpec {
    PipelineEdgeSpec {
        from: from.to_string(),
        to: to.to_string(),
        from_output_id: Some(from_output_id.to_string()),
        to_input_id: Some(to_input_id.to_string()),
    }
}
