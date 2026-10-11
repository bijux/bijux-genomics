//! Owner: bijux-dna-bench-workflows
//! Core germline observations truth.
use super::super::{
    bail, combined_gc_percent, combined_mean_quality, json, length_histogram,
    path_relative_to_repo, read_fastq_records, total_bases, write_fastq_records, Context,
    FastqProfileStageReport, FastqRecord, FilterDecision, FilterDropReason, FilterEffectiveParams,
    FilterReadsReportV1, PairedMode, Path, Result, FASTQ_FILTER_TOOL_ID, FASTQ_PROFILE_TOOL_ID,
    FILTER_READS_REPORT_SCHEMA_VERSION, PIPELINE_SAMPLE_ID,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn build_profile_report(
    input_r1: &Path,
    input_r2: Option<&Path>,
) -> Result<FastqProfileStageReport> {
    let records_r1 = read_fastq_records(input_r1)?;
    let records_r2 = input_r2.map(read_fastq_records).transpose()?;
    let bases_r1 = total_bases(&records_r1);
    let bases_r2 = records_r2.as_deref().map_or(0, total_bases);
    let reads_r1 = u64::try_from(records_r1.len()).unwrap_or(u64::MAX);
    let reads_r2 =
        records_r2.as_ref().map_or(0, |records| u64::try_from(records.len()).unwrap_or(u64::MAX));
    let length_histogram = length_histogram(&records_r1, records_r2.as_deref());
    Ok(FastqProfileStageReport {
        schema_version: "bijux.bench.local_core_germline_micro_pipeline.fastq_profile.v1"
            .to_string(),
        stage_id: stage::FASTQ_STATS_NEUTRAL.to_string(),
        tool_id: FASTQ_PROFILE_TOOL_ID.to_string(),
        sample_id: PIPELINE_SAMPLE_ID.to_string(),
        input_r1: input_r1.display().to_string(),
        input_r2: input_r2.map(|path| path.display().to_string()),
        reads_total: reads_r1 + reads_r2,
        bases_total: bases_r1 + bases_r2,
        mean_q: combined_mean_quality(&records_r1, records_r2.as_deref()),
        gc_percent: combined_gc_percent(&records_r1, records_r2.as_deref()),
        length_histogram,
    })
}

pub(crate) fn write_filter_reads_report(
    repo_root: &Path,
    paths: &FilterReadPaths<'_>,
    effective_params: &FilterEffectiveParams,
) -> Result<FilterReadsReportV1> {
    let FilterReadPaths {
        input_r1,
        input_r2,
        output_r1,
        output_r2,
        report_path,
        raw_backend_report,
    } = *paths;

    let FilterReadSelection {
        input_records,
        mate_records,
        decisions,
        mate_decisions,
        retained_r1,
        retained_r2,
    } = select_reads(input_r1, input_r2, effective_params)?;
    write_fastq_records(output_r1, &retained_r1)?;
    if let (Some(output_r2), Some(retained_r2)) = (output_r2, retained_r2.as_ref()) {
        write_fastq_records(output_r2, retained_r2)?;
    }

    let reads_removed_by_n = decisions
        .iter()
        .chain(mate_decisions.as_deref().unwrap_or(&[]).iter())
        .filter(|decision| decision.drop_reason == Some(FilterDropReason::TooManyN))
        .count() as u64;
    let reads_removed_low_complexity = decisions
        .iter()
        .chain(mate_decisions.as_deref().unwrap_or(&[]).iter())
        .filter(|decision| decision.drop_reason == Some(FilterDropReason::LowComplexity))
        .count() as u64;
    let reads_in = u64::try_from(input_records.len() + mate_records.as_ref().map_or(0, Vec::len))
        .context("count FASTQ filter input reads")?;
    let reads_out = u64::try_from(retained_r1.len() + retained_r2.as_ref().map_or(0, Vec::len))
        .context("count FASTQ filter output reads")?;
    let bases_in = total_bases(&input_records) + mate_records.as_deref().map_or(0, total_bases);
    let bases_out = total_bases(&retained_r1) + retained_r2.as_deref().map_or(0, total_bases);
    let backend_metrics = json!({
        "passed_filter_reads": reads_out,
        "too_many_n_reads": reads_removed_by_n,
        "low_complexity_reads": reads_removed_low_complexity,
    });
    let report = FilterReadsReportV1 {
        schema_version: FILTER_READS_REPORT_SCHEMA_VERSION.to_string(),
        stage: stage::FASTQ_FILTER.to_string(),
        stage_id: stage::FASTQ_FILTER.to_string(),
        tool_id: FASTQ_FILTER_TOOL_ID.to_string(),
        paired_mode: if input_r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: effective_params.threads,
        input_r1: path_relative_to_repo(repo_root, input_r1),
        input_r2: input_r2.map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, output_r1),
        output_r2: output_r2.map(|path| path_relative_to_repo(repo_root, path)),
        report_json: path_relative_to_repo(repo_root, report_path),
        max_n: effective_params.max_n,
        max_n_fraction: effective_params.max_n_fraction,
        max_n_count: effective_params.max_n_count,
        low_complexity_threshold: effective_params.low_complexity_threshold,
        entropy_threshold: effective_params.entropy_threshold,
        n_policy: effective_params.n_policy.clone(),
        polyx_policy: effective_params.polyx_policy.clone(),
        contaminant_db: effective_params.contaminant_db.clone(),
        reads_in,
        reads_out,
        reads_dropped: reads_in.saturating_sub(reads_out),
        reads_removed_by_n,
        reads_removed_by_entropy: 0,
        reads_removed_low_complexity,
        reads_removed_by_kmer: 0,
        reads_removed_contaminant_kmer: 0,
        reads_removed_by_length: 0,
        bases_in,
        bases_out,
        pairs_in: input_r2.map(|_| input_records.len() as u64),
        pairs_out: input_r2.map(|_| retained_r1.len() as u64),
        mean_q_before: combined_mean_quality(&input_records, mate_records.as_deref()),
        mean_q_after: combined_mean_quality(&retained_r1, retained_r2.as_deref()),
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: Some(path_relative_to_repo(repo_root, raw_backend_report)),
        raw_backend_report_format: Some("fastp_json".to_string()),
        backend_metrics: Some(backend_metrics.clone()),
    };
    bijux_dna_infra::atomic_write_json(report_path, &report)?;
    bijux_dna_infra::write_bytes(
        raw_backend_report,
        json!({
            "filtering_result": {
                "passed_filter_reads": backend_metrics["passed_filter_reads"],
                "too_many_N_reads": backend_metrics["too_many_n_reads"],
                "low_complexity_reads": backend_metrics["low_complexity_reads"],
            }
        })
        .to_string(),
    )?;
    Ok(report)
}

pub(crate) fn filter_drop_reason(
    record: &FastqRecord,
    effective_params: &FilterEffectiveParams,
) -> Option<FilterDropReason> {
    let max_n_count = effective_params.max_n_count.or(effective_params.max_n);
    if let Some(limit) = max_n_count {
        let n_count = u32::try_from(
            record.sequence.bytes().filter(|base| matches!(*base, b'N' | b'n')).count(),
        )
        .unwrap_or(u32::MAX);
        if n_count > limit {
            return Some(FilterDropReason::TooManyN);
        }
    }

    if let Some(threshold) =
        effective_params.low_complexity_threshold.or(effective_params.entropy_threshold)
    {
        let complexity = local_complexity_score(&record.sequence);
        if complexity < threshold {
            return Some(FilterDropReason::LowComplexity);
        }
    }

    None
}

#[expect(
    clippy::cast_precision_loss,
    reason = "Approximate normalized metrics retain the existing floating-point calculation; integer counts remain separate"
)]
pub(crate) fn local_complexity_score(sequence: &str) -> f64 {
    let bytes = sequence.as_bytes();
    if bytes.len() <= 1 {
        return 0.0;
    }
    let transitions = bytes.windows(2).filter(|window| window[0] != window[1]).count() as f64;
    (transitions / (bytes.len() as f64 - 1.0)) * 100.0
}

struct FilterReadSelection {
    input_records: Vec<FastqRecord>,
    mate_records: Option<Vec<FastqRecord>>,
    decisions: Vec<FilterDecision>,
    mate_decisions: Option<Vec<FilterDecision>>,
    retained_r1: Vec<FastqRecord>,
    retained_r2: Option<Vec<FastqRecord>>,
}

fn select_reads(
    input_r1: &Path,
    input_r2: Option<&Path>,
    effective_params: &FilterEffectiveParams,
) -> Result<FilterReadSelection> {
    let input_records = read_fastq_records(input_r1)?;
    let mate_records = input_r2.map(read_fastq_records).transpose()?;
    let decisions = input_records
        .iter()
        .cloned()
        .map(|record| FilterDecision {
            drop_reason: filter_drop_reason(&record, effective_params),
            record,
        })
        .collect::<Vec<_>>();
    let mate_decisions = mate_records.as_ref().map(|records| {
        records
            .iter()
            .cloned()
            .map(|record| FilterDecision {
                drop_reason: filter_drop_reason(&record, effective_params),
                record,
            })
            .collect::<Vec<_>>()
    });
    if let Some(mate_decisions) = mate_decisions.as_ref() {
        if mate_decisions.len() != decisions.len() {
            bail!("paired FASTQ decisions must keep synchronized record counts");
        }
    }

    let retained_r1 = decisions
        .iter()
        .enumerate()
        .filter(|(index, decision)| {
            decision.drop_reason.is_none()
                && mate_decisions
                    .as_ref()
                    .and_then(|mate| mate.get(*index))
                    .is_none_or(|mate_decision| mate_decision.drop_reason.is_none())
        })
        .map(|(_, decision)| decision.record.clone())
        .collect::<Vec<_>>();
    let retained_r2 = mate_decisions.as_ref().map(|mate| {
        mate.iter()
            .enumerate()
            .filter(|(index, decision)| {
                decision.drop_reason.is_none() && decisions[*index].drop_reason.is_none()
            })
            .map(|(_, decision)| decision.record.clone())
            .collect::<Vec<_>>()
    });
    Ok(FilterReadSelection {
        input_records,
        mate_records,
        decisions,
        mate_decisions,
        retained_r1,
        retained_r2,
    })
}

#[derive(Clone, Copy)]
pub(crate) struct FilterReadPaths<'a> {
    pub(crate) input_r1: &'a Path,
    pub(crate) input_r2: Option<&'a Path>,
    pub(crate) output_r1: &'a Path,
    pub(crate) output_r2: Option<&'a Path>,
    pub(crate) report_path: &'a Path,
    pub(crate) raw_backend_report: &'a Path,
}
