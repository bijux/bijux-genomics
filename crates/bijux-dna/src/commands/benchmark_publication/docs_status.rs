use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::models::{BenchmarkPublicationStatusReport, PublicationCorpusSpec, StageAuditIssue};
use super::{
    audit_publication_docs, benchmark_corpus_spec_path, benchmark_publication_contracts,
    benchmark_publication_exclusions, load_json_value, publication_artifact_file_name,
    render_publication_docs_markdown,
};

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
        load_supplemental_findings(&docs_root.join(publication_artifact_file_name(
            corpus_id,
            "publication-findings.json",
        )))?;
    let (results_by_stage, results_warnings) = load_results_status(&docs_root.join(
        publication_artifact_file_name(corpus_id, "results-status.json"),
    ))?;
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
    write_publication_docs_status(docs_root, corpus_id, &report)
}

pub(super) fn load_publication_corpus_spec(
    cwd: &Path,
    explicit_config: Option<&Path>,
    corpus_id: &str,
) -> Result<PublicationCorpusSpec> {
    let path = benchmark_corpus_spec_path(cwd, explicit_config, corpus_id)?;
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(super) fn expected_counts_for_scope(
    spec: &PublicationCorpusSpec,
    sample_scope: &str,
) -> Result<(usize, BTreeMap<String, usize>)> {
    let full_counts = BTreeMap::from([
        ("ancient_pe".to_string(), spec.target_ancient_pe),
        ("ancient_se".to_string(), spec.target_ancient_se),
        ("modern_pe".to_string(), spec.target_modern_pe),
        ("modern_se".to_string(), spec.target_modern_se),
    ]);
    match sample_scope {
        "full" => Ok((full_counts.values().sum(), full_counts)),
        "paired" => {
            let paired_counts = BTreeMap::from([
                ("ancient_pe".to_string(), spec.target_ancient_pe),
                ("modern_pe".to_string(), spec.target_modern_pe),
            ]);
            Ok((paired_counts.values().sum(), paired_counts))
        }
        other => Err(anyhow!(
            "unsupported corpus publication sample_scope: {other}"
        )),
    }
}

pub(super) fn load_supplemental_findings(
    path: &Path,
) -> Result<(
    BTreeMap<String, Vec<StageAuditIssue>>,
    Vec<String>,
    Option<String>,
)> {
    if !path.is_file() {
        return Ok((
            BTreeMap::new(),
            vec![format!(
                "missing supplemental findings file: {}",
                path.display()
            )],
            None,
        ));
    }
    let payload = load_json_value(path)?;
    let mut warnings = Vec::new();
    let generated_at_utc = payload
        .get("generated_at_utc")
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    if generated_at_utc.is_none() {
        warnings.push(format!(
            "supplemental findings freshness is untracked in {}; add generated_at_utc",
            path.display()
        ));
    }
    let findings = payload
        .get("findings")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            anyhow!(
                "supplemental findings in {} must declare a findings array",
                path.display()
            )
        })?;

    let mut findings_by_stage = BTreeMap::<String, Vec<StageAuditIssue>>::new();
    for finding in findings {
        let invalid_message = || {
            anyhow!(
                "invalid supplemental finding in {}: stage_id, issue_id, and detail are required",
                path.display()
            )
        };
        let stage_id = finding
            .get("stage_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(invalid_message)?;
        let issue_id = finding
            .get("issue_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(invalid_message)?;
        let detail = finding
            .get("detail")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(invalid_message)?;
        findings_by_stage
            .entry(stage_id.to_string())
            .or_default()
            .push(StageAuditIssue {
                stage_id: stage_id.to_string(),
                issue_id: issue_id.to_string(),
                severity: finding
                    .get("severity")
                    .and_then(|value| value.as_str())
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("error")
                    .to_string(),
                detail: detail.to_string(),
            });
    }
    Ok((findings_by_stage, warnings, generated_at_utc))
}

fn load_results_status(path: &Path) -> Result<(BTreeMap<String, serde_json::Value>, Vec<String>)> {
    if !path.is_file() {
        return Ok((
            BTreeMap::new(),
            vec![format!("missing results status file: {}", path.display())],
        ));
    }
    let payload = load_json_value(path)?;
    let stages = payload
        .get("stages")
        .and_then(|value| value.as_array())
        .ok_or_else(|| {
            anyhow!(
                "invalid results status payload in {}: missing stages list",
                path.display()
            )
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
    fs::write(
        &json_path,
        format!("{}\n", serde_json::to_string_pretty(report)?),
    )
    .with_context(|| format!("write {}", json_path.display()))?;
    let markdown_path = docs_root.join(publication_artifact_file_name(corpus_id, "status.md"));
    fs::write(&markdown_path, render_publication_docs_markdown(report))
        .with_context(|| format!("write {}", markdown_path.display()))?;
    Ok(())
}
