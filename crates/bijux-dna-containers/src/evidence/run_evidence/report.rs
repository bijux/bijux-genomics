//! Deterministic Markdown publication for selected container runs.

use super::RunEvidenceIndex;

#[must_use]
pub fn render_run_evidence_markdown(index: &RunEvidenceIndex) -> String {
    let execution = counts(&index.summary.execution_status);
    let science = counts(&index.summary.scientific_status);
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-dev containers run run-evidence -- generate -->".to_string(),
        "# LUNARC container run evidence".to_string(),
        String::new(),
        "This report indexes the final successful immutable execution admitted for each"
            .to_string(),
        "tool. A successful process is not scientific validation, and no tool is qualified"
            .to_string(),
        "by this report.".to_string(),
        String::new(),
        "## Archive boundary".to_string(),
        String::new(),
        format!("- Final governed runs: **{}**", index.summary.run_count),
        format!("- Tools represented: **{}**", index.summary.tool_count),
        format!("- Execution status: {execution}"),
        format!("- Scientific status: {science}"),
        String::new(),
        "Superseded successful, failed, and incomplete attempts are summarized in the".to_string(),
        "checksummed attempt ledger without presenting them as current evidence. Their raw"
            .to_string(),
        "results remain on the governed execution site. Large results stay there; each final"
            .to_string(),
        "run directory records paths, sizes, and SHA-256 digests.".to_string(),
        String::new(),
        "The machine-readable [attempt ledger](../runs/attempt-ledger.json) preserves".to_string(),
        "the identities and lock hashes of superseded attempts.".to_string(),
        String::new(),
        "## Evidence by tool".to_string(),
        String::new(),
        "| Tool | Final run | Corpus | Declared stage | Execution | Scientific claim |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    let mut records = index.runs.iter().collect::<Vec<_>>();
    records.sort_by(|left, right| left.tool.cmp(&right.tool));
    for record in records {
        lines.push(format!(
            "| `{}` | [{}](../runs/{}-{}/) | `{}` | `{}` | {} | {} |",
            record.tool,
            record.run_id,
            record.run_id,
            record.tool,
            record.corpus,
            record.stage,
            record.execution_status.as_str(),
            record.scientific_status.as_str(),
        ));
    }
    lines.extend([
        String::new(),
        "## Interpretation".to_string(),
        String::new(),
        "`passed` means the archived Slurm execution completed its declared runtime".to_string(),
        "contract. `not_assessed` means no predeclared scientific acceptance threshold".to_string(),
        "was admitted for that run. Release readiness additionally requires reproducible"
            .to_string(),
        "build, security, benchmark, and freshness evidence; none of those claims can be"
            .to_string(),
        "inferred from process exit status.".to_string(),
        String::new(),
        "The machine-readable companion is [run-evidence.json](run-evidence.json).".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

fn counts(values: &std::collections::BTreeMap<String, usize>) -> String {
    values
        .iter()
        .map(|(status, count)| format!("**{status} {count}**"))
        .collect::<Vec<_>>()
        .join(", ")
}
