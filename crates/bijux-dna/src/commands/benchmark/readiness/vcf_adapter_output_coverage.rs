use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::vcf_angsd_adapter::{render_vcf_angsd_adapter, DEFAULT_VCF_ANGSD_ADAPTER_PATH};
use super::vcf_bcftools_adapter::{render_vcf_bcftools_adapter, DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH};
use super::vcf_descent_family_adapter::{
    render_vcf_descent_family_adapter, DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
};
use super::vcf_eigensoft_adapter::{
    render_vcf_eigensoft_adapter, DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH,
};
use super::vcf_imputation_family_adapter::{
    render_vcf_imputation_family_adapter, DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH,
};
use super::vcf_phasing_family_adapter::{
    render_vcf_phasing_family_adapter, DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
    DEFAULT_VCF_EAGLE_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
};
use super::vcf_plink_family_adapter::{
    render_vcf_plink_family_adapter, DEFAULT_VCF_PLINK2_ADAPTER_PATH,
    DEFAULT_VCF_PLINK_ADAPTER_PATH,
};
use super::vcf_tool_serving_map::{render_vcf_tool_serving_map, DEFAULT_VCF_TOOL_SERVING_MAP_PATH};
use crate::commands::benchmark::local_slurm_run_paths::LOCAL_SLURM_DRY_RUN_RUN_ID;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH: &str =
    "benchmarks/readiness/vcf-adapter-output-coverage.tsv";
const VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_adapter_output_coverage.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum VcfAdapterOutputCoverageStatus {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAdapterOutputCoverageRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) raw_outputs: Vec<String>,
    pub(crate) normalized_metrics: Vec<String>,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) manifest: String,
    pub(crate) index_outputs: Vec<String>,
    pub(crate) status: VcfAdapterOutputCoverageStatus,
    pub(crate) benchmark_status: String,
    pub(crate) missing_declarations: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfAdapterOutputCoverageReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) benchmark_ready_row_count: usize,
    pub(crate) benchmark_ready_complete_row_count: usize,
    pub(crate) benchmark_ready_incomplete_row_count: usize,
    pub(crate) complete_row_count: usize,
    pub(crate) incomplete_row_count: usize,
    pub(crate) rows: Vec<VcfAdapterOutputCoverageRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclaredArtifact {
    artifact_id: String,
    role: String,
    path: String,
}

#[derive(Debug, Clone)]
struct VcfAdapterOutputCoverageOwnedRow {
    family_label: String,
    row: VcfAdapterOutputCoverageRow,
}

pub(crate) fn run_render_vcf_adapter_output_coverage(
    args: &parse::BenchReadinessRenderVcfAdapterOutputCoverageArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_adapter_output_coverage(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_adapter_output_coverage(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAdapterOutputCoverageReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_adapter_output_coverage_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_adapter_output_coverage_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let benchmark_ready_row_count =
        rows.iter().filter(|row| row.benchmark_status == "benchmark_ready").count();
    let benchmark_ready_complete_row_count = rows
        .iter()
        .filter(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.status == VcfAdapterOutputCoverageStatus::Complete
        })
        .count();
    let benchmark_ready_incomplete_row_count =
        benchmark_ready_row_count.saturating_sub(benchmark_ready_complete_row_count);
    let complete_row_count =
        rows.iter().filter(|row| row.status == VcfAdapterOutputCoverageStatus::Complete).count();
    let incomplete_row_count = rows.len().saturating_sub(complete_row_count);

    Ok(VcfAdapterOutputCoverageReport {
        schema_version: VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        benchmark_ready_row_count,
        benchmark_ready_complete_row_count,
        benchmark_ready_incomplete_row_count,
        complete_row_count,
        incomplete_row_count,
        rows,
    })
}

pub(crate) fn collect_vcf_adapter_output_coverage_rows(
    repo_root: &Path,
) -> Result<Vec<VcfAdapterOutputCoverageRow>> {
    let tool_serving_map =
        render_vcf_tool_serving_map(repo_root, PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH))?;
    let benchmark_ready_pairs = tool_serving_map
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();

    let mut owned_rows = BTreeMap::<(String, String), VcfAdapterOutputCoverageOwnedRow>::new();
    append_serialized_rows(
        &mut owned_rows,
        "bcftools",
        render_vcf_bcftools_adapter(repo_root, PathBuf::from(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH))?
            .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "angsd",
        render_vcf_angsd_adapter(repo_root, PathBuf::from(DEFAULT_VCF_ANGSD_ADAPTER_PATH))?.rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "descent",
        render_vcf_descent_family_adapter(
            repo_root,
            PathBuf::from(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
        )?
        .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "eigensoft",
        render_vcf_eigensoft_adapter(repo_root, PathBuf::from(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH))?
            .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "imputation_family",
        render_vcf_imputation_family_adapter(
            repo_root,
            PathBuf::from(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH),
        )?
        .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "plink",
        render_vcf_plink_family_adapter(
            repo_root,
            "plink",
            PathBuf::from(DEFAULT_VCF_PLINK_ADAPTER_PATH),
        )?
        .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "plink2",
        render_vcf_plink_family_adapter(
            repo_root,
            "plink2",
            PathBuf::from(DEFAULT_VCF_PLINK2_ADAPTER_PATH),
        )?
        .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "shapeit5",
        render_vcf_phasing_family_adapter(
            repo_root,
            "shapeit5",
            PathBuf::from(DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH),
        )?
        .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "eagle",
        render_vcf_phasing_family_adapter(
            repo_root,
            "eagle",
            PathBuf::from(DEFAULT_VCF_EAGLE_ADAPTER_PATH),
        )?
        .rows,
    )?;
    append_serialized_rows(
        &mut owned_rows,
        "beagle_phasing",
        render_vcf_phasing_family_adapter(
            repo_root,
            "beagle",
            PathBuf::from(DEFAULT_VCF_BEAGLE_ADAPTER_PATH),
        )?
        .rows,
    )?;

    let mut rows = owned_rows
        .into_values()
        .map(|owned_row| {
            let mut row = owned_row.row;
            row.benchmark_status =
                if benchmark_ready_pairs.contains(&(row.stage_id.clone(), row.tool_id.clone())) {
                    "benchmark_ready".to_string()
                } else {
                    "not_benchmark_ready".to_string()
                };
            row
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        left.tool_id.cmp(&right.tool_id).then(left.stage_id.cmp(&right.stage_id))
    });
    ensure_vcf_adapter_output_coverage_contract(&rows, &benchmark_ready_pairs)?;
    Ok(rows)
}

fn append_serialized_rows<T: Serialize>(
    owned_rows: &mut BTreeMap<(String, String), VcfAdapterOutputCoverageOwnedRow>,
    family_label: &str,
    source_rows: Vec<T>,
) -> Result<()> {
    for row in source_rows {
        let normalized_row = normalize_serialized_row(family_label, serde_json::to_value(row)?)?;
        let key = (normalized_row.stage_id.clone(), normalized_row.tool_id.clone());
        match owned_rows.get(&key) {
            None => {
                owned_rows.insert(
                    key,
                    VcfAdapterOutputCoverageOwnedRow {
                        family_label: family_label.to_string(),
                        row: normalized_row,
                    },
                );
            }
            Some(existing) => {
                let canonical_family = canonical_family_for_pair(
                    normalized_row.stage_id.as_str(),
                    normalized_row.tool_id.as_str(),
                )
                .ok_or_else(|| {
                    anyhow!(
                        "VCF adapter output coverage discovered an unmapped duplicate row `{}` / `{}` between `{}` and `{}`",
                        normalized_row.stage_id,
                        normalized_row.tool_id,
                        existing.family_label,
                        family_label
                    )
                })?;
                if existing.family_label == canonical_family && family_label != canonical_family {
                    continue;
                }
                if family_label == canonical_family && existing.family_label != canonical_family {
                    owned_rows.insert(
                        key,
                        VcfAdapterOutputCoverageOwnedRow {
                            family_label: family_label.to_string(),
                            row: normalized_row,
                        },
                    );
                    continue;
                }
                bail!(
                    "VCF adapter output coverage found conflicting owned rows `{}` / `{}` for canonical family `{}` between `{}` and `{}`",
                    normalized_row.stage_id,
                    normalized_row.tool_id,
                    canonical_family,
                    existing.family_label,
                    family_label
                );
            }
        }
    }
    Ok(())
}

fn normalize_serialized_row(
    family_label: &str,
    value: Value,
) -> Result<VcfAdapterOutputCoverageRow> {
    let stage_id = required_str(&value, "stage_id")?;
    let tool_id = required_str(&value, "tool_id")?;
    let benchmark_status = required_str(&value, "benchmark_status")?;
    let stage_output_ids = required_string_vec(&value, "stage_output_ids")?;
    let raw_output_ids = required_string_vec(&value, "raw_output_ids")?;
    let declared_outputs = declared_artifacts(&value, "declared_outputs")?;
    let declared_by_id = declared_outputs
        .iter()
        .cloned()
        .map(|artifact| (artifact.artifact_id.clone(), artifact))
        .collect::<BTreeMap<_, _>>();

    let normalized_metrics = stage_output_ids
        .iter()
        .filter_map(|artifact_id| declared_by_id.get(artifact_id))
        .map(render_artifact_entry)
        .collect::<Vec<_>>();
    let raw_outputs = raw_output_ids
        .iter()
        .filter_map(|artifact_id| declared_by_id.get(artifact_id))
        .filter(|artifact| !is_index_artifact(artifact))
        .map(render_artifact_entry)
        .collect::<Vec<_>>();
    let index_outputs = declared_outputs
        .iter()
        .filter(|artifact| is_index_artifact(artifact))
        .map(render_artifact_entry)
        .collect::<Vec<_>>();

    let stdout = format!("{}/stdout.log", path_template_root(&stage_id, &tool_id));
    let stderr = format!("{}/stderr.log", path_template_root(&stage_id, &tool_id));
    let manifest = format!("{}/stage-result.json", path_template_root(&stage_id, &tool_id));

    let mut missing_declarations = Vec::new();
    if stage_output_ids.is_empty() {
        missing_declarations.push("stage_output_ids".to_string());
    }
    for artifact_id in &stage_output_ids {
        if !declared_by_id.contains_key(artifact_id) {
            missing_declarations.push(format!("tool.outputs:{artifact_id}"));
        }
    }
    if normalized_metrics.is_empty() {
        missing_declarations.push("normalized_metrics".to_string());
    }
    if raw_output_ids.is_empty() {
        missing_declarations.push("raw_outputs".to_string());
    }
    for artifact_id in &raw_output_ids {
        if !declared_by_id.contains_key(artifact_id) {
            missing_declarations.push(format!("tool.outputs:{artifact_id}"));
        }
    }
    if requires_index_outputs(&declared_outputs) && index_outputs.is_empty() {
        missing_declarations.push("index_outputs".to_string());
    }

    let status = if missing_declarations.is_empty() {
        VcfAdapterOutputCoverageStatus::Complete
    } else {
        VcfAdapterOutputCoverageStatus::Incomplete
    };
    let reason = if status == VcfAdapterOutputCoverageStatus::Complete {
        format!(
            "row `{stage_id}` / `{tool_id}` from `{family_label}` keeps raw outputs, normalized outputs, and deterministic stream/result paths explicit"
        )
    } else {
        format!(
            "row `{stage_id}` / `{tool_id}` from `{family_label}` is missing output declarations: {}",
            missing_declarations.join(", ")
        )
    };

    Ok(VcfAdapterOutputCoverageRow {
        stage_id,
        tool_id,
        raw_outputs,
        normalized_metrics,
        stdout,
        stderr,
        manifest,
        index_outputs,
        status,
        benchmark_status,
        missing_declarations,
        reason,
    })
}

fn ensure_vcf_adapter_output_coverage_contract(
    rows: &[VcfAdapterOutputCoverageRow],
    benchmark_ready_pairs: &BTreeSet<(String, String)>,
) -> Result<()> {
    let mut seen = BTreeSet::<(&str, &str)>::new();
    for row in rows {
        if !seen.insert((row.stage_id.as_str(), row.tool_id.as_str())) {
            bail!(
                "VCF adapter output coverage contains duplicate row `{}` / `{}`",
                row.stage_id,
                row.tool_id
            );
        }
    }

    let actual_benchmark_ready_pairs = rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
        .collect::<BTreeSet<_>>();
    if &actual_benchmark_ready_pairs != benchmark_ready_pairs {
        bail!(
            "VCF adapter output coverage drifted from the benchmark-ready matrix rows: expected {:?}, found {:?}",
            benchmark_ready_pairs,
            actual_benchmark_ready_pairs
        );
    }

    for row in rows.iter().filter(|row| row.benchmark_status == "benchmark_ready") {
        if row.status != VcfAdapterOutputCoverageStatus::Complete {
            bail!(
                "benchmark-ready VCF row `{}` / `{}` is missing governed output declarations: {}",
                row.stage_id,
                row.tool_id,
                row.missing_declarations.join(", ")
            );
        }
    }

    Ok(())
}

fn render_vcf_adapter_output_coverage_tsv(rows: &[VcfAdapterOutputCoverageRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\traw_outputs\tnormalized_metrics\tstdout\tstderr\tmanifest\tindex_outputs\tstatus\tbenchmark_status\tmissing_declarations\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.stage_id),
            sanitize_tsv(&row.tool_id),
            sanitize_tsv(&row.raw_outputs.join(",")),
            sanitize_tsv(&row.normalized_metrics.join(",")),
            sanitize_tsv(&row.stdout),
            sanitize_tsv(&row.stderr),
            sanitize_tsv(&row.manifest),
            sanitize_tsv(&row.index_outputs.join(",")),
            sanitize_tsv(status_label(row.status)),
            sanitize_tsv(&row.benchmark_status),
            sanitize_tsv(&row.missing_declarations.join(",")),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn required_str(value: &Value, field: &str) -> Result<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("VCF adapter output coverage row is missing `{field}`"))
}

fn required_string_vec(value: &Value, field: &str) -> Result<Vec<String>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("VCF adapter output coverage row is missing `{field}`"))?
        .iter()
        .map(|item| {
            item.as_str().map(str::to_string).ok_or_else(|| {
                anyhow!("VCF adapter output coverage row `{field}` must contain strings")
            })
        })
        .collect()
}

fn declared_artifacts(value: &Value, field: &str) -> Result<Vec<DeclaredArtifact>> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("VCF adapter output coverage row is missing `{field}`"))?
        .iter()
        .map(|artifact| {
            Ok(DeclaredArtifact {
                artifact_id: artifact
                    .get("artifact_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("declared output is missing `artifact_id`"))?
                    .to_string(),
                role: artifact
                    .get("role")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("declared output is missing `role`"))?
                    .to_string(),
                path: artifact
                    .get("path")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow!("declared output is missing `path`"))?
                    .to_string(),
            })
        })
        .collect()
}

fn render_artifact_entry(artifact: &DeclaredArtifact) -> String {
    format!("{}={}", artifact.artifact_id, artifact.path)
}

fn is_index_artifact(artifact: &DeclaredArtifact) -> bool {
    artifact.role == "index"
        || artifact.path.ends_with(".tbi")
        || artifact.path.ends_with(".csi")
        || artifact.artifact_id.ends_with("_tbi")
        || artifact.artifact_id.ends_with("_csi")
}

fn requires_index_outputs(declared_outputs: &[DeclaredArtifact]) -> bool {
    declared_outputs.iter().any(|artifact| {
        !is_index_artifact(artifact)
            && (artifact.path.ends_with(".vcf.gz") || artifact.path.ends_with(".bcf"))
    })
}

fn path_template_root(stage_id: &str, tool_id: &str) -> String {
    format!(
        "runs/bench/slurm-dry-run/runs/{}/{}/{}/{}/{}",
        LOCAL_SLURM_DRY_RUN_RUN_ID, "{fixture_scope}", stage_id, "{sample_scope}", tool_id
    )
}

fn canonical_family_for_pair(stage_id: &str, tool_id: &str) -> Option<&'static str> {
    match (stage_id, tool_id) {
        ("vcf.roh", "plink2") => Some("descent"),
        _ => None,
    }
}

fn status_label(status: VcfAdapterOutputCoverageStatus) -> &'static str {
    match status {
        VcfAdapterOutputCoverageStatus::Complete => "complete",
        VcfAdapterOutputCoverageStatus::Incomplete => "incomplete",
    }
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_adapter_output_coverage, VcfAdapterOutputCoverageStatus,
        DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH, VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_adapter_output_coverage_tracks_governed_rows() {
        let root = repo_root();
        let report = render_vcf_adapter_output_coverage(
            &root,
            PathBuf::from(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH),
        )
        .expect("render VCF adapter output coverage");

        assert_eq!(report.schema_version, VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION);
        assert_eq!(report.row_count, 39);
        assert_eq!(report.benchmark_ready_row_count, 13);
        assert_eq!(report.benchmark_ready_complete_row_count, 13);
        assert_eq!(report.benchmark_ready_incomplete_row_count, 0);
        assert_eq!(report.complete_row_count, 36);
        assert_eq!(report.incomplete_row_count, 3);

        let call = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.call" && row.tool_id == "bcftools")
            .expect("bcftools call row");
        assert_eq!(call.status, VcfAdapterOutputCoverageStatus::Complete);
        assert_eq!(call.benchmark_status, "benchmark_ready");
        assert!(call.index_outputs.iter().any(|entry| entry.contains(".tbi")));

        let stats = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.stats" && row.tool_id == "bcftools")
            .expect("bcftools stats row");
        assert_eq!(stats.status, VcfAdapterOutputCoverageStatus::Complete);
        assert!(stats.index_outputs.is_empty());
        assert!(stats.normalized_metrics.iter().any(|entry| entry.starts_with("stats_json=")));

        let qc = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.qc" && row.tool_id == "bcftools")
            .expect("bcftools qc row");
        assert_eq!(qc.status, VcfAdapterOutputCoverageStatus::Complete);
        assert_eq!(qc.benchmark_status, "benchmark_ready");
        assert_eq!(qc.raw_outputs.len(), 6);
        assert!(qc.normalized_metrics.iter().any(|entry| entry.starts_with("qc_report=")));

        let reference_panel = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.prepare_reference_panel" && row.tool_id == "bcftools")
            .expect("bcftools reference-panel row");
        assert_eq!(reference_panel.status, VcfAdapterOutputCoverageStatus::Complete);
        assert_eq!(reference_panel.benchmark_status, "benchmark_ready");
        assert!(reference_panel.index_outputs.iter().any(|entry| entry.contains(".tbi")));
        assert!(reference_panel
            .normalized_metrics
            .iter()
            .any(|entry| entry.starts_with("chunks_json=")));

        let shapeit5 = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit5")
            .expect("shapeit5 row");
        assert_eq!(shapeit5.benchmark_status, "not_benchmark_ready");
        assert_eq!(shapeit5.status, VcfAdapterOutputCoverageStatus::Complete);
        assert!(shapeit5.index_outputs.iter().any(|entry| entry.contains(".tbi")));

        let demography = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.demography" && row.tool_id == "ibdne")
            .expect("ibdne row");
        assert_eq!(demography.status, VcfAdapterOutputCoverageStatus::Complete);
        assert!(demography.index_outputs.is_empty());
        assert!(demography.manifest.ends_with("/stage-result.json"));

        let roh_row_count = report
            .rows
            .iter()
            .filter(|row| row.stage_id == "vcf.roh" && row.tool_id == "plink2")
            .count();
        assert_eq!(roh_row_count, 1);

        let angsd_gl = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.call_gl" && row.tool_id == "angsd")
            .expect("angsd call_gl row");
        assert_eq!(angsd_gl.status, VcfAdapterOutputCoverageStatus::Incomplete);
        assert_eq!(angsd_gl.missing_declarations, vec!["index_outputs"]);
    }
}
