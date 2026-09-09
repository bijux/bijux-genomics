//! Owner: bijux-dna-bench
//! Interpret observed BAM alignment report fields without filesystem access.

use super::fields::{
    format_f64, fraction_u64, optional_str, parse_bool, parse_f64, parse_u64, required_array,
    required_bool, required_f64, required_str, required_tsv, required_u64, summarize_metrics,
    unique_strings,
};
use super::merge::insert_evidence;
use super::{BamEvidenceAggregate, EvidenceRows};
use anyhow::Result;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn merge_validate_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = required_str(root, "stage_id")?;
    let cases = required_array(root, "cases")?;
    let case_count = required_u64(root, "case_count")?;
    anyhow::ensure!(
        u64::try_from(cases.len())? == case_count,
        "BAM validation case_count does not match the observed cases"
    );
    let mut matched_cases = 0_u64;
    let mut pass_cases = 0_u64;
    let mut retained_reads = 0_u64;
    for case in cases {
        matched_cases += u64::from(required_bool(case, "expectation_matched")?);
        pass_cases += u64::from(required_str(case, "validation_status")? == "pass");
        if let Some(value) = case.get("total_reads").filter(|value| !value.is_null()) {
            let reads = value.as_u64().ok_or_else(|| {
                anyhow::anyhow!("BAM validation total_reads must be an unsigned count")
            })?;
            retained_reads = retained_reads
                .checked_add(reads)
                .ok_or_else(|| anyhow::anyhow!("BAM validation total_reads overflow"))?;
        }
    }
    anyhow::ensure!(
        required_bool(root, "all_cases_matched")? == (matched_cases == case_count),
        "BAM validation all_cases_matched contradicts observed cases"
    );
    let validation_statuses = unique_strings(
        cases.iter().filter_map(|case| optional_str(case, "validation_status").map(str::to_string)),
    );
    let refusal_codes = unique_strings(cases.iter().flat_map(|case| {
        case.get("refusal_codes")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
    }));
    let summary = summarize_metrics(vec![
        format!("case_count={case_count}"),
        format!("matched_cases={matched_cases}"),
        format!("pass_cases={pass_cases}"),
        format!("validation_statuses={}", validation_statuses.join(",")),
        format!("refusal_codes={}", refusal_codes.join(",")),
    ]);
    insert_evidence(
        rows,
        stage_id,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(matched_cases, case_count)?),
            truth_correctness_basis: Some("case_expectation_match_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(root, "all_cases_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "validation smoke cases covered governed pass and refusal expectations".to_string(),
            ),
            retained_reads: Some(retained_reads),
            alignment_qc_metric_value: Some(fraction_u64(pass_cases, case_count)?),
            alignment_qc_metric_basis: Some("pass_case_fraction".to_string()),
            scientific_metric_summary: summary,
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_qc_pre_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let stage_id = required_str(root, "stage_id")?;
    for case in required_array(root, "cases")? {
        let tool_id = required_str(case, "tool_id")?;
        let total_reads = required_u64(case, "total_reads")?;
        let mapped_reads = required_u64(case, "mapped_reads")?;
        let unmapped_reads = required_u64(case, "unmapped_reads")?;
        let duplicate_flagged_reads = required_u64(case, "duplicate_flagged_reads")?;
        let summary = summarize_metrics(vec![
            format!("mapped_reads={mapped_reads}"),
            format!("unmapped_reads={unmapped_reads}"),
            format!("duplicate_flagged_reads={duplicate_flagged_reads}"),
        ]);
        insert_evidence(
            rows,
            stage_id,
            tool_id,
            BamEvidenceAggregate {
                truth_correctness_score: Some(fraction_u64(mapped_reads, total_reads)?),
                truth_correctness_basis: Some("mapped_fraction".to_string()),
                contract_correctness_score: Some(if required_bool(case, "expectation_matched")? {
                    1.0
                } else {
                    0.0
                }),
                contract_correctness_basis: Some(
                    "pre-QC smoke report preserved governed mapping and duplicate surfaces"
                        .to_string(),
                ),
                retained_reads: Some(total_reads),
                alignment_qc_metric_value: Some(fraction_u64(mapped_reads, total_reads)?),
                alignment_qc_metric_basis: Some("mapped_fraction".to_string()),
                scientific_metric_summary: summary,
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            path,
        );
    }
    Ok(())
}

pub(crate) fn merge_mapping_summary(
    tsv_rows: &[BTreeMap<String, String>],
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    for row in tsv_rows {
        let stage_id = bijux_dna_core::id_catalog::BAM_MAPPING_SUMMARY;
        let total_reads = parse_u64(required_tsv(row, "total_reads")?)?;
        let mapped_reads = parse_u64(required_tsv(row, "mapped_reads")?)?;
        let unmapped_reads = parse_u64(required_tsv(row, "unmapped_reads")?)?;
        let secondary_reads = parse_u64(required_tsv(row, "secondary_reads")?)?;
        let supplementary_reads = parse_u64(required_tsv(row, "supplementary_reads")?)?;
        let mapping_fraction = parse_f64(required_tsv(row, "mapping_fraction")?)?;
        let summary = summarize_metrics(vec![
            format!("mapped_reads={mapped_reads}"),
            format!("unmapped_reads={unmapped_reads}"),
            format!("secondary_reads={secondary_reads}"),
            format!("supplementary_reads={supplementary_reads}"),
        ]);
        insert_evidence(
            rows,
            stage_id,
            "samtools",
            BamEvidenceAggregate {
                truth_correctness_score: Some(mapping_fraction),
                truth_correctness_basis: Some("mapping_fraction".to_string()),
                contract_correctness_score: Some(
                    if parse_bool(required_tsv(row, "expectation_matched")?)? { 1.0 } else { 0.0 },
                ),
                contract_correctness_basis: Some(
                    "mapping summary TSV preserved governed mapped and unmapped count surfaces"
                        .to_string(),
                ),
                retained_reads: Some(total_reads),
                alignment_qc_metric_value: Some(mapping_fraction),
                alignment_qc_metric_basis: Some("mapping_fraction".to_string()),
                scientific_metric_summary: summary,
                source_paths: BTreeSet::new(),
                ..BamEvidenceAggregate::default()
            },
            path,
        );
    }
    Ok(())
}

pub(crate) fn merge_filter_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let input_reads = required_u64(root, "input_reads")?;
    let kept_reads = required_u64(root, "kept_reads")?;
    let removed_reads = required_u64(root, "removed_reads")?;
    let active_filters = unique_strings(
        required_array(root, "active_filters")?
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string),
    );
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(kept_reads, input_reads)?),
            truth_correctness_basis: Some("kept_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "filter smoke report preserved governed kept and removed read counts".to_string(),
            ),
            retained_reads: Some(kept_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some(fraction_u64(kept_reads, input_reads)?),
            alignment_qc_metric_basis: Some("kept_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("input_reads={input_reads}"),
                format!("kept_reads={kept_reads}"),
                format!("removed_reads={removed_reads}"),
                format!("active_filters={}", active_filters.join(",")),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_mapq_filter_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let input_reads = required_u64(root, "input_reads")?;
    let kept_reads = required_u64(root, "kept_reads")?;
    let removed_reads = required_u64(root, "removed_reads")?;
    let mapped_fraction_retained = required_f64(root, "mapped_fraction_retained")?;
    let mapq_threshold = required_u64(root, "mapq_threshold")?;
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(kept_reads, input_reads)?),
            truth_correctness_basis: Some("kept_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "mapq filter report preserved governed retained and removed read counts"
                    .to_string(),
            ),
            retained_reads: Some(kept_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some(mapped_fraction_retained),
            alignment_qc_metric_basis: Some("mapped_fraction_retained".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("kept_reads={kept_reads}"),
                format!("removed_reads={removed_reads}"),
                format!("mapq_threshold={mapq_threshold}"),
                format!("mapped_fraction_retained={}", format_f64(mapped_fraction_retained)),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}

pub(crate) fn merge_length_filter_summary(
    root: &Value,
    path: &Path,
    rows: &mut EvidenceRows,
) -> Result<()> {
    let input_reads = required_u64(root, "input_reads")?;
    let kept_reads = required_u64(root, "kept_reads")?;
    let removed_reads = required_u64(root, "removed_reads")?;
    let min_length_threshold = required_u64(root, "min_length_threshold")?;
    insert_evidence(
        rows,
        required_str(root, "stage_id")?,
        "samtools",
        BamEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(kept_reads, input_reads)?),
            truth_correctness_basis: Some("kept_fraction".to_string()),
            contract_correctness_score: Some(if required_bool(root, "expectation_matched")? {
                1.0
            } else {
                0.0
            }),
            contract_correctness_basis: Some(
                "length filter report preserved governed retained and removed read counts"
                    .to_string(),
            ),
            retained_reads: Some(kept_reads),
            dropped_reads: Some(removed_reads),
            alignment_qc_metric_value: Some(fraction_u64(kept_reads, input_reads)?),
            alignment_qc_metric_basis: Some("kept_fraction".to_string()),
            scientific_metric_summary: summarize_metrics(vec![
                format!("kept_reads={kept_reads}"),
                format!("removed_reads={removed_reads}"),
                format!("min_length_threshold={min_length_threshold}"),
            ]),
            source_paths: BTreeSet::new(),
            ..BamEvidenceAggregate::default()
        },
        path,
    );
    Ok(())
}
