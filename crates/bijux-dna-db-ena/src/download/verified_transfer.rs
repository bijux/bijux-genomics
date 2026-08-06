use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use rayon::prelude::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::client::CRATE_USER_AGENT;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VerifiedDownloadRequest {
    pub source_url: String,
    pub output: PathBuf,
    pub expected_size_bytes: u64,
    pub expected_md5: String,
    pub retries: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerifiedDownloadDisposition {
    Downloaded,
    VerifiedExisting,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VerifiedDownloadOutcome {
    pub output: PathBuf,
    pub size_bytes: u64,
    pub md5: String,
    pub sha256: String,
    pub disposition: VerifiedDownloadDisposition,
}

/// Download one HTTPS object and publish it only after exact size and MD5 verification.
///
/// An existing regular file is re-read and accepted only when its identity matches. New bytes are
/// streamed through bounded memory into a private temporary file and persisted without replacing a
/// concurrent destination.
///
/// # Errors
/// Returns an error for unsafe paths, malformed identities, HTTP failures, checksum drift, or
/// publication conflicts.
pub fn download_verified_file(
    request: &VerifiedDownloadRequest,
) -> Result<VerifiedDownloadOutcome> {
    validate_request(request)?;
    if let Some(outcome) = verify_existing(request)? {
        return Ok(outcome);
    }
    let parent = prepare_parent(&request.output)?;
    let client = Client::builder()
        .user_agent(CRATE_USER_AGENT)
        .timeout(Duration::from_secs(request.timeout_seconds))
        .build()
        .context("create verified ENA transfer client")?;
    let mut last_error = None;
    for _ in 0..=request.retries {
        let attempt = (|| {
            let response = client
                .get(&request.source_url)
                .send()
                .with_context(|| format!("request {}", request.source_url))?
                .error_for_status()
                .with_context(|| format!("receive {}", request.source_url))?;
            publish_reader(request, &parent, response)
        })();
        match attempt {
            Ok(outcome) => return Ok(outcome),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("verified transfer did not execute")))
        .with_context(|| format!("download {}", request.source_url))
}

/// Download a unique set of verified objects with an explicit parallelism bound.
///
/// Returned outcomes preserve request order. Individual transfers retain their own retry and
/// timeout policies.
///
/// # Errors
/// Returns an error when the batch is empty, parallelism is zero, output identities repeat, thread
/// pool construction fails, or any transfer fails.
pub fn download_verified_files(
    requests: &[VerifiedDownloadRequest],
    maximum_parallelism: usize,
) -> Result<Vec<VerifiedDownloadOutcome>> {
    if requests.is_empty() {
        bail!("verified download batch is empty");
    }
    if maximum_parallelism == 0 {
        bail!("verified download parallelism must be positive");
    }
    let mut outputs = BTreeSet::new();
    for request in requests {
        validate_request(request)?;
        if !outputs.insert(request.output.clone()) {
            bail!("verified download output repeats: {}", request.output.display());
        }
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(maximum_parallelism)
        .build()
        .context("create verified ENA transfer pool")?;
    pool.install(|| requests.par_iter().map(download_verified_file).collect())
}

fn publish_reader(
    request: &VerifiedDownloadRequest,
    parent: &Path,
    reader: impl Read,
) -> Result<VerifiedDownloadOutcome> {
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .with_context(|| format!("create private transfer file under {}", parent.display()))?;
    let identity = copy_and_hash(reader, temporary.as_file_mut())?;
    verify_expected(request, &identity)?;
    temporary.as_file_mut().sync_all().context("sync verified transfer bytes")?;
    match temporary.persist_noclobber(&request.output) {
        Ok(_) => Ok(outcome(request, identity, VerifiedDownloadDisposition::Downloaded)),
        Err(error) if error.error.kind() == ErrorKind::AlreadyExists => verify_existing(request)?
            .context("concurrent transfer destination conflicts with verified publication"),
        Err(error) => Err(error.error)
            .with_context(|| format!("publish verified transfer {}", request.output.display())),
    }
}

fn verify_existing(request: &VerifiedDownloadRequest) -> Result<Option<VerifiedDownloadOutcome>> {
    let metadata = match fs::symlink_metadata(&request.output) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect transfer output {}", request.output.display()))
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("verified transfer output is not a regular non-symlink file");
    }
    let canonical = request
        .output
        .canonicalize()
        .with_context(|| format!("canonicalize transfer output {}", request.output.display()))?;
    if canonical != request.output {
        bail!("verified transfer output path is not canonical");
    }
    let identity = hash_reader(
        File::open(&request.output)
            .with_context(|| format!("open transfer output {}", request.output.display()))?,
    )?;
    verify_expected(request, &identity)?;
    Ok(Some(outcome(request, identity, VerifiedDownloadDisposition::VerifiedExisting)))
}

fn prepare_parent(output: &Path) -> Result<PathBuf> {
    let parent = output.parent().context("verified transfer output has no parent")?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create verified transfer parent {}", parent.display()))?;
    let metadata = fs::symlink_metadata(parent)
        .with_context(|| format!("inspect verified transfer parent {}", parent.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("verified transfer parent is not a regular non-symlink directory");
    }
    let canonical = parent
        .canonicalize()
        .with_context(|| format!("canonicalize verified transfer parent {}", parent.display()))?;
    if canonical != parent {
        bail!("verified transfer parent path is not canonical");
    }
    Ok(canonical)
}

fn validate_request(request: &VerifiedDownloadRequest) -> Result<()> {
    if !request.source_url.starts_with("https://")
        || request.source_url.chars().any(|value| matches!(value, '\n' | '\r' | '\0'))
    {
        bail!("verified transfer source must be an HTTPS URL");
    }
    if !request.output.is_absolute() {
        bail!("verified transfer output must be absolute");
    }
    if request.expected_size_bytes == 0 {
        bail!("verified transfer expected size must be positive");
    }
    if request.expected_md5.len() != 32
        || !request
            .expected_md5
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
    {
        bail!("verified transfer MD5 identity is malformed");
    }
    if request.timeout_seconds == 0 {
        bail!("verified transfer timeout must be positive");
    }
    Ok(())
}

#[derive(Debug)]
struct ObservedIdentity {
    size_bytes: u64,
    md5: String,
    sha256: String,
}

fn copy_and_hash(mut reader: impl Read, mut writer: impl Write) -> Result<ObservedIdentity> {
    let mut md5 = md5::Context::new();
    let mut sha256 = Sha256::new();
    let mut size_bytes = 0_u64;
    let mut buffer = vec![0_u8; 1024 * 1024].into_boxed_slice();
    loop {
        let count = reader.read(&mut buffer).context("read verified transfer bytes")?;
        if count == 0 {
            break;
        }
        writer.write_all(&buffer[..count]).context("write verified transfer bytes")?;
        size_bytes = size_bytes
            .checked_add(u64::try_from(count).context("transfer chunk size overflow")?)
            .context("verified transfer size overflow")?;
        md5.consume(&buffer[..count]);
        sha256.update(&buffer[..count]);
    }
    writer.flush().context("flush verified transfer bytes")?;
    Ok(ObservedIdentity {
        size_bytes,
        md5: format!("{:x}", md5.finalize()),
        sha256: lower_hex(&sha256.finalize()),
    })
}

fn lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn hash_reader(reader: impl Read) -> Result<ObservedIdentity> {
    copy_and_hash(reader, std::io::sink())
}

fn verify_expected(request: &VerifiedDownloadRequest, identity: &ObservedIdentity) -> Result<()> {
    if identity.size_bytes != request.expected_size_bytes || identity.md5 != request.expected_md5 {
        bail!(
            "verified transfer identity mismatch for {}: observed {} bytes and MD5 {}",
            request.output.display(),
            identity.size_bytes,
            identity.md5
        );
    }
    Ok(())
}

fn outcome(
    request: &VerifiedDownloadRequest,
    identity: ObservedIdentity,
    disposition: VerifiedDownloadDisposition,
) -> VerifiedDownloadOutcome {
    VerifiedDownloadOutcome {
        output: request.output.clone(),
        size_bytes: identity.size_bytes,
        md5: identity.md5,
        sha256: identity.sha256,
        disposition,
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn publishes_verified_bytes_without_replacement() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let output = directory.path().canonicalize()?.join("read.fastq.gz");
        let request = request(output.clone(), b"governed-fastq")?;

        let downloaded =
            publish_reader(&request, directory.path(), Cursor::new(b"governed-fastq"))?;
        assert_eq!(downloaded.disposition, VerifiedDownloadDisposition::Downloaded);
        assert_eq!(fs::read(&output)?, b"governed-fastq");

        let existing = verify_existing(&request)?.context("verified existing output")?;
        assert_eq!(existing.disposition, VerifiedDownloadDisposition::VerifiedExisting);
        assert_eq!(existing.sha256, downloaded.sha256);
        assert_eq!(download_verified_file(&request)?, existing);
        Ok(())
    }

    #[test]
    fn rejects_identity_drift_without_publishing() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let output = directory.path().canonicalize()?.join("read.fastq.gz");
        let request = request(output.clone(), b"declared")?;

        let Err(error) = publish_reader(&request, directory.path(), Cursor::new(b"different"))
        else {
            bail!("identity drift must fail");
        };
        assert!(format!("{error:#}").contains("identity mismatch"));
        assert!(!output.exists());
        Ok(())
    }

    #[test]
    fn rejects_duplicate_batch_outputs_before_network_access() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let output = directory.path().canonicalize()?.join("read.fastq.gz");
        let request = request(output, b"governed-fastq")?;

        assert!(download_verified_files(&[request.clone(), request], 2)
            .is_err_and(|error| format!("{error:#}").contains("output repeats")));
        Ok(())
    }

    #[test]
    fn rejects_empty_or_zero_parallelism_batches() -> Result<()> {
        assert!(download_verified_files(&[], 1)
            .is_err_and(|error| format!("{error:#}").contains("batch is empty")));
        let directory = tempfile::tempdir()?;
        let output = directory.path().canonicalize()?.join("read.fastq.gz");
        assert!(download_verified_files(&[request(output, b"governed-fastq")?], 0)
            .is_err_and(|error| format!("{error:#}").contains("parallelism must be positive")));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_existing_outputs() -> Result<()> {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir()?;
        let source = directory.path().join("source.fastq.gz");
        fs::write(&source, b"governed-fastq")?;
        let output = directory.path().join("read.fastq.gz");
        symlink(source, &output)?;
        let request = request(output, b"governed-fastq")?;

        assert!(verify_existing(&request)
            .is_err_and(|error| format!("{error:#}").contains("non-symlink")));
        Ok(())
    }

    fn request(output: PathBuf, bytes: &[u8]) -> Result<VerifiedDownloadRequest> {
        Ok(VerifiedDownloadRequest {
            source_url: "https://archive.example/read.fastq.gz".to_string(),
            output,
            expected_size_bytes: u64::try_from(bytes.len()).context("fixture length")?,
            expected_md5: format!("{:x}", md5::compute(bytes)),
            retries: 0,
            timeout_seconds: 10,
        })
    }
}
