use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use chrono::{DateTime, NaiveDate};

use super::model::{
    ContainerSecurityEvidencePublication, ImageSignatureDisposition, RuntimeAccessPolicy,
    SecuritySourceIdentity, ToolSecurityEvidence, VulnerabilityDatabaseStatus,
    VulnerabilitySeverity, CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1,
};
use crate::qualification::{EvidenceVerdict, SupportDisposition};

/// Validate a complete container security publication and its policy verdicts.
///
/// # Errors
/// Returns an error when identity, evidence, waiver, or verdict invariants are violated.
pub fn validate_security_evidence_publication(
    publication: &ContainerSecurityEvidencePublication,
) -> Result<()> {
    if publication.schema_version != CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1 {
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
    validate_artifact_uri(&publication.raw_evidence.uri, "raw evidence")?;
    validate_sha256(&publication.raw_evidence.sha256, "raw evidence")?;
    validate_nonempty(&publication.scanner.name, "scanner name")?;
    validate_nonempty(&publication.scanner.version, "scanner version")?;
    match publication.scanner.database.status {
        VulnerabilityDatabaseStatus::Current => {
            let updated_at = parse_database_timestamp(publication)?;
            validate_sha256(
                publication.scanner.database.sha256.as_deref().unwrap_or_default(),
                "scanner database",
            )?;
            let age = evaluated_at.signed_duration_since(updated_at);
            if age.num_seconds() < 0
                || age.num_days() > i64::from(publication.scanner.database.maximum_age_days)
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
                || age.num_days() <= i64::from(publication.scanner.database.maximum_age_days)
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
    match evidence.support_disposition {
        SupportDisposition::Unsupported => validate_unsupported(evidence),
        SupportDisposition::Retained => validate_retained(publication, evidence, evaluation_date),
    }
}

fn validate_unsupported(evidence: &ToolSecurityEvidence) -> Result<()> {
    if evidence.verdict != EvidenceVerdict::NotApplicable {
        bail!(
            "unsupported tool {} cannot receive security verdict {:?}",
            evidence.tool_id,
            evidence.verdict
        );
    }
    if evidence.image_sha256.is_some()
        || evidence.tool_contract_sha256.is_some()
        || evidence.build_evidence_id.is_some()
        || evidence.source_identity.is_some()
        || evidence.structural.is_some()
        || evidence.sbom.is_some()
        || evidence.vulnerabilities.is_some()
        || evidence.secret_scan.is_some()
        || evidence.runtime_policy.is_some()
        || !evidence.waivers.is_empty()
    {
        bail!(
            "unsupported tool {} must not carry passing-image security evidence",
            evidence.tool_id
        );
    }
    Ok(())
}

fn validate_retained(
    publication: &ContainerSecurityEvidencePublication,
    evidence: &ToolSecurityEvidence,
    evaluation_date: NaiveDate,
) -> Result<()> {
    let image_sha256 = required_digest(evidence.image_sha256.as_deref(), "image")?;
    required_digest(evidence.tool_contract_sha256.as_deref(), "tool contract")?;
    required_digest(evidence.build_evidence_id.as_deref(), "build evidence")?;
    validate_source_identity(evidence.source_identity.as_ref())?;

    let structural = evidence.structural.as_ref().ok_or_else(|| {
        anyhow::anyhow!("retained tool {} lacks structural evidence", evidence.tool_id)
    })?;
    validate_sha256(&structural.inspection_sha256, "structural inspection")?;
    validate_sha256(&structural.signature_evidence_sha256, "signature evidence")?;
    validate_unique_paths(&structural.suid_sgid_paths, "SUID/SGID")?;
    validate_unique_paths(&structural.world_writable_paths, "world-writable")?;

    let sbom = evidence
        .sbom
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("retained tool {} lacks SBOM evidence", evidence.tool_id))?;
    validate_sha256(&sbom.sha256, "SBOM")?;
    validate_nonempty(&sbom.generator, "SBOM generator")?;
    validate_nonempty(&sbom.generator_version, "SBOM generator version")?;
    validate_sha256(&sbom.source_image_sha256, "SBOM source image")?;
    if sbom.source_image_sha256 != image_sha256 {
        bail!("tool {} SBOM is bound to a different image", evidence.tool_id);
    }

    let vulnerabilities = evidence.vulnerabilities.as_ref().ok_or_else(|| {
        anyhow::anyhow!("retained tool {} lacks vulnerability evidence", evidence.tool_id)
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
        anyhow::anyhow!("retained tool {} lacks secret-scan evidence", evidence.tool_id)
    })?;
    validate_nonempty(&secret_scan.scanner, "secret scanner")?;
    validate_nonempty(&secret_scan.scanner_version, "secret scanner version")?;
    validate_sha256(&secret_scan.report_sha256, "secret scan report")?;

    let runtime = evidence.runtime_policy.as_ref().ok_or_else(|| {
        anyhow::anyhow!("retained tool {} lacks runtime security evidence", evidence.tool_id)
    })?;
    validate_sha256(&runtime.probe_sha256, "runtime security probe")?;

    if evidence.verdict == EvidenceVerdict::Passed {
        validate_passing_controls(publication, evidence)?;
    } else if evidence.verdict == EvidenceVerdict::NotApplicable {
        bail!("retained tool {} cannot have a not-applicable security verdict", evidence.tool_id);
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
        anyhow::anyhow!("retained tool {} lacks structural evidence", evidence.tool_id)
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
        anyhow::anyhow!("retained tool {} lacks secret-scan evidence", evidence.tool_id)
    })?;
    if secret_scan.finding_count != 0 {
        bail!("tool {} cannot pass with secret findings", evidence.tool_id);
    }
    let runtime = evidence.runtime_policy.as_ref().ok_or_else(|| {
        anyhow::anyhow!("retained tool {} lacks runtime security evidence", evidence.tool_id)
    })?;
    if runtime.network_policy != RuntimeAccessPolicy::Denied
        || runtime.writable_mount_policy != RuntimeAccessPolicy::Denied
        || runtime.non_root_uid == 0
        || !runtime.non_root_execution_passed
    {
        bail!("tool {} cannot pass incomplete runtime restrictions", evidence.tool_id);
    }
    let vulnerabilities = evidence.vulnerabilities.as_ref().ok_or_else(|| {
        anyhow::anyhow!("retained tool {} lacks vulnerability evidence", evidence.tool_id)
    })?;
    for finding in &vulnerabilities.findings {
        if finding.severity == VulnerabilitySeverity::Critical && finding.waiver_id.is_none() {
            bail!("tool {} has unwaived critical vulnerability {}", evidence.tool_id, finding.id);
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
        anyhow::anyhow!("retained tool {} lacks vulnerability evidence", evidence.tool_id)
    })?;
    let mut finding_ids = BTreeSet::new();
    let mut observed = [0_u64; 5];
    for finding in &vulnerabilities.findings {
        validate_nonempty(&finding.id, "vulnerability id")?;
        validate_nonempty(&finding.package, "vulnerability package")?;
        validate_nonempty(&finding.installed_version, "installed package version")?;
        if !finding_ids.insert((finding.id.as_str(), finding.package.as_str())) {
            bail!(
                "tool {} has duplicate vulnerability {} for {}",
                evidence.tool_id,
                finding.id,
                finding.package
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

    fn valid_publication() -> ContainerSecurityEvidencePublication {
        serde_json::from_value(serde_json::json!({
            "schema_version": "bijux.container.security_evidence.v1",
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
                    "support_disposition": "retained",
                    "image_sha256": "1".repeat(64),
                    "tool_contract_sha256": "2".repeat(64),
                    "build_evidence_id": "3".repeat(64),
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
                        "source_image_sha256": "1".repeat(64)
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
        }))
        .expect("valid security evidence fixture")
    }

    #[test]
    fn complete_security_evidence_passes() -> Result<()> {
        validate_security_evidence_publication(&valid_publication())
    }

    #[test]
    fn stale_database_cannot_inherit_a_passing_verdict() {
        let mut publication = valid_publication();
        publication.scanner.database.status = VulnerabilityDatabaseStatus::Stale;
        publication.scanner.database.updated_at_utc = Some("2026-06-01T00:00:00Z".to_string());
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("stale or missing")));
    }

    #[test]
    fn expired_waiver_cannot_pass() {
        let mut publication = valid_publication();
        publication.tools.get_mut("samtools").expect("samtools fixture").waivers[0].expires_on =
            "2026-08-03".to_string();
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("expired waiver")));
    }

    #[test]
    fn unwaived_critical_finding_cannot_pass() {
        let mut publication = valid_publication();
        let evidence = publication.tools.get_mut("samtools").expect("samtools fixture");
        evidence.vulnerabilities.as_mut().expect("vulnerability fixture").findings[0].waiver_id =
            None;
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("unwaived critical")));
    }

    #[test]
    fn unsupported_tool_cannot_receive_passing_evidence() {
        let mut publication = valid_publication();
        let evidence = publication.tools.get_mut("samtools").expect("samtools fixture");
        evidence.support_disposition = SupportDisposition::Unsupported;
        assert!(validate_security_evidence_publication(&publication)
            .is_err_and(|error| error.to_string().contains("cannot receive")));
    }

    #[test]
    fn unknown_fields_fail_deserialization() {
        let mut payload = serde_json::to_value(valid_publication()).expect("serialize fixture");
        payload["unowned"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ContainerSecurityEvidencePublication>(payload).is_err());
    }

    #[test]
    fn checked_security_evidence_schema_matches_rust_contract() -> Result<()> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .ok_or_else(|| anyhow::anyhow!("policies crate is outside the repository"))?;
        let path = root.join("containers/contracts/schemas/security-evidence.v1.json");
        let generated = serde_json::to_value(super::super::security_evidence_schema())?;
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let encoded = format!("{}\n", serde_json::to_string_pretty(&generated)?);
            std::fs::write(&path, encoded)?;
        }
        let checked: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        assert_eq!(checked, generated);
        Ok(())
    }
}
