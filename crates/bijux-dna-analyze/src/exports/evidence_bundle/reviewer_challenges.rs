//! Owner: bijux-dna-analyze
//! Validate reviewer references and maintain the clocked challenge journal.

use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;

use super::model::{EvidenceBundleV1, ReviewerChallengeRecordV1, ReviewerChallengeRequestV1};

/// Submit a reviewer challenge tied to a governed evidence/report location.
///
/// # Errors
/// Returns an error if required evidence files are missing or challenge references are invalid.
#[allow(clippy::too_many_lines)]
pub fn submit_reviewer_challenge(
    base_dir: &Path,
    request: &ReviewerChallengeRequestV1,
) -> Result<ReviewerChallengeRecordV1> {
    if request.artifact_id.trim().is_empty() {
        return Err(anyhow!("artifact_id cannot be empty"));
    }
    if request.evidence_path.trim().is_empty() {
        return Err(anyhow!("evidence_path cannot be empty"));
    }
    if request.report_field.trim().is_empty() {
        return Err(anyhow!("report_field cannot be empty"));
    }
    if request.caveat.trim().is_empty() {
        return Err(anyhow!("caveat cannot be empty"));
    }
    if request.question.trim().is_empty() {
        return Err(anyhow!("question cannot be empty"));
    }
    if request.requested_by.trim().is_empty() {
        return Err(anyhow!("requested_by cannot be empty"));
    }

    let artifact_inventory_path = base_dir.join("artifact_inventory.json");
    let report_path = base_dir.join("report.json");
    let evidence_bundle_path = base_dir.join("evidence_bundle.json");
    if !artifact_inventory_path.exists() {
        return Err(anyhow!(
            "reviewer challenge requires artifact inventory at {}",
            artifact_inventory_path.display()
        ));
    }
    if !report_path.exists() {
        return Err(anyhow!("reviewer challenge requires report at {}", report_path.display()));
    }
    if !evidence_bundle_path.exists() {
        return Err(anyhow!(
            "reviewer challenge requires evidence bundle at {}",
            evidence_bundle_path.display()
        ));
    }

    let inventory: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&artifact_inventory_path)
            .with_context(|| format!("read {}", artifact_inventory_path.display()))?,
    )
    .with_context(|| format!("parse {}", artifact_inventory_path.display()))?;
    let artifact_exists =
        inventory.get("artifacts").and_then(serde_json::Value::as_array).is_some_and(|rows| {
            rows.iter().any(|row| {
                row.get("artifact_id").and_then(serde_json::Value::as_str)
                    == Some(request.artifact_id.as_str())
            })
        });
    if !artifact_exists {
        return Err(anyhow!(
            "artifact_id `{}` is not present in artifact_inventory.json",
            request.artifact_id
        ));
    }

    let report: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&report_path)
            .with_context(|| format!("read {}", report_path.display()))?,
    )
    .with_context(|| format!("parse {}", report_path.display()))?;
    if report_field(&report, &request.report_field).is_none() {
        return Err(anyhow!(
            "report field `{}` is not present in report.json",
            request.report_field
        ));
    }
    let evidence_relative = validate_reviewer_evidence_path(base_dir, &request.evidence_path)?;

    let bundle: EvidenceBundleV1 = serde_json::from_str(
        &std::fs::read_to_string(&evidence_bundle_path)
            .with_context(|| format!("read {}", evidence_bundle_path.display()))?,
    )
    .with_context(|| format!("parse {}", evidence_bundle_path.display()))?;
    let caveat_matches = bundle
        .health
        .gaps
        .iter()
        .any(|gap| gap.code == request.caveat || gap.message.contains(&request.caveat));
    if !caveat_matches {
        return Err(anyhow!(
            "caveat `{}` does not match any evidence gap code/message",
            request.caveat
        ));
    }

    let mut existing = list_reviewer_challenges(base_dir)?;
    existing.sort_by(|left, right| left.challenge_id.cmp(&right.challenge_id));
    let sequence = existing.len() + 1;
    let challenge_id = format!(
        "challenge-{}-{}-{:04}",
        Utc::now().format("%Y%m%d%H%M%S%3f"),
        sanitized_suffix(&request.artifact_id),
        sequence
    );
    let record = ReviewerChallengeRecordV1 {
        schema_version: "bijux.reviewer_challenge.v1".to_string(),
        challenge_id,
        created_at: Utc::now().to_rfc3339(),
        artifact_id: request.artifact_id.clone(),
        evidence_path: evidence_relative,
        report_field: request.report_field.clone(),
        caveat: request.caveat.clone(),
        question: request.question.clone(),
        requested_by: request.requested_by.clone(),
        state: "open".to_string(),
    };
    append_reviewer_challenge(base_dir, &record)?;
    Ok(record)
}

/// Load reviewer challenges from the run root.
///
/// # Errors
/// Returns an error if the challenge log exists but cannot be parsed.
pub fn list_reviewer_challenges(base_dir: &Path) -> Result<Vec<ReviewerChallengeRecordV1>> {
    let path = base_dir.join("reviewer_challenges.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let challenge_log =
        std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let mut rows = Vec::new();
    for (index, line) in challenge_log.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let challenge: ReviewerChallengeRecordV1 = serde_json::from_str(line)
            .with_context(|| format!("parse {} line {}", path.display(), index + 1))?;
        rows.push(challenge);
    }
    Ok(rows)
}

fn report_field<'a>(
    value: &'a serde_json::Value,
    dotted_path: &str,
) -> Option<&'a serde_json::Value> {
    let mut current = value;
    for segment in dotted_path.split('.').filter(|segment| !segment.is_empty()) {
        current = current.get(segment)?;
    }
    Some(current)
}

fn append_reviewer_challenge(base_dir: &Path, record: &ReviewerChallengeRecordV1) -> Result<()> {
    let path = base_dir.join("reviewer_challenges.jsonl");
    let mut rows = list_reviewer_challenges(base_dir)?;
    rows.push(record.clone());
    rows.sort_by(|left, right| left.challenge_id.cmp(&right.challenge_id));
    let payload = rows
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n")
        + "\n";
    bijux_dna_infra::write_bytes(&path, payload)?;
    let latest_path = base_dir.join("reviewer_challenges.latest.json");
    bijux_dna_infra::atomic_write_json(&latest_path, record)
        .with_context(|| format!("write {}", latest_path.display()))?;
    Ok(())
}

fn validate_reviewer_evidence_path(base_dir: &Path, value: &str) -> Result<String> {
    let candidate = Path::new(value);
    if candidate.is_absolute() {
        return Err(anyhow!(
            "evidence_path must be repository-relative, got absolute path `{value}`"
        ));
    }
    if candidate.components().any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(anyhow!(
            "evidence_path must not contain parent traversal (`..`), got `{value}`"
        ));
    }
    let relative = candidate
        .to_str()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| anyhow!("evidence_path must be valid UTF-8"))?;
    let full = base_dir.join(relative);
    if !full.exists() {
        return Err(anyhow!("evidence_path `{relative}` is not present under run directory"));
    }
    Ok(relative.to_string())
}

fn sanitized_suffix(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == '_' || ch == '-' {
            out.push(ch);
        }
    }
    if out.is_empty() {
        "artifact".to_string()
    } else {
        out
    }
}
