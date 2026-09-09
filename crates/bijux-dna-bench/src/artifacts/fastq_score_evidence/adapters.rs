//! Owner: bijux-dna-bench
//! Adapters for observed FASTQ stage report formats.

use super::fields::{fraction_u64, required_f64, required_stage, required_str, required_u64};
use anyhow::{Context, Result};
use bijux_dna_bench_model::FastqScoreEvidence as FastqEvidenceAggregate;
use serde_json::Value;
use std::collections::BTreeSet;

type ReportAdapter = fn(&Value) -> Result<(String, String, FastqEvidenceAggregate)>;
pub(super) const REPORTS: &[(&str, ReportAdapter)] = &[
    ("artifacts/runs/bench/local-smoke/fastq.filter_reads/report.json", parse_filter_reads),
    ("artifacts/runs/bench/local-smoke/fastq.filter_low_complexity/report.json", parse_filter_low_complexity),
    ("artifacts/runs/bench/local-smoke/fastq.profile_overrepresented_sequences/report.json", parse_profile_overrepresented_sequences),
    ("artifacts/runs/bench/local-smoke/fastq.remove_duplicates/report.json", parse_remove_duplicates),
    ("artifacts/runs/bench/local-smoke/fastq.detect_adapters/adapters.json", parse_detect_adapters),
    ("artifacts/runs/bench/local-smoke/fastq.extract_umis/report.json", parse_extract_umis),
    ("artifacts/runs/bench/local-smoke/fastq.detect_duplicates_premerge/duplicates.json", parse_detect_duplicates_premerge),
    ("artifacts/runs/bench/local-smoke/fastq.estimate_library_complexity_prealign/complexity.json", parse_estimate_library_complexity_prealign),
    ("artifacts/runs/bench/local-smoke/fastq.merge_pairs/report.json", parse_merge_pairs),
    ("artifacts/runs/bench/local-smoke/fastq.normalize_abundance/report.json", parse_normalize_abundance),
    ("artifacts/runs/bench/local-smoke/fastq.remove_chimeras/report.json", parse_remove_chimeras),
    ("artifacts/runs/bench/local-smoke/fastq.cluster_otus/report.json", parse_cluster_otus),
    ("artifacts/runs/bench/local-smoke/fastq.infer_asvs/report.json", parse_infer_asvs),
    ("artifacts/runs/bench/local-smoke/fastq.trim_terminal_damage/metrics.json", parse_trim_terminal_damage),
];

fn parse_filter_reads(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.filter_reads")?;
    let tool_id = required_str(root, "tool_id")?;
    let input_reads = required_u64(root, "input_reads")?;
    let output_reads = required_u64(root, "output_reads")?;
    let dropped_reads = required_u64(root, "reads_dropped")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
            truth_correctness_basis: Some("retained_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "report retained reads and dropped reads without violating input bounds"
                    .to_string(),
            ),
            retained_reads: Some(output_reads),
            dropped_reads: Some(dropped_reads),
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_filter_low_complexity(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.filter_low_complexity")?;
    let tool_id = required_str(root, "report_tool_id")?;
    let input_reads = required_u64(root, "input_reads")?;
    let output_reads = required_u64(root, "output_reads")?;
    let dropped_reads = required_u64(root, "reads_removed_low_complexity")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
            truth_correctness_basis: Some("retained_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "report low-complexity removal without inflating the read count".to_string(),
            ),
            retained_reads: Some(output_reads),
            dropped_reads: Some(dropped_reads),
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_profile_overrepresented_sequences(
    root: &Value,
) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.profile_overrepresented_sequences")?;
    let tool_id = required_str(root, "report_tool_id")?;
    let top_fraction = required_f64(root, "top_fraction")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(top_fraction),
            truth_correctness_basis: Some("top_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "surface publishes governed comparable metrics for overrepresented sequences"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_remove_duplicates(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.remove_duplicates")?;
    let tool_id = required_str(root, "report_tool_id")?;
    let input_reads = required_u64(root, "input_reads")?;
    let output_reads = required_u64(root, "output_reads")?;
    let duplicate_reads = required_u64(root, "duplicate_reads")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
            truth_correctness_basis: Some("retained_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "duplicate removal preserves the governed post-dedup read count surface"
                    .to_string(),
            ),
            retained_reads: Some(output_reads),
            dropped_reads: Some(duplicate_reads),
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_detect_adapters(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.detect_adapters")?;
    let case_count = required_u64(root, "case_count")?;
    let detected_case_count = required_u64(root, "detected_case_count")?;
    Ok((
        stage_id.to_string(),
        "fastqc".to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(detected_case_count, case_count)?),
            truth_correctness_basis: Some("detected_case_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "adapter detection summary keeps the governed case-count surface complete"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_extract_umis(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.extract_umis")?;
    let tool_id = required_str(root, "report_tool_id")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "UMI extraction summary and governed report artifacts materialized".to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_detect_duplicates_premerge(
    root: &Value,
) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.detect_duplicates_premerge")?;
    Ok((
        stage_id.to_string(),
        "bijux_dna".to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "duplicate-signal smoke report covers both duplicate and distinct pair cases"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_estimate_library_complexity_prealign(
    root: &Value,
) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.estimate_library_complexity_prealign")?;
    Ok((
        stage_id.to_string(),
        "bijux_dna".to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "complexity smoke report emits governed estimated- and insufficient-data counts"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_merge_pairs(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.merge_pairs")?;
    let tool_id = required_str(root, "report_tool_id")?;
    let input_pair_count = required_u64(root, "input_pair_count")?;
    let merged_count = required_u64(root, "merged_count")?;
    let unmerged_r1_count = required_u64(root, "unmerged_r1_count")?;
    let unmerged_r2_count = required_u64(root, "unmerged_r2_count")?;
    let discarded_count = required_u64(root, "discarded_count")?;
    let input_reads =
        input_pair_count.checked_mul(2).context("FASTQ merge input read count overflows u64")?;
    let retained_reads = merged_count
        .checked_add(unmerged_r1_count)
        .and_then(|count| count.checked_add(unmerged_r2_count))
        .context("FASTQ merge retained read count overflows u64")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(retained_reads, input_reads)?),
            truth_correctness_basis: Some("retained_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "merge report preserves merged, unmerged, and discarded counts".to_string(),
            ),
            retained_reads: Some(retained_reads),
            dropped_reads: Some(discarded_count),
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_normalize_abundance(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.normalize_abundance")?;
    let tool_id = required_str(root, "report_tool_id")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "normalized abundance table materialized and retained numeric validity".to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_remove_chimeras(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.remove_chimeras")?;
    let tool_id = required_str(root, "report_tool_id")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "chimera summary emits governed non-chimeric and chimera artifact paths"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_cluster_otus(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.cluster_otus")?;
    let tool_id = required_str(root, "report_tool_id")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "OTU clustering report materialized representatives and abundance outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_infer_asvs(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.infer_asvs")?;
    let tool_id = required_str(root, "report_tool_id")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: None,
            truth_correctness_basis: None,
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "ASV inference report materialized representatives and abundance outputs"
                    .to_string(),
            ),
            retained_reads: None,
            dropped_reads: None,
            source_paths: BTreeSet::new(),
        },
    ))
}

fn parse_trim_terminal_damage(root: &Value) -> Result<(String, String, FastqEvidenceAggregate)> {
    let stage_id = required_stage(root, "fastq.trim_terminal_damage")?;
    let tool_id = required_str(root, "tool_id")?;
    let input_reads = required_u64(root, "input_reads")?;
    let output_reads = required_u64(root, "output_reads")?;
    Ok((
        stage_id.to_string(),
        tool_id.to_string(),
        FastqEvidenceAggregate {
            truth_correctness_score: Some(fraction_u64(output_reads, input_reads)?),
            truth_correctness_basis: Some("retained_fraction".to_string()),
            contract_correctness_score: Some(1.0),
            contract_correctness_basis: Some(
                "terminal-damage trimming report preserves the governed output count surface"
                    .to_string(),
            ),
            retained_reads: Some(output_reads),
            dropped_reads: Some(input_reads.saturating_sub(output_reads)),
            source_paths: BTreeSet::new(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn report_stage_identity_is_bound_to_each_adapter() -> Result<()> {
        let mut value = serde_json::json!({
            "stage_id":"fastq.filter_reads", "tool_id":"fastp", "planned_tool_id":"fastp", "report_tool_id":"fastp",
            "top_fraction":0.25, "case_count":3, "detected_case_count":1,
            "input_reads":3, "output_reads":1, "reads_dropped":2,
            "reads_removed_low_complexity":2, "duplicate_reads":2,
            "input_pair_count":3, "merged_count":1, "unmerged_r1_count":1,
            "unmerged_r2_count":1, "discarded_count":0
        });
        for (path, adapter) in REPORTS {
            let stage = std::path::Path::new(path)
                .parent()
                .and_then(std::path::Path::file_name)
                .and_then(std::ffi::OsStr::to_str)
                .context("declared report stage")?;
            value["stage_id"] = serde_json::json!(stage);
            assert_eq!(adapter(&value)?.0, stage);
            for wrong_stage in ["vcf.filter", "fastq.validate_reads"] {
                value["stage_id"] = serde_json::json!(wrong_stage);
                let error = adapter(&value).err().context("cross-stage report must reject")?;
                assert!(error.to_string().contains("expected stage"), "{error}");
            }
        }
        Ok(())
    }
    #[test]
    fn merge_report_counts_reject_overflow_and_preserve_record_units() -> Result<()> {
        let original = serde_json::json!({"stage_id":"fastq.merge_pairs", "planned_tool_id":"pear", "report_tool_id":"pear",
            "input_pair_count":2,"merged_count":1,"unmerged_r1_count":1,"unmerged_r2_count":1,"discarded_count":0});
        let observed = parse_merge_pairs(&original)?.2;
        assert_eq!((observed.retained_reads, observed.dropped_reads), (Some(3), Some(0)));
        for field in ["input_pair_count", "merged_count", "unmerged_r2_count"] {
            let mut value = original.clone();
            value[field] = serde_json::json!(u64::MAX);
            let error = parse_merge_pairs(&value).err().context("count overflow must reject")?;
            assert!(error.to_string().contains("overflows u64"), "{error}");
        }
        Ok(())
    }
    #[test]
    fn local_report_producer_is_never_replaced_by_the_planned_backend() -> Result<()> {
        let adapters: [ReportAdapter; 9] = [
            parse_filter_low_complexity,
            parse_profile_overrepresented_sequences,
            parse_remove_duplicates,
            parse_extract_umis,
            parse_merge_pairs,
            parse_normalize_abundance,
            parse_remove_chimeras,
            parse_cluster_otus,
            parse_infer_asvs,
        ];
        let stages = [
            "fastq.filter_low_complexity",
            "fastq.profile_overrepresented_sequences",
            "fastq.remove_duplicates",
            "fastq.extract_umis",
            "fastq.merge_pairs",
            "fastq.normalize_abundance",
            "fastq.remove_chimeras",
            "fastq.cluster_otus",
            "fastq.infer_asvs",
        ];
        for (adapter, stage) in adapters.into_iter().zip(stages) {
            let mut value = serde_json::json!({
                "stage_id": stage, "planned_tool_id": "pear", "report_tool_id": "bijux",
                "top_fraction":0.25, "input_reads":3, "output_reads":1,
                "reads_removed_low_complexity":2, "duplicate_reads":2,
                "input_pair_count":3, "merged_count":1, "unmerged_r1_count":1,
                "unmerged_r2_count":1, "discarded_count":0
            });
            assert_eq!(adapter(&value)?.1, "bijux", "{stage}");
            value["planned_tool_id"] = serde_json::json!("vsearch");
            assert_eq!(adapter(&value)?.1, "bijux", "planning cannot change observed producer");
            for invalid in
                [serde_json::Value::Null, serde_json::json!(""), serde_json::json!(" bijux")]
            {
                value["report_tool_id"] = invalid;
                assert!(adapter(&value).is_err(), "{stage}: invalid producer must reject");
            }
        }
        Ok(())
    }
}
