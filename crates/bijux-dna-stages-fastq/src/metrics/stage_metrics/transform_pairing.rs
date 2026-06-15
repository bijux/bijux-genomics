#![allow(clippy::items_after_test_module)]
#![cfg_attr(test, allow(clippy::expect_used))]

use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_fastq::metrics::{
    FastqDeduplicateMetricsV1, FastqDeltaMetricsV1, FastqMergeMetricsV1, FastqValidateMetricsV1,
    RetentionReportMetricV1,
};
use bijux_dna_stage_contract::StagePlanV1;

use crate::metrics::envelope_support::{
    f64_from_u64, path_from_params, retention_conditions_from_effective, stats_for_paths,
    zero_seqkit_metrics,
};

pub(super) fn deduplicate_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        outputs.first().map(PathBuf::as_path),
    ])?;
    let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let output = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
    let parsed_report = std::fs::read_to_string(plan.out_dir.join("deduplicate_report.json"))
        .ok()
        .and_then(|raw| crate::observer::parse_remove_duplicates_report(&raw).ok());
    let reads_in = parsed_report.as_ref().map_or(input.reads, |report| report.reads_in);
    let reads_out = parsed_report.as_ref().map_or(output.reads, |report| report.reads_out);
    let (pairs_in, pairs_out) = (
        parsed_report.as_ref().and_then(|report| report.pairs_in),
        parsed_report.as_ref().and_then(|report| report.pairs_out),
    );
    let read_retention =
        if reads_in > 0 { f64_from_u64(reads_out) / f64_from_u64(reads_in) } else { 0.0 };
    let base_retention =
        if input.bases > 0 { f64_from_u64(output.bases) / f64_from_u64(input.bases) } else { 0.0 };
    let delta = FastqDeltaMetricsV1 {
        read_retention,
        base_retention,
        mean_q_delta: output.mean_q - input.mean_q,
        gc_delta: output.gc_percent - input.gc_percent,
    };
    let retention = RetentionReportMetricV1 {
        value: read_retention,
        numerator_reads: reads_out,
        denominator_reads: reads_in,
        numerator_bases: output.bases,
        denominator_bases: input.bases,
        definition: "reads_out / reads_in".to_string(),
        stage_boundary: plan.stage_id.to_string(),
        conditions: retention_conditions_from_effective(
            &plan.stage_id,
            &plan.effective_params,
            &plan.params,
        ),
    };
    Ok(serde_json::to_value(FastqDeduplicateMetricsV1 {
        reads_in,
        reads_out,
        reads_removed_duplicates: reads_in.saturating_sub(reads_out),
        bases_in: input.bases,
        bases_out: output.bases,
        pairs_in,
        pairs_out,
        mean_q_before: input.mean_q,
        mean_q_after: output.mean_q,
        paired_mode: parsed_report
            .as_ref()
            .and_then(|report| serde_json::to_value(report.paired_mode).ok())
            .and_then(|value| value.as_str().map(ToString::to_string)),
        dedup_mode: parsed_report.as_ref().map(|report| match &report.dedup_mode {
            bijux_dna_domain_fastq::params::remove_duplicates::DedupMode::Exact => {
                "exact".to_string()
            }
            bijux_dna_domain_fastq::params::remove_duplicates::DedupMode::SequenceIdentity => {
                "sequence_identity".to_string()
            }
            bijux_dna_domain_fastq::params::remove_duplicates::DedupMode::OpticalAware => {
                "optical_aware".to_string()
            }
        }),
        keep_order: parsed_report.as_ref().map(|report| report.keep_order),
        pair_count_match: parsed_report.as_ref().and_then(|report| report.pair_count_match),
        duplicate_class_count: parsed_report
            .as_ref()
            .map(|report| report.duplicate_classes.len() as u64),
        duplicate_provenance_json: parsed_report
            .as_ref()
            .and_then(|report| report.duplicate_provenance_json.clone()),
        raw_backend_report_format: parsed_report
            .as_ref()
            .and_then(|report| report.raw_backend_report_format.clone()),
        delta_metrics: delta,
        retention,
    })?)
}

pub(super) fn merge_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        inputs.get(1).map(PathBuf::as_path),
        outputs.first().map(PathBuf::as_path),
        outputs.get(1).map(PathBuf::as_path),
        outputs.get(2).map(PathBuf::as_path),
    ])?;
    let r1 = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let r2 = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
    let merged = stats.get(2).copied().unwrap_or_else(zero_seqkit_metrics);
    let unmerged_r1 = stats.get(3).copied().unwrap_or_else(zero_seqkit_metrics);
    let unmerged_r2 = stats.get(4).copied().unwrap_or_else(zero_seqkit_metrics);
    let parsed_report = std::fs::read_to_string(plan.out_dir.join("merge_report.json"))
        .ok()
        .and_then(|raw| crate::observer::parse_merge_pairs_report(&raw).ok());
    let reads_r1 = parsed_report.as_ref().map_or(r1.reads, |report| report.reads_r1);
    let reads_r2 = parsed_report.as_ref().map_or(r2.reads, |report| report.reads_r2);
    let reads_merged = parsed_report.as_ref().map_or(merged.reads, |report| report.reads_merged);
    let reads_unmerged = parsed_report
        .as_ref()
        .map_or_else(|| unmerged_r1.reads.min(unmerged_r2.reads), |report| report.reads_unmerged);
    let (input_pair_count, merged_pair_count, unmerged_pair_count, discarded_pair_count) =
        parsed_report.as_ref().map_or_else(
            || {
                let input_pair_count = reads_r1.min(reads_r2);
                let merged_pair_count = reads_merged.min(input_pair_count);
                let unmerged_pair_count =
                    reads_unmerged.min(input_pair_count.saturating_sub(merged_pair_count));
                let discarded_pair_count =
                    input_pair_count.saturating_sub(merged_pair_count + unmerged_pair_count);
                (input_pair_count, merged_pair_count, unmerged_pair_count, discarded_pair_count)
            },
            |report| {
                let pair_counts = report.canonical_pair_counts();
                (
                    pair_counts.input_pair_count,
                    pair_counts.merged_pair_count,
                    pair_counts.unmerged_pair_count,
                    pair_counts.discarded_pair_count,
                )
            },
        );
    let min_reads = input_pair_count;
    let merge_rate = parsed_report.as_ref().map_or_else(
        || {
            if min_reads > 0 {
                f64_from_u64(reads_merged) / f64_from_u64(min_reads)
            } else {
                0.0
            }
        },
        |report| report.merge_rate,
    );
    let bases_in = r1.bases.min(r2.bases);
    let mean_q_in = f64::midpoint(r1.mean_q, r2.mean_q);
    let merge_q_delta = merged.mean_q - mean_q_in;
    Ok(serde_json::to_value(FastqMergeMetricsV1 {
        reads_in: min_reads,
        reads_out: reads_merged,
        bases_in,
        bases_out: merged.bases,
        pairs_in: Some(input_pair_count),
        pairs_out: Some(merged_pair_count),
        reads_r1,
        reads_r2,
        reads_merged,
        reads_unmerged,
        reads_discarded: discarded_pair_count,
        input_pair_count,
        merged_pair_count,
        unmerged_pair_count,
        discarded_pair_count,
        merge_rate,
        merge_q_delta,
    })?)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use bijux_dna_core::contract::{ArtifactRole, StageIO, StageOperatingMode, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
    use bijux_dna_domain_fastq::metrics::FastqMergeMetricsV1;
    use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StagePlanV1};

    use super::merge_metrics;

    fn write_fastq(path: &std::path::Path, read_id: &str, sequence: &str) {
        let quality = "#".repeat(sequence.len());
        bijux_dna_infra::write_bytes(path, format!("@{read_id}\n{sequence}\n+\n{quality}\n"))
            .expect("write fastq");
    }

    fn merge_plan(out_dir: &std::path::Path) -> StagePlanV1 {
        StagePlanV1 {
            stage_id: StageId::from_static("fastq.merge_pairs"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("pear"),
            tool_version: "test".to_string(),
            image: serde_json::from_value(serde_json::json!({
                "image": "bijuxdna/test",
                "digest": null
            }))
            .expect("image"),
            command: serde_json::from_value(serde_json::json!({
                "template": ["echo", "ok"]
            }))
            .expect("command"),
            resources: ToolConstraints::default(),
            io: StageIO {
                inputs: vec![
                    ArtifactRef::required(
                        ArtifactId::new("reads_r1"),
                        PathBuf::from("reads_R1.fastq"),
                        ArtifactRole::Reads,
                    ),
                    ArtifactRef::required(
                        ArtifactId::new("reads_r2"),
                        PathBuf::from("reads_R2.fastq"),
                        ArtifactRole::Reads,
                    ),
                ],
                outputs: vec![],
            },
            out_dir: out_dir.to_path_buf(),
            params: serde_json::json!({}),
            effective_params: serde_json::json!({}),
            operating_mode: StageOperatingMode::Enforced,
            aux_images: BTreeMap::new(),
            canonical_contract: None,
            provenance: None,
            reason: PlanDecisionReason::default(),
        }
    }

    #[test]
    fn merge_metrics_emit_canonical_pair_counts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_r1 = temp.path().join("reads_R1.fastq");
        let reads_r2 = temp.path().join("reads_R2.fastq");
        let merged_reads = temp.path().join("merged.fastq");
        let unmerged_r1 = temp.path().join("unmerged_R1.fastq");
        let unmerged_r2 = temp.path().join("unmerged_R2.fastq");
        write_fastq(&reads_r1, "r1", "ACGT");
        write_fastq(&reads_r2, "r1", "TGCA");
        write_fastq(&merged_reads, "merged", "ACGTTGCA");
        write_fastq(&unmerged_r1, "left", "AAAA");
        write_fastq(&unmerged_r2, "right", "TTTT");
        bijux_dna_infra::write_bytes(
            temp.path().join("merge_report.json"),
            serde_json::json!({
                "schema_version": "bijux.fastq.merge_pairs.report.v2",
                "stage": "fastq.merge_pairs",
                "stage_id": "fastq.merge_pairs",
                "tool_id": "pear",
                "paired_mode": "paired_end",
                "merge_engine": "pear",
                "threads": 2,
                "merge_overlap": 12,
                "min_len": 30,
                "unmerged_read_policy": "emit_unmerged_pairs",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "merged_reads": "merged.fastq.gz",
                "unmerged_reads_r1": "unmerged_R1.fastq.gz",
                "unmerged_reads_r2": "unmerged_R2.fastq.gz",
                "reads_r1": 100,
                "reads_r2": 96,
                "reads_merged": 88,
                "reads_unmerged": 6,
                "merge_rate": 0.916_666_666_7
            })
            .to_string(),
        )
        .expect("write report");

        let metrics: FastqMergeMetricsV1 = serde_json::from_value(
            merge_metrics(
                &merge_plan(temp.path()),
                &[reads_r1, reads_r2],
                &[merged_reads, unmerged_r1, unmerged_r2],
            )
            .expect("merge metrics"),
        )
        .expect("decode metrics");

        assert_eq!(metrics.input_pair_count, 96);
        assert_eq!(metrics.merged_pair_count, 88);
        assert_eq!(metrics.unmerged_pair_count, 6);
        assert_eq!(metrics.discarded_pair_count, 2);
        assert_eq!(metrics.reads_discarded, 2);
        assert_eq!(metrics.pairs_in, Some(96));
        assert_eq!(metrics.pairs_out, Some(88));
    }
}

pub(super) fn validate_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[
        inputs.first().map(PathBuf::as_path),
        inputs.get(1).map(PathBuf::as_path),
    ])?;
    let input_r1 = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let input_r2 = stats.get(1).copied().unwrap_or_else(zero_seqkit_metrics);
    let pairs_in = inputs.get(1).map(|_| input_r1.reads.min(input_r2.reads));
    let pairs_out = pairs_in;
    let reads_in = input_r1.reads + inputs.get(1).map_or(0, |_| input_r2.reads);
    let bases_in = input_r1.bases + inputs.get(1).map_or(0, |_| input_r2.bases);
    let mean_q = if inputs.get(1).is_some() {
        f64::midpoint(input_r1.mean_q, input_r2.mean_q)
    } else {
        input_r1.mean_q
    };
    let report_metrics = path_from_params(&plan.params, "report_json")
        .and_then(|report_path| std::fs::read_to_string(&report_path).ok())
        .and_then(|raw| crate::observer::parse_validation_report(&raw).ok())
        .map(|report| {
            let reads_total = report.validated_reads_r1 + report.validated_reads_r2.unwrap_or(0);
            let reads_invalid = match report.failure_class {
                bijux_dna_domain_fastq::ValidateFailureClass::None
                | bijux_dna_domain_fastq::ValidateFailureClass::HeaderSyncMismatch => 0,
                bijux_dna_domain_fastq::ValidateFailureClass::UnsupportedCompression
                | bijux_dna_domain_fastq::ValidateFailureClass::EmptyInput
                | bijux_dna_domain_fastq::ValidateFailureClass::MalformedRecord
                | bijux_dna_domain_fastq::ValidateFailureClass::InvalidQualityEncoding => {
                    reads_total
                }
                bijux_dna_domain_fastq::ValidateFailureClass::PairCountMismatch => {
                    report.validated_reads_r1.abs_diff(report.validated_reads_r2.unwrap_or(0))
                }
                bijux_dna_domain_fastq::ValidateFailureClass::ValidatorError => {
                    let mut invalid = 0;
                    if report.status_r1 != 0 {
                        invalid += report.validated_reads_r1;
                    }
                    if report.status_r2 != 0 {
                        invalid += report.validated_reads_r2.unwrap_or(0);
                    }
                    invalid
                }
            }
            .min(reads_total);
            FastqValidateMetricsV1 {
                reads_in,
                reads_out: reads_in,
                bases_in,
                bases_out: bases_in,
                pairs_in,
                pairs_out,
                reads_total,
                reads_valid: reads_total.saturating_sub(reads_invalid),
                reads_invalid,
                mean_q,
                validated_inputs: Some(report.validated_inputs),
                validated_pairs: report.validated_pairs,
                pair_sync_checked: Some(report.pair_sync_checked),
                pair_sync_pass: report.pair_sync_pass,
                pair_count_match: report.pair_count_match,
                strict_pass: Some(report.strict_pass),
                failure_class: serde_json::to_value(&report.failure_class)
                    .ok()
                    .and_then(|value| value.as_str().map(ToOwned::to_owned)),
            }
        });
    Ok(serde_json::to_value(report_metrics.unwrap_or(FastqValidateMetricsV1 {
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in,
        pairs_out,
        reads_total: reads_in,
        reads_valid: reads_in,
        reads_invalid: 0,
        mean_q,
        validated_inputs: None,
        validated_pairs: None,
        pair_sync_checked: None,
        pair_sync_pass: None,
        pair_count_match: None,
        strict_pass: None,
        failure_class: None,
    }))?)
}
