use super::Result;
use crate::request_args::{
    SignedBundleRequestV1, SignedBundleResponseV1, SignedBundleVerifyRequestV1,
    SignedBundleVerifyResponseV1,
};
use anyhow::Context;
use sha2::Digest;

/// Create a prototype signature file for a run bundle.
///
/// # Errors
/// Returns an error if governed run files cannot be read or written.
pub fn sign_bundle_prototype(request: &SignedBundleRequestV1) -> Result<SignedBundleResponseV1> {
    let signature_path = request.run_dir.join("bundle_signature.json");
    let payload_sha256 = payload_sha256(&request.run_dir)?;
    let signature = compute_signature(&request.shared_secret, &payload_sha256);
    let response = SignedBundleResponseV1 {
        schema_version: "bijux.signed_bundle.v1".to_string(),
        signature_path: signature_path.clone(),
        key_id: request.key_id.clone(),
        algorithm: "sha256(secret||payload_sha256)".to_string(),
        payload_sha256,
        signature,
    };
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&response)?;
    bijux_dna_infra::atomic_write_bytes(&signature_path, bytes.as_slice())?;
    Ok(response)
}

/// Verify a prototype signed bundle using a shared secret.
///
/// # Errors
/// Returns an error if signature or run contracts cannot be read.
pub fn verify_signed_bundle_prototype(
    request: &SignedBundleVerifyRequestV1,
) -> Result<SignedBundleVerifyResponseV1> {
    let signature_path = request
        .signature_path
        .clone()
        .unwrap_or_else(|| request.run_dir.join("bundle_signature.json"));
    let signed: SignedBundleResponseV1 = serde_json::from_slice(
        &std::fs::read(&signature_path)
            .with_context(|| format!("read {}", signature_path.display()))?,
    )
    .context("parse signed bundle")?;

    let actual_payload_sha256 = payload_sha256(&request.run_dir)?;
    if actual_payload_sha256 != signed.payload_sha256 {
        return Ok(SignedBundleVerifyResponseV1 {
            schema_version: "bijux.signed_bundle_verify.v1".to_string(),
            verified: false,
            reason: Some("payload_sha256_mismatch".to_string()),
        });
    }
    let expected_signature = compute_signature(&request.shared_secret, &actual_payload_sha256);
    let verified = expected_signature == signed.signature;
    Ok(SignedBundleVerifyResponseV1 {
        schema_version: "bijux.signed_bundle_verify.v1".to_string(),
        verified,
        reason: (!verified).then_some("signature_mismatch".to_string()),
    })
}

fn payload_sha256(run_dir: &std::path::Path) -> Result<String> {
    let layout = bijux_dna_runtime::run_layout::RunLayout::from_run_dir(run_dir.to_path_buf());
    let manifest_sha256 = file_hash_or_missing(&layout.manifest_path)?;
    let artifact_inventory_sha256 = file_hash_or_missing(&layout.artifact_inventory_path)?;
    let hash_ledger_sha256 = file_hash_or_missing(&layout.hash_ledger_path)?;
    let payload = serde_json::json!({
        "schema_version": "bijux.signed_bundle_payload.v1",
        "manifest_sha256": manifest_sha256,
        "artifact_inventory_sha256": artifact_inventory_sha256,
        "hash_ledger_sha256": hash_ledger_sha256,
    });
    let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&payload)?;
    Ok(sha256_hex(sha2::Sha256::digest(bytes)))
}

fn file_hash_or_missing(path: &std::path::Path) -> Result<String> {
    if !path.exists() {
        return Ok("missing".to_string());
    }
    Ok(bijux_dna_infra::hash_file_sha256(path)?)
}

fn compute_signature(secret: &str, payload_sha256: &str) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b"::");
    hasher.update(payload_sha256.as_bytes());
    sha256_hex(hasher.finalize())
}

fn sha256_hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}
