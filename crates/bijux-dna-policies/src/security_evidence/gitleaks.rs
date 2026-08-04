use anyhow::{bail, Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedSecretScanReport {
    pub finding_count: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GitleaksFinding {
    #[serde(rename = "RuleID")]
    rule_id: String,
    file: String,
    start_line: u64,
    #[serde(default)]
    secret: String,
}

/// Parse a fully redacted Gitleaks JSON report without importing secret material.
///
/// # Errors
/// Returns an error when JSON is malformed, finding identity is incomplete, or a secret is visible.
pub fn parse_gitleaks_report(raw: &[u8]) -> Result<ParsedSecretScanReport> {
    let findings: Vec<GitleaksFinding> =
        serde_json::from_slice(raw).context("parse Gitleaks secret report")?;
    for finding in &findings {
        if finding.rule_id.trim().is_empty() || finding.file.trim().is_empty() {
            bail!("Gitleaks report contains a finding without rule or file identity");
        }
        if finding.start_line == 0 {
            bail!("Gitleaks report contains a finding without a source line");
        }
        let secret = finding.secret.trim();
        if !secret.is_empty() && !is_redacted(secret) {
            bail!("Gitleaks report contains unredacted secret material");
        }
    }
    let finding_count = u64::try_from(findings.len()).context("Gitleaks finding count overflow")?;
    Ok(ParsedSecretScanReport { finding_count })
}

fn is_redacted(value: &str) -> bool {
    value.eq_ignore_ascii_case("redacted")
        || value.bytes().all(|byte| matches!(byte, b'*' | b'x' | b'X' | b'.' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::parse_gitleaks_report;

    #[test]
    fn counts_redacted_findings() {
        let report = parse_gitleaks_report(
            br#"[
              {"RuleID":"generic-api-key","File":"etc/tool.conf","StartLine":4,"Secret":"REDACTED"},
              {"RuleID":"private-key","File":"opt/key","StartLine":1,"Secret":"********"}
            ]"#,
        )
        .expect("parse report");
        assert_eq!(report.finding_count, 2);
    }

    #[test]
    fn accepts_empty_report() {
        let report = parse_gitleaks_report(b"[]").expect("parse empty report");
        assert_eq!(report.finding_count, 0);
    }

    #[test]
    fn rejects_unredacted_secret_material() {
        let raw = br#"[
          {"RuleID":"generic-api-key","File":"etc/tool.conf","StartLine":4,"Secret":"actual-token"}
        ]"#;
        let error = parse_gitleaks_report(raw).expect_err("secret material must fail");
        assert!(error.to_string().contains("unredacted secret"));
    }

    #[test]
    fn rejects_incomplete_finding_identity() {
        let raw = br#"[
          {"RuleID":"","File":"etc/tool.conf","StartLine":0,"Secret":"REDACTED"}
        ]"#;
        let error = parse_gitleaks_report(raw).expect_err("identity must fail");
        assert!(error.to_string().contains("without rule or file identity"));
    }

    #[test]
    fn rejects_malformed_json() {
        let error = parse_gitleaks_report(b"{}").expect_err("object must fail");
        assert!(error.to_string().contains("parse Gitleaks"));
    }
}
