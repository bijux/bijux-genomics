use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDate};

use super::model::{
    ContainerSecurityEvidencePublication, ImageSignatureDisposition, RuntimeAccessPolicy,
    SecuritySourceIdentity, ToolSecurityEvidence, VulnerabilityDatabaseStatus,
    VulnerabilitySeverity, CONTAINER_SECURITY_EVIDENCE_SCHEMA_V2,
};
use super::security_evidence_policy_sha256;
use crate::qualification::{
    ArtifactAvailability, ArtifactDelivery, EvidenceVerdict, ProductSupport,
};

/// Validate a complete container security publication and its policy verdicts.
///
/// # Errors
/// Returns an error when identity, evidence, waiver, or verdict invariants are violated.
pub fn validate_security_evidence_publication(
    publication: &ContainerSecurityEvidencePublication,
) -> Result<()> {
    if publication.schema_version != CONTAINER_SECURITY_EVIDENCE_SCHEMA_V2 {
        bail!("unsupported container security evidence schema: {}", publication.schema_version);
    }
    let evaluated_at =
        DateTime::parse_from_rfc3339(&publication.evaluated_at_utc).map_err(|error| {
            anyhow::anyhow!("security evidence evaluation timestamp is invalid: {error}")
        })?;
    validate_git_commit(&publication.repository_commit, "repository commit")?;
    validate_sha256(&publication.catalog_sha256, "catalog")?;
    validate_sha256(&publication.inventory_sha256, "inventory")?;
    validate_sha256(&publication.policy_sha256, "policy")?;
    if publication.policy_sha256 != security_evidence_policy_sha256() {
        bail!("security publication policy identity differs from the implemented policy");
    }
    validate_artifact_uri(&publication.raw_evidence.uri, "raw evidence")?;
    validate_sha256(&publication.raw_evidence.sha256, "raw evidence")?;
    validate_nonempty(&publication.scanner.name, "scanner name")?;
    validate_nonempty(&publication.scanner.version, "scanner version")?;
    if publication.scanner.database.maximum_age_days == 0
        || publication.scanner.database.maximum_age_days > 30
    {
        bail!("vulnerability database maximum age is outside policy bounds");
    }
    match publication.scanner.database.status {
        VulnerabilityDatabaseStatus::Current => {
            let updated_at = parse_database_timestamp(publication)?;
            validate_sha256(
                publication.scanner.database.sha256.as_deref().unwrap_or_default(),
                "scanner database",
            )?;
            let age = evaluated_at.signed_duration_since(updated_at);
            if age.num_seconds() < 0
                || age.num_seconds()
                    > i64::from(publication.scanner.database.maximum_age_days) * 86_400
            {
                bail!("vulnerability database marked current outside its maximum age");
            }
        }
        VulnerabilityDatabaseStatus::Stale => {
            let updated_at = parse_database_timestamp(publication)?;
            validate_sha256(
                publication.scanner.database.sha256.as_deref().unwrap_or_default(),
                "scanner database",
            )?;
            let age = evaluated_at.signed_duration_since(updated_at);
            if age.num_seconds() < 0
                || age.num_seconds()
                    <= i64::from(publication.scanner.database.maximum_age_days) * 86_400
            {
                bail!("vulnerability database marked stale while inside its maximum age");
            }
        }
        VulnerabilityDatabaseStatus::Missing => {
            if publication.scanner.database.updated_at_utc.is_some()
                || publication.scanner.database.sha256.is_some()
            {
                bail!("missing vulnerability database must not carry identity fields");
            }
        }
    }
    if publication.tools.is_empty() {
        bail!("security evidence publication contains no tools");
    }
    for (tool_id, evidence) in &publication.tools {
        if tool_id != &evidence.tool_id {
            bail!("security evidence key {tool_id} does not match tool {}", evidence.tool_id);
        }
        validate_tool(publication, evidence, evaluated_at.date_naive())?;
    }
    Ok(())
}

fn validate_tool(
    publication: &ContainerSecurityEvidencePublication,
    evidence: &ToolSecurityEvidence,
    evaluation_date: NaiveDate,
) -> Result<()> {
    validate_identifier(&evidence.tool_id, "tool id")?;
    validate_nonempty(&evidence.version, "tool version")?;
    validate_reasons(evidence)?;
    validate_support_dimensions(evidence)?;
    match (evidence.product_support, evidence.availability) {
        (ProductSupport::Supported, ArtifactAvailability::Admitted) => {
            validate_admitted(publication, evidence, evaluation_date)
        }
        (ProductSupport::Supported, _) => validate_supported_without_artifact(evidence),
        (ProductSupport::Retired | ProductSupport::Excluded, _) => validate_non_supported(evidence),
    }
}

fn validate_support_dimensions(evidence: &ToolSecurityEvidence) -> Result<()> {
    if evidence.redistribution_permitted
        != (evidence.artifact_delivery == ArtifactDelivery::RedistributableImage)
    {
        bail!("tool {} security delivery contradicts redistribution policy", evidence.tool_id);
    }
    if evidence.availability == ArtifactAvailability::BlockedExternalAccess
        && (evidence.product_support != ProductSupport::Supported
            || evidence.artifact_delivery == ArtifactDelivery::RedistributableImage)
    {
        bail!("tool {} has an invalid external-access blocker", evidence.tool_id);
    }
    if evidence.product_support != ProductSupport::Supported
        && evidence.availability != ArtifactAvailability::Unavailable
    {
        bail!("non-supported tool {} cannot carry artifact availability", evidence.tool_id);
    }
    Ok(())
}

fn validate_non_supported(evidence: &ToolSecurityEvidence) -> Result<()> {
    if evidence.verdict != EvidenceVerdict::NotApplicable {
        bail!(
            "non-supported tool {} cannot receive security verdict {:?}",
            evidence.tool_id,
            evidence.verdict
        );
    }
    validate_absent_image_evidence(evidence)
}

fn validate_supported_without_artifact(evidence: &ToolSecurityEvidence) -> Result<()> {
    let expected = match evidence.availability {
        ArtifactAvailability::BlockedExternalAccess => EvidenceVerdict::Blocked,
        ArtifactAvailability::Unavailable => EvidenceVerdict::Missing,
        ArtifactAvailability::Admitted => unreachable!("admitted evidence is validated separately"),
    };
    if evidence.verdict != expected {
        bail!(
            "supported tool {} with {:?} availability requires {:?} security verdict",
            evidence.tool_id,
            evidence.availability,
            expected
        );
    }
    validate_absent_image_evidence(evidence)
}

fn validate_absent_image_evidence(evidence: &ToolSecurityEvidence) -> Result<()> {
    if evidence.artifact_sha256.is_some()
        || evidence.tool_contract_sha256.is_some()
        || evidence.admission_evidence_sha256.is_some()
        || evidence.source_identity.is_some()
        || evidence.structural.is_some()
        || evidence.sbom.is_some()
        || evidence.vulnerabilities.is_some()
        || evidence.secret_scan.is_some()
        || evidence.runtime_policy.is_some()
        || !evidence.waivers.is_empty()
    {
        bail!(
            "tool {} without an admitted artifact must not carry artifact security evidence",
            evidence.tool_id
        );
    }
    Ok(())
}

fn validate_admitted(
    publication: &ContainerSecurityEvidencePublication,
    evidence: &ToolSecurityEvidence,
    evaluation_date: NaiveDate,
) -> Result<()> {
    let artifact_sha256 = required_digest(evidence.artifact_sha256.as_deref(), "artifact")?;
    required_digest(evidence.tool_contract_sha256.as_deref(), "tool contract")?;
    required_digest(evidence.admission_evidence_sha256.as_deref(), "artifact admission evidence")?;
    validate_source_identity(evidence.source_identity.as_ref())?;

    let structural = evidence.structural.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks structural evidence", evidence.tool_id)
    })?;
    validate_sha256(&structural.inspection_sha256, "structural inspection")?;
    validate_sha256(&structural.signature_evidence_sha256, "signature evidence")?;
    validate_unique_paths(&structural.suid_sgid_paths, "SUID/SGID")?;
    validate_unique_paths(&structural.world_writable_paths, "world-writable")?;

    let sbom = evidence
        .sbom
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("admitted tool {} lacks SBOM evidence", evidence.tool_id))?;
    validate_sha256(&sbom.sha256, "SBOM")?;
    validate_nonempty(&sbom.generator, "SBOM generator")?;
    validate_nonempty(&sbom.generator_version, "SBOM generator version")?;
    validate_sha256(&sbom.source_artifact_sha256, "SBOM source artifact")?;
    if sbom.source_artifact_sha256 != artifact_sha256 {
        bail!("tool {} SBOM is bound to a different artifact", evidence.tool_id);
    }

    let vulnerabilities = evidence.vulnerabilities.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks vulnerability evidence", evidence.tool_id)
    })?;
    validate_sha256(&vulnerabilities.database_sha256, "vulnerability database")?;
    validate_sha256(&vulnerabilities.report_sha256, "vulnerability report")?;
    if vulnerabilities.scanner != publication.scanner.name
        || vulnerabilities.scanner_version != publication.scanner.version
    {
        bail!("tool {} vulnerability scanner identity differs from publication", evidence.tool_id);
    }
    if publication.scanner.database.sha256.as_deref() != Some(&vulnerabilities.database_sha256) {
        bail!("tool {} vulnerability database differs from publication", evidence.tool_id);
    }
    validate_findings(evidence, evaluation_date)?;

    let secret_scan = evidence.secret_scan.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks secret-scan evidence", evidence.tool_id)
    })?;
    validate_nonempty(&secret_scan.scanner, "secret scanner")?;
    validate_nonempty(&secret_scan.scanner_version, "secret scanner version")?;
    validate_sha256(&secret_scan.report_sha256, "secret scan report")?;

    let runtime = evidence.runtime_policy.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks runtime security evidence", evidence.tool_id)
    })?;
    validate_sha256(&runtime.probe_sha256, "runtime security probe")?;

    if evidence.verdict == EvidenceVerdict::Passed {
        validate_passing_controls(publication, evidence)?;
    } else if evidence.verdict == EvidenceVerdict::NotApplicable {
        bail!("admitted tool {} cannot have a not-applicable security verdict", evidence.tool_id);
    }
    Ok(())
}

fn validate_passing_controls(
    publication: &ContainerSecurityEvidencePublication,
    evidence: &ToolSecurityEvidence,
) -> Result<()> {
    if publication.scanner.database.status != VulnerabilityDatabaseStatus::Current {
        bail!(
            "tool {} cannot pass with a stale or missing vulnerability database",
            evidence.tool_id
        );
    }
    let structural = evidence.structural.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks structural evidence", evidence.tool_id)
    })?;
    if !matches!(
        structural.signature,
        ImageSignatureDisposition::SignedVerified
            | ImageSignatureDisposition::UnsignedPolicyAccepted
    ) {
        bail!("tool {} cannot pass signature policy", evidence.tool_id);
    }
    if !structural.suid_sgid_paths.is_empty() || !structural.world_writable_paths.is_empty() {
        bail!("tool {} cannot pass with forbidden filesystem paths", evidence.tool_id);
    }
    let secret_scan = evidence.secret_scan.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks secret-scan evidence", evidence.tool_id)
    })?;
    if secret_scan.finding_count != 0 {
        bail!("tool {} cannot pass with secret findings", evidence.tool_id);
    }
    let runtime = evidence.runtime_policy.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks runtime security evidence", evidence.tool_id)
    })?;
    if runtime.network_policy != RuntimeAccessPolicy::Denied
        || runtime.writable_mount_policy != RuntimeAccessPolicy::Denied
        || runtime.non_root_uid == 0
        || !runtime.non_root_execution_passed
    {
        bail!("tool {} cannot pass incomplete runtime restrictions", evidence.tool_id);
    }
    let vulnerabilities = evidence.vulnerabilities.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks vulnerability evidence", evidence.tool_id)
    })?;
    for finding in &vulnerabilities.findings {
        if finding.waiver_id.is_none() {
            match finding.severity {
                VulnerabilitySeverity::Critical => bail!(
                    "tool {} has unwaived critical vulnerability {}",
                    evidence.tool_id,
                    finding.id
                ),
                VulnerabilitySeverity::High => bail!(
                    "tool {} has unwaived high vulnerability {}",
                    evidence.tool_id,
                    finding.id
                ),
                VulnerabilitySeverity::Medium
                | VulnerabilitySeverity::Low
                | VulnerabilitySeverity::Unknown => {}
            }
        }
    }
    Ok(())
}

fn validate_findings(evidence: &ToolSecurityEvidence, evaluation_date: NaiveDate) -> Result<()> {
    let mut waivers = BTreeMap::new();
    for waiver in &evidence.waivers {
        validate_identifier(&waiver.id, "waiver id")?;
        validate_nonempty(&waiver.vulnerability_id, "waiver vulnerability id")?;
        validate_nonempty(&waiver.rationale, "waiver rationale")?;
        let expires_on = parse_date(&waiver.expires_on, "waiver expiry")?;
        if expires_on < evaluation_date {
            bail!("tool {} has expired waiver {}", evidence.tool_id, waiver.id);
        }
        validate_sha256(&waiver.approval_sha256, "waiver approval")?;
        if waivers.insert(waiver.id.as_str(), waiver).is_some() {
            bail!("tool {} has duplicate waiver {}", evidence.tool_id, waiver.id);
        }
    }
    let vulnerabilities = evidence.vulnerabilities.as_ref().ok_or_else(|| {
        anyhow::anyhow!("admitted tool {} lacks vulnerability evidence", evidence.tool_id)
    })?;
    let mut finding_ids = BTreeSet::new();
    let mut observed = [0_u64; 5];
    for finding in &vulnerabilities.findings {
        validate_nonempty(&finding.id, "vulnerability id")?;
        validate_nonempty(&finding.package, "vulnerability package")?;
        validate_nonempty(&finding.installed_version, "installed package version")?;
        if !finding_ids.insert((
            finding.id.as_str(),
            finding.package.as_str(),
            finding.installed_version.as_str(),
        )) {
            bail!(
                "tool {} has duplicate vulnerability {} for {} version {}",
                evidence.tool_id,
                finding.id,
                finding.package,
                finding.installed_version
            );
        }
        observed[severity_index(finding.severity)] += 1;
        if let Some(waiver_id) = &finding.waiver_id {
            let waiver = waivers.get(waiver_id.as_str()).ok_or_else(|| {
                anyhow::anyhow!(
                    "tool {} finding {} references unknown waiver {waiver_id}",
                    evidence.tool_id,
                    finding.id
                )
            })?;
            if waiver.vulnerability_id != finding.id {
                bail!(
                    "tool {} waiver {waiver_id} belongs to a different vulnerability",
                    evidence.tool_id
                );
            }
        }
    }
    let counts = vulnerabilities.severity_counts;
    let declared = [counts.critical, counts.high, counts.medium, counts.low, counts.unknown];
    if observed != declared {
        bail!("tool {} vulnerability severity counts do not match findings", evidence.tool_id);
    }
    Ok(())
}

const fn severity_index(severity: VulnerabilitySeverity) -> usize {
    match severity {
        VulnerabilitySeverity::Critical => 0,
        VulnerabilitySeverity::High => 1,
        VulnerabilitySeverity::Medium => 2,
        VulnerabilitySeverity::Low => 3,
        VulnerabilitySeverity::Unknown => 4,
    }
}

fn validate_reasons(evidence: &ToolSecurityEvidence) -> Result<()> {
    for reason in &evidence.reasons {
        validate_nonempty(reason, "security verdict reason")?;
    }
    match evidence.verdict {
        EvidenceVerdict::Passed | EvidenceVerdict::NotApplicable => {
            if !evidence.reasons.is_empty() {
                bail!(
                    "tool {} verdict {:?} must not carry failure reasons",
                    evidence.tool_id,
                    evidence.verdict
                );
            }
        }
        EvidenceVerdict::Failed | EvidenceVerdict::Blocked | EvidenceVerdict::Missing => {
            if evidence.reasons.is_empty() {
                bail!("tool {} verdict {:?} requires reasons", evidence.tool_id, evidence.verdict);
            }
        }
    }
    Ok(())
}

fn validate_source_identity(identity: Option<&SecuritySourceIdentity>) -> Result<()> {
    match identity {
        Some(SecuritySourceIdentity::SourceSha256(value)) => validate_sha256(value, "source"),
        Some(SecuritySourceIdentity::PinnedCommit(value)) => {
            validate_git_commit(value, "pinned source")
        }
        None => bail!("retained security evidence has no source identity"),
    }
}

fn required_digest<'a>(value: Option<&'a str>, label: &str) -> Result<&'a str> {
    let value =
        value.ok_or_else(|| anyhow::anyhow!("retained security evidence lacks {label} digest"))?;
    validate_sha256(value, label)?;
    Ok(value)
}

fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("security evidence {label} is not a SHA-256 digest: {value}");
    }
    Ok(())
}

fn validate_git_commit(value: &str, label: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().all(|byte| byte == b'0')
    {
        bail!("security evidence {label} is not a full Git commit: {value}");
    }
    Ok(())
}

fn validate_identifier(value: &str, label: &str) -> Result<()> {
    validate_nonempty(value, label)?;
    if !value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        bail!("security evidence {label} is not lowercase snake case: {value}");
    }
    Ok(())
}

fn validate_nonempty(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() || value != value.trim() || value.chars().any(char::is_control) {
        bail!("security evidence {label} is empty or malformed");
    }
    Ok(())
}

fn validate_artifact_uri(value: &str, label: &str) -> Result<()> {
    validate_nonempty(value, label)?;
    if !value.starts_with("study://") && !value.starts_with("repository://") {
        bail!("security evidence {label} must use a portable governed URI");
    }
    Ok(())
}

fn validate_unique_paths(paths: &[String], label: &str) -> Result<()> {
    let mut unique = BTreeSet::new();
    for path in paths {
        if !path.starts_with('/') || path.contains('\n') || path.contains('\r') {
            bail!("security evidence {label} path is not an absolute safe path: {path}");
        }
        if !unique.insert(path) {
            bail!("security evidence contains duplicate {label} path: {path}");
        }
    }
    Ok(())
}

fn parse_date(value: &str, label: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|error| anyhow::anyhow!("security evidence {label} must use YYYY-MM-DD: {error}"))
}

fn parse_database_timestamp(
    publication: &ContainerSecurityEvidencePublication,
) -> Result<DateTime<chrono::FixedOffset>> {
    let value = publication.scanner.database.updated_at_utc.as_deref().unwrap_or_default();
    DateTime::parse_from_rfc3339(value)
        .map_err(|error| anyhow::anyhow!("scanner database update timestamp is invalid: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_publication() -> Result<ContainerSecurityEvidencePublication> {
        let mut publication: ContainerSecurityEvidencePublication =
            serde_json::from_value(serde_json::json!({
            "schema_version": "bijux.container.security_evidence.v2",
            "evaluated_at_utc": "2026-08-04T00:00:00Z",
            "repository_commit": "a".repeat(40),
            "catalog_sha256": "b".repeat(64),
            "inventory_sha256": "c".repeat(64),
            "policy_sha256": "d".repeat(64),
            "raw_evidence": {
                "uri": "study://evidence/security/raw.json",
                "sha256": "e".repeat(64)
            },
            "scanner": {
                "name": "grype",
                "version": "0.104.0",
                "database": {
                    "status": "current",
                    "updated_at_utc": "2026-08-04T00:00:00Z",
                    "sha256": "f".repeat(64),
                    "maximum_age_days": 30
                }
            },
            "tools": {
                "samtools": {
                    "tool_id": "samtools",
                    "version": "1.20",
                    "product_support": "supported",
                    "artifact_delivery": "redistributable_image",
                    "redistribution_permitted": true,
                    "availability": "admitted",
                    "artifact_sha256": "1".repeat(64),
                    "tool_contract_sha256": "2".repeat(64),
                    "admission_evidence_sha256": "3".repeat(64),
                    "source_identity": {"kind": "source_sha256", "value": "4".repeat(64)},
                    "structural": {
                        "inspection_sha256": "5".repeat(64),
                        "signature": "unsigned_policy_accepted",
                        "signature_evidence_sha256": "6".repeat(64),
                        "suid_sgid_paths": [],
                        "world_writable_paths": []
                    },
                    "sbom": {
                        "format": "cyclone_dx_json",
                        "sha256": "7".repeat(64),
                        "generator": "syft",
                        "generator_version": "1.29.0",
                        "source_artifact_sha256": "1".repeat(64)
                    },
                    "vulnerabilities": {
                        "scanner": "grype",
                        "scanner_version": "0.104.0",
                        "database_sha256": "f".repeat(64),
                        "report_sha256": "8".repeat(64),
                        "severity_counts": {
                            "critical": 1,
                            "high": 0,
                            "medium": 0,
                            "low": 0,
                            "unknown": 0
                        },
                        "findings": [{
                            "id": "CVE-2026-0001",
                            "severity": "critical",
                            "package": "libexample",
                            "installed_version": "1.0",
                            "fixed_versions": [],
                            "waiver_id": "accepted_cve_2026_0001"
                        }]
                    },
                    "secret_scan": {
                        "scanner": "gitleaks",
                        "scanner_version": "8.24.0",
                        "report_sha256": "9".repeat(64),
                        "finding_count": 0
                    },
                    "runtime_policy": {
                        "network_policy": "denied",
                        "writable_mount_policy": "denied",
                        "non_root_uid": 65534,
                        "non_root_execution_passed": true,
                        "probe_sha256": "a".repeat(64)
                    },
                    "waivers": [{
                        "id": "accepted_cve_2026_0001",
                        "vulnerability_id": "CVE-2026-0001",
                        "rationale": "No reachable vulnerable code path in the declared interface.",
                        "expires_on": "2026-09-01",
                        "approval_sha256": "b".repeat(64)
                    }],
                    "verdict": "passed",
                    "reasons": []
                }
            }
            }))?;
        publication.policy_sha256 = security_evidence_policy_sha256();
        Ok(publication)
    }

    #[test]
    fn complete_security_evidence_passes() -> Result<()> {
        validate_security_evidence_publication(&valid_publication()?)
    }

    #[test]
    fn publication_cannot_claim_a_different_policy_identity() -> Result<()> {
        let mut publication = valid_publication()?;
        publication.policy_sha256 = "d".repeat(64);
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("implemented policy")));
        Ok(())
    }

    #[test]
    fn exact_security_publication_admits_an_artifact_bound_pass() -> Result<()> {
        let publication = valid_publication()?;
        let admitted = super::super::admission::admit_security_evidence(
            &publication,
            &"7".repeat(64),
            &"a".repeat(40),
            &"b".repeat(64),
            &"c".repeat(64),
        )?;
        let decision = admitted.tool_decision("samtools", &"1".repeat(64));
        assert_eq!(decision.verdict, EvidenceVerdict::Passed);
        assert_eq!(decision.evidence_sha256, Some("7".repeat(64)));

        let stale_artifact = admitted.tool_decision("samtools", &"2".repeat(64));
        assert_eq!(stale_artifact.verdict, EvidenceVerdict::Failed);
        assert!(stale_artifact.audit.reason.contains("different artifact"));
        Ok(())
    }

    #[test]
    fn security_admission_rejects_authority_drift() -> Result<()> {
        let publication = valid_publication()?;
        let result = super::super::admission::admit_security_evidence(
            &publication,
            &"7".repeat(64),
            &"0".repeat(40),
            &"b".repeat(64),
            &"c".repeat(64),
        );
        assert!(result.is_err_and(|error| error.to_string().contains("repository commit")));
        Ok(())
    }

    #[test]
    fn stale_database_cannot_inherit_a_passing_verdict() -> Result<()> {
        let mut publication = valid_publication()?;
        publication.scanner.database.status = VulnerabilityDatabaseStatus::Stale;
        publication.scanner.database.updated_at_utc = Some("2026-06-01T00:00:00Z".to_string());
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("stale or missing")));
        Ok(())
    }

    #[test]
    fn database_age_must_not_round_down_to_whole_days() -> Result<()> {
        let mut publication = valid_publication()?;
        publication.scanner.database.maximum_age_days = 30;
        publication.scanner.database.updated_at_utc = Some("2026-07-04T23:59:59Z".to_string());
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("outside its maximum age")));
        Ok(())
    }

    #[test]
    fn expired_waiver_cannot_pass() -> Result<()> {
        let mut publication = valid_publication()?;
        let Some(evidence) = publication.tools.get_mut("samtools") else {
            anyhow::bail!("security fixture lacks samtools evidence");
        };
        evidence.waivers[0].expires_on = "2026-08-03".to_string();
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("expired waiver")));
        Ok(())
    }

    #[test]
    fn unwaived_critical_finding_cannot_pass() -> Result<()> {
        let mut publication = valid_publication()?;
        let Some(evidence) = publication.tools.get_mut("samtools") else {
            anyhow::bail!("security fixture lacks samtools evidence");
        };
        let Some(vulnerabilities) = evidence.vulnerabilities.as_mut() else {
            anyhow::bail!("security fixture lacks vulnerability evidence");
        };
        vulnerabilities.findings[0].waiver_id = None;
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("unwaived critical")));
        Ok(())
    }

    #[test]
    fn unwaived_high_finding_cannot_pass() -> Result<()> {
        let mut publication = valid_publication()?;
        let Some(evidence) = publication.tools.get_mut("samtools") else {
            anyhow::bail!("security fixture lacks samtools evidence");
        };
        let Some(vulnerabilities) = evidence.vulnerabilities.as_mut() else {
            anyhow::bail!("security fixture lacks vulnerability evidence");
        };
        vulnerabilities.findings[0].severity = VulnerabilitySeverity::High;
        vulnerabilities.findings[0].waiver_id = None;
        vulnerabilities.severity_counts.critical = 0;
        vulnerabilities.severity_counts.high = 1;
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("unwaived high")));
        Ok(())
    }

    #[test]
    fn non_supported_tool_cannot_receive_passing_evidence() -> Result<()> {
        let mut publication = valid_publication()?;
        let Some(evidence) = publication.tools.get_mut("samtools") else {
            anyhow::bail!("security fixture lacks samtools evidence");
        };
        evidence.product_support = ProductSupport::Retired;
        evidence.redistribution_permitted = false;
        evidence.artifact_delivery = ArtifactDelivery::ExternalArtifact;
        evidence.availability = ArtifactAvailability::Unavailable;
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("cannot receive")));
        Ok(())
    }

    #[test]
    fn supported_external_blocker_cannot_be_not_applicable() -> Result<()> {
        let mut publication = valid_publication()?;
        let Some(evidence) = publication.tools.get_mut("samtools") else {
            anyhow::bail!("security fixture lacks samtools evidence");
        };
        evidence.product_support = ProductSupport::Supported;
        evidence.artifact_delivery = ArtifactDelivery::ExternalArtifact;
        evidence.redistribution_permitted = false;
        evidence.availability = ArtifactAvailability::BlockedExternalAccess;
        evidence.artifact_sha256 = None;
        evidence.tool_contract_sha256 = None;
        evidence.admission_evidence_sha256 = None;
        evidence.source_identity = None;
        evidence.structural = None;
        evidence.sbom = None;
        evidence.vulnerabilities = None;
        evidence.secret_scan = None;
        evidence.runtime_policy = None;
        evidence.waivers.clear();
        evidence.verdict = EvidenceVerdict::NotApplicable;
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("requires Blocked")));
        Ok(())
    }

    #[test]
    fn unknown_fields_fail_deserialization() -> Result<()> {
        let mut payload = serde_json::to_value(valid_publication()?)?;
        payload["unowned"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ContainerSecurityEvidencePublication>(payload).is_err());
        Ok(())
    }

    #[test]
    fn checked_security_evidence_schema_matches_rust_contract() -> Result<()> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| anyhow::anyhow!("policies crate is outside the repository"))?;
        let path = root.join("containers/contracts/schemas/security-evidence.v2.json");
        let generated = serde_json::to_value(super::super::security_evidence_schema())?;
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let encoded = format!("{}\n", serde_json::to_string_pretty(&generated)?);
            std::fs::write(&path, encoded)?;
        }
        let checked: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        assert_eq!(checked, generated);
        Ok(())
    }

    #[test]
    fn historical_security_evidence_schema_remains_v1() -> Result<()> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| anyhow::anyhow!("policies crate is outside the repository"))?;
        let historical: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
            root.join("containers/contracts/schemas/security-evidence.v1.json"),
        )?)?;
        assert_eq!(
            historical
                .pointer("/properties/schema_version/const")
                .and_then(serde_json::Value::as_str),
            Some(super::super::CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1)
        );
        let tool = historical
            .pointer("/$defs/ToolSecurityEvidence/properties")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| anyhow::anyhow!("historical security schema omits tool properties"))?;
        assert!(tool.contains_key("support_disposition"));
        assert!(!tool.contains_key("product_support"));
        Ok(())
    }
}
