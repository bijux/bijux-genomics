#![allow(clippy::items_after_test_module)]
#![cfg_attr(test, allow(clippy::expect_used))]

use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_core::id_catalog;
use bijux_dna_domain_fastq::metrics::{
    FastqCorrectMetricsV1, FastqDeltaMetricsV1, FastqDetectAdaptersMetricsV1,
    FastqPreprocessMetricsV1, FastqQcPostMetricsV1, FastqStatsNeutralMetricsV1, FastqUmiMetricsV1,
    RetentionReportMetricV1,
};
use bijux_dna_stage_contract::StagePlanV1;

use crate::metrics::envelope_support::{
    distributions_for_path, f64_from_u64, pair_counts_from_paths, path_from_params,
    retention_conditions_from_effective, stats_for_paths, stats_or_zero, zero_seqkit_metrics,
};
use crate::metrics::fastqc::fastqc_metrics_v2_from_dir;

pub(super) fn stage_metrics_for_stage(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Option<Result<serde_json::Value>> {
    match plan.stage_id.as_str() {
        "fastq.normalize_primers" => Some(normalize_primers_metrics(plan, inputs, outputs)),
        "fastq.profile_overrepresented_sequences" => {
            Some(profile_overrepresented_metrics(plan, inputs))
        }
        id_catalog::FASTQ_DETECT_ADAPTERS => Some(detect_adapters_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_CORRECT => Some(correct_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_UMI => Some(umi_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_PREPROCESS => Some(preprocess_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_QC_POST => Some(qc_post_metrics(plan, inputs, outputs)),
        id_catalog::FASTQ_STATS_NEUTRAL => Some(stats_neutral_metrics(plan, inputs, outputs)),
        "fastq.profile_read_lengths" => Some(profile_read_lengths_metrics(plan, inputs)),
        _ => None,
    }
}

fn normalize_primers_metrics(
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
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("normalize_primers_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_normalize_primers_report(&raw).ok());
    let reads_in =
        governed_report.as_ref().and_then(|report| report.reads_in).unwrap_or(input.reads);
    let reads_out =
        governed_report.as_ref().and_then(|report| report.reads_out).unwrap_or(output.reads);
    let bases_in =
        governed_report.as_ref().and_then(|report| report.bases_in).unwrap_or(input.bases);
    let bases_out =
        governed_report.as_ref().and_then(|report| report.bases_out).unwrap_or(output.bases);
    let read_retention =
        if reads_in > 0 { f64_from_u64(reads_out) / f64_from_u64(reads_in) } else { 0.0 };
    let base_retention =
        if bases_in > 0 { f64_from_u64(bases_out) / f64_from_u64(bases_in) } else { 0.0 };
    let retention = RetentionReportMetricV1 {
        value: read_retention,
        numerator_reads: reads_out,
        denominator_reads: reads_in,
        numerator_bases: bases_out,
        denominator_bases: bases_in,
        definition: "reads_out / reads_in".to_string(),
        stage_boundary: plan.stage_id.to_string(),
        conditions: retention_conditions_from_effective(
            &plan.stage_id,
            &plan.effective_params,
            &plan.params,
        ),
    };
    Ok(serde_json::json!({
        "reads_in": reads_in,
        "reads_out": reads_out,
        "bases_in": bases_in,
        "bases_out": bases_out,
        "pairs_in": governed_report.as_ref().and_then(|report| report.pairs_in),
        "pairs_out": governed_report.as_ref().and_then(|report| report.pairs_out),
        "paired_mode": governed_report.as_ref().map(|report| report.paired_mode),
        "primer_set_id": governed_report.as_ref().map(|report| report.primer_set_id.clone()),
        "marker_id": governed_report.as_ref().and_then(|report| report.marker_id.clone()),
        "orientation_policy": governed_report.as_ref().map(|report| report.orientation_policy.clone()),
        "primer_trimmed_reads": governed_report.as_ref().and_then(|report| report.primer_trimmed_reads),
        "primer_trimmed_fraction": governed_report.as_ref().and_then(|report| report.primer_trimmed_fraction),
        "orientation_forward_fraction": governed_report.as_ref().and_then(|report| report.orientation_forward_fraction),
        "raw_backend_report_format": governed_report.as_ref().and_then(|report| report.raw_backend_report_format.clone()),
        "delta_metrics": {
            "read_retention": read_retention,
            "base_retention": base_retention,
            "mean_q_delta": output.mean_q - input.mean_q,
            "gc_delta": output.gc_percent - input.gc_percent,
        },
        "retention": retention,
    }))
}

fn profile_overrepresented_metrics(
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
    let report = path_from_params(&plan.params, "report_json")
        .and_then(|report_path| std::fs::read_to_string(report_path).ok())
        .and_then(|raw| crate::observer::parse_profile_overrepresented_report(&raw).ok());
    let legacy = path_from_params(&plan.params, "output_json")
        .and_then(|json_path| std::fs::read_to_string(json_path).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok());
    Ok(serde_json::json!({
        "reads_in": reads_in,
        "reads_out": reads_in,
        "bases_in": bases_in,
        "bases_out": bases_in,
        "pairs_in": pairs_in,
        "pairs_out": pairs_out,
        "paired_mode": report.as_ref().map(|payload| payload.paired_mode),
        "threads": report.as_ref().map(|payload| payload.threads),
        "top_k": report.as_ref().map(|payload| payload.top_k).or_else(|| {
            legacy
                .as_ref()
                .and_then(|value| value.get("top_k"))
                .and_then(serde_json::Value::as_u64)
                .and_then(|value| u32::try_from(value).ok())
        }),
        "sequence_count": report
            .as_ref()
            .map(|payload| payload.sequence_count)
            .or_else(|| {
                legacy
                    .as_ref()
                    .and_then(|value| value.get("sequence_count"))
                    .and_then(serde_json::Value::as_u64)
            })
            .unwrap_or(0),
        "flagged_sequences": report
            .as_ref()
            .map(|payload| payload.flagged_sequences)
            .or_else(|| {
                legacy
                    .as_ref()
                    .and_then(|value| value.get("flagged_sequences"))
                    .and_then(serde_json::Value::as_u64)
            })
            .unwrap_or(0),
        "top_fraction": report
            .as_ref()
            .map(|payload| payload.top_fraction)
            .or_else(|| {
                legacy
                    .as_ref()
                    .and_then(|value| value.get("top_fraction"))
                    .and_then(serde_json::Value::as_f64)
            })
            .unwrap_or(0.0),
        "row_count": report.as_ref().map(|payload| payload.rows.len()).or_else(|| {
            legacy
                .as_ref()
                .and_then(|value| value.get("rows"))
                .and_then(serde_json::Value::as_array)
                .map(std::vec::Vec::len)
        }),
    }))
}

fn detect_adapters_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("adapter_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_detect_adapters_report(&raw).ok());
    if let Some(report) = governed_report {
        Ok(serde_json::to_value(FastqDetectAdaptersMetricsV1 {
            reads_in: report.reads_in,
            reads_out: report.reads_out,
            bases_in: report.bases_in,
            bases_out: report.bases_out,
            pairs_in: report.pairs_in,
            pairs_out: report.pairs_out,
            mean_q: report.mean_q,
            adapter_content_max: report.adapter_content_max,
            adapter_content_mean: report.adapter_content_mean,
            duplication_rate: report.duplication_rate,
            n_rate: report.n_rate,
            kmer_warning_count: report.kmer_warning_count,
            overrepresented_sequence_count: report.overrepresented_sequence_count,
        })?)
    } else {
        let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
        let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
        let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
        let out_dir = path_from_params(&plan.params, "out_dir")
            .unwrap_or_else(|| inputs.first().cloned().unwrap_or_default());
        let metrics = fastqc_metrics_v2_from_dir(&out_dir).or_else(|| {
            let subdir = out_dir.join("fastqc");
            fastqc_metrics_v2_from_dir(&subdir)
        });
        let adapter_content_max =
            metrics.as_ref().and_then(|m| m.adapter_content.as_ref().map(|a| a.max_percent));
        let adapter_content_mean =
            metrics.as_ref().and_then(|m| m.adapter_content.as_ref().map(|a| a.mean_percent));
        let duplication_rate =
            metrics.as_ref().and_then(|m| m.duplication.as_ref().map(|d| d.duplication_rate));
        let n_rate =
            metrics.as_ref().and_then(|m| m.n_content.as_ref().map(|n| n.mean_percent / 100.0));
        let kmer_warning_count =
            metrics.as_ref().and_then(|m| m.kmer_content.as_ref().map(|k| k.warning_count));
        let overrepresented_sequence_count =
            metrics.as_ref().and_then(|m| m.overrepresented_sequences.as_ref().map(|o| o.count));
        Ok(serde_json::to_value(FastqDetectAdaptersMetricsV1 {
            reads_in: input.reads,
            reads_out: input.reads,
            bases_in: input.bases,
            bases_out: input.bases,
            pairs_in,
            pairs_out,
            mean_q: input.mean_q,
            adapter_content_max,
            adapter_content_mean,
            duplication_rate,
            n_rate,
            kmer_warning_count,
            overrepresented_sequence_count,
        })?)
    }
}

fn correct_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("correct_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_correct_errors_report(&raw).ok());
    if let Some(report) = governed_report {
        let reads_in = report.reads_in.unwrap_or(0);
        let reads_out = report.reads_out.unwrap_or(reads_in);
        let bases_in = report.bases_in.unwrap_or(0);
        let bases_out = report.bases_out.unwrap_or(bases_in);
        let reads_corrected = report.corrected_reads.unwrap_or(reads_out);
        let bases_corrected = report.bases_out.unwrap_or(bases_out);
        Ok(serde_json::to_value(FastqCorrectMetricsV1 {
            reads_in,
            reads_out,
            bases_in,
            bases_out,
            pairs_in: report.pairs_in,
            pairs_out: report.pairs_out,
            reads_corrected,
            reads_uncorrected: reads_in.saturating_sub(reads_corrected),
            bases_corrected,
            bases_uncorrected: bases_in.saturating_sub(bases_corrected),
        })?)
    } else {
        let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
        let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
        let output = if outputs.is_empty() {
            input
        } else {
            let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
            stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
        };
        let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
        Ok(serde_json::to_value(FastqCorrectMetricsV1 {
            reads_in: input.reads,
            reads_out: output.reads,
            bases_in: input.bases,
            bases_out: output.bases,
            pairs_in,
            pairs_out,
            reads_corrected: output.reads,
            reads_uncorrected: input.reads.saturating_sub(output.reads),
            bases_corrected: output.bases,
            bases_uncorrected: input.bases.saturating_sub(output.bases),
        })?)
    }
}

#[allow(clippy::too_many_lines)]
fn umi_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let parsed_report = std::fs::read_to_string(plan.out_dir.join("umi_report.json"))
        .ok()
        .and_then(|raw| crate::observer::parse_extract_umis_report(&raw).ok());
    if let Some(report) = parsed_report {
        let umi_summary = report.canonical_umi_summary();
        let read_retention = if report.reads_in > 0 {
            f64_from_u64(report.reads_out) / f64_from_u64(report.reads_in)
        } else {
            0.0
        };
        let base_retention = if report.bases_in > 0 {
            f64_from_u64(report.bases_out) / f64_from_u64(report.bases_in)
        } else {
            0.0
        };
        let delta = FastqDeltaMetricsV1 {
            read_retention,
            base_retention,
            mean_q_delta: report.mean_q_after - report.mean_q_before,
            gc_delta: 0.0,
        };
        let retention = RetentionReportMetricV1 {
            value: read_retention,
            numerator_reads: report.reads_out,
            denominator_reads: report.reads_in,
            numerator_bases: report.bases_out,
            denominator_bases: report.bases_in,
            definition: "reads_out / reads_in".to_string(),
            stage_boundary: plan.stage_id.to_string(),
            conditions: retention_conditions_from_effective(
                &plan.stage_id,
                &plan.effective_params,
                &plan.params,
            ),
        };
        Ok(serde_json::to_value(FastqUmiMetricsV1 {
            reads_in: report.reads_in,
            reads_out: report.reads_out,
            bases_in: report.bases_in,
            bases_out: report.bases_out,
            pairs_in: report.pairs_in,
            pairs_out: report.pairs_out,
            umi_pattern: report.umi_pattern,
            tag_header_format: umi_summary.tag_header_format,
            reads_with_umi: report.reads_with_umi,
            extracted_umi_count: umi_summary.extracted_umi_count,
            invalid_umi_count: umi_summary.invalid_umi_count,
            mean_q_before: report.mean_q_before,
            mean_q_after: report.mean_q_after,
            delta_metrics: delta,
            retention,
        })?)
    } else {
        let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
        let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
        let output = if outputs.is_empty() {
            input
        } else {
            let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
            stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
        };
        let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
        let read_retention = if input.reads > 0 {
            f64_from_u64(output.reads) / f64_from_u64(input.reads)
        } else {
            0.0
        };
        let base_retention = if input.bases > 0 {
            f64_from_u64(output.bases) / f64_from_u64(input.bases)
        } else {
            0.0
        };
        let delta = FastqDeltaMetricsV1 {
            read_retention,
            base_retention,
            mean_q_delta: output.mean_q - input.mean_q,
            gc_delta: output.gc_percent - input.gc_percent,
        };
        let retention = RetentionReportMetricV1 {
            value: read_retention,
            numerator_reads: output.reads,
            denominator_reads: input.reads,
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
        Ok(serde_json::to_value(FastqUmiMetricsV1 {
            reads_in: input.reads,
            reads_out: output.reads,
            bases_in: input.bases,
            bases_out: output.bases,
            pairs_in,
            pairs_out,
            umi_pattern: plan
                .effective_params
                .get("umi_pattern")
                .and_then(serde_json::Value::as_str)
                .map_or_else(String::new, ToString::to_string),
            tag_header_format: plan
                .effective_params
                .get("read_name_transform")
                .and_then(serde_json::Value::as_str)
                .map_or_else(String::new, ToString::to_string),
            reads_with_umi: output.reads,
            extracted_umi_count: output.reads,
            invalid_umi_count: input.reads.saturating_sub(output.reads),
            mean_q_before: input.mean_q,
            mean_q_after: output.mean_q,
            delta_metrics: delta,
            retention,
        })?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use bijux_dna_core::contract::{ArtifactRole, StageIO, StageOperatingMode, ToolConstraints};
    use bijux_dna_core::ids::{ArtifactId, StageId, StageVersion, ToolId};
    use bijux_dna_domain_fastq::metrics::FastqUmiMetricsV1;
    use bijux_dna_stage_contract::{ArtifactRef, PlanDecisionReason, StagePlanV1};

    use super::umi_metrics;

    fn write_fastq(path: &std::path::Path, read_id: &str, sequence: &str) {
        let quality = "#".repeat(sequence.len());
        bijux_dna_infra::write_bytes(path, format!("@{read_id}\n{sequence}\n+\n{quality}\n"))
            .expect("write fastq");
    }

    fn umi_plan(out_dir: &std::path::Path) -> StagePlanV1 {
        StagePlanV1 {
            stage_id: StageId::from_static("fastq.extract_umis"),
            stage_instance_id: None,
            stage_version: StageVersion(1),
            tool_id: ToolId::from_static("umi_tools"),
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
                inputs: vec![ArtifactRef::required(
                    ArtifactId::new("reads_r1"),
                    PathBuf::from("reads_R1.fastq"),
                    ArtifactRole::Reads,
                )],
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
    fn umi_metrics_emit_canonical_umi_fields() {
        let temp = tempfile::tempdir().expect("tempdir");
        let reads_r1 = temp.path().join("reads_R1.fastq");
        let umi_reads_r1 = temp.path().join("umi_reads_R1.fastq");
        write_fastq(&reads_r1, "r1", "ACGT");
        write_fastq(&umi_reads_r1, "r1 umi:AAAA", "ACGT");
        bijux_dna_infra::write_bytes(
            temp.path().join("umi_report.json"),
            serde_json::json!({
                "schema_version": "bijux.fastq.extract_umis.report.v2",
                "stage": "fastq.extract_umis",
                "stage_id": "fastq.extract_umis",
                "tool_id": "umi_tools",
                "paired_mode": "single_end",
                "threads": 2,
                "umi_pattern": "NNNNNNNN",
                "extraction_location": "read1_prefix",
                "read_name_transform": "append_to_header",
                "failed_extraction_policy": "retain_unmodified",
                "grouping_policy": "pair_aware",
                "downstream_dedup_policy": "sequence_identity_recommended",
                "downstream_propagation": "header_and_report",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": null,
                "output_r1": "umi_reads_R1.fastq.gz",
                "output_r2": null,
                "report_json": "umi_report.json",
                "reads_in": 100,
                "reads_out": 98,
                "bases_in": 1000,
                "bases_out": 980,
                "pairs_in": null,
                "pairs_out": null,
                "reads_with_umi": 98,
                "failed_extractions": 2,
                "mean_q_before": 30.0,
                "mean_q_after": 30.0
            })
            .to_string(),
        )
        .expect("write report");

        let metrics: FastqUmiMetricsV1 = serde_json::from_value(
            umi_metrics(&umi_plan(temp.path()), &[reads_r1], &[umi_reads_r1]).expect("umi metrics"),
        )
        .expect("decode metrics");

        assert_eq!(metrics.umi_pattern, "NNNNNNNN");
        assert_eq!(metrics.tag_header_format, "append_to_header");
        assert_eq!(metrics.reads_with_umi, 98);
        assert_eq!(metrics.extracted_umi_count, 98);
        assert_eq!(metrics.invalid_umi_count, 2);
    }
}

fn preprocess_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
    let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let output = if outputs.is_empty() {
        input
    } else {
        let stats = stats_for_paths(&[outputs.first().map(PathBuf::as_path)])?;
        stats.first().copied().unwrap_or_else(zero_seqkit_metrics)
    };
    let (pairs_in, pairs_out) = pair_counts_from_paths(inputs, outputs)?;
    let read_retention =
        if input.reads > 0 { f64_from_u64(output.reads) / f64_from_u64(input.reads) } else { 0.0 };
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
        numerator_reads: output.reads,
        denominator_reads: input.reads,
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
    Ok(serde_json::to_value(FastqPreprocessMetricsV1 {
        reads_in: input.reads,
        reads_out: output.reads,
        bases_in: input.bases,
        bases_out: output.bases,
        pairs_in,
        pairs_out,
        mean_q_before: input.mean_q,
        mean_q_after: output.mean_q,
        delta_metrics: delta,
        retention,
    })?)
}

#[allow(clippy::too_many_lines)]
fn qc_post_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
    let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let output = input;
    let pairs_in = qc_post_pairs_from_inputs(inputs)?;
    let pairs_out = pairs_in;
    let paths = qc_post_paths(plan, outputs);
    let raw_metrics = fastqc_metrics_v2_from_dir(&paths.raw_dir);
    let trimmed_metrics = fastqc_metrics_v2_from_dir(&paths.trimmed_dir);
    let metrics_source = trimmed_metrics.as_ref().or(raw_metrics.as_ref());
    let governed_report = qc_post_report(plan, &paths.out_dir);
    let adapter_content_max = governed_report
        .as_ref()
        .and_then(|report| report.adapter_content_max)
        .or_else(|| metrics_source.and_then(|m| m.adapter_content.as_ref().map(|a| a.max_percent)));
    let adapter_content_mean =
        governed_report.as_ref().and_then(|report| report.adapter_content_mean).or_else(|| {
            metrics_source.and_then(|m| m.adapter_content.as_ref().map(|a| a.mean_percent))
        });
    let duplication_rate =
        governed_report.as_ref().and_then(|report| report.duplication_rate).or_else(|| {
            metrics_source.and_then(|m| m.duplication.as_ref().map(|d| d.duplication_rate))
        });
    let n_rate = governed_report.as_ref().and_then(|report| report.n_rate).or_else(|| {
        metrics_source.and_then(|m| m.n_content.as_ref().map(|n| n.mean_percent / 100.0))
    });
    let kmer_warning_count = governed_report
        .as_ref()
        .and_then(|report| report.kmer_warning_count)
        .or_else(|| metrics_source.and_then(|m| m.kmer_content.as_ref().map(|k| k.warning_count)));
    let overrepresented_sequence_count =
        governed_report.as_ref().and_then(|report| report.overrepresented_sequence_count).or_else(
            || metrics_source.and_then(|m| m.overrepresented_sequences.as_ref().map(|o| o.count)),
        );
    let reads_in = governed_report.as_ref().map_or(input.reads, |report| report.reads_in);
    let reads_out = governed_report.as_ref().map_or(output.reads, |report| report.reads_out);
    let bases_in = governed_report.as_ref().map_or(input.bases, |report| report.bases_in);
    let bases_out = governed_report.as_ref().map_or(output.bases, |report| report.bases_out);
    let mean_q_before = input.mean_q;
    let mean_q_after = governed_report.as_ref().map_or(output.mean_q, |report| report.mean_q);
    let read_retention =
        if reads_in > 0 { f64_from_u64(reads_out) / f64_from_u64(reads_in) } else { 0.0 };
    let base_retention =
        if bases_in > 0 { f64_from_u64(bases_out) / f64_from_u64(bases_in) } else { 0.0 };
    let delta = FastqDeltaMetricsV1 {
        read_retention,
        base_retention,
        mean_q_delta: mean_q_after - mean_q_before,
        gc_delta: output.gc_percent - input.gc_percent,
    };
    let retention = RetentionReportMetricV1 {
        value: read_retention,
        numerator_reads: reads_out,
        denominator_reads: reads_in,
        numerator_bases: bases_out,
        denominator_bases: bases_in,
        definition: "reads_out / reads_in".to_string(),
        stage_boundary: plan.stage_id.to_string(),
        conditions: retention_conditions_from_effective(
            &plan.stage_id,
            &plan.effective_params,
            &plan.params,
        ),
    };
    Ok(serde_json::to_value(FastqQcPostMetricsV1 {
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in: governed_report
            .as_ref()
            .and_then(|report| {
                if report.pairs_in.is_some_and(|value| value > 0)
                    || matches!(
                        report.paired_mode,
                        bijux_dna_domain_fastq::params::PairedMode::PairedEnd
                    )
                {
                    report.pairs_in
                } else {
                    None
                }
            })
            .or(pairs_in),
        pairs_out: governed_report
            .as_ref()
            .and_then(|report| {
                if report.pairs_out.is_some_and(|value| value > 0)
                    || matches!(
                        report.paired_mode,
                        bijux_dna_domain_fastq::params::PairedMode::PairedEnd
                    )
                {
                    report.pairs_out
                } else {
                    None
                }
            })
            .or(pairs_out),
        mean_q: Some(mean_q_after),
        mean_q_before,
        mean_q_after,
        delta_metrics: delta,
        retention,
        contamination_rate: governed_report
            .as_ref()
            .map(|report| report.contamination_rate)
            .or(Some(0.0)),
        aggregation_engine: governed_report.as_ref().and_then(|report| {
            serde_json::to_value(report.aggregation_engine.clone())
                .ok()
                .and_then(|value| value.as_str().map(ToString::to_string))
        }),
        aggregation_scope: governed_report.as_ref().and_then(|report| {
            serde_json::to_value(report.aggregation_scope.clone())
                .ok()
                .and_then(|value| value.as_str().map(ToString::to_string))
        }),
        adapter_content_max,
        adapter_content_mean,
        duplication_rate,
        n_rate,
        kmer_warning_count,
        overrepresented_sequence_count,
        raw_fastqc_dir: governed_report
            .as_ref()
            .and_then(|report| report.raw_fastqc_dir.clone())
            .or_else(|| paths.raw_dir.exists().then_some(paths.raw_dir.display().to_string())),
        trimmed_fastqc_dir: governed_report
            .as_ref()
            .and_then(|report| report.trimmed_fastqc_dir.clone())
            .or_else(|| {
                paths.trimmed_dir.exists().then_some(paths.trimmed_dir.display().to_string())
            }),
        multiqc_report: governed_report
            .as_ref()
            .and_then(|report| report.multiqc_report.clone())
            .or_else(|| {
                paths.multiqc_report.exists().then_some(paths.multiqc_report.display().to_string())
            }),
        multiqc_data: governed_report
            .as_ref()
            .and_then(|report| report.multiqc_data.clone())
            .or_else(|| {
                paths.multiqc_data.exists().then_some(paths.multiqc_data.display().to_string())
            }),
        governed_qc_input_count: governed_report
            .as_ref()
            .map(|report| report.governed_qc_input_count),
        governed_qc_contributor_stage_ids: governed_report
            .as_ref()
            .map(|report| report.governed_qc_contributor_stage_ids.clone())
            .unwrap_or_default(),
        governed_qc_contributor_tool_ids: governed_report
            .as_ref()
            .map(|report| report.governed_qc_contributor_tool_ids.clone())
            .unwrap_or_default(),
        governed_qc_lineage_hash: governed_report
            .as_ref()
            .and_then(|report| report.governed_qc_lineage_hash.clone()),
        multiqc_sample_count: governed_report
            .as_ref()
            .and_then(|report| report.multiqc_sample_count),
        multiqc_module_count: governed_report
            .as_ref()
            .and_then(|report| report.multiqc_module_count),
    })?)
}

struct QcPostPaths {
    out_dir: PathBuf,
    raw_dir: PathBuf,
    trimmed_dir: PathBuf,
    multiqc_report: PathBuf,
    multiqc_data: PathBuf,
}

fn qc_post_paths(plan: &StagePlanV1, outputs: &[PathBuf]) -> QcPostPaths {
    let out_dir = path_from_params(&plan.params, "out_dir")
        .unwrap_or_else(|| outputs.first().cloned().unwrap_or_default());
    QcPostPaths {
        raw_dir: out_dir.join("fastqc_raw"),
        trimmed_dir: out_dir.join("fastqc_trimmed"),
        multiqc_report: out_dir.join("multiqc_report.html"),
        multiqc_data: out_dir.join("multiqc_data"),
        out_dir,
    }
}

fn qc_post_pairs_from_inputs(inputs: &[PathBuf]) -> Result<Option<u64>> {
    if inputs.len() < 2 {
        return Ok(None);
    }
    let r1 = stats_or_zero(inputs.first().map(PathBuf::as_path))?;
    let r2 = stats_or_zero(inputs.get(1).map(PathBuf::as_path))?;
    Ok(Some(r1.reads.min(r2.reads)))
}

fn qc_post_report(
    plan: &StagePlanV1,
    out_dir: &std::path::Path,
) -> Option<bijux_dna_domain_fastq::ReportQcReportV1> {
    path_from_params(&plan.params, "report_json")
        .or_else(|| {
            let fallback = out_dir.join("report_qc_report.json");
            fallback.exists().then_some(fallback)
        })
        .and_then(|report_path| std::fs::read_to_string(report_path).ok())
        .and_then(|raw| crate::observer::parse_report_qc_report(&raw).ok())
}

fn stats_neutral_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
    outputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let stats = stats_for_paths(&[inputs.first().map(PathBuf::as_path)])?;
    let input = stats.first().copied().unwrap_or_else(zero_seqkit_metrics);
    let output = input;
    let governed_report = path_from_params(&plan.params, "qc_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "qc_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("qc.json");
            fallback.exists().then_some(fallback)
        })
        .and_then(|report_path| std::fs::read_to_string(&report_path).ok())
        .and_then(|raw| crate::observer::parse_profile_reads_report(&raw).ok());
    let (pairs_in, pairs_out) = if let Some(report) = governed_report.as_ref() {
        match report.paired_mode {
            bijux_dna_domain_fastq::params::PairedMode::PairedEnd => {
                let pairs = report.mate_summaries.iter().map(|mate| mate.reads).min().unwrap_or(0);
                (Some(pairs), Some(pairs))
            }
            bijux_dna_domain_fastq::params::PairedMode::SingleEnd
            | bijux_dna_domain_fastq::params::PairedMode::Unknown => (None, None),
        }
    } else {
        pair_counts_from_paths(inputs, outputs)?
    };
    let (read_length_distribution, gc_distribution) = if let Some(report) = governed_report.as_ref()
    {
        (
            report.length_histogram.iter().map(|bin| (bin.length, bin.count)).collect::<Vec<_>>(),
            distributions_for_path(inputs.first().map(PathBuf::as_path))?.1,
        )
    } else {
        distributions_for_path(inputs.first().map(PathBuf::as_path))?
    };
    Ok(serde_json::to_value(FastqStatsNeutralMetricsV1 {
        reads_in: governed_report.as_ref().map_or(input.reads, |report| report.reads_total),
        reads_out: governed_report.as_ref().map_or(output.reads, |report| report.reads_total),
        bases_in: governed_report.as_ref().map_or(input.bases, |report| report.bases_total),
        bases_out: governed_report.as_ref().map_or(output.bases, |report| report.bases_total),
        pairs_in,
        pairs_out,
        read_length_distribution,
        gc_distribution,
    })?)
}

fn profile_read_lengths_metrics(
    plan: &StagePlanV1,
    inputs: &[PathBuf],
) -> Result<serde_json::Value> {
    let report_path = path_from_params(&plan.params, "report_json")
        .or_else(|| {
            plan.io
                .outputs
                .iter()
                .find(|artifact| artifact.name.as_str() == "report_json")
                .map(|artifact| artifact.path.clone())
        })
        .or_else(|| {
            let fallback = plan.out_dir.join("profile_read_lengths_report.json");
            fallback.exists().then_some(fallback)
        });
    let governed_report = report_path
        .and_then(|path| std::fs::read_to_string(&path).ok())
        .and_then(|raw| crate::observer::parse_profile_read_lengths_report(&raw).ok());
    if let Some(report) = governed_report {
        Ok(serde_json::json!({
            "read_count": report.read_count,
            "mean_read_length": report.mean_read_length,
            "max_read_length": report.max_read_length,
            "distinct_lengths": report.distinct_lengths,
            "threads": report.threads,
            "histogram_bins": report.histogram_bins,
            "paired_mode": report.paired_mode,
            "pairs_in": matches!(report.paired_mode, bijux_dna_domain_fastq::params::PairedMode::PairedEnd)
                .then_some(report.read_count / 2),
            "pairs_out": matches!(report.paired_mode, bijux_dna_domain_fastq::params::PairedMode::PairedEnd)
                .then_some(report.read_count / 2),
            "read_length_distribution": report
                .histogram
                .into_iter()
                .map(|bin| (bin.read_length, bin.count))
                .collect::<Vec<_>>(),
        }))
    } else {
        let (read_length_distribution, _gc_distribution) =
            distributions_for_path(inputs.first().map(PathBuf::as_path))?;
        let read_count = read_length_distribution.iter().map(|(_, count)| *count).sum::<u64>();
        let weighted_total = read_length_distribution
            .iter()
            .map(|(length, count)| length.saturating_mul(*count))
            .sum::<u64>();
        let mean_read_length = if read_count == 0 {
            0.0
        } else {
            super::super::f64_from_u64(weighted_total) / super::super::f64_from_u64(read_count)
        };
        let max_read_length =
            read_length_distribution.iter().map(|(length, _count)| *length).max().unwrap_or(0);
        Ok(serde_json::json!({
            "read_count": read_count,
            "mean_read_length": mean_read_length,
            "max_read_length": max_read_length,
            "distinct_lengths": read_length_distribution.len(),
            "read_length_distribution": read_length_distribution,
        }))
    }
}
