//! Owner: runner bundle transport lifecycle, publication, and replay integrity.
mod age;
mod contracts;
mod mock;
mod storage;
use age::{decrypt_age_cli, encrypt_age_cli};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::BUNDLE_SIDECAR_SCHEMA_VERSION;
pub use contracts::{BundleDecryptRequest, BundleSidecar, BundleWriteRequest};
use mock::{decrypt_mock_envelope, encrypt_mock_envelope};
use std::path::{Path, PathBuf};
pub use storage::{digest_file_sha256, sha256_hex, sidecar_path_for};
use storage::{ensure_parent, recipient_fingerprint, temp_output_path};
#[derive(Debug, Clone)]
enum EncryptionBackend {
    MockEnvelopeV1,
    AgeCli,
}

fn parse_backend(value: &str) -> Result<EncryptionBackend> {
    match value {
        "mock-envelope-v1" => Ok(EncryptionBackend::MockEnvelopeV1),
        "age-cli" => Ok(EncryptionBackend::AgeCli),
        other => Err(anyhow!("unsupported encryption backend `{other}`")),
    }
}
fn encrypt_with_backend(
    backend: &EncryptionBackend,
    recipients: &[String],
    plaintext: &[u8],
    output_path: &Path,
) -> Result<()> {
    match backend {
        EncryptionBackend::MockEnvelopeV1 => {
            let ciphertext = encrypt_mock_envelope(recipients, plaintext)?;
            bijux_dna_infra::write_bytes(output_path, ciphertext)
                .with_context(|| format!("write {}", output_path.display()))?;
            Ok(())
        }
        EncryptionBackend::AgeCli => encrypt_age_cli(recipients, plaintext, output_path),
    }
}
fn decrypt_with_backend(
    backend: &EncryptionBackend,
    bundle_path: &Path,
    identity_files: &[PathBuf],
) -> Result<Vec<u8>> {
    match backend {
        EncryptionBackend::MockEnvelopeV1 => {
            let ciphertext = std::fs::read(bundle_path)
                .with_context(|| format!("read {}", bundle_path.display()))?;
            decrypt_mock_envelope(&ciphertext)
        }
        EncryptionBackend::AgeCli => decrypt_age_cli(bundle_path, identity_files),
    }
}
/// Publish a ciphertext and its matching transport sidecar.
///
/// # Errors
/// Returns backend, filesystem, serialization, or recipient admission errors.
pub fn write_encrypted_bundle(request: &BundleWriteRequest<'_>) -> Result<BundleSidecar> {
    let backend = parse_backend(request.backend)?;
    if request.recipients.is_empty() {
        return Err(anyhow!("bundle encryption recipients list is empty"));
    }

    ensure_parent(request.output_path)?;
    let temp_path = temp_output_path(request.output_path);
    let plaintext_sha256 = sha256_hex(request.plaintext);

    let encrypt_result =
        encrypt_with_backend(&backend, request.recipients, request.plaintext, &temp_path);

    if let Err(error) = encrypt_result {
        let _ = bijux_dna_infra::remove_file(&temp_path);
        return Err(error).with_context(|| format!("encrypt {}", request.output_path.display()));
    }

    let ciphertext_sha256 = digest_file_sha256(&temp_path)?;
    bijux_dna_infra::rename(&temp_path, request.output_path).with_context(|| {
        format!(
            "move encrypted bundle {} -> {}",
            temp_path.display(),
            request.output_path.display()
        )
    })?;

    let sidecar = BundleSidecar {
        schema_version: BUNDLE_SIDECAR_SCHEMA_VERSION.to_string(),
        bundle_kind: request.bundle_kind.to_string(),
        campaign_id: request.campaign_id.to_string(),
        domain: request.domain.to_string(),
        stage: request.stage.to_string(),
        tool: request.tool.to_string(),
        sample: request.sample.to_string(),
        planned_job_id: request.planned_job_id.to_string(),
        scheduler_job_id: request.scheduler_job_id.to_string(),
        submitted_at: request.submitted_at.to_string(),
        backend: request.backend.to_string(),
        recipients: request.recipients.to_vec(),
        recipient_fingerprints: request
            .recipients
            .iter()
            .map(|recipient| recipient_fingerprint(recipient))
            .collect(),
        ciphertext_path: request.output_path.display().to_string(),
        plaintext_sha256,
        ciphertext_sha256,
        plaintext_bytes: request.plaintext.len(),
    };

    let sidecar_path = sidecar_path_for(request.output_path);
    let payload = serde_json::to_vec_pretty(&sidecar).context("serialize sidecar")?;
    bijux_dna_infra::atomic_write_bytes(&sidecar_path, &payload)
        .with_context(|| format!("write {}", sidecar_path.display()))?;

    Ok(sidecar)
}
/// Verify the selected sidecar and ciphertext before decoding the declared backend.
///
/// # Errors
/// Returns path, schema, hash, backend, or IO errors.
pub fn decrypt_bundle(request: &BundleDecryptRequest<'_>) -> Result<(BundleSidecar, Vec<u8>)> {
    let sidecar_path =
        request.sidecar_path.map_or_else(|| sidecar_path_for(request.bundle_path), PathBuf::from);
    let raw_sidecar =
        std::fs::read(&sidecar_path).with_context(|| format!("read {}", sidecar_path.display()))?;
    let sidecar: BundleSidecar =
        serde_json::from_slice(&raw_sidecar).context("parse bundle sidecar")?;
    if sidecar.schema_version != BUNDLE_SIDECAR_SCHEMA_VERSION {
        return Err(anyhow!("unsupported bundle sidecar schema `{}`", sidecar.schema_version));
    }
    if Path::new(&sidecar.ciphertext_path) != request.bundle_path {
        return Err(anyhow!(
            "bundle sidecar ciphertext path mismatch: expected {}, found {}",
            request.bundle_path.display(),
            sidecar.ciphertext_path
        ));
    }
    let ciphertext_sha256 = digest_file_sha256(request.bundle_path)?;
    if ciphertext_sha256 != sidecar.ciphertext_sha256 {
        return Err(anyhow!("ciphertext hash mismatch"));
    }

    let backend = parse_backend(&sidecar.backend)?;
    let plaintext = decrypt_with_backend(&backend, request.bundle_path, request.identity_files)?;
    if plaintext.len() != sidecar.plaintext_bytes {
        return Err(anyhow!("bundle plaintext byte count mismatch"));
    }
    let plaintext_sha256 = sha256_hex(&plaintext);
    if plaintext_sha256 != sidecar.plaintext_sha256 {
        return Err(anyhow!("plaintext hash mismatch"));
    }
    Ok((sidecar, plaintext))
}

#[cfg(test)]
#[path = "../../../tests/contracts/bundle/owner.rs"]
mod tests;
