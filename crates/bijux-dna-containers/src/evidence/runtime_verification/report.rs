use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::evidence::{
    load_runtime_verification_publication, RuntimeEvidenceStatus, RuntimeVerificationPublication,
};

#[must_use]
pub fn render_runtime_verification_markdown(
    publication: &RuntimeVerificationPublication,
) -> String {
    let passed = publication
        .tools
        .values()
        .filter(|evidence| evidence.status == RuntimeEvidenceStatus::Passed)
        .count();
    let failed = publication.tools.len() - passed;
    let mut output = String::from(
        "<!-- GENERATED FILE - DO NOT EDIT -->\n\
<!-- source: bijux-dna-containers::evidence::runtime_verification -->\n\
# Apptainer Runtime Verification\n\n\
This report verifies embedded identity contracts, image labels, and catalog-owned\n\
version and help entry points for the exact canonical images selected by the\n\
admitted inventory. It does not establish\n\
real-data, scientific, benchmark, security, or release qualification.\n\n\
## Evidence Boundary\n\n",
    );
    let _ = writeln!(output, "- Repository commit: `{}`", publication.repository_commit);
    let _ = writeln!(output, "- Catalog SHA-256: `{}`", publication.catalog.sha256);
    let _ = writeln!(output, "- Raw inventory SHA-256: `{}`", publication.inventory.sha256);
    let _ = writeln!(output, "- Tool results: **passed {passed}**, **failed {failed}**");
    output.push_str("- Runtime network: **disabled**\n\n## Tool Results\n\n");
    output.push_str("| Tool | Version | Image SHA-256 | Identity | Manifest | Result |\n");
    output.push_str("|---|---|---|---|---|---|\n");
    for (tool_id, evidence) in &publication.tools {
        let result = match evidence.status {
            RuntimeEvidenceStatus::Passed => "passed",
            RuntimeEvidenceStatus::Failed => "failed",
            RuntimeEvidenceStatus::Blocked => "blocked",
        };
        let manifest =
            evidence.manifest.as_ref().map_or("unpublished", |identity| identity.path.as_str());
        let identity =
            evidence.identity.as_ref().map_or("unpublished", |identity| identity.path.as_str());
        let _ = writeln!(
            output,
            "| `{tool_id}` | `{}` | `{}` | `{identity}` | `{manifest}` | **{result}** |",
            evidence.version, evidence.image_sha256
        );
    }
    output.push_str(
        "\n## Interpretation\n\nA pass means the image digest, embedded contract, and image labels matched the\ncurrent catalog and recipe identity; the immutable image then started without\nnetwork access, its declared version and help probes completed within the\nexecution timeout, and version output matched the catalog contract. Full identity\nevidence, probe manifests, and immutable logs remain in the governed execution-site\nevidence tree.\n",
    );
    output
}

/// Verify a checked report is the exact deterministic rendering of its publication.
///
/// # Errors
/// Returns an error when the publication is invalid or the Markdown differs.
pub fn verify_runtime_verification_report(
    publication_path: &Path,
    markdown_path: &Path,
) -> Result<()> {
    let publication = load_runtime_verification_publication(publication_path)?;
    let expected = render_runtime_verification_markdown(&publication);
    let observed = fs::read_to_string(markdown_path)
        .with_context(|| format!("read runtime report {}", markdown_path.display()))?;
    if observed != expected {
        bail!("runtime-verification Markdown differs from its typed publication");
    }
    Ok(())
}
