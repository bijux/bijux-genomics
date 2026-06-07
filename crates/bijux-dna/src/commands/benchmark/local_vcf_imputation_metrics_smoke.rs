use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::local_stage_result_manifest::{
    path_relative_to_repo, validate_stage_result_manifest, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_impute_smoke::run_local_vcf_impute_smoke;
use crate::commands::cli::parse;
use crate::commands::cli::render;

const DEFAULT_VCF_IMPUTATION_METRICS_SMOKE_ROOT: &str =
    "runs/bench/local-smoke/vcf.imputation_metrics";
const LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_imputation_metrics_smoke.v1";
const LOCAL_VCF_IMPUTATION_METRICS_COMMAND: &str =
    "bijux-dna bench local run-vcf-imputation-metrics-smoke";
const LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID: &str = "vcf.imputation_metrics";
const DEFAULT_OUTPUT_REPORT_NAME: &str = "imputation_metrics.json";
const DEFAULT_OUTPUT_SOURCE_QC_NAME: &str = "source_imputation_qc.json";
const DEFAULT_OUTPUT_SOURCE_SMOKE_NAME: &str = "source_impute_smoke_metrics.json";
const DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME: &str = "source_imputation_manifest.json";
const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct QualityFieldAvailability {
    concordance: bool,
    dosage_r2: bool,
    maf_strata: bool,
    masked_truth_sites: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub(crate) struct LocalVcfImputationMetricsSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) command: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_fixture_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) output_root: String,
    pub(crate) imputation_metrics_path: String,
    pub(crate) source_imputation_qc_path: String,
    pub(crate) source_impute_smoke_metrics_path: String,
    pub(crate) source_imputation_manifest_path: String,
    pub(crate) stage_result_manifest_path: String,
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) elapsed_seconds: f64,
    pub(crate) exit_code: i32,
    pub(crate) concordance: Option<f64>,
    pub(crate) mean_info_score: f64,
    pub(crate) r2_available: bool,
    pub(crate) dosage_r2: Option<f64>,
    pub(crate) low_confidence_sites: u64,
    pub(crate) masked_truth_sites: u64,
    pub(crate) quality_field_availability: QualityFieldAvailability,
    pub(crate) missing_quality_fields: Vec<String>,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq)]
struct ImputationMetricsSummary {
    concordance: Option<f64>,
    mean_info_score: f64,
    r2_available: bool,
    dosage_r2: Option<f64>,
    low_confidence_sites: u64,
    masked_truth_sites: u64,
    quality_field_availability: QualityFieldAvailability,
    missing_quality_fields: Vec<String>,
    status: String,
}

pub(crate) fn run_vcf_imputation_metrics_smoke(
    args: &parse::BenchLocalRunVcfImputationMetricsSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = run_local_vcf_imputation_metrics_smoke(&repo_root, &args.tool_id)?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.imputation_metrics_path);
    }
    Ok(())
}

pub(crate) fn run_local_vcf_imputation_metrics_smoke(
    repo_root: &Path,
    tool_id: &str,
) -> Result<LocalVcfImputationMetricsSmokeReport> {
    let source_report = run_local_vcf_impute_smoke(repo_root, tool_id)?;
    let output_root =
        repo_root.join(DEFAULT_VCF_IMPUTATION_METRICS_SMOKE_ROOT).join(&source_report.tool_id);
    if output_root.exists() {
        fs::remove_dir_all(&output_root)
            .with_context(|| format!("remove {}", output_root.display()))?;
    }
    fs::create_dir_all(&output_root)
        .with_context(|| format!("create {}", output_root.display()))?;

    let source_imputation_qc_source = repo_root.join(&source_report.imputation_qc_path);
    let source_impute_smoke_metrics_source = repo_root.join(&source_report.metrics_path);
    let source_imputation_manifest_source = repo_root.join(&source_report.imputation_manifest_path);
    let source_imputation_qc_path = output_root.join(DEFAULT_OUTPUT_SOURCE_QC_NAME);
    let source_impute_smoke_metrics_path = output_root.join(DEFAULT_OUTPUT_SOURCE_SMOKE_NAME);
    let source_imputation_manifest_path = output_root.join(DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME);

    fs::copy(&source_imputation_qc_source, &source_imputation_qc_path).with_context(|| {
        format!(
            "copy {} to {}",
            source_imputation_qc_source.display(),
            source_imputation_qc_path.display()
        )
    })?;
    fs::copy(&source_impute_smoke_metrics_source, &source_impute_smoke_metrics_path).with_context(
        || {
            format!(
                "copy {} to {}",
                source_impute_smoke_metrics_source.display(),
                source_impute_smoke_metrics_path.display()
            )
        },
    )?;
    fs::copy(&source_imputation_manifest_source, &source_imputation_manifest_path).with_context(
        || {
            format!(
                "copy {} to {}",
                source_imputation_manifest_source.display(),
                source_imputation_manifest_path.display()
            )
        },
    )?;

    let started_at = timestamp_marker();
    let started = Instant::now();
    let source_imputation_qc = read_json(&source_imputation_qc_path)?;
    let summary = summarize_imputation_metrics(&source_imputation_qc)?;

    let imputation_metrics_path = output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
    let report_payload = LocalVcfImputationMetricsSmokeReport {
        schema_version: LOCAL_VCF_IMPUTATION_METRICS_SMOKE_SCHEMA_VERSION,
        command: format!(
            "{LOCAL_VCF_IMPUTATION_METRICS_COMMAND} --tool-id {}",
            source_report.tool_id
        ),
        stage_id: LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID.to_string(),
        tool_id: source_report.tool_id.clone(),
        corpus_id: source_report.corpus_id.clone(),
        input_fixture_id: source_report.input_fixture_id.clone(),
        panel_id: source_report.panel_id.clone(),
        map_id: source_report.map_id.clone(),
        output_root: path_relative_to_repo(repo_root, &output_root),
        imputation_metrics_path: path_relative_to_repo(repo_root, &imputation_metrics_path),
        source_imputation_qc_path: path_relative_to_repo(repo_root, &source_imputation_qc_path),
        source_impute_smoke_metrics_path: path_relative_to_repo(
            repo_root,
            &source_impute_smoke_metrics_path,
        ),
        source_imputation_manifest_path: path_relative_to_repo(
            repo_root,
            &source_imputation_manifest_path,
        ),
        stage_result_manifest_path: path_relative_to_repo(
            repo_root,
            &output_root.join(DEFAULT_STAGE_RESULT_NAME),
        ),
        started_at: started_at.clone(),
        finished_at: String::new(),
        elapsed_seconds: 0.0,
        exit_code: 0,
        concordance: summary.concordance,
        mean_info_score: summary.mean_info_score,
        r2_available: summary.r2_available,
        dosage_r2: summary.dosage_r2,
        low_confidence_sites: summary.low_confidence_sites,
        masked_truth_sites: summary.masked_truth_sites,
        quality_field_availability: summary.quality_field_availability.clone(),
        missing_quality_fields: summary.missing_quality_fields.clone(),
        status: summary.status.clone(),
    };
    bijux_dna_infra::atomic_write_json(&imputation_metrics_path, &report_payload)?;

    let elapsed_seconds = started.elapsed().as_secs_f64();
    let finished_at = timestamp_marker();
    let stage_result_manifest_path = output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let stage_result_manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: LOCAL_VCF_IMPUTATION_METRICS_STAGE_ID.to_string(),
        tool: BenchStageResultToolV1 { id: source_report.tool_id.clone() },
        command: BenchStageResultCommandV1 {
            rendered: format!(
                "{LOCAL_VCF_IMPUTATION_METRICS_COMMAND} --tool-id {}",
                source_report.tool_id
            ),
        },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: started_at.clone(),
            finished_at: finished_at.clone(),
            elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: vec![
            BenchStageResultOutputV1 {
                artifact_id: "imputation_metrics_json".to_string(),
                declared_path: DEFAULT_OUTPUT_REPORT_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &imputation_metrics_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_imputation_qc_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_QC_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_imputation_qc_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_impute_smoke_metrics_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_SMOKE_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_impute_smoke_metrics_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
            BenchStageResultOutputV1 {
                artifact_id: "source_imputation_manifest_json".to_string(),
                declared_path: DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME.to_string(),
                realized_path: path_relative_to_repo(repo_root, &source_imputation_manifest_path),
                role: "report_output".to_string(),
                optional: false,
                exists: true,
            },
        ],
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    bijux_dna_infra::atomic_write_json(&stage_result_manifest_path, &stage_result_manifest)?;

    Ok(LocalVcfImputationMetricsSmokeReport {
        finished_at,
        elapsed_seconds,
        stage_result_manifest_path: path_relative_to_repo(repo_root, &stage_result_manifest_path),
        ..report_payload
    })
}

fn summarize_imputation_metrics(
    imputation_qc: &serde_json::Value,
) -> Result<ImputationMetricsSummary> {
    let concordance = imputation_qc
        .pointer("/concordance/genotype_concordance")
        .and_then(serde_json::Value::as_f64);
    let dosage_r2 =
        imputation_qc.pointer("/concordance/dosage_r2").and_then(serde_json::Value::as_f64);
    let maf_strata_present = imputation_qc
        .pointer("/concordance/maf_strata")
        .and_then(serde_json::Value::as_array)
        .is_some();
    let masked_truth_sites = imputation_qc
        .pointer("/concordance/masked_truth_site_count")
        .and_then(serde_json::Value::as_u64);
    let mean_info_score = imputation_qc
        .get("imputation_info_mean")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing `imputation_info_mean` in imputation QC payload"))?;
    let low_confidence_sites = imputation_qc
        .get("low_confidence_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("missing `low_confidence_count` in imputation QC payload"))?;

    let quality_field_availability = QualityFieldAvailability {
        concordance: concordance.is_some(),
        dosage_r2: dosage_r2.is_some(),
        maf_strata: maf_strata_present,
        masked_truth_sites: masked_truth_sites.is_some(),
    };
    let mut missing_quality_fields = Vec::<String>::new();
    if !quality_field_availability.concordance {
        missing_quality_fields.push("concordance".to_string());
    }
    if !quality_field_availability.dosage_r2 {
        missing_quality_fields.push("dosage_r2".to_string());
    }
    if !quality_field_availability.maf_strata {
        missing_quality_fields.push("maf_strata".to_string());
    }
    if !quality_field_availability.masked_truth_sites {
        missing_quality_fields.push("masked_truth_sites".to_string());
    }
    let status = if missing_quality_fields.is_empty() {
        "complete".to_string()
    } else {
        "explicit_missing_quality_fields".to_string()
    };

    Ok(ImputationMetricsSummary {
        concordance,
        mean_info_score,
        r2_available: dosage_r2.is_some(),
        dosage_r2,
        low_confidence_sites,
        masked_truth_sites: masked_truth_sites.unwrap_or(0),
        quality_field_availability,
        missing_quality_fields,
        status,
    })
}

fn read_json(path: &Path) -> Result<serde_json::Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

fn timestamp_marker() -> String {
    let seconds =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_secs());
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::{run_local_vcf_imputation_metrics_smoke, summarize_imputation_metrics};

    #[test]
    fn imputation_metrics_summary_reports_missing_quality_fields_explicitly() {
        let qc = serde_json::json!({
            "imputation_info_mean": 0.81,
            "low_confidence_count": 3,
            "concordance": {
                "genotype_concordance": serde_json::Value::Null,
                "dosage_r2": serde_json::Value::Null
            }
        });
        let summary = summarize_imputation_metrics(&qc).expect("summarize quality payload");
        assert_eq!(summary.concordance, None);
        assert!(!summary.r2_available);
        assert_eq!(summary.masked_truth_sites, 0);
        assert_eq!(
            summary.missing_quality_fields,
            vec![
                "concordance".to_string(),
                "dosage_r2".to_string(),
                "maf_strata".to_string(),
                "masked_truth_sites".to_string()
            ]
        );
        assert_eq!(summary.status, "explicit_missing_quality_fields");
    }

    #[test]
    fn governed_vcf_imputation_metrics_smoke_reports_quality_surface() {
        let repo_root = tempfile::tempdir().expect("tempdir");
        let report = run_local_vcf_imputation_metrics_smoke(repo_root.path(), "beagle")
            .expect("run local imputation metrics smoke");
        assert_eq!(report.stage_id, "vcf.imputation_metrics");
        assert_eq!(report.tool_id, "beagle");
        assert_eq!(report.concordance, Some(1.0));
        assert!(report.mean_info_score > 0.8);
        assert!(report.r2_available);
        assert!(report.dosage_r2.is_some_and(|value| value > 0.7));
        assert_eq!(report.low_confidence_sites, 1);
        assert_eq!(report.masked_truth_sites, 1);
        assert!(report.missing_quality_fields.is_empty());
        assert_eq!(report.status, "complete");
    }
}
