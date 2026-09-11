#[derive(Debug, Clone)]
pub(in crate::readiness) struct LocalProof {
    pub(in crate::readiness) proof_kind: String,
    pub(in crate::readiness) proof_path: String,
    pub(in crate::readiness) tool_id: String,
    pub(in crate::readiness) sample_id: String,
    pub(in crate::readiness) artifact_paths: Vec<String>,
    pub(in crate::readiness) observed_fields: Vec<String>,
    pub(in crate::readiness) declared_output_ids: Vec<String>,
}
