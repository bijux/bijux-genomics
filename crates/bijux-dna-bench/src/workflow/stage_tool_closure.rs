//! Owner: bijux-dna-bench
//! Deterministic publication of catalog, domain, and benchmark protocol closure.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    benchmark_stage_tool_closure_schema, validate_benchmark_stage_tool_closure,
    BenchmarkStageToolClosureDisposition, BenchmarkStageToolClosureReport,
    BenchmarkStageToolClosureRow, BenchmarkStageToolClosureSource,
    BenchmarkStageToolClosureSummary, BenchmarkStageToolProtocolDisposition,
    BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1,
};
use bijux_dna_containers::api::{load_container_catalog, validate_container_catalog};
use serde::Deserialize;

use super::load_benchmark_portfolio;

const CLOSURE_JSON: &str = "benchmarks/reports/STAGE_TOOL_CLOSURE.json";
const CLOSURE_MARKDOWN: &str = "benchmarks/reports/STAGE_TOOL_CLOSURE.md";
const CLOSURE_SCHEMA: &str = "benchmarks/contracts/schemas/stage-tool-closure.schema.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkStageToolClosureWriteMode {
    Write,
    Check,
}

#[derive(Debug, Deserialize)]
struct DomainIndex {
    stage_ids: Vec<String>,
    stage_tool_compatibility: BTreeMap<String, Vec<String>>,
}

/// Derive the complete product-supported catalog pair closure.
///
/// # Errors
/// Returns an error when catalog, domain, portfolio, or aggregate contracts drift.
pub fn derive_benchmark_stage_tool_closure(
    repository_root: &Path,
) -> Result<BenchmarkStageToolClosureReport> {
    let catalog_validation = validate_container_catalog(repository_root)?;
    if !catalog_validation.is_valid() {
        bail!("container catalog validation failed: {:?}", catalog_validation.errors);
    }
    let catalog = load_container_catalog(repository_root)?;
    let portfolio_path = repository_root.join("benchmarks/contracts/portfolio.toml");
    let portfolio = load_benchmark_portfolio(&portfolio_path)?;

    let mut sources = Vec::new();
    let catalog_path = repository_root.join("containers/contracts/catalog.toml");
    sources.push(source_identity(repository_root, &catalog_path)?);
    sources.push(source_identity(repository_root, &portfolio_path)?);

    let mut domain_pairs = BTreeSet::new();
    let mut stage_ids = BTreeSet::new();
    for domain in ["bam", "fastq", "vcf"] {
        let path = repository_root.join("domain").join(domain).join("index.yaml");
        sources.push(source_identity(repository_root, &path)?);
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read domain index {}", path.display()))?;
        let index: DomainIndex = serde_yaml::from_str(&raw)
            .with_context(|| format!("parse domain index {}", path.display()))?;
        stage_ids.extend(index.stage_ids);
        domain_pairs.extend(index.stage_tool_compatibility.into_iter().flat_map(
            |(stage_id, tool_ids)| {
                tool_ids.into_iter().map(move |tool_id| (stage_id.clone(), tool_id))
            },
        ));
    }
    sources.sort_by(|left, right| left.path.cmp(&right.path));

    let supported_tools =
        catalog.tools.iter().filter(|tool| tool.is_product_supported()).collect::<Vec<_>>();
    let mut rows = supported_tools
        .iter()
        .flat_map(|tool| {
            tool.stage_ids.iter().map(|stage_id| {
                let domain = stage_id.split_once('.').map_or("", |(domain, _)| domain);
                let domain_declared = domain_pairs.contains(&(stage_id.clone(), tool.id.clone()));
                let benchmark_run_id = portfolio.stage_runs.get(stage_id).cloned();
                BenchmarkStageToolClosureRow {
                    domain: domain.to_string(),
                    stage_id: stage_id.clone(),
                    tool_id: tool.id.clone(),
                    product_support: tool.product_support.as_str().to_string(),
                    artifact_delivery: tool.artifact_delivery.as_str().to_string(),
                    redistribution_permitted: tool.redistribution_permitted,
                    domain_declared,
                    protocol_disposition: if benchmark_run_id.is_some() {
                        BenchmarkStageToolProtocolDisposition::BenchmarkRequired
                    } else {
                        BenchmarkStageToolProtocolDisposition::QualificationOnly
                    },
                    benchmark_run_id,
                    closure: if domain_declared {
                        BenchmarkStageToolClosureDisposition::Aligned
                    } else {
                        BenchmarkStageToolClosureDisposition::MissingDomainBinding
                    },
                }
            })
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });

    let aligned_pair_count = rows.iter().filter(|row| row.domain_declared).count();
    let benchmark_pair_count = rows.iter().filter(|row| row.benchmark_run_id.is_some()).count();
    let report = BenchmarkStageToolClosureReport {
        schema_version: BENCHMARK_STAGE_TOOL_CLOSURE_SCHEMA_V1.to_string(),
        sources,
        summary: BenchmarkStageToolClosureSummary {
            supported_tool_count: supported_tools.len(),
            stage_count: stage_ids.len(),
            catalog_pair_count: rows.len(),
            domain_pair_count: domain_pairs.len(),
            aligned_pair_count,
            drift_pair_count: rows.len() - aligned_pair_count,
            benchmark_pair_count,
            qualification_only_pair_count: rows.len() - benchmark_pair_count,
            external_artifact_pair_count: rows
                .iter()
                .filter(|row| row.artifact_delivery == "external_artifact")
                .count(),
        },
        rows,
    };
    validate_benchmark_stage_tool_closure(&report)
        .context("validate benchmark stage/tool closure")?;
    Ok(report)
}

/// Write or check the checked-in stage/tool closure publication and schema.
///
/// # Errors
/// Returns an error when strict product counts drift or generated files cannot be written/checked.
pub fn publish_checked_in_benchmark_stage_tool_closure(
    repository_root: &Path,
    mode: BenchmarkStageToolClosureWriteMode,
) -> Result<BenchmarkStageToolClosureReport> {
    let report = derive_benchmark_stage_tool_closure(repository_root)?;
    let summary = &report.summary;
    if summary.supported_tool_count != 89
        || summary.stage_count != 89
        || summary.catalog_pair_count != 159
        || summary.domain_pair_count != 159
        || summary.aligned_pair_count != 159
        || summary.drift_pair_count != 0
        || summary.external_artifact_pair_count != 8
    {
        bail!("strict stage/tool closure is incomplete: {summary:?}");
    }
    let json = serde_json::to_string_pretty(&report)? + "\n";
    let markdown = render_benchmark_stage_tool_closure_markdown(&report);
    let schema = serde_json::to_string_pretty(&benchmark_stage_tool_closure_schema())? + "\n";
    for (relative, content) in [
        (CLOSURE_JSON, json.as_bytes()),
        (CLOSURE_MARKDOWN, markdown.as_bytes()),
        (CLOSURE_SCHEMA, schema.as_bytes()),
    ] {
        let path = repository_root.join(relative);
        match mode {
            BenchmarkStageToolClosureWriteMode::Write => {
                bijux_dna_infra::atomic_write_bytes(&path, content)
                    .with_context(|| format!("write stage/tool closure {relative}"))?;
            }
            BenchmarkStageToolClosureWriteMode::Check => {
                let current = fs::read(&path)
                    .with_context(|| format!("read stage/tool closure {relative}"))?;
                if current != content {
                    bail!("generated stage/tool closure drifted: {relative}");
                }
            }
        }
    }
    Ok(report)
}

#[must_use]
pub fn render_benchmark_stage_tool_closure_markdown(
    report: &BenchmarkStageToolClosureReport,
) -> String {
    let summary = &report.summary;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- source: bijux-dna-bench::workflow::stage_tool_closure -->".to_string(),
        "# Stage/tool closure".to_string(),
        String::new(),
        format!("- Supported tools: **{}**", summary.supported_tool_count),
        format!("- Domain stages: **{}**", summary.stage_count),
        format!("- Catalog pairs: **{}**", summary.catalog_pair_count),
        format!("- Domain pairs: **{}**", summary.domain_pair_count),
        format!("- Aligned pairs: **{}**", summary.aligned_pair_count),
        format!("- Drift pairs: **{}**", summary.drift_pair_count),
        format!("- Licensed external pairs: **{}**", summary.external_artifact_pair_count),
        String::new(),
        "| Stage | Tool | Delivery | Protocol | Run | Closure |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    for row in &report.rows {
        let protocol = match row.protocol_disposition {
            BenchmarkStageToolProtocolDisposition::BenchmarkRequired => "benchmark_required",
            BenchmarkStageToolProtocolDisposition::QualificationOnly => "qualification_only",
        };
        let closure = match row.closure {
            BenchmarkStageToolClosureDisposition::Aligned => "aligned",
            BenchmarkStageToolClosureDisposition::MissingDomainBinding => "missing_domain_binding",
        };
        lines.push(format!(
            "| `{}` | `{}` | `{}` | `{protocol}` | {} | `{closure}` |",
            row.stage_id,
            row.tool_id,
            row.artifact_delivery,
            row.benchmark_run_id.as_deref().map_or("-".to_string(), |run_id| format!("`{run_id}`")),
        ));
    }
    lines.join("\n") + "\n"
}

fn source_identity(repository_root: &Path, path: &Path) -> Result<BenchmarkStageToolClosureSource> {
    let relative = path
        .strip_prefix(repository_root)
        .with_context(|| format!("source is outside repository: {}", path.display()))?;
    Ok(BenchmarkStageToolClosureSource {
        path: relative.to_string_lossy().replace('\\', "/"),
        sha256: bijux_dna_infra::hash_file_sha256(path)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository_root() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("repository root")
            .to_path_buf()
    }

    #[test]
    fn closure_covers_every_supported_catalog_pair_and_resolved_drift() {
        let report =
            derive_benchmark_stage_tool_closure(&repository_root()).expect("derive closure");
        assert_eq!(report.summary.supported_tool_count, 89);
        assert_eq!(report.summary.stage_count, 89);
        assert_eq!(report.summary.catalog_pair_count, 159);
        assert_eq!(report.summary.domain_pair_count, 159);
        assert_eq!(report.summary.aligned_pair_count, 159);
        assert_eq!(report.summary.drift_pair_count, 0);
        assert_eq!(report.summary.external_artifact_pair_count, 8);
        for (stage_id, tool_id) in [
            ("fastq.merge_pairs", "seqprep"),
            ("fastq.trim_reads", "seqprep"),
            ("vcf.ibd", "ibdhap"),
        ] {
            assert!(report.rows.iter().any(|row| {
                row.stage_id == stage_id
                    && row.tool_id == tool_id
                    && row.domain_declared
                    && row.closure == BenchmarkStageToolClosureDisposition::Aligned
            }));
        }
    }
}
