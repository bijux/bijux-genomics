use std::collections::BTreeSet;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use super::{VulnerabilityFinding, VulnerabilitySeverity, VulnerabilitySeverityCounts};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVulnerabilityReport {
    pub findings: Vec<VulnerabilityFinding>,
    pub severity_counts: VulnerabilitySeverityCounts,
}

#[derive(Debug, Deserialize)]
struct GrypeReport {
    #[serde(default)]
    matches: Vec<GrypeMatch>,
}

#[derive(Debug, Deserialize)]
struct GrypeMatch {
    vulnerability: GrypeVulnerability,
    artifact: GrypeArtifact,
}

#[derive(Debug, Deserialize)]
struct GrypeVulnerability {
    id: String,
    severity: String,
    #[serde(default)]
    fix: Option<GrypeFix>,
}

#[derive(Debug, Deserialize)]
struct GrypeFix {
    #[serde(default)]
    versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GrypeArtifact {
    name: String,
    version: String,
}

/// Parse a Grype JSON report without discarding package or remediation identity.
///
/// # Errors
/// Returns an error when JSON is malformed, required identities are empty, or findings repeat.
pub fn parse_grype_vulnerability_report(raw: &[u8]) -> Result<ParsedVulnerabilityReport> {
    let report: GrypeReport =
        serde_json::from_slice(raw).context("parse Grype vulnerability report")?;
    let mut observed = BTreeSet::new();
    let mut findings = Vec::with_capacity(report.matches.len());
    let mut severity_counts =
        VulnerabilitySeverityCounts { critical: 0, high: 0, medium: 0, low: 0, unknown: 0 };

    for item in report.matches {
        let id = required_value(&item.vulnerability.id, "vulnerability ID")?;
        let package = required_value(&item.artifact.name, "package name")?;
        let installed_version = required_value(&item.artifact.version, "installed version")?;
        if !observed.insert((id.clone(), package.clone())) {
            bail!("Grype report repeats vulnerability {id} for package {package}");
        }
        let severity = parse_severity(&item.vulnerability.severity);
        increment_severity(&mut severity_counts, severity);
        let mut fixed_versions = item
            .vulnerability
            .fix
            .map_or_else(Vec::new, |fix| fix.versions)
            .into_iter()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        fixed_versions.sort();
        fixed_versions.dedup();
        findings.push(VulnerabilityFinding {
            id,
            severity,
            package,
            installed_version,
            fixed_versions,
            waiver_id: None,
        });
    }
    findings.sort_by(|left, right| {
        (&left.id, &left.package, &left.installed_version).cmp(&(
            &right.id,
            &right.package,
            &right.installed_version,
        ))
    });
    Ok(ParsedVulnerabilityReport { findings, severity_counts })
}

fn required_value(value: &str, label: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("Grype report has empty {label}");
    }
    Ok(value.to_string())
}

fn parse_severity(value: &str) -> VulnerabilitySeverity {
    match value.trim().to_ascii_lowercase().as_str() {
        "critical" => VulnerabilitySeverity::Critical,
        "high" => VulnerabilitySeverity::High,
        "medium" => VulnerabilitySeverity::Medium,
        "low" => VulnerabilitySeverity::Low,
        _ => VulnerabilitySeverity::Unknown,
    }
}

fn increment_severity(counts: &mut VulnerabilitySeverityCounts, severity: VulnerabilitySeverity) {
    match severity {
        VulnerabilitySeverity::Critical => counts.critical += 1,
        VulnerabilitySeverity::High => counts.high += 1,
        VulnerabilitySeverity::Medium => counts.medium += 1,
        VulnerabilitySeverity::Low => counts.low += 1,
        VulnerabilitySeverity::Unknown => counts.unknown += 1,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_grype_vulnerability_report;
    use crate::security_evidence::VulnerabilitySeverity;

    #[test]
    fn preserves_finding_identity_and_computes_counts() {
        let report = parse_grype_vulnerability_report(
            br#"{
              "matches": [
                {
                  "vulnerability": {
                    "id": "CVE-2026-0002",
                    "severity": "High",
                    "fix": {"versions": ["2.0", "2.0", ""]}
                  },
                  "artifact": {"name": "libbeta", "version": "1.4"}
                },
                {
                  "vulnerability": {
                    "id": "CVE-2026-0001",
                    "severity": "Critical"
                  },
                  "artifact": {"name": "libalpha", "version": "1.0"}
                }
              ]
            }"#,
        )
        .expect("parse report");
        assert_eq!(report.severity_counts.critical, 1);
        assert_eq!(report.severity_counts.high, 1);
        assert_eq!(report.findings[0].id, "CVE-2026-0001");
        assert_eq!(report.findings[0].severity, VulnerabilitySeverity::Critical);
        assert_eq!(report.findings[1].package, "libbeta");
        assert_eq!(report.findings[1].fixed_versions, ["2.0"]);
    }

    #[test]
    fn rejects_duplicate_vulnerability_package_pairs() {
        let raw = br#"{
          "matches": [
            {
              "vulnerability": {"id": "CVE-1", "severity": "Low"},
              "artifact": {"name": "libalpha", "version": "1"}
            },
            {
              "vulnerability": {"id": "CVE-1", "severity": "High"},
              "artifact": {"name": "libalpha", "version": "2"}
            }
          ]
        }"#;
        let error = parse_grype_vulnerability_report(raw).expect_err("duplicate must fail");
        assert!(error.to_string().contains("repeats vulnerability"));
    }

    #[test]
    fn rejects_missing_package_identity() {
        let raw = br#"{
          "matches": [{
            "vulnerability": {"id": "CVE-1", "severity": "High"},
            "artifact": {"name": " ", "version": "1"}
          }]
        }"#;
        let error = parse_grype_vulnerability_report(raw).expect_err("identity must fail");
        assert!(error.to_string().contains("empty package name"));
    }

    #[test]
    fn rejects_malformed_json() {
        let error = parse_grype_vulnerability_report(b"not-json").expect_err("JSON must fail");
        assert!(error.to_string().contains("parse Grype"));
    }
}
