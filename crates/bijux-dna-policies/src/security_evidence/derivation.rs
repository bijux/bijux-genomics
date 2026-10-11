use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use chrono::DateTime;
use sha2::{Digest, Sha256};

use super::{
    parse_gitleaks_report, parse_grype_vulnerability_report,
    validate_security_evidence_publication, ContainerSecurityEvidencePublication,
    ImageSignatureDisposition, RuntimeAccessPolicy, RuntimeSecurityEvidence, SbomEvidence,
    SbomFormat, SecretScanEvidence, SecurityArtifactIdentity, SecuritySourceIdentity,
    StructuralSecurityEvidence, ToolSecurityEvidence, VulnerabilityDatabaseStatus,
    VulnerabilityEvidence, VulnerabilityScannerIdentity, VulnerabilitySeverity,
    VulnerabilityWaiver, CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1,
};
use crate::qualification::{EvidenceVerdict, SupportDisposition};

#[derive(Debug, Clone)]
pub struct SecurityEvidenceDerivationRequest {
    pub evaluated_at_utc: String,
    pub repository_commit: String,
    pub catalog_sha256: String,
    pub inventory_sha256: String,
    pub raw_evidence: SecurityArtifactIdentity,
    pub scanner: VulnerabilityScannerIdentity,
    pub sbom_generator_version: String,
    pub secret_scanner_version: String,
    pub tools: BTreeMap<String, SecurityToolObservation>,
}

const SECURITY_EVIDENCE_POLICY_V1: &[u8] = br#"{"database":"current","runtime":{"network":"denied","non_root_uid":"greater_than_zero","system":"read_only"},"secrets":{"maximum_findings":0},"signature":["signed_verified","unsigned_policy_accepted"],"structural":{"suid_sgid_paths":0,"world_writable_paths":0},"vulnerabilities":{"maximum_unwaived_critical":0,"maximum_unwaived_high":0}}"#;

/// Return the immutable identity of the policy implemented by this derivation.
#[must_use]
pub fn security_evidence_policy_sha256() -> String {
    sha256_hex(SECURITY_EVIDENCE_POLICY_V1)
}

/// Classify one immutable vulnerability database relative to the evidence evaluation time.
///
/// # Errors
/// Returns an error for malformed timestamps, impossible future identities, or invalid policy
/// bounds.
pub fn classify_vulnerability_database(
    evaluated_at_utc: &str,
    updated_at_utc: &str,
    sha256: String,
    maximum_age_days: u16,
) -> Result<super::VulnerabilityDatabaseIdentity> {
    let evaluated_at = DateTime::parse_from_rfc3339(evaluated_at_utc)?;
    let updated_at = DateTime::parse_from_rfc3339(updated_at_utc)?;
    if maximum_age_days == 0 || maximum_age_days > 30 || updated_at > evaluated_at {
        bail!("vulnerability database classification identity is invalid");
    }
    let maximum_age_seconds = i64::from(maximum_age_days) * 86_400;
    let status =
        if evaluated_at.signed_duration_since(updated_at).num_seconds() <= maximum_age_seconds {
            VulnerabilityDatabaseStatus::Current
        } else {
            VulnerabilityDatabaseStatus::Stale
        };
    Ok(super::VulnerabilityDatabaseIdentity {
        status,
        updated_at_utc: Some(updated_at_utc.to_string()),
        sha256: Some(sha256),
        maximum_age_days,
    })
}

#[derive(Debug, Clone)]
pub enum SecurityToolObservation {
    Unsupported { tool_id: String, version: String },
    Collected(Box<CollectedSecurityToolObservation>),
}

#[derive(Debug, Clone)]
pub struct CollectedSecurityToolObservation {
    pub tool_id: String,
    pub version: String,
    pub image_sha256: String,
    pub tool_contract_sha256: String,
    pub build_evidence_id: String,
    pub source_identity: SecuritySourceIdentity,
    pub structural: StructuralProbeObservation,
    pub sbom_sha256: String,
    pub vulnerability_report: ScannerReportObservation,
    pub secret_report: ScannerReportObservation,
    pub runtime: RuntimeProbeObservation,
    pub waivers: Vec<VulnerabilityWaiver>,
}

#[derive(Debug, Clone)]
pub struct StructuralProbeObservation {
    pub inspection_sha256: String,
    pub signature_evidence_sha256: String,
    pub signature: ImageSignatureDisposition,
    pub suid_sgid_paths: Vec<String>,
    pub world_writable_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ScannerReportObservation {
    pub sha256: String,
    pub raw_json: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct RuntimeProbeObservation {
    pub probe_sha256: String,
    pub network_isolated: bool,
    pub system_read_only: bool,
    pub non_root_probe_passed: bool,
    pub observed_uid: Option<u32>,
}

/// Derive one complete policy-owned security publication from effect-free observations.
///
/// # Errors
/// Returns an error for incomplete identities, malformed scanner reports, ambiguous waivers, or a
/// derived publication that fails the policy-owned evidence contract.
pub fn derive_security_evidence(
    request: SecurityEvidenceDerivationRequest,
) -> Result<ContainerSecurityEvidencePublication> {
    let mut tools = BTreeMap::new();
    for (tool_id, observation) in request.tools {
        let evidence = match observation {
            SecurityToolObservation::Unsupported { tool_id: observed, version } => {
                if observed != tool_id {
                    bail!("security observation key {tool_id} differs from tool {observed}");
                }
                unsupported_evidence(observed, version)
            }
            SecurityToolObservation::Collected(observed) => {
                if observed.tool_id != tool_id {
                    bail!(
                        "security observation key {tool_id} differs from tool {}",
                        observed.tool_id
                    );
                }
                collected_evidence(
                    *observed,
                    &request.scanner,
                    &request.sbom_generator_version,
                    &request.secret_scanner_version,
                )?
            }
        };
        tools.insert(tool_id, evidence);
    }
    let publication = ContainerSecurityEvidencePublication {
        schema_version: CONTAINER_SECURITY_EVIDENCE_SCHEMA_V1.to_string(),
        evaluated_at_utc: request.evaluated_at_utc,
        repository_commit: request.repository_commit,
        catalog_sha256: request.catalog_sha256,
        inventory_sha256: request.inventory_sha256,
        policy_sha256: security_evidence_policy_sha256(),
        raw_evidence: request.raw_evidence,
        scanner: request.scanner,
        tools,
    };
    validate_security_evidence_publication(&publication)?;
    Ok(publication)
}

fn unsupported_evidence(tool_id: String, version: String) -> ToolSecurityEvidence {
    ToolSecurityEvidence {
        tool_id,
        version,
        support_disposition: SupportDisposition::Unsupported,
        image_sha256: None,
        tool_contract_sha256: None,
        build_evidence_id: None,
        source_identity: None,
        structural: None,
        sbom: None,
        vulnerabilities: None,
        secret_scan: None,
        runtime_policy: None,
        waivers: Vec::new(),
        verdict: EvidenceVerdict::NotApplicable,
        reasons: Vec::new(),
    }
}

fn collected_evidence(
    observation: CollectedSecurityToolObservation,
    scanner: &VulnerabilityScannerIdentity,
    sbom_generator_version: &str,
    secret_scanner_version: &str,
) -> Result<ToolSecurityEvidence> {
    if scanner.name != "grype" {
        bail!("security derivation requires the Grype vulnerability scanner");
    }
    verify_report_digest("vulnerability", &observation.vulnerability_report)?;
    verify_report_digest("secret", &observation.secret_report)?;
    let mut parsed_vulnerabilities =
        parse_grype_vulnerability_report(&observation.vulnerability_report.raw_json)?;
    apply_waivers(&mut parsed_vulnerabilities.findings, &observation.waivers)?;
    let parsed_secrets = parse_gitleaks_report(&observation.secret_report.raw_json)?;
    let mut reasons = Vec::new();
    if scanner.database.status != VulnerabilityDatabaseStatus::Current {
        reasons.push("vulnerability database is not current".to_string());
    }
    if !matches!(
        observation.structural.signature,
        ImageSignatureDisposition::SignedVerified
            | ImageSignatureDisposition::UnsignedPolicyAccepted
    ) {
        reasons.push("image signature is not verified".to_string());
    }
    if !observation.structural.suid_sgid_paths.is_empty() {
        reasons.push("image contains SUID or SGID paths".to_string());
    }
    if !observation.structural.world_writable_paths.is_empty() {
        reasons.push("image contains world-writable paths".to_string());
    }
    if parsed_secrets.finding_count != 0 {
        reasons.push("secret scanner reported findings".to_string());
    }
    if !observation.runtime.network_isolated {
        reasons.push("runtime network isolation probe failed".to_string());
    }
    if !observation.runtime.system_read_only {
        reasons.push("runtime system read-only probe failed".to_string());
    }
    let non_root_uid = observation.runtime.observed_uid.unwrap_or(0);
    if !observation.runtime.non_root_probe_passed || non_root_uid == 0 {
        reasons.push("runtime non-root probe failed".to_string());
    }
    if parsed_vulnerabilities.findings.iter().any(|finding| {
        matches!(finding.severity, VulnerabilitySeverity::Critical | VulnerabilitySeverity::High)
            && finding.waiver_id.is_none()
    }) {
        reasons
            .push("vulnerability report contains unwaived critical or high findings".to_string());
    }
    reasons.sort();
    reasons.dedup();
    let verdict =
        if reasons.is_empty() { EvidenceVerdict::Passed } else { EvidenceVerdict::Failed };
    Ok(ToolSecurityEvidence {
        tool_id: observation.tool_id,
        version: observation.version,
        support_disposition: SupportDisposition::Retained,
        image_sha256: Some(observation.image_sha256.clone()),
        tool_contract_sha256: Some(observation.tool_contract_sha256),
        build_evidence_id: Some(observation.build_evidence_id),
        source_identity: Some(observation.source_identity),
        structural: Some(StructuralSecurityEvidence {
            inspection_sha256: observation.structural.inspection_sha256,
            signature: observation.structural.signature,
            signature_evidence_sha256: observation.structural.signature_evidence_sha256,
            suid_sgid_paths: observation.structural.suid_sgid_paths,
            world_writable_paths: observation.structural.world_writable_paths,
        }),
        sbom: Some(SbomEvidence {
            format: SbomFormat::CycloneDxJson,
            sha256: observation.sbom_sha256,
            generator: "syft".to_string(),
            generator_version: sbom_generator_version.to_string(),
            source_image_sha256: observation.image_sha256,
        }),
        vulnerabilities: Some(VulnerabilityEvidence {
            scanner: scanner.name.clone(),
            scanner_version: scanner.version.clone(),
            database_sha256: scanner.database.sha256.clone().unwrap_or_default(),
            report_sha256: observation.vulnerability_report.sha256,
            severity_counts: parsed_vulnerabilities.severity_counts,
            findings: parsed_vulnerabilities.findings,
        }),
        secret_scan: Some(SecretScanEvidence {
            scanner: "gitleaks".to_string(),
            scanner_version: secret_scanner_version.to_string(),
            report_sha256: observation.secret_report.sha256,
            finding_count: parsed_secrets.finding_count,
        }),
        runtime_policy: Some(RuntimeSecurityEvidence {
            network_policy: access_policy(observation.runtime.network_isolated),
            writable_mount_policy: access_policy(observation.runtime.system_read_only),
            non_root_uid,
            non_root_execution_passed: observation.runtime.non_root_probe_passed,
            probe_sha256: observation.runtime.probe_sha256,
        }),
        waivers: observation.waivers,
        verdict,
        reasons,
    })
}

fn verify_report_digest(label: &str, report: &ScannerReportObservation) -> Result<()> {
    let observed = sha256_hex(&report.raw_json);
    if observed != report.sha256 {
        bail!("{label} report digest differs from the parsed bytes");
    }
    Ok(())
}

fn sha256_hex(raw: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(raw);
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn apply_waivers(
    findings: &mut [super::VulnerabilityFinding],
    waivers: &[VulnerabilityWaiver],
) -> Result<()> {
    let mut waiver_by_vulnerability = BTreeMap::new();
    for waiver in waivers {
        if waiver_by_vulnerability
            .insert(waiver.vulnerability_id.as_str(), waiver.id.as_str())
            .is_some()
        {
            bail!("multiple waivers target vulnerability {}", waiver.vulnerability_id);
        }
    }
    let mut matched_vulnerabilities = BTreeSet::new();
    for finding in findings {
        finding.waiver_id = waiver_by_vulnerability.get(finding.id.as_str()).map(|waiver_id| {
            matched_vulnerabilities.insert(finding.id.clone());
            (*waiver_id).to_string()
        });
    }
    for vulnerability_id in waiver_by_vulnerability.keys() {
        if !matched_vulnerabilities.contains(*vulnerability_id) {
            bail!("waiver targets absent vulnerability {vulnerability_id}");
        }
    }
    Ok(())
}

const fn access_policy(denied: bool) -> RuntimeAccessPolicy {
    if denied {
        RuntimeAccessPolicy::Denied
    } else {
        RuntimeAccessPolicy::Allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(value: char) -> String {
        std::iter::repeat_n(value, 64).collect()
    }

    fn collected() -> CollectedSecurityToolObservation {
        CollectedSecurityToolObservation {
            tool_id: "samtools".to_string(),
            version: "1.20".to_string(),
            image_sha256: digest('1'),
            tool_contract_sha256: digest('2'),
            build_evidence_id: digest('3'),
            source_identity: SecuritySourceIdentity::SourceSha256(digest('4')),
            structural: StructuralProbeObservation {
                inspection_sha256: digest('5'),
                signature_evidence_sha256: digest('6'),
                signature: ImageSignatureDisposition::SignedVerified,
                suid_sgid_paths: Vec::new(),
                world_writable_paths: Vec::new(),
            },
            sbom_sha256: digest('7'),
            vulnerability_report: report(br#"{"matches":[]}"#),
            secret_report: report(b"[]"),
            runtime: RuntimeProbeObservation {
                probe_sha256: digest('a'),
                network_isolated: true,
                system_read_only: true,
                non_root_probe_passed: true,
                observed_uid: Some(65_534),
            },
            waivers: Vec::new(),
        }
    }

    fn report(raw: &[u8]) -> ScannerReportObservation {
        ScannerReportObservation { sha256: sha256_hex(raw), raw_json: raw.to_vec() }
    }

    fn request(observation: CollectedSecurityToolObservation) -> SecurityEvidenceDerivationRequest {
        SecurityEvidenceDerivationRequest {
            evaluated_at_utc: "2026-08-08T22:00:00Z".to_string(),
            repository_commit: "a".repeat(40),
            catalog_sha256: digest('b'),
            inventory_sha256: digest('c'),
            raw_evidence: SecurityArtifactIdentity {
                uri: "study://security/raw.json".to_string(),
                sha256: digest('e'),
            },
            scanner: VulnerabilityScannerIdentity {
                name: "grype".to_string(),
                version: "0.116.1".to_string(),
                database: super::super::VulnerabilityDatabaseIdentity {
                    status: VulnerabilityDatabaseStatus::Current,
                    updated_at_utc: Some("2026-08-08T21:00:00Z".to_string()),
                    sha256: Some(digest('f')),
                    maximum_age_days: 5,
                },
            },
            sbom_generator_version: "1.50.0".to_string(),
            secret_scanner_version: "8.29.1".to_string(),
            tools: BTreeMap::from([(
                "samtools".to_string(),
                SecurityToolObservation::Collected(Box::new(observation)),
            )]),
        }
    }

    #[test]
    fn complete_clean_observation_derives_a_pass() -> Result<()> {
        let publication = derive_security_evidence(request(collected()))?;
        assert_eq!(publication.tools["samtools"].verdict, EvidenceVerdict::Passed);
        assert_eq!(publication.policy_sha256, security_evidence_policy_sha256());
        Ok(())
    }

    #[test]
    fn database_classification_preserves_fractional_day_age() -> Result<()> {
        let database = classify_vulnerability_database(
            "2026-08-08T22:00:01Z",
            "2026-08-03T22:00:00Z",
            digest('f'),
            5,
        )?;
        assert_eq!(database.status, VulnerabilityDatabaseStatus::Stale);
        Ok(())
    }

    #[test]
    fn unverified_signature_derives_a_failed_verdict() -> Result<()> {
        let mut observation = collected();
        observation.structural.signature = ImageSignatureDisposition::Invalid;
        let publication = derive_security_evidence(request(observation))?;
        assert_eq!(publication.tools["samtools"].verdict, EvidenceVerdict::Failed);
        assert!(publication.tools["samtools"]
            .reasons
            .contains(&"image signature is not verified".to_string()));
        Ok(())
    }

    #[test]
    fn duplicate_vulnerability_waivers_fail_closed() {
        let mut observation = collected();
        observation.waivers = vec![
            VulnerabilityWaiver {
                id: "first".to_string(),
                vulnerability_id: "CVE-1".to_string(),
                rationale: "first approval".to_string(),
                expires_on: "2026-09-01".to_string(),
                approval_sha256: digest('a'),
            },
            VulnerabilityWaiver {
                id: "second".to_string(),
                vulnerability_id: "CVE-1".to_string(),
                rationale: "second approval".to_string(),
                expires_on: "2026-09-01".to_string(),
                approval_sha256: digest('b'),
            },
        ];
        assert!(derive_security_evidence(request(observation))
            .is_err_and(|error| error.to_string().contains("multiple waivers")));
    }

    #[test]
    fn waiver_for_absent_vulnerability_fails_closed() {
        let mut observation = collected();
        observation.waivers = vec![VulnerabilityWaiver {
            id: "unused".to_string(),
            vulnerability_id: "CVE-ABSENT".to_string(),
            rationale: "approval must match collected evidence".to_string(),
            expires_on: "2026-09-01".to_string(),
            approval_sha256: digest('a'),
        }];
        assert!(derive_security_evidence(request(observation))
            .is_err_and(|error| error.to_string().contains("absent vulnerability")));
    }

    #[test]
    fn scanner_report_digest_drift_fails_closed() {
        let mut observation = collected();
        observation.secret_report.sha256 = digest('9');
        assert!(derive_security_evidence(request(observation))
            .is_err_and(|error| error.to_string().contains("digest differs")));
    }
}
