use std::fmt::Write as _;

use anyhow::Result;

use super::{
    ArtifactAvailability, ArtifactDelivery, EvidenceDimension, EvidenceVerdict, ProductSupport,
    QualificationClaim, QualificationReport, REQUIRED_EVIDENCE_DIMENSIONS,
};

/// Render a validated qualification report as deterministic JSON.
///
/// # Errors
/// Returns an error when the report is invalid or cannot be serialized.
pub fn render_qualification_report_json(report: &QualificationReport) -> Result<String> {
    report.validate()?;
    Ok(format!("{}\n", serde_json::to_string_pretty(report)?))
}

/// Render a validated qualification report as deterministic reviewer-facing Markdown.
///
/// # Errors
/// Returns an error when the report is invalid.
pub fn render_qualification_report_markdown(report: &QualificationReport) -> Result<String> {
    report.validate()?;
    let summary = &report.summary;
    let mut output = String::from(
        "<!-- GENERATED FILE - DO NOT EDIT -->\n<!-- source: bijux-dna-policies qualification report -->\n# Container evidence qualification\n\n",
    );
    write!(
        output,
        "- Catalog tools: **{}**\n- Product-supported: **{}**\n- Qualified: **{}**\n- Supported but unverified: **{}**\n- Retired: **{}**\n- Excluded: **{}**\n- Blocked: **{}**\n- Failed: **{}**\n\n",
        summary.tool_count,
        summary.supported_count,
        summary.qualified_count,
        summary.unverified_count,
        summary.retired_count,
        summary.excluded_count,
        summary.blocked_count,
        summary.failed_count,
    )?;
    output.push_str("## Dimension coverage\n\n| Dimension | Passing tools |\n|---|---:|\n");
    for dimension in REQUIRED_EVIDENCE_DIMENSIONS {
        let count = summary.dimension_passes.get(&dimension).copied().unwrap_or_default();
        writeln!(output, "| `{}` | {count} |", dimension_name(dimension))?;
    }
    output.push_str(
        "\n## Tool matrix\n\n| Tool | Product support | Artifact delivery | Redistribution | Availability | Build | Runtime | Real data | Science | Benchmark | Security | Release | Qualification |\n|---|---|---|---|---|---|---|---|---|---|---|---|---|\n",
    );
    for assessment in &report.assessments {
        let decision = |dimension| {
            assessment
                .dimensions
                .get(&dimension)
                .map_or("invalid", |value| verdict_name(value.verdict))
        };
        let qualification = if assessment.claims.contains(&QualificationClaim::Qualified) {
            "qualified"
        } else if assessment.product_support != ProductSupport::Supported {
            "unsupported"
        } else if assessment.claims.contains(&QualificationClaim::Blocked) {
            "blocked"
        } else {
            "unverified"
        };
        writeln!(
            output,
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | **{}** |",
            assessment.subject_id,
            product_support_name(assessment.product_support),
            artifact_delivery_name(assessment.artifact_delivery),
            if assessment.redistribution_permitted { "permitted" } else { "prohibited" },
            availability_name(assessment.availability),
            decision(EvidenceDimension::Build),
            decision(EvidenceDimension::Runtime),
            decision(EvidenceDimension::RealData),
            decision(EvidenceDimension::Science),
            decision(EvidenceDimension::Benchmark),
            decision(EvidenceDimension::Security),
            decision(EvidenceDimension::Release),
            qualification,
        )?;
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

fn product_support_name(support: ProductSupport) -> &'static str {
    match support {
        ProductSupport::Supported => "supported",
        ProductSupport::Retired => "retired",
        ProductSupport::Excluded => "excluded",
    }
}

fn artifact_delivery_name(delivery: ArtifactDelivery) -> &'static str {
    match delivery {
        ArtifactDelivery::RedistributableImage => "redistributable_image",
        ArtifactDelivery::ExternalArtifact => "external_artifact",
        ArtifactDelivery::PrivateSiteBuild => "private_site_build",
        ArtifactDelivery::SiteModule => "site_module",
    }
}

fn availability_name(availability: ArtifactAvailability) -> &'static str {
    match availability {
        ArtifactAvailability::Admitted => "admitted",
        ArtifactAvailability::BlockedExternalAccess => "blocked_external_access",
        ArtifactAvailability::Unavailable => "unavailable",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{render_qualification_report_json, render_qualification_report_markdown};
    use crate::qualification::{
        ArtifactAvailability, ArtifactDelivery, EvidenceDecision, ProductSupport,
        PublicationDisposition, QualificationAssessment, QualificationEvidenceSource,
        QualificationReport, REQUIRED_EVIDENCE_DIMENSIONS,
    };

    fn report() -> anyhow::Result<QualificationReport> {
        let dimensions = REQUIRED_EVIDENCE_DIMENSIONS
            .into_iter()
            .map(|dimension| {
                (
                    dimension,
                    EvidenceDecision::passed(
                        "qualification.render.test.v1",
                        BTreeMap::from([("evidence_state".to_string(), "passed".to_string())]),
                        BTreeMap::from([("required_state".to_string(), "passed".to_string())]),
                        "a".repeat(64),
                        "test evidence passes",
                    ),
                )
            })
            .collect();
        QualificationReport::v4(
            BTreeMap::from([(
                "catalog".to_string(),
                QualificationEvidenceSource {
                    uri: "repository://containers/contracts/catalog.toml".to_string(),
                    sha256: "b".repeat(64),
                },
            )]),
            vec![QualificationAssessment::v3(
                "samtools".to_string(),
                ProductSupport::Supported,
                ArtifactDelivery::RedistributableImage,
                true,
                ArtifactAvailability::Admitted,
                PublicationDisposition::Current,
                dimensions,
            )?],
        )
    }

    #[test]
    fn renders_deterministic_validated_publications() -> anyhow::Result<()> {
        let report = report()?;
        let json = render_qualification_report_json(&report)?;
        let markdown = render_qualification_report_markdown(&report)?;
        assert_eq!(json, render_qualification_report_json(&report)?);
        assert!(json.ends_with('\n'));
        assert!(markdown.contains("<!-- source: bijux-dna-policies qualification report -->"));
        assert!(markdown
            .contains("| `samtools` | supported | redistributable_image | permitted | admitted"));
        assert!(markdown.contains("**qualified**"));
        Ok(())
    }
}
