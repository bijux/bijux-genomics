use anyhow::Result;
use bijux_dna_api::v1::api::qualification::{
    EvidenceDimension, EvidenceVerdict, QualificationClaim, QualificationReport,
    SupportDisposition, REQUIRED_EVIDENCE_DIMENSIONS,
};

use super::super::{read_utf8, Workspace};

pub(in crate::commands::containers) fn qualification_json(
    report: &QualificationReport,
) -> Result<String> {
    report.validate()?;
    Ok(format!("{}\n", serde_json::to_string_pretty(report)?))
}

pub(in crate::commands::containers) fn qualification_markdown(
    report: &QualificationReport,
) -> Result<String> {
    report.validate()?;
    let summary = &report.summary;
    let mut output = String::from(
        "<!-- GENERATED FILE - DO NOT EDIT -->\n<!-- source: bijux-dna-dev containers qualification -->\n# Container evidence qualification\n\n",
    );
    output.push_str(&format!(
        "- Catalog tools: **{}**\n- Qualified: **{}**\n- Retained but unverified: **{}**\n- Unsupported: **{}**\n- Blocked: **{}**\n- Failed: **{}**\n\n",
        summary.tool_count,
        summary.qualified_count,
        summary.unverified_count,
        summary.unsupported_count,
        summary.blocked_count,
        summary.failed_count,
    ));
    output.push_str("## Dimension coverage\n\n| Dimension | Passing tools |\n|---|---:|\n");
    for dimension in REQUIRED_EVIDENCE_DIMENSIONS {
        let count = summary.dimension_passes.get(&dimension).copied().unwrap_or_default();
        output.push_str(&format!("| `{}` | {count} |\n", dimension_name(dimension)));
    }
    output.push_str(
        "\n## Tool matrix\n\n| Tool | Disposition | Build | Runtime | Real data | Science | Benchmark | Security | Release | Qualification |\n|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for assessment in &report.assessments {
        let decision = |dimension| {
            assessment
                .dimensions
                .get(&dimension)
                .map(|value| verdict_name(value.verdict))
                .unwrap_or("invalid")
        };
        let qualification = if assessment.claims.contains(&QualificationClaim::Qualified) {
            "qualified"
        } else if assessment.support_disposition == SupportDisposition::Unsupported {
            "unsupported"
        } else {
            "unverified"
        };
        output.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | **{}** |\n",
            assessment.subject_id,
            disposition_name(assessment.support_disposition),
            decision(EvidenceDimension::Build),
            decision(EvidenceDimension::Runtime),
            decision(EvidenceDimension::RealData),
            decision(EvidenceDimension::Science),
            decision(EvidenceDimension::Benchmark),
            decision(EvidenceDimension::Security),
            decision(EvidenceDimension::Release),
            qualification,
        ));
    }
    Ok(output)
}

fn dimension_name(dimension: EvidenceDimension) -> &'static str {
    match dimension {
        EvidenceDimension::Catalog => "catalog",
        EvidenceDimension::Source => "source",
        EvidenceDimension::Build => "build",
        EvidenceDimension::Runtime => "runtime",
        EvidenceDimension::Stage => "stage",
        EvidenceDimension::RealData => "real_data",
        EvidenceDimension::Science => "science",
        EvidenceDimension::Benchmark => "benchmark",
        EvidenceDimension::Security => "security",
        EvidenceDimension::Freshness => "freshness",
        EvidenceDimension::Release => "release",
    }
}

fn verdict_name(verdict: EvidenceVerdict) -> &'static str {
    match verdict {
        EvidenceVerdict::Passed => "pass",
        EvidenceVerdict::Failed => "failed",
        EvidenceVerdict::Blocked => "blocked",
        EvidenceVerdict::Missing => "missing",
        EvidenceVerdict::NotApplicable => "n/a",
    }
}

fn disposition_name(disposition: SupportDisposition) -> &'static str {
    match disposition {
        SupportDisposition::Retained => "retained",
        SupportDisposition::Unsupported => "unsupported",
    }
}

pub(in crate::commands::containers) fn checked_report_matches(
    workspace: &Workspace,
    relative: &str,
    expected: &str,
) -> Result<bool> {
    Ok(read_utf8(&workspace.path(relative))? == expected)
}
