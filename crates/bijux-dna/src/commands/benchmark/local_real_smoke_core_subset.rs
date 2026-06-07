use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::local_stage_commands::materialize_local_stage;
use super::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, path_relative_to_repo, BenchStageResultManifestV1,
    BenchStageResultStatus,
};
use super::local_vcf_call_smoke::run_local_vcf_call_smoke;
use super::local_vcf_stats_smoke::run_local_vcf_stats_smoke;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH: &str =
    "target/local-real-smoke/core-subset/REAL_SMOKE_SUMMARY.json";
const REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION: &str = "bijux.bench.local_real_smoke_core_subset.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RealSmokeCoreSubsetExecutionKind {
    Stage,
    PipelineBridge,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RealSmokeCoreSubsetRow {
    pub(crate) execution_id: String,
    pub(crate) execution_kind: RealSmokeCoreSubsetExecutionKind,
    pub(crate) domain: String,
    pub(crate) bridge_source_domain: Option<String>,
    pub(crate) bridge_target_domain: Option<String>,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) evidence_path: String,
    pub(crate) parsed_schema_version: String,
    pub(crate) stage_result_manifest_path: Option<String>,
    pub(crate) manifest_status: Option<String>,
    pub(crate) manifest_exit_code: Option<i32>,
    pub(crate) normalized_metric_count: usize,
    pub(crate) normalized_metrics: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RealSmokeCoreSubsetReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) execution_count: usize,
    pub(crate) stage_execution_count: usize,
    pub(crate) pipeline_bridge_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) passes_behavior_test: bool,
    pub(crate) rows: Vec<RealSmokeCoreSubsetRow>,
}

pub(crate) fn run_real_smoke_core_subset(
    args: &parse::BenchLocalRunRealSmokeCoreSubsetArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_real_smoke_core_subset(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_real_smoke_core_subset(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<RealSmokeCoreSubsetReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut rows = Vec::with_capacity(4);
    rows.push(collect_fastq_validate_row(
        repo_root,
        &materialize_local_stage(repo_root, "fastq.validate_reads")
            .context("materialize fastq.validate_reads real-smoke report")?,
    )?);
    rows.push(collect_bam_validate_row(
        repo_root,
        &materialize_local_stage(repo_root, "bam.validate")
            .context("materialize bam.validate real-smoke report")?,
    )?);

    let vcf_stats_report = run_local_vcf_stats_smoke(repo_root, "bcftools")
        .context("run governed vcf.stats real-smoke stage")?;
    rows.push(collect_vcf_stats_row(
        repo_root,
        &vcf_stats_report.metrics_path,
        &vcf_stats_report.stage_result_manifest_path,
    )?);

    let vcf_call_report = run_local_vcf_call_smoke(repo_root, "bcftools")
        .context("run governed bam-to-vcf bridge real-smoke stage")?;
    rows.push(collect_vcf_call_bridge_row(
        repo_root,
        &vcf_call_report.metrics_path,
        &vcf_call_report.stage_result_manifest_path,
    )?);

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.execution_id.cmp(&right.execution_id))
    });

    let stage_execution_count = rows
        .iter()
        .filter(|row| row.execution_kind == RealSmokeCoreSubsetExecutionKind::Stage)
        .count();
    let pipeline_bridge_count = rows.len().saturating_sub(stage_execution_count);
    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    let report = RealSmokeCoreSubsetReport {
        schema_version: REAL_SMOKE_CORE_SUBSET_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        execution_count: rows.len(),
        stage_execution_count,
        pipeline_bridge_count,
        domain_counts,
        passes_behavior_test: false,
        rows,
    };
    let report = ensure_real_smoke_core_subset_contract(repo_root, report)?;
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}

fn collect_fastq_validate_row(
    repo_root: &Path,
    report_path: &Path,
) -> Result<RealSmokeCoreSubsetRow> {
    let report = read_json_document(report_path)?;
    let normalized_metrics = BTreeMap::from([
        ("case_count".to_string(), Value::from(json_u64_field(&report, "case_count")?)),
        (
            "all_cases_passed".to_string(),
            Value::from(json_bool_field(&report, "all_cases_passed")?),
        ),
        (
            "missing_output_marker_present".to_string(),
            Value::from(json_bool_field(&report, "missing_output_marker_present")?),
        ),
    ]);

    Ok(RealSmokeCoreSubsetRow {
        execution_id: "fastq.validate_reads".to_string(),
        execution_kind: RealSmokeCoreSubsetExecutionKind::Stage,
        domain: "fastq".to_string(),
        bridge_source_domain: None,
        bridge_target_domain: None,
        stage_id: "fastq.validate_reads".to_string(),
        tool_id: "fastqc".to_string(),
        corpus_id: "local_smoke".to_string(),
        asset_profile_id: "sample_set".to_string(),
        evidence_path: path_relative_to_repo(repo_root, report_path),
        parsed_schema_version: json_string_field(&report, "schema_version")?,
        stage_result_manifest_path: None,
        manifest_status: None,
        manifest_exit_code: None,
        normalized_metric_count: normalized_metrics.len(),
        normalized_metrics,
    })
}

fn collect_bam_validate_row(
    repo_root: &Path,
    report_path: &Path,
) -> Result<RealSmokeCoreSubsetRow> {
    let report = read_json_document(report_path)?;
    let cases = report
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing `cases`", report_path.display()))?;
    let pass_case_count = cases
        .iter()
        .filter(|case| case.get("validation_status").and_then(Value::as_str) == Some("pass"))
        .count() as u64;
    let refusal_case_count = cases
        .iter()
        .filter(|case| case.get("validation_status").and_then(Value::as_str) == Some("refusal"))
        .count() as u64;
    let normalized_metrics = BTreeMap::from([
        ("case_count".to_string(), Value::from(json_u64_field(&report, "case_count")?)),
        (
            "all_cases_matched".to_string(),
            Value::from(json_bool_field(&report, "all_cases_matched")?),
        ),
        ("pass_case_count".to_string(), Value::from(pass_case_count)),
        ("refusal_case_count".to_string(), Value::from(refusal_case_count)),
    ]);

    Ok(RealSmokeCoreSubsetRow {
        execution_id: "bam.validate".to_string(),
        execution_kind: RealSmokeCoreSubsetExecutionKind::Stage,
        domain: "bam".to_string(),
        bridge_source_domain: None,
        bridge_target_domain: None,
        stage_id: "bam.validate".to_string(),
        tool_id: "samtools".to_string(),
        corpus_id: "local_smoke".to_string(),
        asset_profile_id: "sample_set".to_string(),
        evidence_path: path_relative_to_repo(repo_root, report_path),
        parsed_schema_version: json_string_field(&report, "schema_version")?,
        stage_result_manifest_path: None,
        manifest_status: None,
        manifest_exit_code: None,
        normalized_metric_count: normalized_metrics.len(),
        normalized_metrics,
    })
}

fn collect_vcf_stats_row(
    repo_root: &Path,
    metrics_relative_path: &str,
    manifest_relative_path: &str,
) -> Result<RealSmokeCoreSubsetRow> {
    let metrics_path = repo_root.join(metrics_relative_path);
    let metrics = read_json_document(&metrics_path)?;
    let manifest_path = repo_root.join(manifest_relative_path);
    let manifest = load_validated_stage_result_manifest_path(&manifest_path)
        .with_context(|| format!("load {}", manifest_path.display()))?;
    let normalized_metrics = BTreeMap::from([
        ("variant_count".to_string(), Value::from(json_u64_field(&metrics, "variant_count")?)),
        ("snp_count".to_string(), Value::from(json_u64_field(&metrics, "snp_count")?)),
        ("indel_count".to_string(), Value::from(json_u64_field(&metrics, "indel_count")?)),
        (
            "transition_count".to_string(),
            Value::from(json_u64_field(&metrics, "transition_count")?),
        ),
        (
            "transversion_count".to_string(),
            Value::from(json_u64_field(&metrics, "transversion_count")?),
        ),
        ("ti_tv".to_string(), Value::from(json_f64_field(&metrics, "ti_tv")?)),
        ("sample_count".to_string(), Value::from(json_u64_field(&metrics, "sample_count")?)),
    ]);

    Ok(RealSmokeCoreSubsetRow {
        execution_id: "vcf.stats".to_string(),
        execution_kind: RealSmokeCoreSubsetExecutionKind::Stage,
        domain: "vcf".to_string(),
        bridge_source_domain: None,
        bridge_target_domain: None,
        stage_id: "vcf.stats".to_string(),
        tool_id: "bcftools".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        asset_profile_id: "vcf_cohort".to_string(),
        evidence_path: metrics_relative_path.to_string(),
        parsed_schema_version: json_string_field(&metrics, "schema_version")?,
        stage_result_manifest_path: Some(manifest_relative_path.to_string()),
        manifest_status: Some(manifest_status_label(&manifest)),
        manifest_exit_code: Some(manifest.runtime.exit_code),
        normalized_metric_count: normalized_metrics.len(),
        normalized_metrics,
    })
}

fn collect_vcf_call_bridge_row(
    repo_root: &Path,
    metrics_relative_path: &str,
    manifest_relative_path: &str,
) -> Result<RealSmokeCoreSubsetRow> {
    let metrics_path = repo_root.join(metrics_relative_path);
    let metrics = read_json_document(&metrics_path)?;
    let manifest_path = repo_root.join(manifest_relative_path);
    let manifest = load_validated_stage_result_manifest_path(&manifest_path)
        .with_context(|| format!("load {}", manifest_path.display()))?;
    let normalized_metrics = BTreeMap::from([
        ("variant_count".to_string(), Value::from(json_u64_field(&metrics, "variant_count")?)),
        ("snp_count".to_string(), Value::from(json_u64_field(&metrics, "snp_count")?)),
        ("indel_count".to_string(), Value::from(json_u64_field(&metrics, "indel_count")?)),
        ("sample_count".to_string(), Value::from(json_u64_field(&metrics, "sample_count")?)),
        ("exit_code".to_string(), Value::from(json_i64_field(&metrics, "exit_code")?)),
    ]);

    Ok(RealSmokeCoreSubsetRow {
        execution_id: "bridge:bam-to-vcf.call".to_string(),
        execution_kind: RealSmokeCoreSubsetExecutionKind::PipelineBridge,
        domain: "vcf".to_string(),
        bridge_source_domain: Some("bam".to_string()),
        bridge_target_domain: Some("vcf".to_string()),
        stage_id: "vcf.call".to_string(),
        tool_id: "bcftools".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        asset_profile_id: "bam_bundle".to_string(),
        evidence_path: metrics_relative_path.to_string(),
        parsed_schema_version: json_string_field(&metrics, "schema_version")?,
        stage_result_manifest_path: Some(manifest_relative_path.to_string()),
        manifest_status: Some(manifest_status_label(&manifest)),
        manifest_exit_code: Some(manifest.runtime.exit_code),
        normalized_metric_count: normalized_metrics.len(),
        normalized_metrics,
    })
}

fn ensure_real_smoke_core_subset_contract(
    repo_root: &Path,
    mut report: RealSmokeCoreSubsetReport,
) -> Result<RealSmokeCoreSubsetReport> {
    if report.execution_count != 4 {
        return Err(anyhow!(
            "real-smoke core subset must keep exactly 4 executions, found {}",
            report.execution_count
        ));
    }
    if report.stage_execution_count != 3 {
        return Err(anyhow!(
            "real-smoke core subset must keep exactly 3 stage executions, found {}",
            report.stage_execution_count
        ));
    }
    if report.pipeline_bridge_count != 1 {
        return Err(anyhow!(
            "real-smoke core subset must keep exactly 1 pipeline bridge, found {}",
            report.pipeline_bridge_count
        ));
    }
    ensure_row_exists(
        &report.rows,
        RealSmokeCoreSubsetExecutionKind::Stage,
        "fastq",
        "fastq.validate_reads",
    )?;
    ensure_row_exists(
        &report.rows,
        RealSmokeCoreSubsetExecutionKind::Stage,
        "bam",
        "bam.validate",
    )?;
    ensure_row_exists(&report.rows, RealSmokeCoreSubsetExecutionKind::Stage, "vcf", "vcf.stats")?;
    let bridge_row = ensure_row_exists(
        &report.rows,
        RealSmokeCoreSubsetExecutionKind::PipelineBridge,
        "vcf",
        "vcf.call",
    )?;
    if bridge_row.bridge_source_domain.as_deref() != Some("bam")
        || bridge_row.bridge_target_domain.as_deref() != Some("vcf")
    {
        return Err(anyhow!(
            "real-smoke core subset bridge must stay bam->vcf, found {:?}->{:?}",
            bridge_row.bridge_source_domain,
            bridge_row.bridge_target_domain
        ));
    }

    for row in &report.rows {
        if row.normalized_metric_count == 0 || row.normalized_metrics.is_empty() {
            return Err(anyhow!(
                "real-smoke core subset row `{}` is missing normalized metrics",
                row.execution_id
            ));
        }
        let evidence_path = repo_root.join(&row.evidence_path);
        if !evidence_path.is_file() {
            return Err(anyhow!(
                "real-smoke core subset evidence path `{}` is missing for `{}`",
                row.evidence_path,
                row.execution_id
            ));
        }
        if let Some(manifest_path) = &row.stage_result_manifest_path {
            let manifest_abs = repo_root.join(manifest_path);
            if !manifest_abs.is_file() {
                return Err(anyhow!(
                    "real-smoke core subset manifest path `{manifest_path}` is missing for `{}`",
                    row.execution_id
                ));
            }
            if row.manifest_status.as_deref() != Some("succeeded") {
                return Err(anyhow!(
                    "real-smoke core subset manifest for `{}` must be succeeded, found {:?}",
                    row.execution_id,
                    row.manifest_status
                ));
            }
            if row.manifest_exit_code != Some(0) {
                return Err(anyhow!(
                    "real-smoke core subset manifest for `{}` must keep exit_code=0, found {:?}",
                    row.execution_id,
                    row.manifest_exit_code
                ));
            }
        }
    }

    report.passes_behavior_test = true;
    Ok(report)
}

fn ensure_row_exists<'a>(
    rows: &'a [RealSmokeCoreSubsetRow],
    execution_kind: RealSmokeCoreSubsetExecutionKind,
    domain: &str,
    stage_id: &str,
) -> Result<&'a RealSmokeCoreSubsetRow> {
    rows.iter()
        .find(|row| {
            row.execution_kind == execution_kind && row.domain == domain && row.stage_id == stage_id
        })
        .ok_or_else(|| {
            anyhow!(
                "real-smoke core subset is missing `{}` `{}` `{}`",
                execution_kind_label(execution_kind),
                domain,
                stage_id
            )
        })
}

fn execution_kind_label(kind: RealSmokeCoreSubsetExecutionKind) -> &'static str {
    match kind {
        RealSmokeCoreSubsetExecutionKind::Stage => "stage",
        RealSmokeCoreSubsetExecutionKind::PipelineBridge => "pipeline_bridge",
    }
}

fn manifest_status_label(manifest: &BenchStageResultManifestV1) -> String {
    match manifest.runtime.status {
        BenchStageResultStatus::Succeeded => "succeeded",
        BenchStageResultStatus::Failed => "failed",
    }
    .to_string()
}

fn read_json_document(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parse json from {}", path.display()))
}

fn json_string_field(document: &Value, field: &str) -> Result<String> {
    document
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("json document is missing string field `{field}`"))
}

fn json_bool_field(document: &Value, field: &str) -> Result<bool> {
    document
        .get(field)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("json document is missing bool field `{field}`"))
}

fn json_u64_field(document: &Value, field: &str) -> Result<u64> {
    document
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow!("json document is missing u64 field `{field}`"))
}

fn json_i64_field(document: &Value, field: &str) -> Result<i64> {
    document
        .get(field)
        .and_then(Value::as_i64)
        .ok_or_else(|| anyhow!("json document is missing i64 field `{field}`"))
}

fn json_f64_field(document: &Value, field: &str) -> Result<f64> {
    document
        .get(field)
        .and_then(Value::as_f64)
        .ok_or_else(|| anyhow!("json document is missing f64 field `{field}`"))
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}
