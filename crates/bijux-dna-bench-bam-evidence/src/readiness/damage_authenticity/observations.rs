#[derive(Debug, Clone)]
pub(in crate::readiness) struct LocalSmokeProof {
    pub(in crate::readiness) proof_path: String,
    pub(in crate::readiness) sample_id: String,
    pub(in crate::readiness) artifact_paths: Vec<String>,
    pub(in crate::readiness) observed_fields: Vec<String>,
}
