//! Owner: bijux-dna-bench
//! Acquire optional BAM scoring reports and dispatch format-specific interpretation.

use crate::artifacts::bam_score_evidence::{self as adapters, EvidenceRows};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

type JsonAdapter = fn(&Value, &Path, &mut EvidenceRows) -> Result<()>;
type TsvAdapter = fn(&[BTreeMap<String, String>], &Path, &mut EvidenceRows) -> Result<()>;
enum ReportAdapter {
    Json(JsonAdapter),
    Tsv(TsvAdapter),
}

const REPORTS: &[(&str, ReportAdapter)] = &[
    (
        "artifacts/runs/bench/local-smoke/bam.validate/validation.json",
        ReportAdapter::Json(adapters::merge_validate_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.qc_pre/qc_pre.json",
        ReportAdapter::Json(adapters::merge_qc_pre_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.mapping_summary/mapping_summary.tsv",
        ReportAdapter::Tsv(adapters::merge_mapping_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.filter/filter_metrics.json",
        ReportAdapter::Json(adapters::merge_filter_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.mapq_filter/mapq_filter.json",
        ReportAdapter::Json(adapters::merge_mapq_filter_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.length_filter/length_filter.json",
        ReportAdapter::Json(adapters::merge_length_filter_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.markdup/duplicates.json",
        ReportAdapter::Json(adapters::merge_markdup_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.duplication_metrics/duplication_metrics.json",
        ReportAdapter::Json(adapters::merge_duplication_metrics_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.coverage/coverage.tsv",
        ReportAdapter::Tsv(adapters::merge_coverage_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.damage/damage.json",
        ReportAdapter::Json(adapters::merge_damage_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.authenticity/authenticity.json",
        ReportAdapter::Json(adapters::merge_authenticity_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.contamination/local_smoke.json",
        ReportAdapter::Json(adapters::merge_contamination_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.sex/tool_smoke.json",
        ReportAdapter::Json(adapters::merge_sex_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.haplogroups/haplogroups.json",
        ReportAdapter::Json(adapters::merge_haplogroups_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.complexity/complexity.json",
        ReportAdapter::Json(adapters::merge_complexity_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.endogenous_content/endogenous_content.json",
        ReportAdapter::Json(adapters::merge_endogenous_content_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.insert_size/insert_size.json",
        ReportAdapter::Json(adapters::merge_insert_size_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.kinship/kinship.json",
        ReportAdapter::Json(adapters::merge_kinship_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.overlap_correction/overlap_correction.json",
        ReportAdapter::Json(adapters::merge_overlap_correction_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.recalibration/recalibration.json",
        ReportAdapter::Json(adapters::merge_recalibration_summary),
    ),
    (
        "artifacts/runs/bench/local-smoke/bam.gc_bias/gc_bias.tsv",
        ReportAdapter::Tsv(adapters::merge_gc_bias_summary),
    ),
];

/// Acquire optional BAM reports in their governed merge order.
///
/// # Errors
/// Returns report read, format or required-field errors with source context.
pub fn load_bam_tool_score_evidence(repo_root: &Path) -> Result<EvidenceRows> {
    let mut rows = EvidenceRows::new();
    for (relative_path, adapter) in REPORTS {
        let path = repo_root.join(relative_path);
        match adapter {
            ReportAdapter::Json(parse) => {
                if let Some(value) = load_optional_json_value(&path)? {
                    parse(&value, &path, &mut rows)?;
                }
            }
            ReportAdapter::Tsv(parse) => {
                if path.is_file() {
                    let raw = fs::read_to_string(&path)
                        .with_context(|| format!("read {}", path.display()))?;
                    let records = adapters::parse_tsv_rows(&raw, &path)?;
                    parse(&records, &path, &mut rows)?;
                }
            }
        }
    }
    Ok(rows)
}

fn load_optional_json_value(path: &Path) -> Result<Option<Value>> {
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let value =
        serde_json::from_str::<Value>(&raw).with_context(|| format!("parse {}", path.display()))?;
    if matches!(value, Value::Object(ref map) if map.is_empty()) {
        return Ok(None);
    }
    Ok(Some(value))
}
