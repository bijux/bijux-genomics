use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::model::{EvidenceDecision, EvidenceVerdict};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublicationArtifactIdentity {
    pub uri: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImmutablePublicationIdentity {
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub source_contracts: BTreeMap<String, PublicationArtifactIdentity>,
    pub raw_evidence: PublicationArtifactIdentity,
    pub generated_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicationAuthority {
    pub schema_version: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub source_contracts: BTreeMap<String, PublicationArtifactIdentity>,
    pub raw_evidence: PublicationArtifactIdentity,
    pub maximum_age: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PublicationFreshnessVerdict {
    Current,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum PublicationIdentityMismatch {
    InvalidAuthority { field: String, reason: String },
    InvalidPublication { field: String, reason: String },
    ValueMismatch { field: String, expected: String, observed: String },
    MissingSourceContract { contract: String },
    UnexpectedSourceContract { contract: String },
    GeneratedInFuture { generated_at_utc: String, evaluated_at_utc: String },
    Expired { generated_at_utc: String, evaluated_at_utc: String, maximum_age_seconds: i64 },
}

impl PublicationIdentityMismatch {
    #[must_use]
    pub fn reason(&self) -> String {
        match self {
            Self::InvalidAuthority { field, reason } => {
                format!("publication authority {field} is invalid: {reason}")
            }
            Self::InvalidPublication { field, reason } => {
                format!("publication {field} is invalid: {reason}")
            }
            Self::ValueMismatch { field, expected, observed } => {
                format!("publication {field} mismatch: expected {expected}, observed {observed}")
            }
            Self::MissingSourceContract { contract } => {
                format!("publication omits required source contract {contract}")
            }
            Self::UnexpectedSourceContract { contract } => {
                format!("publication contains unexpected source contract {contract}")
            }
            Self::GeneratedInFuture { generated_at_utc, evaluated_at_utc } => format!(
                "publication timestamp {generated_at_utc} is later than evaluation time {evaluated_at_utc}"
            ),
            Self::Expired {
                generated_at_utc,
                evaluated_at_utc,
                maximum_age_seconds,
            } => format!(
                "publication generated at {generated_at_utc} is older than {maximum_age_seconds} seconds at {evaluated_at_utc}"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PublicationFreshnessEvaluation {
    pub verdict: PublicationFreshnessVerdict,
    pub evaluated_at_utc: String,
    pub mismatches: Vec<PublicationIdentityMismatch>,
}

impl PublicationFreshnessEvaluation {
    #[must_use]
    pub fn is_current(&self) -> bool {
        self.verdict == PublicationFreshnessVerdict::Current
    }

    #[must_use]
    pub fn qualification_decision(&self, evaluation_sha256: String) -> EvidenceDecision {
        if self.is_current() {
            EvidenceDecision {
                verdict: EvidenceVerdict::Passed,
                evidence_sha256: Some(evaluation_sha256),
                reasons: Vec::new(),
            }
        } else {
            EvidenceDecision {
                verdict: EvidenceVerdict::Failed,
                evidence_sha256: None,
                reasons: self.mismatches.iter().map(PublicationIdentityMismatch::reason).collect(),
            }
        }
    }
}

#[must_use]
pub fn evaluate_publication_freshness(
    publication: &ImmutablePublicationIdentity,
    authority: &PublicationAuthority,
    evaluated_at: DateTime<Utc>,
) -> PublicationFreshnessEvaluation {
    let mut mismatches = Vec::new();
    validate_authority(authority, &mut mismatches);
    validate_publication(publication, &mut mismatches);
    compare_value(
        "schema_version",
        &authority.schema_version,
        &publication.schema_version,
        &mut mismatches,
    );
    compare_value(
        "repository_commit",
        &authority.repository_commit,
        &publication.repository_commit,
        &mut mismatches,
    );
    compare_value(
        "catalog_sha256",
        &authority.catalog_sha256,
        &publication.catalog_sha256,
        &mut mismatches,
    );
    compare_source_contracts(publication, authority, &mut mismatches);
    compare_artifact(
        "raw_evidence",
        &authority.raw_evidence,
        &publication.raw_evidence,
        &mut mismatches,
    );
    evaluate_age(publication, authority, evaluated_at, &mut mismatches);

    PublicationFreshnessEvaluation {
        verdict: if mismatches.is_empty() {
            PublicationFreshnessVerdict::Current
        } else {
            PublicationFreshnessVerdict::Rejected
        },
        evaluated_at_utc: evaluated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        mismatches,
    }
}

fn validate_authority(
    authority: &PublicationAuthority,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    validate_nonempty_authority("schema_version", &authority.schema_version, mismatches);
    validate_git_authority("repository_commit", &authority.repository_commit, mismatches);
    validate_sha256_authority("catalog_sha256", &authority.catalog_sha256, mismatches);
    if authority.source_contracts.is_empty() {
        invalid_authority("source_contracts", "at least one contract is required", mismatches);
    }
    for (contract, artifact) in &authority.source_contracts {
        validate_contract_name(contract, true, mismatches);
        validate_artifact(contract, artifact, true, mismatches);
    }
    validate_artifact("raw_evidence", &authority.raw_evidence, true, mismatches);
    if authority.maximum_age <= Duration::zero() {
        invalid_authority("maximum_age", "duration must be positive", mismatches);
    }
}

fn validate_publication(
    publication: &ImmutablePublicationIdentity,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    validate_nonempty_publication("schema_version", &publication.schema_version, mismatches);
    validate_git_publication("repository_commit", &publication.repository_commit, mismatches);
    validate_sha256_publication("catalog_sha256", &publication.catalog_sha256, mismatches);
    if publication.source_contracts.is_empty() {
        invalid_publication("source_contracts", "at least one contract is required", mismatches);
    }
    for (contract, artifact) in &publication.source_contracts {
        validate_contract_name(contract, false, mismatches);
        validate_artifact(contract, artifact, false, mismatches);
    }
    validate_artifact("raw_evidence", &publication.raw_evidence, false, mismatches);
}

fn compare_source_contracts(
    publication: &ImmutablePublicationIdentity,
    authority: &PublicationAuthority,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    let expected = authority.source_contracts.keys().collect::<BTreeSet<_>>();
    let observed = publication.source_contracts.keys().collect::<BTreeSet<_>>();
    for contract in expected.difference(&observed) {
        mismatches.push(PublicationIdentityMismatch::MissingSourceContract {
            contract: (*contract).clone(),
        });
    }
    for contract in observed.difference(&expected) {
        mismatches.push(PublicationIdentityMismatch::UnexpectedSourceContract {
            contract: (*contract).clone(),
        });
    }
    for contract in expected.intersection(&observed) {
        compare_artifact(
            &format!("source_contracts.{contract}"),
            &authority.source_contracts[*contract],
            &publication.source_contracts[*contract],
            mismatches,
        );
    }
}

fn compare_artifact(
    field: &str,
    expected: &PublicationArtifactIdentity,
    observed: &PublicationArtifactIdentity,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    compare_value(&format!("{field}.uri"), &expected.uri, &observed.uri, mismatches);
    compare_value(&format!("{field}.sha256"), &expected.sha256, &observed.sha256, mismatches);
}

fn compare_value(
    field: &str,
    expected: &str,
    observed: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if expected != observed {
        mismatches.push(PublicationIdentityMismatch::ValueMismatch {
            field: field.to_string(),
            expected: expected.to_string(),
            observed: observed.to_string(),
        });
    }
}

fn evaluate_age(
    publication: &ImmutablePublicationIdentity,
    authority: &PublicationAuthority,
    evaluated_at: DateTime<Utc>,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    let Ok(generated_at) = DateTime::parse_from_rfc3339(&publication.generated_at_utc) else {
        invalid_publication("generated_at_utc", "timestamp must be RFC 3339", mismatches);
        return;
    };
    let generated_at = generated_at.with_timezone(&Utc);
    if generated_at > evaluated_at {
        mismatches.push(PublicationIdentityMismatch::GeneratedInFuture {
            generated_at_utc: publication.generated_at_utc.clone(),
            evaluated_at_utc: evaluated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        });
    } else if authority.maximum_age > Duration::zero()
        && evaluated_at.signed_duration_since(generated_at) > authority.maximum_age
    {
        mismatches.push(PublicationIdentityMismatch::Expired {
            generated_at_utc: publication.generated_at_utc.clone(),
            evaluated_at_utc: evaluated_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            maximum_age_seconds: authority.maximum_age.num_seconds(),
        });
    }
}

fn validate_contract_name(
    value: &str,
    authority: bool,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if value.trim().is_empty() || value != value.trim() || value.chars().any(char::is_control) {
        invalid(authority, "source_contracts", "contract name is empty or malformed", mismatches);
    }
}

fn validate_artifact(
    field: &str,
    artifact: &PublicationArtifactIdentity,
    authority: bool,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if let Err(reason) = portable_uri_error(&artifact.uri) {
        invalid(authority, &format!("{field}.uri"), reason, mismatches);
    }
    if !is_sha256(&artifact.sha256) {
        invalid(
            authority,
            &format!("{field}.sha256"),
            "value is not a nonzero SHA-256 digest",
            mismatches,
        );
    }
}

fn portable_uri_error(value: &str) -> Result<(), &'static str> {
    if value.trim().is_empty() || value != value.trim() || value.chars().any(char::is_control) {
        return Err("URI is empty or malformed");
    }
    let relative = value
        .strip_prefix("study://")
        .or_else(|| value.strip_prefix("repository://"))
        .ok_or("URI must use study:// or repository://")?;
    if relative.is_empty()
        || relative.starts_with('/')
        || relative.contains('\\')
        || relative.contains('?')
        || relative.contains('#')
        || relative
            .split('/')
            .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err("URI must contain a normalized relative path");
    }
    Ok(())
}

fn validate_nonempty_authority(
    field: &str,
    value: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if value.trim().is_empty() || value != value.trim() || value.chars().any(char::is_control) {
        invalid_authority(field, "value is empty or malformed", mismatches);
    }
}

fn validate_nonempty_publication(
    field: &str,
    value: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if value.trim().is_empty() || value != value.trim() || value.chars().any(char::is_control) {
        invalid_publication(field, "value is empty or malformed", mismatches);
    }
}

fn validate_git_authority(
    field: &str,
    value: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if !is_git_commit(value) {
        invalid_authority(field, "value is not a full nonzero Git commit", mismatches);
    }
}

fn validate_git_publication(
    field: &str,
    value: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if !is_git_commit(value) {
        invalid_publication(field, "value is not a full nonzero Git commit", mismatches);
    }
}

fn validate_sha256_authority(
    field: &str,
    value: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if !is_sha256(value) {
        invalid_authority(field, "value is not a nonzero SHA-256 digest", mismatches);
    }
}

fn validate_sha256_publication(
    field: &str,
    value: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if !is_sha256(value) {
        invalid_publication(field, "value is not a nonzero SHA-256 digest", mismatches);
    }
}

fn is_git_commit(value: &str) -> bool {
    is_hex_identity(value, 40)
}

fn is_sha256(value: &str) -> bool {
    is_hex_identity(value, 64)
}

fn is_hex_identity(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| byte.is_ascii_hexdigit())
        && !value.bytes().all(|byte| byte == b'0')
}

fn invalid(
    authority: bool,
    field: &str,
    reason: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    if authority {
        invalid_authority(field, reason, mismatches);
    } else {
        invalid_publication(field, reason, mismatches);
    }
}

fn invalid_authority(field: &str, reason: &str, mismatches: &mut Vec<PublicationIdentityMismatch>) {
    mismatches.push(PublicationIdentityMismatch::InvalidAuthority {
        field: field.to_string(),
        reason: reason.to_string(),
    });
}

fn invalid_publication(
    field: &str,
    reason: &str,
    mismatches: &mut Vec<PublicationIdentityMismatch>,
) {
    mismatches.push(PublicationIdentityMismatch::InvalidPublication {
        field: field.to_string(),
        reason: reason.to_string(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(uri: &str, digest: char) -> PublicationArtifactIdentity {
        PublicationArtifactIdentity { uri: uri.to_string(), sha256: digest.to_string().repeat(64) }
    }

    fn publication() -> ImmutablePublicationIdentity {
        ImmutablePublicationIdentity {
            schema_version: "bijux.container.report.v1".to_string(),
            repository_commit: "a".repeat(40),
            catalog_sha256: "b".repeat(64),
            source_contracts: BTreeMap::from([
                (
                    "catalog".to_string(),
                    artifact("repository://containers/contracts/catalog.toml", 'b'),
                ),
                ("data_index".to_string(), artifact("study://data/index.json", 'c')),
            ]),
            raw_evidence: artifact("study://evidence/raw/manifest.json", 'd'),
            generated_at_utc: "2026-08-04T12:00:00Z".to_string(),
        }
    }

    fn authority(publication: &ImmutablePublicationIdentity) -> PublicationAuthority {
        PublicationAuthority {
            schema_version: publication.schema_version.clone(),
            repository_commit: publication.repository_commit.clone(),
            catalog_sha256: publication.catalog_sha256.clone(),
            source_contracts: publication.source_contracts.clone(),
            raw_evidence: publication.raw_evidence.clone(),
            maximum_age: Duration::hours(24),
        }
    }

    fn evaluated_at() -> anyhow::Result<DateTime<Utc>> {
        Ok("2026-08-04T13:00:00Z".parse()?)
    }

    #[test]
    fn exact_current_identity_passes() -> anyhow::Result<()> {
        let publication = publication();
        let result =
            evaluate_publication_freshness(&publication, &authority(&publication), evaluated_at()?);
        assert!(result.is_current());
        assert!(result.mismatches.is_empty());
        assert_eq!(result.qualification_decision("e".repeat(64)).verdict, EvidenceVerdict::Passed);
        Ok(())
    }

    #[test]
    fn repository_catalog_and_schema_drift_are_reported_exactly() -> anyhow::Result<()> {
        let mut publication = publication();
        let authority = authority(&publication);
        publication.schema_version = "bijux.container.report.v0".to_string();
        publication.repository_commit = "e".repeat(40);
        publication.catalog_sha256 = "f".repeat(64);
        let result = evaluate_publication_freshness(&publication, &authority, evaluated_at()?);
        let fields = result
            .mismatches
            .iter()
            .filter_map(|mismatch| match mismatch {
                PublicationIdentityMismatch::ValueMismatch { field, .. } => Some(field.as_str()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(
            fields,
            BTreeSet::from(["catalog_sha256", "repository_commit", "schema_version"])
        );
        Ok(())
    }

    #[test]
    fn mutated_and_changed_source_contracts_are_rejected() -> anyhow::Result<()> {
        let mut publication = publication();
        let authority = authority(&publication);
        publication
            .source_contracts
            .get_mut("catalog")
            .ok_or_else(|| anyhow::anyhow!("fixture has no catalog contract"))?
            .sha256 = "f".repeat(64);
        publication.source_contracts.remove("data_index");
        publication.source_contracts.insert(
            "unreviewed".to_string(),
            artifact("repository://containers/contracts/unreviewed.json", '9'),
        );
        let result = evaluate_publication_freshness(&publication, &authority, evaluated_at()?);
        assert!(!result.is_current());
        assert!(result.mismatches.iter().any(|mismatch| matches!(
            mismatch,
            PublicationIdentityMismatch::ValueMismatch { field, .. }
                if field == "source_contracts.catalog.sha256"
        )));
        assert!(result.mismatches.iter().any(|mismatch| matches!(
            mismatch,
            PublicationIdentityMismatch::MissingSourceContract { contract }
                if contract == "data_index"
        )));
        assert!(result.mismatches.iter().any(|mismatch| matches!(
            mismatch,
            PublicationIdentityMismatch::UnexpectedSourceContract { contract }
                if contract == "unreviewed"
        )));
        Ok(())
    }

    #[test]
    fn raw_evidence_mutation_and_nonportable_paths_are_rejected() -> anyhow::Result<()> {
        let mut publication = publication();
        let authority = authority(&publication);
        publication.raw_evidence.sha256 = "f".repeat(64);
        publication.raw_evidence.uri = "/home/bijan/STUDY/raw.json".to_string();
        let result = evaluate_publication_freshness(&publication, &authority, evaluated_at()?);
        assert!(result.mismatches.iter().any(|mismatch| matches!(
            mismatch,
            PublicationIdentityMismatch::InvalidPublication { field, .. }
                if field == "raw_evidence.uri"
        )));
        assert!(result.mismatches.iter().any(|mismatch| matches!(
            mismatch,
            PublicationIdentityMismatch::ValueMismatch { field, .. }
                if field == "raw_evidence.sha256"
        )));
        Ok(())
    }

    #[test]
    fn expired_and_future_publications_are_rejected() -> anyhow::Result<()> {
        let mut publication = publication();
        let authority = authority(&publication);
        publication.generated_at_utc = "2026-08-01T00:00:00Z".to_string();
        let expired = evaluate_publication_freshness(&publication, &authority, evaluated_at()?);
        assert!(expired
            .mismatches
            .iter()
            .any(|mismatch| matches!(mismatch, PublicationIdentityMismatch::Expired { .. })));

        publication.generated_at_utc = "2026-08-05T00:00:00Z".to_string();
        let future = evaluate_publication_freshness(&publication, &authority, evaluated_at()?);
        assert!(future.mismatches.iter().any(|mismatch| matches!(
            mismatch,
            PublicationIdentityMismatch::GeneratedInFuture { .. }
        )));
        Ok(())
    }

    #[test]
    fn rejected_evaluation_preserves_reasons_for_qualification() -> anyhow::Result<()> {
        let mut publication = publication();
        let authority = authority(&publication);
        publication.catalog_sha256 = "f".repeat(64);
        let result = evaluate_publication_freshness(&publication, &authority, evaluated_at()?);
        let decision = result.qualification_decision("e".repeat(64));
        assert_eq!(decision.verdict, EvidenceVerdict::Failed);
        assert_eq!(decision.evidence_sha256, None);
        assert_eq!(decision.reasons.len(), result.mismatches.len());
        assert!(decision.reasons[0].contains("catalog_sha256"));
        Ok(())
    }
}
