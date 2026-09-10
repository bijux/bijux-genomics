//! Canonical external-file identity observations shared by admission and later verification.
use super::{model::ExternalArtifactIdentity, request::ExternalArtifactInput};
use anyhow::{bail, Context, Result};
use std::path::Path;

pub(super) fn observe(
    repository: &Path,
    distributable_images: &Path,
    input: &ExternalArtifactInput<'_>,
) -> Result<ExternalArtifactIdentity> {
    let path = input.path.canonicalize().context("canonicalize external artifact")?;
    admit_location(repository, distributable_images, &path)?;
    let metadata = path.metadata().context("inspect external artifact")?;
    if !metadata.is_file() {
        bail!("external artifact is not a regular file");
    }
    let sha256 = bijux_dna_infra::hash_file_sha256(&path).context("hash external artifact")?;
    if sha256 != input.expected_sha256.to_ascii_lowercase() {
        bail!("external artifact checksum does not match operator declaration");
    }
    Ok(ExternalArtifactIdentity {
        kind: input.kind,
        path: path.to_str().context("external artifact path is not UTF-8")?.to_owned(),
        sha256,
        size_bytes: metadata.len(),
    })
}

pub(super) fn verify(
    repository: &Path,
    distributable_images: &Path,
    identity: &ExternalArtifactIdentity,
) -> Result<()> {
    let path = Path::new(&identity.path);
    let resolved = path.canonicalize().context("resolve current external artifact location")?;
    if resolved != path {
        bail!("external artifact canonical location changed");
    }
    admit_location(repository, distributable_images, path)?;
    let metadata = path.metadata().context("inspect external artifact")?;
    if !metadata.is_file()
        || metadata.len() != identity.size_bytes
        || bijux_dna_infra::hash_file_sha256(path).context("rehash external artifact")?
            != identity.sha256
    {
        bail!("external artifact identity changed");
    }
    Ok(())
}

fn admit_location(repository: &Path, distributable_images: &Path, path: &Path) -> Result<()> {
    if path.starts_with(repository) {
        bail!("external artifact must remain outside the product repository");
    }
    if path.starts_with(distributable_images) {
        bail!("external artifact must remain outside the distributable active image root");
    }
    Ok(())
}
