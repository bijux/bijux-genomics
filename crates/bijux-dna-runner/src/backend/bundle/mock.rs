//! Owner: runner deterministic mock transport; this format does not provide confidentiality.
use super::storage::{hex_encode, sha256_hex};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
const MOCK_ENVELOPE_SCHEMA_VERSION: &str = "bijux.hpc.bundle.mock_envelope.v1";
#[derive(Debug, Serialize, Deserialize)]
struct MockEnvelope {
    schema_version: String,
    recipients: Vec<String>,
    payload_sha256: String,
    ciphertext_hex: String,
}

fn hex_decode(value: &str) -> Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
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
pub(super) fn encrypt_mock_envelope(recipients: &[String], plaintext: &[u8]) -> Result<Vec<u8>> {
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
pub(super) fn decrypt_mock_envelope(ciphertext: &[u8]) -> Result<Vec<u8>> {
    let envelope: MockEnvelope =
        serde_json::from_slice(ciphertext).context("parse mock envelope ciphertext")?;
    if envelope.schema_version != MOCK_ENVELOPE_SCHEMA_VERSION {
        return Err(anyhow!("unsupported mock envelope schema `{}`", envelope.schema_version));
    }
    let key =
        Sha256::digest(format!("mock-envelope-v1|{}", envelope.recipients.join("|")).as_bytes());
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
