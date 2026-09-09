//! Owner: bijux-dna-bench
//! Acquire optional VCF scoring reports and dispatch format interpretation.

use crate::artifacts::vcf_score_evidence::{self as adapters, EvidenceRows};
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

type JsonAdapter = fn(&Value, &Path, &mut EvidenceRows) -> Result<()>;
enum ReportAdapter {
    Json(JsonAdapter),
    Phasing,
}
const REPORTS: &[(&str,ReportAdapter)] = &[
    (
        "artifacts/runs/bench/local-smoke/vcf.admixture/plink2/admixture.json",
        ReportAdapter::Json(adapters::merge_admixture_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.call/bcftools/metrics.json",
        ReportAdapter::Json(|root,path,rows| adapters::merge_call_summary(root,path,rows,adapters::CallCountContract::Variants)),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.call_diploid/bcftools/metrics.json",
        ReportAdapter::Json(|root,path,rows| adapters::merge_call_summary(root,path,rows,adapters::CallCountContract::DiploidGenotypes)),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.call_gl/bcftools/metrics.json",
        ReportAdapter::Json(adapters::merge_call_gl_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.call_pseudohaploid/bcftools/metrics.json",
        ReportAdapter::Json(adapters::merge_call_pseudohaploid_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.damage_filter/bcftools/metrics.json",
        ReportAdapter::Json(adapters::merge_damage_filter_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.demography/ibdne/demography.json",
        ReportAdapter::Json(adapters::merge_demography_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.filter/bcftools/metrics.json",
        ReportAdapter::Json(adapters::merge_filter_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.gl_propagation/bcftools/metrics.json",
        ReportAdapter::Json(adapters::merge_gl_propagation_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.ibd/germline/ibd.json",
        ReportAdapter::Json(adapters::merge_ibd_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.ibd/ibdseq/ibd.json",
        ReportAdapter::Json(adapters::merge_ibd_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json",
        ReportAdapter::Json(adapters::merge_imputation_metrics_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.impute/beagle/metrics.json",
        ReportAdapter::Json(adapters::merge_impute_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.pca/plink2/pca.json",
        ReportAdapter::Json(adapters::merge_pca_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.pca/eigensoft/pca.json",
        ReportAdapter::Json(adapters::merge_pca_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.phasing/shapeit5/metrics.json",
        ReportAdapter::Phasing,
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json",
        ReportAdapter::Json(adapters::merge_population_structure_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.postprocess/bcftools/metrics.json",
        ReportAdapter::Json(adapters::merge_postprocess_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/metrics.json",
        ReportAdapter::Json(adapters::merge_prepare_reference_panel_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.qc/bcftools/qc_summary.json",
        ReportAdapter::Json(|root,path,rows| adapters::merge_qc_summary(root,path,rows,"bcftools")),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.qc/plink/qc_summary.json",
        ReportAdapter::Json(|root,path,rows| adapters::merge_qc_summary(root,path,rows,"plink")),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.qc/plink2/qc_summary.json",
        ReportAdapter::Json(|root,path,rows| adapters::merge_qc_summary(root,path,rows,"plink2")),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.roh/plink2/roh.json",
        ReportAdapter::Json(adapters::merge_roh_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/vcf.stats/bcftools/stats.json",
        ReportAdapter::Json(adapters::merge_stats_summary),
    ),
];

/// Acquire VCF report evidence in governed source priority order.
///
/// # Errors
/// Returns acquisition, JSON shape or observation interpretation errors with source context.
pub fn load_vcf_tool_score_evidence(repo_root: &Path) -> Result<EvidenceRows> {
    let mut rows = EvidenceRows::new();
    for (relative_path, adapter) in REPORTS {
        let path = repo_root.join(relative_path);
        let Some(value) = load_optional_json_value(&path)? else {
            continue;
        };
        match adapter {
            ReportAdapter::Json(parse) => parse(&value, &path, &mut rows)?,
            ReportAdapter::Phasing => {
                let result_path = repo_root.join(
                    "artifacts/runs/bench/local-smoke/vcf/vcf.phasing/shapeit5/stage-result.json",
                );
                let result = load_optional_json_value(&result_path)?;
                adapters::merge_phasing_summary(
                    &value,
                    &path,
                    result.as_ref(),
                    &result_path,
                    &mut rows,
                )?;
            }
        }
    }
    Ok(rows)
}

fn load_optional_json_value(path: &Path) -> Result<Option<Value>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display())).map(Some)
}
