//! PCA adapter observations keep full binding identity separate from publication addresses.
use crate::{model::commands::VcfRenderedCommandRow, snapshot::VcfStageReadinessBindingKey};
/// Declared output addresses and adapter status; this record is not executed scientific proof.
#[derive(Debug, Clone)]
pub struct PcaAdapterOutputProof {
    pub output_proof_path: String,
    pub benchmark_status: String,
    pub raw_outputs: Vec<String>,
    pub normalized_metrics_outputs: Vec<String>,
    pub manifest_output: String,
    pub index_outputs: Vec<String>,
}
/// One typed adapter observation, keyed by its complete retained binding and normalized schema.
#[derive(Debug, Clone)]
pub struct PcaAdapterEvidence {
    pub binding: VcfStageReadinessBindingKey,
    pub adapter_id: String,
    pub parser_id: String,
    pub schema_id: String,
    pub output: PcaAdapterOutputProof,
    pub command: VcfRenderedCommandRow,
}
