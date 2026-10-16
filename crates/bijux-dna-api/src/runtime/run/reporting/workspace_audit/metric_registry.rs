use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_domain_vcf::{contracts::stage_metrics_contract, VcfDomainStage};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod contracts;

#[derive(Debug, Serialize)]
pub struct MetricRegistryReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub domain_count: usize,
    pub stage_count: usize,
    pub row_count: usize,
    pub rows: Vec<MetricRegistryRow>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MetricRegistryRow {
    pub domain: String,
    pub stage_id: String,
    pub metric_id: String,
    pub meaning: String,
    pub contract_kind: String,
    pub stage_contract_surface: String,
    pub domain_registry_surface: String,
}

#[derive(Debug, Deserialize, Default)]
struct MetricRegistryStageDoc {
    #[serde(default)]
    stage_id: String,
    #[serde(default)]
    metrics: Vec<MetricRegistryStageMetricDoc>,
}

#[derive(Debug, Deserialize, Default)]
struct MetricRegistryStageMetricDoc {
    #[serde(default)]
    name: String,
    #[serde(default)]
    meaning: String,
}

#[derive(Debug, Deserialize, Default)]
struct MetricRegistryVocabularyDoc {
    #[serde(default)]
    metric_ids: Vec<String>,
    #[serde(default)]
    metrics: Vec<MetricRegistryVocabularyEntryDoc>,
}

#[derive(Debug, Deserialize, Default)]
struct MetricRegistryVocabularyEntryDoc {
    #[serde(default)]
    id: String,
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn collect_yaml_sources(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_sources(&path, files)?;
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("yaml") {
            files.push(path);
        }
    }
    Ok(())
}

fn load_domain_metric_vocabulary(path: &Path) -> Result<BTreeSet<String>> {
    let payload =
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let doc: MetricRegistryVocabularyDoc =
        serde_yaml::from_str(&payload).with_context(|| format!("parse {}", path.display()))?;
    let mut metric_ids = doc
        .metric_ids
        .into_iter()
        .chain(doc.metrics.into_iter().map(|entry| entry.id))
        .filter(|metric_id| !metric_id.trim().is_empty())
        .collect::<BTreeSet<_>>();
    if metric_ids.is_empty() {
        bail!("metric vocabulary `{}` is empty", path.display());
    }
    Ok(std::mem::take(&mut metric_ids))
}

fn collect_stage_yaml_metric_registry_rows(
    cwd: &Path,
    domain: &str,
) -> Result<Vec<MetricRegistryRow>> {
    let domain_root = cwd.join("domain").join(domain);
    let vocabulary_path = domain_root.join("metrics.yaml");
    let registered_metric_ids = load_domain_metric_vocabulary(&vocabulary_path)?;

    let mut stage_paths = Vec::new();
    collect_yaml_sources(&domain_root.join("stages"), &mut stage_paths)?;
    stage_paths.sort();

    let mut rows = Vec::new();
    for stage_path in stage_paths {
        if stage_path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let payload = std::fs::read_to_string(&stage_path)
            .with_context(|| format!("read {}", stage_path.display()))?;
        let stage_doc: MetricRegistryStageDoc = serde_yaml::from_str(&payload)
            .with_context(|| format!("parse {}", stage_path.display()))?;
        if stage_doc.stage_id.trim().is_empty() {
            continue;
        }
        let mut stage_metric_ids = BTreeSet::new();
        for metric in stage_doc.metrics {
            if metric.name.trim().is_empty() {
                bail!(
                    "stage metric declaration in `{}` is missing a metric name",
                    stage_path.display()
                );
            }
            if !stage_metric_ids.insert(metric.name.clone()) {
                bail!(
                    "stage `{}` declares duplicate metric `{}` in `{}`",
                    stage_doc.stage_id,
                    metric.name,
                    stage_path.display()
                );
            }
            if !registered_metric_ids.contains(&metric.name) {
                bail!(
                    "stage `{}` declares unregistered metric `{}` in `{}`",
                    stage_doc.stage_id,
                    metric.name,
                    stage_path.display()
                );
            }
            rows.push(MetricRegistryRow {
                domain: domain.to_string(),
                stage_id: stage_doc.stage_id.clone(),
                metric_id: metric.name,
                meaning: metric.meaning,
                contract_kind: "yaml_stage_metrics".to_string(),
                stage_contract_surface: relative_display(&stage_path, cwd),
                domain_registry_surface: relative_display(&vocabulary_path, cwd),
            });
        }
    }
    Ok(rows)
}

fn collect_vcf_metric_registry_rows(cwd: &Path) -> Result<Vec<MetricRegistryRow>> {
    let vocabulary_path = cwd.join("domain").join("vcf").join("metrics.yaml");
    let registered_metric_ids = load_domain_metric_vocabulary(&vocabulary_path)?;
    let mut rows = Vec::new();

    for stage in VcfDomainStage::all() {
        let contract = stage_metrics_contract(*stage);
        let mut stage_metric_ids = BTreeSet::new();
        for metric_id in contract.required_metrics {
            if !stage_metric_ids.insert((*metric_id).to_string()) {
                bail!(
                    "VCF stage `{}` declares duplicate metric `{metric_id}` in the Rust contract",
                    stage.as_str()
                );
            }
            if !registered_metric_ids.contains(*metric_id) {
                bail!(
                    "VCF stage `{}` references unregistered metric `{metric_id}` in `domain/vcf/metrics.yaml`",
                    stage.as_str()
                );
            }
            rows.push(MetricRegistryRow {
                domain: "vcf".to_string(),
                stage_id: stage.as_str().to_string(),
                metric_id: (*metric_id).to_string(),
                meaning: String::new(),
                contract_kind: "rust_stage_metrics_contract".to_string(),
                stage_contract_surface:
                    "crates/bijux-dna-domain-vcf/src/contracts/stage_metrics.rs".to_string(),
                domain_registry_surface: relative_display(&vocabulary_path, cwd),
            });
        }
    }

    Ok(rows)
}

fn collect_metric_registry_rows(cwd: &Path) -> Result<Vec<MetricRegistryRow>> {
    let mut rows = Vec::new();
    rows.extend(collect_stage_yaml_metric_registry_rows(cwd, "fastq")?);
    rows.extend(collect_stage_yaml_metric_registry_rows(cwd, "bam")?);
    rows.extend(collect_vcf_metric_registry_rows(cwd)?);
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.metric_id.cmp(&right.metric_id))
    });

    let mut row_keys = BTreeSet::new();
    for row in &rows {
        let key = (row.domain.clone(), row.stage_id.clone(), row.metric_id.clone());
        if !row_keys.insert(key) {
            bail!(
                "metric registry produced duplicate row for `{}` / `{}` / `{}`",
                row.domain,
                row.stage_id,
                row.metric_id
            );
        }
    }

    Ok(rows)
}

fn render_metric_registry_tsv(rows: &[MetricRegistryRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\tmetric_id\tmeaning\tcontract_kind\tstage_contract_surface\tdomain_registry_surface\n",
    );
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.metric_id),
            sanitize_tsv(&row.meaning),
            sanitize_tsv(&row.contract_kind),
            sanitize_tsv(&row.stage_contract_surface),
            sanitize_tsv(&row.domain_registry_surface),
        );
    }
    rendered
}

/// # Errors
/// Returns an error if the governed stage metric registry cannot be resolved or written.
pub fn write_metric_registry_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<MetricRegistryReport> {
    let rows = collect_metric_registry_rows(cwd)?;
    let report = MetricRegistryReport {
        schema_version: "bijux.crates.metric_registry.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        domain_count: rows.iter().map(|row| row.domain.as_str()).collect::<BTreeSet<_>>().len(),
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        row_count: rows.len(),
        rows: rows.clone(),
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_bytes(output_path, render_metric_registry_tsv(&rows).as_bytes())?;
    Ok(report)
}
