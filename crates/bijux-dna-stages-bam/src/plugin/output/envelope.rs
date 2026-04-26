use anyhow::Result;
use bijux_dna_core::contract::canonical::parameters_json_canonicalization;
use bijux_dna_core::contract::ContractVersion;
use bijux_dna_core::metrics::MetricsEnvelope;
use bijux_dna_core::prelude::hashing::{input_fingerprint, parameters_fingerprint};
use bijux_dna_domain_bam::metrics::BamMetricsV1;
use bijux_dna_stage_contract::StagePlanV1;

pub(super) fn build_metrics_envelope(
    plan: &StagePlanV1,
    metrics: BamMetricsV1,
) -> Result<MetricsEnvelope<serde_json::Value>> {
    let metrics_json = serde_json::to_value(metrics)?;
    let mut input_hashes = Vec::new();
    for input in &plan.io.inputs {
        if input.path.exists() {
            if let Ok(hash) = bijux_dna_infra::hash_file_sha256(&input.path) {
                input_hashes.push(hash);
            }
        }
    }
    input_hashes.sort();
    let input_fingerprint = input_fingerprint(&input_hashes);
    let parameters_fingerprint = parameters_fingerprint(&plan.params)?;
    let parameters_json_normalized = parameters_json_canonicalization(&plan.params);
    let image_digest = plan.image.digest.clone().unwrap_or_else(|| plan.image.image.clone());
    Ok(MetricsEnvelope {
        schema_version: "bijux.metrics_envelope.v2".to_string(),
        contract_version: ContractVersion::v1(),
        stage_id: plan.stage_id.0.to_string(),
        stage_version: plan.stage_version.0,
        tool_id: plan.tool_id.0.to_string(),
        tool_version: plan.tool_version.clone(),
        image_digest,
        parameters_fingerprint,
        input_fingerprint,
        parameters_json_normalized,
        input_hashes,
        metric_provenance: None,
        metrics: metrics_json,
    })
}
