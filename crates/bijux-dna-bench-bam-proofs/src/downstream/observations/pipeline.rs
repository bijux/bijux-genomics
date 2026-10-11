/// Only the canonical validation fields consumed by BAM downstream proof.
#[derive(Debug, Clone)]
pub struct PipelineNode {
    pub stage_id: String,
    pub depends_on: Vec<String>,
    pub external_inputs: Vec<String>,
    pub upstream_inputs: Vec<String>,
    pub outputs: Vec<String>,
}
/// Multiplicity and input/output order remain observable until scientific admission.
#[derive(Debug, Clone)]
pub struct PipelineObservation {
    pub output_path: String,
    pub pipeline_id: String,
    pub valid: bool,
    pub nodes: Vec<PipelineNode>,
}
