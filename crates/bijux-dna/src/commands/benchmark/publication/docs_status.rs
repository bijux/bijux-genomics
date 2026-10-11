use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    validate_benchmark_publication_findings, validate_benchmark_publication_status,
};

use super::{
    audit_publication_docs, benchmark_publication_contracts, benchmark_publication_exclusions,
    load_json_value, publication_artifact_file_name, render_publication_docs_markdown,
    BenchmarkPublicationFindings, BenchmarkPublicationStatusReport, StageAuditIssue,
};
use crate::commands::benchmark::corpus_metadata::{
    expected_counts_for_scope as expected_corpus_counts_for_scope, load_corpus_spec, CorpusSpec,
};

type SupplementalFindingsPayload =
    (BTreeMap<String, Vec<StageAuditIssue>>, Vec<String>, Option<String>);

pub(super) fn write_corpus_fastq_docs_status(
    cwd: &Path,
    explicit_config: Option<&Path>,
    docs_root: &Path,
    corpus_id: &str,
) -> Result<()> {
    let contracts = benchmark_publication_contracts(cwd, explicit_config, corpus_id)?;
    if contracts.is_empty() {
        return Ok(());
    }
    let exclusions = benchmark_publication_exclusions(cwd, explicit_config, corpus_id)?;
    let corpus_spec = load_publication_corpus_spec(cwd, explicit_config, corpus_id)?;
    let (supplemental_findings, mut audit_warnings, findings_generated_at_utc) =
        load_supplemental_findings(
            &docs_root.join(publication_artifact_file_name(corpus_id, "publication-findings.json")),
            corpus_id,
        )?;
    let (results_by_stage, results_warnings) = load_results_status(
        &docs_root.join(publication_artifact_file_name(corpus_id, "results-status.json")),
    )?;
    audit_warnings.extend(results_warnings);
    let report = audit_publication_docs(
        cwd,
        docs_root,
        corpus_id,
        &contracts,
        &exclusions,
        &corpus_spec,
        &supplemental_findings,
        &results_by_stage,
        &audit_warnings,
        findings_generated_at_utc,
    )?;
    validate_benchmark_publication_status(&report)?;
    write_publication_docs_status(docs_root, corpus_id, &report)
}

pub(super) fn load_publication_corpus_spec(
    cwd: &Path,
    explicit_config: Option<&Path>,
    corpus_id: &str,
) -> Result<CorpusSpec> {
    load_corpus_spec(cwd, explicit_config, corpus_id)
}

pub(super) fn expected_counts_for_scope(
    spec: &CorpusSpec,
    sample_scope: &str,
) -> Result<(usize, BTreeMap<String, usize>)> {
    let counts = expected_corpus_counts_for_scope(spec, sample_scope)?;
    Ok((counts.values().sum(), counts))
}

pub(super) fn load_supplemental_findings(
    path: &Path,
    corpus_id: &str,
) -> Result<SupplementalFindingsPayload> {
    if !path.is_file() {
        return Ok((
            BTreeMap::new(),
            vec![format!("missing supplemental findings file: {}", path.display())],
            None,
        ));
    }
    let payload = load_publication_findings(path, corpus_id)?;
    let generated_at_utc = Some(payload.generated_at_utc);
    let mut findings_by_stage = BTreeMap::<String, Vec<StageAuditIssue>>::new();
    for finding in payload.findings {
        findings_by_stage.entry(finding.stage_id.clone()).or_default().push(finding);
    }
    Ok((findings_by_stage, Vec::new(), generated_at_utc))
}

pub(super) fn load_publication_findings(
    path: &Path,
    corpus_id: &str,
) -> Result<BenchmarkPublicationFindings> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let findings: BenchmarkPublicationFindings = serde_json::from_str(&raw)
        .with_context(|| format!("parse typed publication findings from {}", path.display()))?;
    validate_benchmark_publication_findings(&findings)?;
    if findings.corpus_id != corpus_id {
        return Err(anyhow!(
            "publication findings corpus `{}` does not match requested corpus `{corpus_id}`",
            findings.corpus_id
        ));
    }
    Ok(findings)
}

fn load_results_status(path: &Path) -> Result<(BTreeMap<String, serde_json::Value>, Vec<String>)> {
    if !path.is_file() {
        return Ok((
            BTreeMap::new(),
            vec![format!("missing results status file: {}", path.display())],
        ));
    }
    let payload = load_json_value(path)?;
    let stages = payload.get("stages").and_then(|value| value.as_array()).ok_or_else(|| {
        anyhow!("invalid results status payload in {}: missing stages list", path.display())
    })?;
    Ok((
        stages
            .iter()
            .filter_map(|stage| {
                stage
                    .get("stage_id")
                    .and_then(|value| value.as_str())
                    .map(|stage_id| (stage_id.to_string(), stage.clone()))
            })
            .collect(),
        Vec::new(),
    ))
}

fn write_publication_docs_status(
    docs_root: &Path,
    corpus_id: &str,
    report: &BenchmarkPublicationStatusReport,
) -> Result<()> {
    let json_path = docs_root.join(publication_artifact_file_name(corpus_id, "status.json"));
    fs::write(&json_path, format!("{}\n", serde_json::to_string_pretty(report)?))
        .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(corpus_id, "status.md"));
    fs::write(&markdown_path, render_publication_docs_markdown(report))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}
