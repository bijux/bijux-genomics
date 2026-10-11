use anyhow::{ensure, Context, Result};
use chrono::DateTime;

use super::{
    release_evidence_policy_sha256, ContainerReleaseEvidencePublication,
    CONTAINER_RELEASE_EVIDENCE_SCHEMA_V1,
};
use crate::qualification::{ArtifactAvailability, EvidenceVerdict, ProductSupport};

/// Validate the structure and policy invariants of a release publication.
///
/// # Errors
/// Returns an error for invalid identities, timestamps, tool keys, or verdict semantics.
pub fn validate_release_evidence_publication(
    publication: &ContainerReleaseEvidencePublication,
) -> Result<()> {
    ensure!(
        publication.schema_version == CONTAINER_RELEASE_EVIDENCE_SCHEMA_V1,
        "unsupported release evidence schema: {}",
        publication.schema_version
    );
    DateTime::parse_from_rfc3339(&publication.released_at_utc)
        .context("release publication timestamp is not RFC 3339")?;
    ensure!(valid_git_commit(&publication.repository_commit), "invalid release repository commit");
    ensure!(valid_sha256(&publication.catalog_sha256), "invalid release catalog SHA-256");
    ensure!(
        valid_sha256(&publication.qualification_sources_sha256),
        "invalid qualification sources SHA-256"
    );
    ensure!(
        publication.policy_sha256 == release_evidence_policy_sha256(),
        "release policy identity differs from the current policy"
    );
    ensure!(!publication.tools.is_empty(), "release publication has no tools");
    for (tool_id, evidence) in &publication.tools {
        ensure!(!tool_id.trim().is_empty(), "release publication has an empty tool ID");
        ensure!(tool_id == &evidence.tool_id, "release tool key differs from its tool ID");
        ensure!(
            valid_sha256(&evidence.qualification_evidence_set_sha256),
            "release evidence set identity is invalid for {tool_id}"
        );
        ensure!(
            evidence.reasons.iter().all(|reason| !reason.trim().is_empty()),
            "release evidence contains an empty reason for {tool_id}"
        );
        match evidence.product_support {
            ProductSupport::Supported => {
                ensure!(
                    evidence.verdict != EvidenceVerdict::NotApplicable,
                    "supported release evidence cannot be not applicable for {tool_id}"
                );
                if evidence.availability != ArtifactAvailability::Admitted {
                    ensure!(
                        evidence.verdict == EvidenceVerdict::Blocked,
                        "unadmitted supported release evidence must be blocked for {tool_id}"
                    );
                }
                ensure!(
                    (evidence.verdict == EvidenceVerdict::Passed) == evidence.reasons.is_empty(),
                    "passing release evidence must omit reasons and non-passing evidence must explain them for {tool_id}"
                );
            }
            ProductSupport::Retired | ProductSupport::Excluded => {
                ensure!(
                    evidence.verdict == EvidenceVerdict::NotApplicable,
                    "unsupported release evidence must be not applicable for {tool_id}"
                );
                ensure!(
                    evidence.reasons.is_empty(),
                    "not-applicable release evidence must omit reasons for {tool_id}"
                );
            }
        }
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}

fn valid_git_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    #[test]
    fn checked_release_evidence_schema_matches_rust_contract() -> Result<()> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| anyhow::anyhow!("policies crate is outside the repository"))?;
        let path = root.join("containers/contracts/schemas/release-evidence.v1.json");
        let checked: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        assert_eq!(checked, super::super::release_evidence_schema());
        Ok(())
    }
}
