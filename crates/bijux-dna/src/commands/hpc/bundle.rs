use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const BUNDLE_SIDECAR_SCHEMA_VERSION: &str = "bijux.hpc.bundle.sidecar.v1";
const MOCK_ENVELOPE_SCHEMA_VERSION: &str = "bijux.hpc.bundle.mock_envelope.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSidecar {
    pub schema_version: String,
    pub bundle_kind: String,
    pub campaign_id: String,
    pub domain: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub planned_job_id: String,
    pub scheduler_job_id: String,
    pub submitted_at: String,
    pub backend: String,
    pub recipients: Vec<String>,
    pub recipient_fingerprints: Vec<String>,
    pub ciphertext_path: String,
    pub plaintext_sha256: String,
    pub ciphertext_sha256: String,
    pub plaintext_bytes: usize,
}

#[derive(Debug, Clone)]
pub struct BundleWriteRequest<'a> {
    pub output_path: &'a Path,
    pub bundle_kind: &'a str,
    pub campaign_id: &'a str,
    pub domain: &'a str,
    pub stage: &'a str,
    pub tool: &'a str,
    pub sample: &'a str,
    pub planned_job_id: &'a str,
    pub scheduler_job_id: &'a str,
    pub submitted_at: &'a str,
    pub backend: &'a str,
    pub recipients: &'a [String],
    pub plaintext: &'a [u8],
}

#[derive(Debug, Clone)]
pub struct BundleDecryptRequest<'a> {
    pub bundle_path: &'a Path,
    pub sidecar_path: Option<&'a Path>,
    pub identity_files: &'a [PathBuf],
}

#[derive(Debug, Clone)]
enum EncryptionBackend {
    MockEnvelopeV1,
    AgeCli,
}

#[derive(Debug, Serialize, Deserialize)]
struct MockEnvelope {
    schema_version: String,
    recipients: Vec<String>,
    payload_sha256: String,
    ciphertext_hex: String,
}

fn parse_backend(value: &str) -> Result<EncryptionBackend> {
    match value {
        "mock-envelope-v1" => Ok(EncryptionBackend::MockEnvelopeV1),
        "age-cli" => Ok(EncryptionBackend::AgeCli),
        other => Err(anyhow!("unsupported encryption backend `{other}`")),
    }
}

pub fn sidecar_path_for(bundle_path: &Path) -> PathBuf {
    let mut path = bundle_path.to_path_buf();
    let suffix = bundle_path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "bundle.sidecar.json".to_string(), |name| format!("{name}.sidecar.json"));
    path.set_file_name(suffix);
    path
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

pub fn digest_file_sha256(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let count = file
            .read(&mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buf[..count]);
    }
    Ok(hex_encode(&hasher.finalize()))
}

fn recipient_fingerprint(value: &str) -> String {
    let digest = sha256_hex(value.as_bytes());
    digest.chars().take(16).collect()
}

fn ensure_parent(path: &Path) -> Result<()> {
    let Some(parent) = path.parent() else {
        return Err(anyhow!("path has no parent: {}", path.display()));
    };
    bijux_dna_infra::ensure_dir(parent).with_context(|| format!("create {}", parent.display()))
}

fn temp_output_path(output_path: &Path) -> PathBuf {
    let mut path = output_path.to_path_buf();
    let suffix = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .map_or_else(|| "bundle.tmp".to_string(), |name| {
            format!("{name}.tmp.{}.{}", std::process::id(), monotonic_nanos())
        });
    path.set_file_name(suffix);
    path
}

fn monotonic_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |delta| delta.as_nanos())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn hex_decode(value: &str) -> Result<Vec<u8>> {
    if value.len() % 2 != 0 {
        return Err(anyhow!("hex payload has odd length"));
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    let bytes = value.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let hi = decode_nibble(bytes[i])?;
        let lo = decode_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn decode_nibble(value: u8) -> Result<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        b'A'..=b'F' => Ok(value - b'A' + 10),
        _ => Err(anyhow!("invalid hex digit `{}`", value as char)),
    }
}

fn encrypt_mock_envelope(recipients: &[String], plaintext: &[u8]) -> Result<Vec<u8>> {
    let key = Sha256::digest(format!("mock-envelope-v1|{}", recipients.join("|")).as_bytes());
    let ciphertext = plaintext
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ key[index % key.len()])
        .collect::<Vec<_>>();
    let envelope = MockEnvelope {
        schema_version: MOCK_ENVELOPE_SCHEMA_VERSION.to_string(),
        recipients: recipients.to_vec(),
        payload_sha256: sha256_hex(plaintext),
        ciphertext_hex: hex_encode(&ciphertext),
    };
    serde_json::to_vec_pretty(&envelope).context("serialize mock envelope")
}

fn decrypt_mock_envelope(ciphertext: &[u8]) -> Result<Vec<u8>> {
    let envelope: MockEnvelope =
        serde_json::from_slice(ciphertext).context("parse mock envelope ciphertext")?;
    if envelope.schema_version != MOCK_ENVELOPE_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported mock envelope schema `{}`",
            envelope.schema_version
        ));
    }
    let key = Sha256::digest(format!("mock-envelope-v1|{}", envelope.recipients.join("|")).as_bytes());
    let cipher_bytes = hex_decode(&envelope.ciphertext_hex)?;
    let plaintext = cipher_bytes
        .iter()
        .enumerate()
        .map(|(index, byte)| byte ^ key[index % key.len()])
        .collect::<Vec<_>>();
    let digest = sha256_hex(&plaintext);
    if digest != envelope.payload_sha256 {
        return Err(anyhow!("mock envelope plaintext hash mismatch"));
    }
    Ok(plaintext)
}

fn encrypt_age_cli(
    recipients: &[String],
    plaintext: &[u8],
    output_path: &Path,
) -> Result<()> {
    if recipients.is_empty() {
        return Err(anyhow!("age-cli backend requires at least one recipient"));
    }
    let mut command = Command::new("age");
    command.arg("--encrypt");
    for recipient in recipients {
        command.arg("-r").arg(recipient);
    }
    command.arg("-o").arg(output_path).arg("-");
    command.stdin(Stdio::piped()).stdout(Stdio::null()).stderr(Stdio::piped());
    let mut child = command.spawn().context("spawn age --encrypt")?;
    {
        let stdin = child.stdin.as_mut().ok_or_else(|| anyhow!("open age stdin"))?;
        stdin.write_all(plaintext).context("write plaintext to age stdin")?;
    }
    let output = child.wait_with_output().context("wait for age --encrypt")?;
    if !output.status.success() {
        return Err(anyhow!(
            "age --encrypt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn decrypt_age_cli(bundle_path: &Path, identity_files: &[PathBuf]) -> Result<Vec<u8>> {
    if identity_files.is_empty() {
        return Err(anyhow!(
            "age-cli decrypt requires at least one identity file; pass --identity-file"
        ));
    }
    let mut command = Command::new("age");
    command.arg("--decrypt");
    for identity_file in identity_files {
        command.arg("-i").arg(identity_file);
    }
    command.arg(bundle_path);
    let output = command
        .output()
        .with_context(|| format!("run age --decrypt for {}", bundle_path.display()))?;
    if !output.status.success() {
        return Err(anyhow!(
            "age --decrypt failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(output.stdout)
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
            std::fs::write(output_path, ciphertext)
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

pub fn write_encrypted_bundle(request: &BundleWriteRequest<'_>) -> Result<BundleSidecar> {
    let backend = parse_backend(request.backend)?;
    if request.recipients.is_empty() {
        return Err(anyhow!("bundle encryption recipients list is empty"));
    }

    ensure_parent(request.output_path)?;
    let temp_path = temp_output_path(request.output_path);
    let plaintext_sha256 = sha256_hex(request.plaintext);

    let encrypt_result = encrypt_with_backend(
        &backend,
        request.recipients,
        request.plaintext,
        &temp_path,
    );

    if let Err(error) = encrypt_result {
        let _ = std::fs::remove_file(&temp_path);
        return Err(error)
            .with_context(|| format!("encrypt {}", request.output_path.display()));
    }

    let ciphertext_sha256 = digest_file_sha256(&temp_path)?;
    std::fs::rename(&temp_path, request.output_path).with_context(|| {
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
    bijux_dna_api::v1::api::run::atomic_write_bytes(&sidecar_path, &payload)
        .with_context(|| format!("write {}", sidecar_path.display()))?;

    Ok(sidecar)
}

pub fn decrypt_bundle(request: &BundleDecryptRequest<'_>) -> Result<(BundleSidecar, Vec<u8>)> {
    let sidecar_path =
        request.sidecar_path.map_or_else(|| sidecar_path_for(request.bundle_path), PathBuf::from);
    let raw_sidecar =
        std::fs::read(&sidecar_path).with_context(|| format!("read {}", sidecar_path.display()))?;
    let sidecar: BundleSidecar =
        serde_json::from_slice(&raw_sidecar).context("parse bundle sidecar")?;
    if sidecar.schema_version != BUNDLE_SIDECAR_SCHEMA_VERSION {
        return Err(anyhow!(
            "unsupported bundle sidecar schema `{}`",
            sidecar.schema_version
        ));
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
    let plaintext_sha256 = sha256_hex(&plaintext);
    if plaintext_sha256 != sidecar.plaintext_sha256 {
        return Err(anyhow!("plaintext hash mismatch"));
    }
    Ok((sidecar, plaintext))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        decrypt_bundle, sha256_hex, sidecar_path_for, write_encrypted_bundle, BundleDecryptRequest,
        BundleWriteRequest,
    };

    #[test]
    fn mock_bundle_round_trip_and_sidecar_hash_validation() {
        let root = tempfile::tempdir().expect("tempdir");
        let bundle_path = root.path().join("bundle.results");
        let plaintext = br#"{"metrics":{"walltime_sec":10}}"#;
        let recipients = vec!["alice".to_string(), "bob".to_string()];

        let sidecar = write_encrypted_bundle(&BundleWriteRequest {
            output_path: &bundle_path,
            bundle_kind: "results",
            campaign_id: "mini",
            domain: "fastq",
            stage: "fastq.validate_reads",
            tool: "seqkit_v2",
            sample: "sample-1",
            planned_job_id: "dryrun-0001",
            scheduler_job_id: "mock-0001",
            submitted_at: "1700000000",
            backend: "mock-envelope-v1",
            recipients: &recipients,
            plaintext,
        })
        .expect("encrypt bundle");
        assert_eq!(sidecar.plaintext_sha256, sha256_hex(plaintext));
        assert!(bundle_path.is_file());

        let (decoded_sidecar, decoded) = decrypt_bundle(&BundleDecryptRequest {
            bundle_path: &bundle_path,
            sidecar_path: None,
            identity_files: &[],
        })
        .expect("decrypt bundle");
        assert_eq!(decoded_sidecar.bundle_kind, "results");
        assert_eq!(decoded, plaintext);
    }

    #[test]
    fn decrypt_bundle_detects_ciphertext_tamper() {
        let root = tempfile::tempdir().expect("tempdir");
        let bundle_path = root.path().join("bundle.results");
        let plaintext = br#"{"metrics":{"walltime_sec":10}}"#;
        let recipients = vec!["alice".to_string()];

        write_encrypted_bundle(&BundleWriteRequest {
            output_path: &bundle_path,
            bundle_kind: "results",
            campaign_id: "mini",
            domain: "fastq",
            stage: "fastq.validate_reads",
            tool: "seqkit_v2",
            sample: "sample-1",
            planned_job_id: "dryrun-0001",
            scheduler_job_id: "mock-0001",
            submitted_at: "1700000000",
            backend: "mock-envelope-v1",
            recipients: &recipients,
            plaintext,
        })
        .expect("encrypt bundle");

        let mut ciphertext = std::fs::read(&bundle_path).expect("read ciphertext");
        ciphertext[0] ^= 1;
        std::fs::write(&bundle_path, ciphertext).expect("write tampered ciphertext");

        let err = decrypt_bundle(&BundleDecryptRequest {
            bundle_path: &bundle_path,
            sidecar_path: None,
            identity_files: &[],
        })
        .expect_err("must fail integrity validation");
        assert!(err.to_string().contains("ciphertext hash mismatch"));
    }

    #[test]
    fn age_backend_refuses_decrypt_without_identity_files() {
        let root = tempfile::tempdir().expect("tempdir");
        let bundle_path = root.path().join("bundle.code");
        let plaintext = br#"{"code":{"state":"frozen"}}"#;
        let recipients = vec!["alice".to_string()];

        write_encrypted_bundle(&BundleWriteRequest {
            output_path: &bundle_path,
            bundle_kind: "code",
            campaign_id: "mini",
            domain: "fastq",
            stage: "fastq.validate_reads",
            tool: "seqkit_v2",
            sample: "sample-1",
            planned_job_id: "dryrun-0001",
            scheduler_job_id: "mock-0001",
            submitted_at: "1700000000",
            backend: "mock-envelope-v1",
            recipients: &recipients,
            plaintext,
        })
        .expect("encrypt bundle");

        let sidecar_path = sidecar_path_for(&bundle_path);
        let mut sidecar: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&sidecar_path).expect("read sidecar"))
                .expect("parse sidecar");
        sidecar["backend"] = serde_json::Value::String("age-cli".to_string());
        std::fs::write(
            &sidecar_path,
            serde_json::to_vec_pretty(&sidecar).expect("serialize sidecar"),
        )
        .expect("write sidecar");

        let err = decrypt_bundle(&BundleDecryptRequest {
            bundle_path: &bundle_path,
            sidecar_path: Some(&sidecar_path),
            identity_files: &[],
        })
        .expect_err("must reject missing age identity");
        assert!(err.to_string().contains("requires at least one identity file"));
    }
}
