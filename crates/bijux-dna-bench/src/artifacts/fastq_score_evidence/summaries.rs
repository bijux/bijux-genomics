//! Owner: bijux-dna-bench
//! Normalize case collections and stage summary observations.

use super::fields::{
    fraction_u64, optional_str, optional_u64, required_f64, required_observed_tool, required_stage,
    sum_report_counts,
};
use super::merge_evidence_row;
use crate::repo::load_optional_fastq_score_report as load_optional_json_value;
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::FastqScoreEvidence as FastqEvidenceAggregate;
use bijux_dna_core::prelude::numeric::rounded_f64_to_u64;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(super) fn merge_case_stage_report(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
    relative_path: &str,
    contract_label: &str,
) -> Result<()> {
    let absolute_path = repo_root.join(relative_path);
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_stage(&root, &format!("fastq.{contract_label}"))?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let mut tool_totals = BTreeMap::<String, (u64, u64, u64, u64)>::new();
    for case in cases {
        let tool_id = required_observed_tool(case)?.to_string();
        let input_reads = optional_u64(case, "input_read_count_total")
            .or_else(|| optional_u64(case, "input_reads"))
            .ok_or_else(|| {
                anyhow!("{} is missing input read counts for `{tool_id}`", absolute_path.display())
            })?;
        let output_reads = optional_u64(case, "output_read_count_total")
            .or_else(|| optional_u64(case, "output_reads"))
            .ok_or_else(|| {
                anyhow!("{} is missing output read counts for `{tool_id}`", absolute_path.display())
            })?;
        let retained_reads = optional_u64(case, "reads_retained").unwrap_or(output_reads);
        let dropped_reads = optional_u64(case, "reads_dropped")
            .unwrap_or_else(|| input_reads.saturating_sub(output_reads));
        let entry = tool_totals.entry(tool_id).or_default();
        entry.0 = entry
            .0
            .checked_add(input_reads)
            .with_context(|| format!("{} overflows input reads", absolute_path.display()))?;
        entry.1 = entry
            .1
            .checked_add(retained_reads)
            .with_context(|| format!("{} overflows retained reads", absolute_path.display()))?;
        entry.2 = entry
            .2
            .checked_add(dropped_reads)
            .with_context(|| format!("{} overflows dropped reads", absolute_path.display()))?;
        entry.3 = entry
            .3
            .checked_add(1)
            .with_context(|| format!("{} overflows case count", absolute_path.display()))?;
    }
    for (tool_id, (input_reads, retained_reads, dropped_reads, _case_count)) in tool_totals {
        merge_evidence_row(
            rows,
            (stage_id.clone(), tool_id),
            absolute_path.display().to_string(),
            FastqEvidenceAggregate {
                truth_correctness_score: Some(fraction_u64(retained_reads, input_reads)?),
                truth_correctness_basis: Some("retained_fraction".to_string()),
                contract_correctness_score: Some(1.0),
                contract_correctness_basis: Some(format!(
                    "{contract_label} smoke cases preserve the governed output-count surface"
                )),
                retained_reads: Some(retained_reads),
                dropped_reads: Some(dropped_reads),
                source_paths: BTreeSet::new(),
            },
        );
    }
    Ok(())
}

pub(super) fn merge_profile_reads_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.profile_reads/profile.json");
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_stage(&root, "fastq.profile_reads")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let total_reads = sum_report_counts(cases, "reads_total")?;
    merge_evidence_row(
        rows,
        (stage_id, "seqkit_stats".to_string()),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "profile report emits governed read-count and aggregate QC metrics".to_string(),
            ),
            retained_reads: Some(total_reads),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

pub(super) fn merge_validate_reads_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path =
        repo_root.join("artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json");
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_stage(&root, "fastq.validate_reads")?.to_string();
    let cases = root
        .get("cases")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing cases", absolute_path.display()))?;
    let total_reads = sum_report_counts(cases, "input_read_count_total")?;
    let passed_cases = u64::try_from(
        cases.iter().filter(|case| optional_str(case, "validation_status") == Some("pass")).count(),
    )?;
    let case_count = u64::try_from(cases.len())?;
    merge_evidence_row(
        rows,
        (stage_id, "fastqvalidator".to_string()),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(passed_cases, case_count)?),
            truth_correctness_basis: Some("validation_pass_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "validation smoke report covers every governed case without missing outputs"
                    .to_string(),
            ),
            retained_reads: Some(total_reads),
            dropped_reads: Some(0),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}

pub(super) fn merge_screen_taxonomy_summary(
    repo_root: &Path,
    rows: &mut BTreeMap<(String, String), FastqEvidenceAggregate>,
) -> Result<()> {
    let absolute_path = repo_root
        .join("artifacts/benchmark-smoke/micro/pipelines/edna/artifacts/fastq.screen_taxonomy/report.json");
    let Some(root) = load_optional_json_value(&absolute_path)? else {
        return Ok(());
    };
    let stage_id = required_stage(&root, "fastq.screen_taxonomy")?.to_string();
    let tool_id = required_observed_tool(&root)?.to_string();
    let samples = root
        .get("samples")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} is missing samples", absolute_path.display()))?;
    let mut total_reads = 0_f64;
    let mut classified_reads = 0_f64;
    for sample in samples {
        let reads_in = required_f64(sample, "reads_in")?;
        let unclassified_fraction = required_f64(sample, "unclassified_fraction")?;
        total_reads += reads_in;
        classified_reads += reads_in * (1.0 - unclassified_fraction);
    }
    merge_evidence_row(
        rows,
        (stage_id, tool_id),
        absolute_path.display().to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: (total_reads > 0.0).then_some(classified_reads / total_reads),
            truth_correctness_basis: Some("classified_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "taxonomy screen report materialized the governed classified and unclassified surfaces"
                    .to_string(),
            ),
            retained_reads: Some(rounded_f64_to_u64(classified_reads, "retained reads")?),
            dropped_reads: Some(rounded_f64_to_u64(total_reads - classified_reads, "dropped reads")?),
            source_paths: BTreeSet::new(),
        },
    );
    Ok(())
}
