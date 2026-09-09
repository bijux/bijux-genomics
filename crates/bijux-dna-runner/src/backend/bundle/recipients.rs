//! Owner: bundle recipient declaration consistency before transport effects.
use super::{storage::recipient_fingerprint, BundleSidecar};
use anyhow::{ensure, Result};
use std::collections::BTreeSet;

pub(super) fn validate_recipients(recipients: &[String]) -> Result<()> {
    ensure!(!recipients.is_empty(), "bundle encryption recipients list is empty");
    let mut seen = BTreeSet::new();
    for recipient in recipients {
        ensure!(
            !recipient.trim().is_empty() && recipient.trim() == recipient,
            "bundle recipient must be nonblank and have no surrounding whitespace"
        );
        ensure!(seen.insert(recipient), "bundle recipient list contains duplicates");
    }
    Ok(())
}

/// Fingerprints describe the declarations; an unsigned sidecar does not authenticate age recipients.
pub(super) fn validate_sidecar_recipients(sidecar: &BundleSidecar) -> Result<()> {
    validate_recipients(&sidecar.recipients)?;
    let expected = sidecar
        .recipients
        .iter()
        .map(|recipient| recipient_fingerprint(recipient))
        .collect::<Vec<_>>();
    ensure!(
        sidecar.recipient_fingerprints == expected,
        "bundle recipient fingerprints disagree with recipient declarations"
    );
    Ok(())
}
