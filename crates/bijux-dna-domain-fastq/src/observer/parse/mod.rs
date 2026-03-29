//! Owner: bijux-dna-domain-fastq
//! Observer parsers for tool stdout and report artifacts.
//! Supported formats must be deterministic and stable across versions.

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use tracing::warn;

use crate::metrics::{
    AdapterRemovalToolMetricsV1, FastpToolMetricsV1, FastqcToolMetricsV1, MultiqcToolMetricsV1,
    SamtoolsFlagstatMetricsV1, SeqkitToolMetricsV1,
};
use crate::params::{
    detect_adapters::{AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode},
    PairedMode,
};
use crate::DuplicateClassEntryV1;
use crate::{
    ClusterOtusReportV1, CorrectErrorsReportV1, DepleteHostReportV1,
    DepleteReferenceContaminantsReportV1, DepleteRrnaReportV1, DetectAdaptersReportV1,
    ExtractUmisReportV1, FilterLowComplexityReportV1, FilterReadsReportV1, IndexReferenceReportV1,
    InferAsvsReportV1, MergePairsReportV1, NormalizeAbundanceReportV1, NormalizePrimersReportV1,
    OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1, ProfileReadLengthBinV1,
    ProfileReadLengthsReportV1, ProfileReadsHistogramBinV1, ProfileReadsMateSummaryV1,
    ProfileReadsReportV1, RemoveChimerasReportV1, RemoveDuplicatesProvenanceV1,
    RemoveDuplicatesReportV1, ReportQcReportV1, ScreenTaxonomyReportV1,
    TaxonomyScreenSummaryEntryV1, TerminalDamageReportV1, TrimPolygReportV1, TrimReadsReportV1,
    ValidatedReadsManifestV1, ValidationReportV1,
};
use bijux_dna_core::prelude::measure::SeqkitMetrics;

mod sequence;
mod reports;
mod correct_errors;
mod depletion;

pub use self::sequence::{
    parse_fastqvalidator_count, parse_length_histogram, parse_seqkit_stats,
};


pub use self::reports::{
    parse_cluster_otus_report, parse_filter_reads_report, parse_index_reference_report,
    parse_infer_asvs_report, parse_merge_pairs_report, parse_normalize_abundance_report,
    parse_normalize_primers_report, parse_report_qc_report, parse_screen_taxonomy_report,
    parse_terminal_damage_report, parse_trim_polyg_report, parse_trim_reads_report,
    parse_validated_reads_manifest, parse_validation_report,
};


pub use self::correct_errors::parse_correct_errors_report;




/// # Errors
/// Returns an error if the governed profile-reads report JSON cannot be parsed.
pub fn parse_profile_reads_report(report_json: &str) -> Result<ProfileReadsReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_reads_report(report_json))
        .context("parse profile reads report")
}

/// # Errors
/// Returns an error if the governed profile-read-lengths report JSON cannot be parsed.
pub fn parse_profile_read_lengths_report(report_json: &str) -> Result<ProfileReadLengthsReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_read_lengths_report(report_json))
        .context("parse profile read lengths report")
}

/// # Errors
/// Returns an error if the governed overrepresented-sequence report JSON cannot be parsed.
pub fn parse_profile_overrepresented_report(
    report_json: &str,
) -> Result<ProfileOverrepresentedReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_overrepresented_report(report_json))
        .context("parse profile overrepresented report")
}


pub use self::depletion::{
    parse_deplete_host_report, parse_deplete_reference_contaminants_report,
    parse_deplete_rrna_report,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDetectAdaptersReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    inspection_mode: String,
    report_only: bool,
    evidence_engine: String,
    input_fastq: String,
    paired_input: bool,
    candidate_adapter_count: u64,
    adapter_trimmed_fraction: Option<f64>,
    fastqc_dir: String,
    runtime_s: Option<f64>,
    memory_mb: Option<f64>,
    exit_code: Option<i32>,
}

fn parse_legacy_detect_adapters_report(report_json: &str) -> Result<DetectAdaptersReportV1> {
    let legacy: LegacyDetectAdaptersReportV1 =
        serde_json::from_str(report_json).context("parse legacy detect adapters report")?;
    if legacy.schema_version != "bijux.fastq.detect_adapters.report.v1" {
        return Err(anyhow!(
            "unsupported detect adapters report schema {}",
            legacy.schema_version
        ));
    }
    let inspection_mode = match legacy.inspection_mode.as_str() {
        "evidence_only" => AdapterInspectionMode::EvidenceOnly,
        other => {
            return Err(anyhow!(
                "unsupported detect adapters inspection mode {other}"
            ))
        }
    };
    Ok(DetectAdaptersReportV1 {
        schema_version: "bijux.fastq.detect_adapters.report.v2".to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id,
        paired_mode: if legacy.paired_input {
            PairedMode::PairedEnd
        } else {
            PairedMode::SingleEnd
        },
        threads: 1,
        inspection_mode,
        report_only: legacy.report_only,
        evidence_engine: legacy.evidence_engine,
        evidence_scope: AdapterEvidenceScope::FullInput,
        evidence_format: AdapterEvidenceFormat::FastqcSummary,
        evidence_artifact_id: "report_json".to_string(),
        input_r1: legacy.input_fastq,
        input_r2: None,
        report_json: "adapter_report.json".to_string(),
        adapter_evidence_dir: legacy.fastqc_dir,
        reads_in: 0,
        reads_out: 0,
        bases_in: 0,
        bases_out: 0,
        pairs_in: None,
        pairs_out: None,
        mean_q: 0.0,
        candidate_adapter_count: legacy.candidate_adapter_count,
        adapter_trimmed_fraction: legacy.adapter_trimmed_fraction,
        adapter_content_max: None,
        adapter_content_mean: None,
        duplication_rate: None,
        n_rate: None,
        kmer_warning_count: None,
        overrepresented_sequence_count: None,
        runtime_s: legacy.runtime_s,
        memory_mb: legacy.memory_mb,
        exit_code: legacy.exit_code,
        raw_backend_report: None,
        raw_backend_report_format: None,
    })
}

/// # Errors
/// Returns an error if the detect-adapters report cannot be parsed.
pub fn parse_detect_adapters_report(report_json: &str) -> Result<DetectAdaptersReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_detect_adapters_report(report_json))
        .context("parse detect adapters report")
}

/// # Errors
/// Returns an error if the governed remove-chimeras report JSON cannot be parsed.
pub fn parse_remove_chimeras_report(report_json: &str) -> Result<RemoveChimerasReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_remove_chimeras_report(report_json))
        .context("parse remove chimeras report")
}

/// # Errors
/// Returns an error if a taxonomy summary TSV cannot be reduced to label/percent entries.
pub fn parse_screen_summary_tsv(summary_tsv: &str) -> Result<Vec<TaxonomyScreenSummaryEntryV1>> {
    let mut entries = Vec::new();
    for (idx, line) in summary_tsv.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 3 {
            return Err(anyhow!(
                "screen report line {} has {} columns",
                idx + 1,
                parts.len()
            ));
        }
        let label = parts[0].trim().to_string();
        let percent = parts
            .last()
            .ok_or_else(|| anyhow!("screen report line {} missing percent", idx + 1))?
            .trim()
            .trim_end_matches('%')
            .parse::<f64>()
            .with_context(|| format!("screen report line {} percent parse", idx + 1))?;
        entries.push(TaxonomyScreenSummaryEntryV1 { label, percent });
    }
    Ok(entries)
}

/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_deduplicate_report(report_json: &str) -> Result<(u64, u64)> {
    let report = parse_remove_duplicates_report(report_json)?;
    Ok((report.reads_in, report.reads_out))
}

/// # Errors
/// Returns an error if the governed remove-duplicates report JSON cannot be parsed.
pub fn parse_remove_duplicates_report(report_json: &str) -> Result<RemoveDuplicatesReportV1> {
    match serde_json::from_str(report_json) {
        Ok(report) => Ok(report),
        Err(governed_error) => {
            if looks_like_governed_remove_duplicates_report(report_json) {
                Err(governed_error).context("parse remove duplicates report")
            } else {
                parse_legacy_remove_duplicates_report(report_json)
                    .context("parse remove duplicates report")
            }
        }
    }
}

/// # Errors
/// Returns an error if the governed remove-duplicates provenance JSON cannot be parsed.
pub fn parse_remove_duplicates_provenance(
    provenance_json: &str,
) -> Result<RemoveDuplicatesProvenanceV1> {
    serde_json::from_str(provenance_json).context("parse remove duplicates provenance")
}

/// # Errors
/// Returns an error if the duplicate-classes TSV cannot be parsed.
pub fn parse_duplicate_classes_tsv(classes_tsv: &str) -> Result<Vec<DuplicateClassEntryV1>> {
    let mut entries = Vec::new();
    for (idx, line) in classes_tsv.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if idx == 0 && line.eq_ignore_ascii_case("class\treads_removed\tpaired_mode") {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 3 {
            return Err(anyhow!(
                "duplicate classes line {} has {} columns; expected 3",
                idx + 1,
                parts.len()
            ));
        }
        entries.push(DuplicateClassEntryV1 {
            class: parts[0].trim().to_string(),
            reads_removed: parts[1].trim().parse::<u64>().with_context(|| {
                format!("parse duplicate classes line {} reads_removed", idx + 1)
            })?,
            paired_mode: serde_json::from_value(serde_json::json!(parts[2].trim()))
                .with_context(|| format!("parse duplicate classes line {} paired_mode", idx + 1))?,
        });
    }
    Ok(entries)
}

fn parse_legacy_remove_duplicates_report(report_json: &str) -> Result<RemoveDuplicatesReportV1> {
    let reads_in = parse_report_u64_field(report_json, "reads_in")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_in"))?;
    let reads_out = parse_report_u64_field(report_json, "reads_out")
        .ok_or_else(|| anyhow!("deduplicate report missing reads_out"))?;
    let duplicates_removed = parse_report_u64_field(report_json, "duplicates_removed")
        .unwrap_or_else(|| reads_in.saturating_sub(reads_out));
    Ok(RemoveDuplicatesReportV1 {
        schema_version: "bijux.fastq.remove_duplicates.report.v1_legacy".to_string(),
        stage: "fastq.remove_duplicates".to_string(),
        stage_id: "fastq.remove_duplicates".to_string(),
        tool_id: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| {
                value
                    .get("tool_id")
                    .and_then(serde_json::Value::as_str)
                    .map(ToString::to_string)
            })
            .unwrap_or_else(|| "unknown".to_string()),
        paired_mode: match parse_report_u64_field(report_json, "pairs_in") {
            Some(_) => crate::PairedMode::PairedEnd,
            None => crate::PairedMode::SingleEnd,
        },
        threads: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| value.get("threads").and_then(serde_json::Value::as_u64))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        dedup_mode: crate::params::remove_duplicates::DedupMode::Exact,
        keep_order: true,
        input_r1: String::new(),
        input_r2: None,
        output_r1: String::new(),
        output_r2: None,
        reads_in,
        reads_out,
        reads_in_r2: parse_report_u64_field(report_json, "reads_in_r2"),
        reads_out_r2: parse_report_u64_field(report_json, "reads_out_r2"),
        pairs_in: parse_report_u64_field(report_json, "pairs_in"),
        pairs_out: parse_report_u64_field(report_json, "pairs_out"),
        pair_count_match: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| {
                value
                    .get("pair_count_match")
                    .and_then(serde_json::Value::as_bool)
            }),
        duplicates_removed,
        dedup_rate: serde_json::from_str::<serde_json::Value>(report_json)
            .ok()
            .and_then(|value| value.get("dedup_rate").and_then(serde_json::Value::as_f64))
            .unwrap_or_else(|| {
                if reads_in == 0 {
                    0.0
                } else {
                    u64_to_f64(duplicates_removed) / u64_to_f64(reads_in)
                }
            }),
        duplicate_classes_tsv: None,
        duplicate_provenance_json: None,
        duplicate_classes: Vec::new(),
        raw_backend_report: None,
        raw_backend_report_format: None,
        runtime_s: None,
        memory_mb: None,
    })
}

fn looks_like_governed_remove_duplicates_report(report_json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(report_json)
        .ok()
        .and_then(|value| value.as_object().cloned())
        .is_some_and(|object| {
            object
                .get("schema_version")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|schema| schema.starts_with("bijux.fastq.remove_duplicates.report."))
                || object
                    .get("stage")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|stage| stage == "fastq.remove_duplicates")
                || object
                    .get("stage_id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|stage_id| stage_id == "fastq.remove_duplicates")
        })
}

fn parse_legacy_remove_chimeras_report(report_json: &str) -> Result<RemoveChimerasReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy remove chimeras json")?;
    Ok(RemoveChimerasReportV1 {
        schema_version: "bijux.fastq.remove_chimeras.report.v1_legacy".to_string(),
        stage: "fastq.remove_chimeras".to_string(),
        stage_id: "fastq.remove_chimeras".to_string(),
        tool_id: json
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        paired_mode: crate::PairedMode::SingleEnd,
        threads: json
            .get("threads")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        method: "legacy_metrics_only".to_string(),
        detection_scope: "denovo".to_string(),
        chimera_removed_definition:
            "reads flagged as chimeric are excluded from downstream abundance tables".to_string(),
        input_reads: String::new(),
        output_reads: String::new(),
        chimera_metrics_json: String::new(),
        chimeras_fasta: None,
        uchime_report_tsv: None,
        reads_in: None,
        reads_out: json
            .get("non_chimera_reads")
            .and_then(serde_json::Value::as_u64),
        chimeras_removed: json
            .get("chimeras_removed")
            .and_then(serde_json::Value::as_u64),
        chimera_fraction: json
            .get("chimera_fraction")
            .and_then(serde_json::Value::as_f64),
        used_fallback: json
            .get("used_fallback")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false),
        raw_backend_report: None,
        raw_backend_report_format: None,
        runtime_s: None,
        memory_mb: None,
        exit_code: None,
        backend_metrics: None,
    })
}

fn parse_legacy_profile_reads_report(report_json: &str) -> Result<ProfileReadsReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy profile reads json")?;
    let length_histogram = json
        .get("length_histogram")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let length = entry.get("length").and_then(serde_json::Value::as_u64)?;
            let count = entry.get("count").and_then(serde_json::Value::as_u64)?;
            Some(ProfileReadsHistogramBinV1 { length, count })
        })
        .collect::<Vec<_>>();
    Ok(ProfileReadsReportV1 {
        schema_version: "bijux.fastq.profile_reads.report.v1_legacy".to_string(),
        stage: "fastq.profile_reads".to_string(),
        stage_id: "fastq.profile_reads".to_string(),
        tool_id: json
            .get("tool_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        paired_mode: match json.get("paired_mode").and_then(serde_json::Value::as_str) {
            Some("paired_end") => crate::PairedMode::PairedEnd,
            _ => crate::PairedMode::SingleEnd,
        },
        threads: json
            .get("threads")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        input_r1: String::new(),
        input_r2: None,
        qc_json: String::new(),
        qc_tsv: String::new(),
        qc_plots_dir: None,
        length_histogram_source: "legacy_qc_json".to_string(),
        reads_total: json
            .get("reads_total")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        bases_total: json
            .get("bases_total")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        mean_q: json
            .get("mean_q")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        gc_percent: json
            .get("gc_percent")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        length_histogram,
        mate_summaries: vec![ProfileReadsMateSummaryV1 {
            label: "reads_r1".to_string(),
            reads: json
                .get("reads_total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            bases: json
                .get("bases_total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            mean_q: json.get("mean_q").and_then(serde_json::Value::as_f64),
            gc_percent: json.get("gc_percent").and_then(serde_json::Value::as_f64),
        }],
        runtime_s: json.get("runtime_s").and_then(serde_json::Value::as_f64),
        memory_mb: json.get("memory_mb").and_then(serde_json::Value::as_f64),
        exit_code: json
            .get("exit_code")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok()),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

fn parse_legacy_profile_read_lengths_report(
    report_json: &str,
) -> Result<ProfileReadLengthsReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy profile read lengths json")?;
    let histogram = json
        .get("histogram")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let read_length = entry
                .get("read_length")
                .and_then(serde_json::Value::as_u64)?;
            let count = entry.get("count").and_then(serde_json::Value::as_u64)?;
            Some(ProfileReadLengthBinV1 { read_length, count })
        })
        .collect::<Vec<_>>();
    let read_count = histogram.iter().map(|bin| bin.count).sum::<u64>();
    let total_length = histogram
        .iter()
        .map(|bin| bin.read_length.saturating_mul(bin.count))
        .sum::<u64>();
    let mean_read_length = if read_count == 0 {
        0.0
    } else {
        u64_to_f64(total_length) / u64_to_f64(read_count)
    };
    Ok(ProfileReadLengthsReportV1 {
        schema_version: "bijux.fastq.profile_read_lengths.report.v1_legacy".to_string(),
        stage: "fastq.profile_read_lengths".to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
        tool_id: json
            .get("tool_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        paired_mode: match json.get("paired_mode").and_then(serde_json::Value::as_str) {
            Some("paired_end") => crate::PairedMode::PairedEnd,
            _ => crate::PairedMode::SingleEnd,
        },
        threads: json
            .get("threads")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        histogram_bins: u32::try_from(histogram.len()).unwrap_or(u32::MAX).max(1),
        input_r1: String::new(),
        input_r2: None,
        length_distribution_tsv: String::new(),
        length_distribution_json: String::new(),
        report_json: String::new(),
        read_count,
        mean_read_length,
        max_read_length: histogram
            .iter()
            .map(|bin| bin.read_length)
            .max()
            .unwrap_or(0),
        distinct_lengths: histogram.len() as u64,
        histogram,
        runtime_s: json.get("runtime_s").and_then(serde_json::Value::as_f64),
        memory_mb: json.get("memory_mb").and_then(serde_json::Value::as_f64),
        exit_code: json
            .get("exit_code")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok()),
        raw_backend_report: None,
        raw_backend_report_format: None,
    })
}

fn parse_legacy_profile_overrepresented_report(
    report_json: &str,
) -> Result<ProfileOverrepresentedReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy profile overrepresented json")?;
    let rows = json
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some(OverrepresentedSequenceRowV1 {
                sequence: entry
                    .get("sequence")
                    .and_then(serde_json::Value::as_str)?
                    .to_string(),
                count: entry.get("count").and_then(serde_json::Value::as_u64)?,
                fraction: entry
                    .get("fraction")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0),
                flag: entry
                    .get("flag")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("background")
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();
    Ok(ProfileOverrepresentedReportV1 {
        schema_version: "bijux.fastq.profile_overrepresented.report.v1_legacy".to_string(),
        stage: "fastq.profile_overrepresented_sequences".to_string(),
        stage_id: "fastq.profile_overrepresented_sequences".to_string(),
        tool_id: json
            .get("tool_id")
            .or_else(|| json.get("tool"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        paired_mode: match json.get("paired_mode").and_then(serde_json::Value::as_str) {
            Some("paired_end") => crate::PairedMode::PairedEnd,
            _ => crate::PairedMode::SingleEnd,
        },
        threads: json
            .get("threads")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        top_k: json
            .get("top_k")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or_else(|| u32::try_from(rows.len()).unwrap_or(u32::MAX).max(1)),
        input_r1: String::new(),
        input_r2: None,
        overrepresented_sequences_tsv: String::new(),
        overrepresented_sequences_json: String::new(),
        report_json: String::new(),
        sequence_count: json
            .get("sequence_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(rows.len() as u64),
        flagged_sequences: json
            .get("flagged_sequences")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                rows.iter()
                    .filter(|row| row.flag == "overrepresented")
                    .count() as u64
            }),
        top_fraction: json
            .get("top_fraction")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or_else(|| rows.first().map_or(0.0, |row| row.fraction)),
        rows,
        runtime_s: json.get("runtime_s").and_then(serde_json::Value::as_f64),
        memory_mb: json.get("memory_mb").and_then(serde_json::Value::as_f64),
        exit_code: json
            .get("exit_code")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok()),
        raw_backend_report: None,
        raw_backend_report_format: None,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyFilterLowComplexityReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    input_fastq: String,
    output_fastq: String,
    #[serde(default)]
    output_fastq_r2: Option<String>,
    reads_in: u64,
    reads_out: u64,
    reads_removed_low_complexity: u64,
    #[serde(default)]
    runtime_s: Option<f64>,
    #[serde(default)]
    memory_mb: Option<f64>,
    #[serde(default)]
    exit_code: Option<i32>,
}

fn parse_legacy_filter_low_complexity_report(
    report_json: &str,
) -> Result<FilterLowComplexityReportV1> {
    let legacy: LegacyFilterLowComplexityReportV1 =
        serde_json::from_str(report_json).context("parse legacy low complexity report")?;
    if legacy.schema_version != "bijux.fastq.filter_low_complexity.report.v1" {
        return Err(anyhow!(
            "unsupported low-complexity report schema {}",
            legacy.schema_version
        ));
    }
    let paired_mode = if legacy.output_fastq_r2.is_some() {
        crate::PairedMode::PairedEnd
    } else {
        crate::PairedMode::SingleEnd
    };
    Ok(FilterLowComplexityReportV1 {
        schema_version: crate::FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION.to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id,
        paired_mode,
        threads: 1,
        input_r1: legacy.input_fastq,
        input_r2: None,
        output_r1: legacy.output_fastq,
        output_r2: legacy.output_fastq_r2,
        report_json: "low_complexity_report.json".to_string(),
        entropy_threshold: None,
        polyx_threshold: None,
        reads_in: legacy.reads_in,
        reads_out: legacy.reads_out,
        reads_removed_low_complexity: legacy.reads_removed_low_complexity,
        bases_in: 0,
        bases_out: 0,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 0.0,
        mean_q_after: 0.0,
        runtime_s: legacy.runtime_s,
        memory_mb: legacy.memory_mb,
        exit_code: legacy.exit_code,
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyExtractUmisReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    input_r1: String,
    input_r2: String,
    output_r1: String,
    output_r2: String,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    #[serde(default)]
    pairs_in: Option<u64>,
    #[serde(default)]
    pairs_out: Option<u64>,
    reads_with_umi: u64,
    runtime_s: f64,
    memory_mb: f64,
    exit_code: i32,
}

fn parse_legacy_extract_umis_report(report_json: &str) -> Result<ExtractUmisReportV1> {
    let legacy: LegacyExtractUmisReportV1 =
        serde_json::from_str(report_json).context("parse legacy extract umis report")?;
    if legacy.schema_version != "bijux.fastq.extract_umis.report.v1" {
        return Err(anyhow!(
            "unsupported extract-umis report schema {}",
            legacy.schema_version
        ));
    }
    Ok(ExtractUmisReportV1 {
        schema_version: crate::EXTRACT_UMIS_REPORT_SCHEMA_VERSION.to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id,
        paired_mode: crate::PairedMode::PairedEnd,
        threads: 1,
        umi_pattern: String::new(),
        input_r1: legacy.input_r1,
        input_r2: Some(legacy.input_r2),
        output_r1: legacy.output_r1,
        output_r2: Some(legacy.output_r2),
        report_json: "umi_report.json".to_string(),
        reads_in: legacy.reads_in,
        reads_out: legacy.reads_out,
        bases_in: legacy.bases_in,
        bases_out: legacy.bases_out,
        pairs_in: legacy.pairs_in,
        pairs_out: legacy.pairs_out,
        reads_with_umi: legacy.reads_with_umi,
        mean_q_before: 0.0,
        mean_q_after: 0.0,
        runtime_s: Some(legacy.runtime_s),
        memory_mb: Some(legacy.memory_mb),
        exit_code: Some(legacy.exit_code),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

/// # Errors
/// Returns an error if the governed extract-umis report JSON cannot be parsed.
pub fn parse_extract_umis_report(report_json: &str) -> Result<ExtractUmisReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_extract_umis_report(report_json))
        .context("parse extract umis report")
}

/// # Errors
/// Returns an error if the governed filter-low-complexity report JSON cannot be parsed.
pub fn parse_filter_low_complexity_report(
    report_json: &str,
) -> Result<FilterLowComplexityReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_filter_low_complexity_report(report_json))
        .context("parse filter low complexity report")
}

/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_low_complexity_report(report_json: &str) -> Result<u64> {
    if let Ok(report) = parse_filter_low_complexity_report(report_json) {
        return Ok(report.reads_removed_low_complexity);
    }
    parse_report_u64_field(report_json, "reads_removed_low_complexity")
        .or_else(|| parse_bbduk_reads_removed(report_json).ok())
        .ok_or_else(|| anyhow!("low-complexity report missing reads_removed_low_complexity"))
}

/// # Errors
/// Returns an error if `BBDuk` stats cannot be reduced to a reads-removed count.
pub fn parse_bbduk_reads_removed(stats_txt: &str) -> Result<u64> {
    for line in stats_txt.lines() {
        let line = line.trim();
        if line.starts_with("Reads Removed") || line.starts_with("Reads removed") {
            let digits: String = line.chars().filter(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                return digits.parse::<u64>().context("parse bbduk reads removed");
            }
        }
        if line.starts_with("#Matched") || line.starts_with("Matched") {
            if let Some(field) = line.split_whitespace().nth(1) {
                let digits: String = field.chars().filter(char::is_ascii_digit).collect();
                if !digits.is_empty() {
                    return digits.parse::<u64>().context("parse bbduk matched reads");
                }
            }
        }
    }
    Err(anyhow!("bbduk stats missing reads removed line"))
}

/// # Errors
/// Returns an error if fastp JSON cannot be parsed.
#[allow(dead_code)]
pub fn parse_fastp_metrics(report_json: &str) -> Result<FastpToolMetricsV1> {
    let parsed: serde_json::Value =
        serde_json::from_str(report_json).context("parse fastp json")?;
    let filtering = parsed
        .get("filtering_result")
        .ok_or_else(|| anyhow!("fastp filtering_result missing"))?;
    Ok(FastpToolMetricsV1 {
        schema_version: "bijux.fastp.metrics.v1".to_string(),
        passed_filter_reads: filtering
            .get("passed_filter_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        low_quality_reads: filtering
            .get("low_quality_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        too_many_n_reads: filtering
            .get("too_many_N_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        too_short_reads: filtering
            .get("too_short_reads")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
    })
}

/// Parse `AdapterRemoval` output into canonical merge metrics.
#[allow(dead_code)]
pub fn parse_adapterremoval_metrics(stdout: &str) -> AdapterRemovalToolMetricsV1 {
    let pairs_processed = parse_prefix_u64(stdout, "Total number of read pairs");
    let pairs_merged = parse_prefix_u64(stdout, "Number of fully overlapping pairs");
    let merge_rate = if pairs_processed > 0 {
        u64_to_f64(pairs_merged) / u64_to_f64(pairs_processed)
    } else {
        0.0
    };
    AdapterRemovalToolMetricsV1 {
        schema_version: "bijux.adapterremoval.metrics.v1".to_string(),
        pairs_processed,
        pairs_merged,
        merge_rate,
    }
}

/// # Errors
/// Returns an error if seqkit output cannot be parsed.
#[allow(dead_code)]
pub fn parse_seqkit_tool_metrics(output: &str) -> Result<SeqkitToolMetricsV1> {
    let parsed = parse_seqkit_stats(output)?;
    Ok(SeqkitToolMetricsV1 {
        schema_version: "bijux.seqkit.metrics.v1".to_string(),
        reads: parsed.reads,
        bases: parsed.bases,
        mean_q: Some(parsed.mean_q),
        gc_percent: Some(parsed.gc_percent),
    })
}

/// Parse `samtools flagstat` output into canonical alignment metrics.
#[allow(dead_code)]
pub fn parse_samtools_flagstat_metrics(stdout: &str) -> SamtoolsFlagstatMetricsV1 {
    let total_reads = parse_prefix_u64(stdout, "in total");
    let mapped_reads = parse_prefix_u64(stdout, "mapped (");
    let mapped_rate = if total_reads > 0 {
        u64_to_f64(mapped_reads) / u64_to_f64(total_reads)
    } else {
        0.0
    };
    SamtoolsFlagstatMetricsV1 {
        schema_version: "bijux.samtools.flagstat.v1".to_string(),
        total_reads,
        mapped_reads,
        mapped_rate,
    }
}

/// Parse `FastQC` summary text into canonical summary metrics.
#[allow(dead_code)]
pub fn parse_fastqc_summary_metrics(summary_txt: &str) -> FastqcToolMetricsV1 {
    let mut total_sequences = 0;
    let mut gc_percent = 0.0;
    for line in summary_txt.lines() {
        if let Some((key, value)) = line.split_once('\t') {
            if key.trim() == "Total Sequences" {
                total_sequences = value.trim().parse::<u64>().unwrap_or(0);
            } else if key.trim() == "%GC" {
                gc_percent = value.trim().parse::<f64>().unwrap_or(0.0);
            }
        }
    }
    FastqcToolMetricsV1 {
        schema_version: "bijux.fastqc.metrics.v1".to_string(),
        total_sequences,
        gc_percent,
    }
}

/// # Errors
/// Returns an error if multiqc general stats JSON cannot be parsed.
#[allow(dead_code)]
pub fn parse_multiqc_general_stats_metrics(raw_json: &str) -> Result<MultiqcToolMetricsV1> {
    let parsed: serde_json::Value =
        serde_json::from_str(raw_json).context("parse multiqc general stats json")?;
    let sample_count = parsed.as_object().map_or(0, serde_json::Map::len) as u64;
    let module_count = parsed
        .as_object()
        .and_then(|obj| obj.values().next())
        .and_then(serde_json::Value::as_object)
        .map_or(0, serde_json::Map::len) as u64;
    Ok(MultiqcToolMetricsV1 {
        schema_version: "bijux.multiqc.metrics.v1".to_string(),
        sample_count,
        module_count,
    })
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(f64::MAX)
}

fn parse_report_u64_field(raw: &str, field: &str) -> Option<u64> {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|value| {
            value
                .get(field)
                .and_then(serde_json::Value::as_u64)
                .or_else(|| {
                    value
                        .as_object()
                        .and_then(|obj| obj.get(field))
                        .and_then(serde_json::Value::as_str)
                        .and_then(|s| s.parse::<u64>().ok())
                })
        })
        .or_else(|| parse_kv_u64_field(raw, field))
}

fn parse_kv_u64_field(raw: &str, field: &str) -> Option<u64> {
    raw.lines()
        .filter_map(|line| line.split_once('='))
        .find_map(|(k, v)| {
            if k.trim() == field {
                v.trim().parse::<u64>().ok()
            } else {
                None
            }
        })
}

#[allow(dead_code)]
fn parse_prefix_u64(raw: &str, marker: &str) -> u64 {
    raw.lines()
        .find_map(|line| {
            if line.contains(marker) || line.starts_with(marker) {
                if let Some(value) = line
                    .split_whitespace()
                    .next()
                    .and_then(|value| value.parse::<u64>().ok())
                {
                    return Some(value);
                }
                return line
                    .split_once(':')
                    .and_then(|(_, value)| value.split_whitespace().next())
                    .and_then(|value| value.parse::<u64>().ok());
            }
            None
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        parse_adapterremoval_metrics, parse_bbduk_reads_removed, parse_correct_errors_report,
        parse_deduplicate_report, parse_deplete_host_report,
        parse_deplete_reference_contaminants_report, parse_deplete_rrna_report,
        parse_detect_adapters_report, parse_duplicate_classes_tsv, parse_extract_umis_report,
        parse_fastp_metrics, parse_fastqc_summary_metrics, parse_fastqvalidator_count,
        parse_filter_low_complexity_report, parse_filter_reads_report,
        parse_index_reference_report, parse_infer_asvs_report, parse_length_histogram,
        parse_low_complexity_report, parse_multiqc_general_stats_metrics,
        parse_profile_overrepresented_report, parse_profile_read_lengths_report,
        parse_profile_reads_report, parse_remove_chimeras_report,
        parse_remove_duplicates_provenance, parse_remove_duplicates_report, parse_report_qc_report,
        parse_samtools_flagstat_metrics, parse_screen_summary_tsv, parse_screen_taxonomy_report,
        parse_seqkit_stats, parse_seqkit_tool_metrics, parse_terminal_damage_report,
        parse_trim_reads_report, parse_validated_reads_manifest, parse_validation_report,
    };
    use crate::params::trim::TerminalDamageExecutionPolicy;
    use crate::params::DamageMode;
    use crate::{
        PairedMode, ValidateFailureClass, DEPLETE_RRNA_REPORT_SCHEMA_VERSION,
        REPORT_QC_REPORT_SCHEMA_VERSION, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
        TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION, VALIDATED_READS_MANIFEST_SCHEMA_VERSION,
        VALIDATION_REPORT_SCHEMA_VERSION,
    };
    use anyhow::Result;
    use bijux_dna_core::id_catalog;

    fn assert_f64_eq(left: f64, right: f64) {
        assert!((left - right).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_fastqvalidator_count_parses_fixture() -> Result<()> {
        let stdout =
            include_str!("../../../bijux-dna-stages-fastq/tests/fixtures/fastqvalidator/default/fastqvalidator_v1.txt");
        let count = parse_fastqvalidator_count(stdout)?;
        assert_eq!(count, 12345);
        Ok(())
    }

    #[test]
    fn parse_fastqvalidator_count_rejects_missing_marker() {
        let stdout = "fastqvalidator output without total reads";
        assert!(parse_fastqvalidator_count(stdout).is_err());
    }

    #[test]
    fn parse_validation_report_parses_governed_validate_json() -> Result<()> {
        let parsed = parse_validation_report(
            &serde_json::json!({
                "schema_version": VALIDATION_REPORT_SCHEMA_VERSION,
                "stage": "fastq.validate_reads",
                "stage_id": "fastq.validate_reads",
                "tool_id": "fastqvalidator",
                "validation_mode": "strict",
                "pair_sync_policy": "require_header_sync",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "validation_log_r1": "validation_r1.log",
                "validation_log_r2": "validation_r2.log",
                "validated_inputs": 2,
                "validated_reads_r1": 101,
                "validated_reads_r2": 100,
                "validated_pairs": 100,
                "status_r1": 0,
                "status_r2": 0,
                "pair_sync_checked": true,
                "pair_sync_pass": false,
                "pair_count_match": false,
                "failure_class": "pair_count_mismatch",
                "strict_pass": false,
                "exit_code": 96
            })
            .to_string(),
        )?;
        assert_eq!(
            parsed.failure_class,
            ValidateFailureClass::PairCountMismatch
        );
        assert_eq!(parsed.validated_reads_r2, Some(100));
        Ok(())
    }

    #[test]
    fn parse_validated_reads_manifest_parses_governed_lineage_json() -> Result<()> {
        let parsed = parse_validated_reads_manifest(
            &serde_json::json!({
                "schema_version": VALIDATED_READS_MANIFEST_SCHEMA_VERSION,
                "stage_id": "fastq.validate_reads",
                "tool_id": "seqtk",
                "validation_mode": "report_only",
                "pair_sync_policy": "skip_header_sync",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "validation_report": "validation.json",
                "paired_mode": "paired_end",
                "validated_stream_ids": ["reads_r1", "reads_r2"],
                "pair_sync_checked": false,
                "pair_sync_pass": null,
                "validated_pairs": 120
            })
            .to_string(),
        )?;
        assert_eq!(parsed.paired_mode, PairedMode::PairedEnd);
        assert_eq!(parsed.validated_stream_ids, vec!["reads_r1", "reads_r2"]);
        Ok(())
    }

    #[test]
    fn parse_terminal_damage_report_parses_governed_json() -> Result<()> {
        let parsed = parse_terminal_damage_report(
            &serde_json::json!({
                "schema_version": TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
                "stage": "fastq.trim_terminal_damage",
                "stage_id": "fastq.trim_terminal_damage",
                "tool_id": id_catalog::TOOL_CUTADAPT,
                "paired_mode": "paired_end",
                "threads": 4,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 1,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 1,
                "udg_classification": "non_udg",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "trimmed_R1.fastq.gz",
                "output_r2": "trimmed_R2.fastq.gz",
                "reads_in": 200,
                "reads_out": 198,
                "bases_in": 20000,
                "bases_out": 19100,
                "mean_q_before": 28.0,
                "mean_q_after": 28.5,
                "ct_ga_asymmetry_pre": 0.45,
                "ct_ga_asymmetry_post": 0.12,
                "ct_ga_asymmetry_pre_r1": 0.50,
                "ct_ga_asymmetry_post_r1": 0.15,
                "ct_ga_asymmetry_pre_r2": 0.40,
                "ct_ga_asymmetry_post_r2": 0.09,
                "terminal_base_composition_pre_r1": {"C": 80},
                "terminal_base_composition_post_r1": {"C": 30},
                "terminal_base_composition_pre_r2": {"G": 75},
                "terminal_base_composition_post_r2": {"G": 28},
                "raw_backend_report": "cutadapt.damage.json",
                "raw_backend_report_format": "cutadapt_json",
                "runtime_s": 12.4,
                "memory_mb": 256.0,
                "used_fallback": false,
                "backend_metrics": {"reads_profiled_r1": 200}
            })
            .to_string(),
        )?;
        assert_eq!(parsed.damage_mode, DamageMode::Ancient);
        assert_eq!(parsed.threads, 4);
        assert_eq!(
            parsed.execution_policy,
            TerminalDamageExecutionPolicy::ExplicitTerminalTrim
        );
        assert_eq!(
            parsed.raw_backend_report_format.as_deref(),
            Some("cutadapt_json")
        );
        assert_eq!(parsed.reads_in, Some(200));
        assert!(!parsed.used_fallback);
        Ok(())
    }

    #[test]
    fn parse_trim_reads_report_parses_governed_json() -> Result<()> {
        let parsed = parse_trim_reads_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.trim_reads.report.v2",
                "stage": "fastq.trim_reads",
                "stage_id": "fastq.trim_reads",
                "tool_id": id_catalog::TOOL_FASTP,
                "paired_mode": "paired_end",
                "threads": 4,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "trimmed_R1.fastq.gz",
                "output_r2": "trimmed_R2.fastq.gz",
                "min_length": 30,
                "quality_cutoff": 20,
                "adapter_policy": "bank",
                "polyx_policy": "trim",
                "n_policy": "drop",
                "contaminant_policy": "none",
                "adapter_bank_id": "illumina",
                "adapter_bank_hash": "sha256:adapter",
                "adapter_preset": "default",
                "adapter_overrides": {
                    "enable": ["AGATCGGAAGAGC"],
                    "disable": ["polyA"]
                },
                "polyx_bank_id": "polyx",
                "polyx_bank_hash": "sha256:polyx",
                "polyx_preset": "illumina_twocolor",
                "contaminant_bank_id": "contaminants",
                "contaminant_bank_hash": "sha256:contaminants",
                "contaminant_preset": "illumina_default",
                "reads_in": 100,
                "reads_out": 90,
                "bases_in": 1000,
                "bases_out": 820,
                "pairs_in": 50,
                "pairs_out": 45,
                "mean_q_before": 28.0,
                "mean_q_after": 31.0,
                "runtime_s": 8.4,
                "memory_mb": 128.0,
                "raw_backend_report": "trim.fastp.json",
                "raw_backend_report_format": "fastp_json"
            })
            .to_string(),
        )?;

        assert_eq!(parsed.tool_id, id_catalog::TOOL_FASTP);
        assert_eq!(parsed.threads, 4);
        assert_eq!(parsed.paired_mode, PairedMode::PairedEnd);
        assert_eq!(parsed.adapter_policy, "bank");
        assert_eq!(
            parsed.adapter_overrides,
            Some(serde_json::json!({
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"]
            }))
        );
        assert_eq!(
            parsed.raw_backend_report_format.as_deref(),
            Some("fastp_json")
        );
        Ok(())
    }

    #[test]
    fn parse_screen_taxonomy_report_parses_governed_json() -> Result<()> {
        let parsed = parse_screen_taxonomy_report(
            &serde_json::json!({
                "schema_version": SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
                "stage": "fastq.screen_taxonomy",
                "stage_id": "fastq.screen_taxonomy",
                "tool_id": id_catalog::TOOL_KRAKEN2,
                "paired_mode": "paired_end",
                "threads": 8,
                "classifier": id_catalog::TOOL_KRAKEN2,
                "report_format": "kraken_report",
                "assignment_format": "kraken_assignments",
                "database_catalog_id": "taxonomy_reference",
                "database_artifact_id": "taxonomy_db",
                "database_build_id": "build-2026-03",
                "database_digest": "sha256:taxonomy",
                "database_namespace": "read_screening",
                "database_scope": "read_screening",
                "minimum_confidence": 0.1,
                "emit_unclassified": true,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "screen_report_tsv": "kraken2.report.tsv",
                "classification_report_json": "kraken2.classifications.json",
                "reads_in": 200,
                "reads_out": 200,
                "bases_in": 20000,
                "bases_out": 20000,
                "pairs_in": 100,
                "pairs_out": 100,
                "contamination_rate": 0.23,
                "classified_fraction": 0.77,
                "unclassified_fraction": 0.23,
                "summary_entries": [
                    {"label": "unclassified", "percent": 23.0},
                    {"label": "bacteria", "percent": 77.0}
                ],
                "top_taxa": [
                    {"label": "bacteria", "percent": 77.0}
                ],
                "runtime_s": 12.5,
                "memory_mb": 512.0
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, id_catalog::TOOL_KRAKEN2);
        assert_eq!(parsed.threads, 8);
        assert_eq!(parsed.top_taxa.len(), 1);
        assert_eq!(parsed.top_taxa[0].label, "bacteria");
        Ok(())
    }

    #[test]
    fn parse_detect_adapters_report_round_trips_governed_payload() -> Result<()> {
        let parsed = parse_detect_adapters_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.detect_adapters.report.v2",
                "stage": "fastq.detect_adapters",
                "stage_id": "fastq.detect_adapters",
                "tool_id": "fastqc",
                "paired_mode": "paired_end",
                "threads": 4,
                "inspection_mode": "evidence_only",
                "report_only": true,
                "evidence_engine": "fastqc",
                "evidence_scope": "full_input",
                "evidence_format": "fastqc_summary",
                "evidence_artifact_id": "report_json",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "report_json": "adapter_report.json",
                "adapter_evidence_dir": "fastqc",
                "reads_in": 200_u64,
                "reads_out": 200_u64,
                "bases_in": 20_000_u64,
                "bases_out": 20_000_u64,
                "pairs_in": 100_u64,
                "pairs_out": 100_u64,
                "mean_q": 31.2,
                "candidate_adapter_count": 2_u64,
                "adapter_trimmed_fraction": 0.08,
                "adapter_content_max": 12.5,
                "adapter_content_mean": 3.2,
                "duplication_rate": 0.15,
                "n_rate": 0.001,
                "kmer_warning_count": 4_u64,
                "overrepresented_sequence_count": 3_u64,
                "runtime_s": 4.0,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "fastqc/fastqc_data.txt",
                "raw_backend_report_format": "fastqc_data_txt"
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "fastqc");
        assert_eq!(parsed.candidate_adapter_count, 2);
        assert_eq!(parsed.threads, 4);
        Ok(())
    }

    #[test]
    fn parse_deplete_rrna_report_round_trips_governed_payload() -> Result<()> {
        let parsed = parse_deplete_rrna_report(
            &serde_json::json!({
                "schema_version": DEPLETE_RRNA_REPORT_SCHEMA_VERSION,
                "stage": "fastq.deplete_rrna",
                "stage_id": "fastq.deplete_rrna",
                "tool_id": "sortmerna",
                "paired_mode": "paired_end",
                "threads": 6,
                "rrna_db": "/refs/silva",
                "database_artifact_id": "silva_nr99",
                "database_build_id": "2026.03",
                "screening_engine": "sortmerna",
                "report_format": "summary_tsv_and_json",
                "emit_removed_reads": false,
                "min_identity": 0.95,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "rrna_filtered_R1.fastq.gz",
                "output_r2": "rrna_filtered_R2.fastq.gz",
                "rrna_report_tsv": "rrna_report.tsv",
                "rrna_report_json": "rrna_report.json",
                "reads_in": 200,
                "reads_out": 150,
                "reads_removed": 50,
                "bases_in": 20000,
                "bases_out": 15000,
                "bases_removed": 5000,
                "pairs_in": 100,
                "pairs_out": 75,
                "rrna_fraction_removed": 0.25,
                "runtime_s": 12.3,
                "memory_mb": 256.0,
                "exit_code": 0,
                "raw_backend_report": "sortmerna.log",
                "raw_backend_report_format": "sortmerna_log",
                "backend_metrics": {
                    "reads_removed": 50
                }
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "sortmerna");
        assert_eq!(parsed.database_artifact_id, "silva_nr99");
        assert_eq!(parsed.reads_removed, 50);
        Ok(())
    }

    #[test]
    fn parse_deplete_rrna_report_accepts_legacy_payload() -> Result<()> {
        let parsed = parse_deplete_rrna_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.deplete_rrna.report.v1",
                "stage_id": "fastq.deplete_rrna",
                "tool_id": "sortmerna",
                "rrna_fraction_removed": 0.4,
                "reads_in": 100,
                "reads_out": 60,
                "bases_in": 1000,
                "bases_out": 600,
                "runtime_s": 4.0,
                "memory_mb": 32.0
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "sortmerna");
        assert_eq!(parsed.reads_removed, 40);
        assert_f64_eq(parsed.rrna_fraction_removed, 0.4);
        Ok(())
    }

    #[test]
    fn parse_deplete_reference_contaminants_report_round_trips_governed_payload() -> Result<()> {
        let parsed = parse_deplete_reference_contaminants_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.deplete_reference_contaminants.report.v2",
                "stage": "fastq.deplete_reference_contaminants",
                "stage_id": "fastq.deplete_reference_contaminants",
                "tool_id": "bowtie2",
                "paired_mode": "paired_end",
                "threads": 6,
                "reference_catalog_id": "contaminant_reference",
                "contaminant_reference": "phix_and_spikeins",
                "index_artifact": "reference_index",
                "reference_index_backend": "bowtie2_build",
                "reference_build_id": "2026.03",
                "reference_digest": "sha256:contaminant",
                "retain_unmapped_pairs": true,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "contaminant_screened_R1.fastq.gz",
                "output_r2": "contaminant_screened_R2.fastq.gz",
                "report_json": "contaminant_screen_report.json",
                "reads_in": 200,
                "reads_out": 160,
                "reads_removed": 40,
                "bases_in": 20000,
                "bases_out": 15600,
                "bases_removed": 4400,
                "pairs_in": 100,
                "pairs_out": 80,
                "contaminant_fraction_removed": 0.2,
                "runtime_s": 9.8,
                "memory_mb": 512.0,
                "exit_code": 0,
                "raw_backend_report": "bowtie2.contaminant.metrics.txt",
                "raw_backend_report_format": "bowtie2_met_file",
                "backend_metrics": {
                    "reads_removed": 40
                }
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "bowtie2");
        assert_eq!(parsed.reads_removed, 40);
        assert_eq!(
            parsed.raw_backend_report_format.as_deref(),
            Some("bowtie2_met_file")
        );
        Ok(())
    }

    #[test]
    fn parse_deplete_reference_contaminants_report_accepts_legacy_payload() -> Result<()> {
        let parsed = parse_deplete_reference_contaminants_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.deplete_reference_contaminants.report.v1",
                "stage_id": "fastq.deplete_reference_contaminants",
                "tool_id": "bowtie2",
                "contaminant_fraction_removed": 0.35,
                "reads_in": 100,
                "reads_out": 65,
                "bases_in": 1000,
                "bases_out": 650,
                "runtime_s": 4.0,
                "memory_mb": 32.0
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "bowtie2");
        assert_eq!(parsed.reads_removed, 35);
        assert_f64_eq(parsed.contaminant_fraction_removed, 0.35);
        Ok(())
    }

    #[test]
    fn parse_deplete_host_report_round_trips_governed_payload() -> Result<()> {
        let parsed = parse_deplete_host_report(
            r#"{
                "schema_version": "bijux.fastq.deplete_host.report.v2",
                "stage": "fastq.deplete_host",
                "stage_id": "fastq.deplete_host",
                "tool_id": "bowtie2",
                "paired_mode": "paired_end",
                "threads": 6,
                "reference_scope": "host",
                "reference_catalog_id": "host_reference",
                "reference_index_artifact_id": "reference_index",
                "reference_index_backend": "bowtie2_build",
                "reference_build_id": "2026.03",
                "reference_digest": "sha256:host",
                "masking_policy": "unmasked",
                "decoy_policy": "none",
                "decoy_catalog_id": null,
                "identity_threshold": 0.95,
                "retained_read_policy": "keep_non_host_reads",
                "emit_removed_reads": true,
                "report_format": "bowtie2_metrics_file",
                "retain_unmapped_pairs": true,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "host_depleted_R1.fastq.gz",
                "output_r2": "host_depleted_R2.fastq.gz",
                "removed_host_r1": "removed_host_R1.fastq.gz",
                "removed_host_r2": "removed_host_R2.fastq.gz",
                "report_json": "host_depletion_report.json",
                "reads_in": 200,
                "reads_out": 150,
                "reads_removed": 50,
                "bases_in": 20000,
                "bases_out": 15000,
                "bases_removed": 5000,
                "pairs_in": 100,
                "pairs_out": 75,
                "host_fraction_removed": 0.25,
                "runtime_s": 10.5,
                "memory_mb": 512.0,
                "exit_code": 0,
                "raw_backend_report": "bowtie2.host.metrics.txt",
                "raw_backend_report_format": "bowtie2_met_file",
                "backend_metrics": {
                    "reads_removed": 50
                }
            }"#,
        )?;
        assert_eq!(parsed.tool_id, "bowtie2");
        assert_eq!(parsed.reads_removed, 50);
        assert_eq!(
            parsed.raw_backend_report_format.as_deref(),
            Some("bowtie2_met_file")
        );
        Ok(())
    }

    #[test]
    fn parse_deplete_host_report_accepts_legacy_payload() -> Result<()> {
        let parsed = parse_deplete_host_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.deplete_host.report.v1",
                "stage_id": "fastq.deplete_host",
                "tool_id": "bowtie2",
                "host_fraction_removed": 0.4,
                "reads_in": 100,
                "reads_out": 60,
                "bases_in": 1000,
                "bases_out": 600,
                "runtime_s": 4.0,
                "memory_mb": 32.0
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "bowtie2");
        assert_eq!(parsed.reads_removed, 40);
        assert_f64_eq(parsed.host_fraction_removed, 0.4);
        Ok(())
    }

    #[test]
    fn parse_screen_summary_tsv_extracts_label_percent_pairs() -> Result<()> {
        let parsed = parse_screen_summary_tsv(
            "# taxonomic summary\nunclassified\t123\t23.0%\nbacteria\t410\t77.0%\n",
        )?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].label, "unclassified");
        assert_f64_eq(parsed[1].percent, 77.0);
        Ok(())
    }

    #[test]
    fn parse_report_qc_report_parses_governed_json() -> Result<()> {
        let parsed = parse_report_qc_report(
            &serde_json::json!({
                "schema_version": REPORT_QC_REPORT_SCHEMA_VERSION,
                "stage": "fastq.report_qc",
                "stage_id": "fastq.report_qc",
                "tool_id": "multiqc",
                "paired_mode": "paired_end",
                "aggregation_engine": "multiqc",
                "aggregation_scope": "governed_qc_artifacts",
                "reads_in": 200,
                "reads_out": 200,
                "bases_in": 20000,
                "bases_out": 20000,
                "pairs_in": 100,
                "pairs_out": 100,
                "mean_q": 31.0,
                "contamination_rate": 0.04,
                "adapter_content_max": 0.1,
                "adapter_content_mean": 0.03,
                "duplication_rate": 0.08,
                "n_rate": 0.001,
                "kmer_warning_count": 2,
                "overrepresented_sequence_count": 1,
                "multiqc_sample_count": 2,
                "multiqc_module_count": 5,
                "raw_fastqc_dir": "raw_fastqc",
                "trimmed_fastqc_dir": "trimmed_fastqc",
                "multiqc_report": "multiqc_report.html",
                "multiqc_data": "multiqc_data",
                "governed_qc_input_count": 3,
                "governed_qc_contributor_stage_ids": ["fastq.trim_reads"],
                "governed_qc_contributor_tool_ids": [id_catalog::TOOL_FASTP],
                "governed_qc_contributors": [{
                    "contributor_id": "fastq.trim_reads.fastp",
                    "stage_id": "fastq.trim_reads",
                    "tool_id": id_catalog::TOOL_FASTP,
                    "artifact_id": "report_json",
                    "artifact_role": "report_json",
                    "path": "trim/report.json"
                }],
                "governed_qc_lineage_hash": "lineage",
                "governed_qc_inputs_manifest": "governed_qc_inputs_manifest.json",
                "runtime_s": 3.0,
                "memory_mb": 128.0,
                "exit_code": 0
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "multiqc");
        assert_eq!(parsed.governed_qc_input_count, 3);
        assert_eq!(parsed.multiqc_module_count, Some(5));
        Ok(())
    }

    #[test]
    fn parse_seqkit_stats_parses_fixture() -> Result<()> {
        let stdout = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/seqkit/default/seqkit_stats_v1.txt"
        );
        let metrics = parse_seqkit_stats(stdout)?;
        assert_eq!(metrics.reads, 1000);
        assert_eq!(metrics.bases, 100_000);
        Ok(())
    }

    #[test]
    fn parse_length_histogram_parses_fixture() -> Result<()> {
        let stdout = "readA\t100\nreadB\t100\nreadC\t50\n";
        let metrics = parse_length_histogram(stdout)?;
        assert_eq!(metrics.len(), 2);
        Ok(())
    }

    #[test]
    fn parse_length_histogram_uses_fx2tab_length_column() -> Result<()> {
        let stdout = "readA\tACGT\t####\t4\nreadB\tAC\t##\t2\nreadC\tTT\t!!\t2\n";
        let metrics = parse_length_histogram(stdout)?;
        assert_eq!(metrics, vec![(2, 2), (4, 1)]);
        Ok(())
    }

    #[test]
    fn parse_deduplicate_report_parses_fixture() -> Result<()> {
        let raw =
            include_str!("../../../bijux-dna-stages-fastq/tests/fixtures/deduplicate/default/deduplicate_report_v1.json");
        let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
        assert_eq!(reads_in, 1000);
        assert_eq!(reads_out, 820);
        Ok(())
    }

    #[test]
    fn parse_remove_duplicates_report_parses_governed_json() -> Result<()> {
        let parsed = parse_remove_duplicates_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "single_end",
                "threads": 4,
                "dedup_mode": "optical_aware",
                "keep_order": false,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "dedup.fastq.gz",
                "output_r2": null,
                "reads_in": 100,
                "reads_out": 85,
                "reads_in_r2": null,
                "reads_out_r2": null,
                "pairs_in": null,
                "pairs_out": null,
                "pair_count_match": null,
                "duplicates_removed": 15,
                "dedup_rate": 0.15,
                "duplicate_classes_tsv": "duplicate_classes.tsv",
                "duplicate_provenance_json": "duplicate_provenance.json",
                "duplicate_classes": [
                    {"class": "duplicate", "reads_removed": 11, "paired_mode": "single_end"},
                    {"class": "optical_duplicate", "reads_removed": 4, "paired_mode": "single_end"}
                ],
                "raw_backend_report": "clumpify.log",
                "raw_backend_report_format": "clumpify_log",
                "runtime_s": 2.2,
                "memory_mb": 64.0
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "clumpify");
        assert_eq!(parsed.threads, 4);
        assert_eq!(parsed.duplicate_classes.len(), 2);
        assert_f64_eq(parsed.dedup_rate, 0.15);
        Ok(())
    }

    #[test]
    fn parse_remove_duplicates_report_rejects_incomplete_governed_json() {
        let result = parse_remove_duplicates_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.report.v2",
                "stage": "fastq.remove_duplicates",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "clumpify",
                "paired_mode": "single_end",
                "dedup_mode": "optical_aware",
                "keep_order": false,
                "reads_in": 100,
                "reads_out": 85,
                "duplicates_removed": 15,
                "dedup_rate": 0.15
            })
            .to_string(),
        );
        let Err(error) = result else {
            panic!("incomplete governed remove-duplicates reports must not fall back to legacy parsing");
        };

        assert!(error.to_string().contains("parse remove duplicates report"));
    }

    #[test]
    fn parse_remove_duplicates_provenance_parses_governed_json() -> Result<()> {
        let parsed = parse_remove_duplicates_provenance(
            &serde_json::json!({
                "schema_version": "bijux.fastq.remove_duplicates.provenance.v2",
                "stage_id": "fastq.remove_duplicates",
                "tool_id": "fastuniq",
                "paired_mode": "paired_end",
                "threads": 4,
                "dedup_mode": "exact",
                "keep_order": true,
                "duplicates_removed": 18,
                "dedup_rate": 0.09,
                "backend_log": "fastuniq.log",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "dedup_R1.fastq.gz",
                "output_r2": "dedup_R2.fastq.gz",
                "raw_backend_report": "fastuniq.log",
                "raw_backend_report_format": "fastuniq_log"
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "fastuniq");
        assert_eq!(parsed.duplicates_removed, 18);
        Ok(())
    }

    #[test]
    fn parse_low_complexity_report_parses_fixture() -> Result<()> {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/low_complexity/default/low_complexity_report_v1.json"
        );
        let removed = parse_low_complexity_report(raw)?;
        assert_eq!(removed, 137);
        Ok(())
    }

    #[test]
    fn parse_deduplicate_report_parses_key_value_fixture() -> Result<()> {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/stage_output_bank/default/fastq.remove_duplicates.fastuniq.txt"
        );
        let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
        assert_eq!(reads_in, 1000);
        assert_eq!(reads_out, 820);
        Ok(())
    }

    #[test]
    fn parse_duplicate_classes_tsv_parses_fixture_lines() -> Result<()> {
        let parsed = parse_duplicate_classes_tsv(
            "class\treads_removed\tpaired_mode\nduplicate\t180\tpaired_end\noptical_duplicate\t12\tpaired_end\n",
        )?;
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].class, "duplicate");
        assert_eq!(parsed[1].reads_removed, 12);
        Ok(())
    }

    #[test]
    fn parse_remove_chimeras_report_parses_governed_contract() -> Result<()> {
        let parsed = parse_remove_chimeras_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.remove_chimeras.report.v2",
                "stage": "fastq.remove_chimeras",
                "stage_id": "fastq.remove_chimeras",
                "tool_id": "vsearch",
                "paired_mode": "single_end",
                "threads": 2,
                "method": "vsearch_uchime_denovo",
                "detection_scope": "denovo",
                "chimera_removed_definition": "reads flagged as de_novo chimeras are excluded from downstream abundance tables",
                "input_reads": "merged.fastq.gz",
                "output_reads": "nonchimeras.fastq.gz",
                "chimera_metrics_json": "chimera_metrics.json",
                "chimeras_fasta": "chimeras.fasta",
                "uchime_report_tsv": "uchime.tsv",
                "reads_in": 100,
                "reads_out": 92,
                "chimeras_removed": 8,
                "chimera_fraction": 0.08,
                "used_fallback": false,
                "raw_backend_report": "uchime.tsv",
                "raw_backend_report_format": "vsearch_uchime_tsv",
                "runtime_s": 1.7,
                "memory_mb": 32.0,
                "exit_code": 0,
                "backend_metrics": {
                    "parsed_records": 100,
                    "flagged_records": 8
                }
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "vsearch");
        assert_eq!(parsed.threads, 2);
        assert_eq!(parsed.chimera_fraction, Some(0.08));
        assert_eq!(parsed.uchime_report_tsv.as_deref(), Some("uchime.tsv"));
        Ok(())
    }

    #[test]
    fn parse_remove_chimeras_report_accepts_legacy_metrics_payload() -> Result<()> {
        let parsed = parse_remove_chimeras_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.remove_chimeras.v2",
                "chimera_fraction": 0.12,
                "chimeras_removed": 12,
                "non_chimera_reads": 88,
                "tool": "vsearch",
                "used_fallback": true
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "vsearch");
        assert_eq!(parsed.chimeras_removed, Some(12));
        assert!(parsed.used_fallback);
        Ok(())
    }

    #[test]
    fn parse_profile_reads_report_parses_governed_contract() -> Result<()> {
        let parsed = parse_profile_reads_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.profile_reads.report.v2",
                "stage": "fastq.profile_reads",
                "stage_id": "fastq.profile_reads",
                "tool_id": "seqkit_stats",
                "paired_mode": "paired_end",
                "threads": 2,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "qc_json": "qc.json",
                "qc_tsv": "qc.tsv",
                "qc_plots_dir": "plots",
                "reads_total": 200,
                "bases_total": 20000,
                "mean_q": 31.2,
                "gc_percent": 42.0,
                "length_histogram": [{"length": 100, "count": 200}],
                "mate_summaries": [
                    {"label": "reads_r1", "reads": 100, "bases": 10000, "mean_q": 31.0, "gc_percent": 41.0},
                    {"label": "reads_r2", "reads": 100, "bases": 10000, "mean_q": 31.4, "gc_percent": 43.0}
                ],
                "runtime_s": 1.2,
                "memory_mb": 20.0,
                "exit_code": 0,
                "raw_backend_report": "qc.tsv",
                "raw_backend_report_format": "seqkit_stats_tsv",
                "backend_metrics": [
                    {"schema_version": "bijux.seqkit.metrics.v1", "reads": 100, "bases": 10000, "mean_q": 31.0, "gc_percent": 41.0}
                ]
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "seqkit_stats");
        assert_eq!(parsed.reads_total, 200);
        assert_eq!(parsed.length_histogram.len(), 1);
        Ok(())
    }

    #[test]
    fn parse_profile_reads_report_accepts_legacy_metrics_payload() -> Result<()> {
        let parsed = parse_profile_reads_report(
            &serde_json::json!({
                "reads_total": 100,
                "bases_total": 10000,
                "mean_q": 30.5,
                "gc_percent": 41.5,
                "length_histogram": [{"length": 100, "count": 100}]
            })
            .to_string(),
        )?;
        assert_eq!(parsed.reads_total, 100);
        assert_eq!(parsed.tool_id, "unknown");
        assert_eq!(parsed.length_histogram[0].count, 100);
        Ok(())
    }

    #[test]
    fn parse_profile_read_lengths_report_parses_governed_contract() -> Result<()> {
        let parsed = parse_profile_read_lengths_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.profile_read_lengths.report.v2",
                "stage": "fastq.profile_read_lengths",
                "stage_id": "fastq.profile_read_lengths",
                "tool_id": "seqkit_stats",
                "paired_mode": "paired_end",
                "threads": 2,
                "histogram_bins": 64,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "length_distribution_tsv": "length_distribution.tsv",
                "length_distribution_json": "length_distribution.json",
                "report_json": "profile_read_lengths_report.json",
                "read_count": 200,
                "mean_read_length": 101.5,
                "max_read_length": 150,
                "distinct_lengths": 12,
                "histogram": [{"read_length": 100, "count": 180}],
                "runtime_s": 1.1,
                "memory_mb": 16.0,
                "exit_code": 0,
                "raw_backend_report": "length_distribution.tsv",
                "raw_backend_report_format": "seqkit_fx2tab_tsv"
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "seqkit_stats");
        assert_eq!(parsed.threads, 2);
        assert_eq!(parsed.histogram_bins, 64);
        assert_eq!(parsed.read_count, 200);
        assert_eq!(parsed.histogram.len(), 1);
        Ok(())
    }

    #[test]
    fn parse_profile_read_lengths_report_accepts_legacy_histogram_payload() -> Result<()> {
        let parsed = parse_profile_read_lengths_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.profile_read_lengths.v1",
                "histogram": [
                    {"read_length": 100, "count": 90},
                    {"read_length": 101, "count": 10}
                ]
            })
            .to_string(),
        )?;
        assert_eq!(parsed.read_count, 100);
        assert_eq!(parsed.max_read_length, 101);
        assert_eq!(parsed.distinct_lengths, 2);
        Ok(())
    }

    #[test]
    fn parse_profile_overrepresented_report_parses_governed_contract() -> Result<()> {
        let parsed = parse_profile_overrepresented_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.profile_overrepresented.report.v2",
                "stage": "fastq.profile_overrepresented_sequences",
                "stage_id": "fastq.profile_overrepresented_sequences",
                "tool_id": "fastqc",
                "paired_mode": "paired_end",
                "threads": 4,
                "top_k": 25,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "overrepresented_sequences_tsv": "overrepresented_sequences.tsv",
                "overrepresented_sequences_json": "overrepresented_sequences.json",
                "report_json": "overrepresented_report.json",
                "sequence_count": 25,
                "flagged_sequences": 3,
                "top_fraction": 0.12,
                "rows": [
                    {"sequence": "ACGT", "count": 12, "fraction": 0.12, "flag": "overrepresented"}
                ],
                "runtime_s": 1.4,
                "memory_mb": 48.0,
                "exit_code": 0,
                "raw_backend_report": "fastqc_data.txt",
                "raw_backend_report_format": "fastqc_module_txt"
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "fastqc");
        assert_eq!(parsed.top_k, 25);
        assert_eq!(parsed.rows.len(), 1);
        Ok(())
    }

    #[test]
    fn parse_profile_overrepresented_report_accepts_legacy_payload() -> Result<()> {
        let parsed = parse_profile_overrepresented_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.profile_overrepresented_sequences.v1",
                "tool": "seqkit",
                "sequence_count": 2,
                "flagged_sequences": 1,
                "top_fraction": 0.4,
                "rows": [
                    {"sequence": "AAAA", "count": 40, "fraction": 0.4, "flag": "overrepresented"},
                    {"sequence": "TTTT", "count": 10, "fraction": 0.1, "flag": "background"}
                ]
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "seqkit");
        assert_eq!(parsed.sequence_count, 2);
        assert_eq!(parsed.flagged_sequences, 1);
        assert_f64_eq(parsed.top_fraction, 0.4);
        Ok(())
    }

    #[test]
    fn parse_low_complexity_report_parses_key_value_fixture() -> Result<()> {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/stage_output_bank/default/fastq.filter_low_complexity.bbduk.txt"
        );
        let removed = parse_low_complexity_report(raw)?;
        assert_eq!(removed, 137);
        Ok(())
    }

    #[test]
    fn parse_filter_low_complexity_report_round_trips_governed_payload() -> Result<()> {
        let parsed = parse_filter_low_complexity_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.filter_low_complexity.report.v2",
                "stage": "fastq.filter_low_complexity",
                "stage_id": "fastq.filter_low_complexity",
                "tool_id": "bbduk",
                "paired_mode": "single_end",
                "threads": 8,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "filtered.fastq.gz",
                "output_r2": null,
                "report_json": "low_complexity_report.json",
                "entropy_threshold": 0.5,
                "polyx_threshold": 20,
                "reads_in": 100,
                "reads_out": 92,
                "reads_removed_low_complexity": 8,
                "bases_in": 1000,
                "bases_out": 910,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 28.0,
                "mean_q_after": 29.0,
                "runtime_s": 1.1,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "bbduk.low_complexity.stats",
                "raw_backend_report_format": "bbduk_stats",
                "backend_metrics": {
                    "reads_removed_reported": 8
                }
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "bbduk");
        assert_eq!(parsed.reads_removed_low_complexity, 8);
        assert_eq!(parsed.polyx_threshold, Some(20));
        Ok(())
    }

    #[test]
    fn parse_extract_umis_report_round_trips_governed_payload() -> Result<()> {
        let parsed = parse_extract_umis_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.extract_umis.report.v2",
                "stage": "fastq.extract_umis",
                "stage_id": "fastq.extract_umis",
                "tool_id": "umi_tools",
                "paired_mode": "paired_end",
                "threads": 2,
                "umi_pattern": "NNNNNNNN",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "umi_reads_R1.fastq.gz",
                "output_r2": "umi_reads_R2.fastq.gz",
                "report_json": "umi_report.json",
                "reads_in": 200,
                "reads_out": 200,
                "bases_in": 20000,
                "bases_out": 20000,
                "pairs_in": 100,
                "pairs_out": 100,
                "reads_with_umi": 200,
                "mean_q_before": 30.0,
                "mean_q_after": 30.0,
                "runtime_s": 1.4,
                "memory_mb": 64.0,
                "exit_code": 0,
                "raw_backend_report": "umi_tools.extract.log",
                "raw_backend_report_format": "umi_tools_log",
                "backend_metrics": {
                    "reads_with_umi_fraction": 1.0
                }
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "umi_tools");
        assert_eq!(parsed.umi_pattern, "NNNNNNNN");
        assert_eq!(parsed.reads_with_umi, 200);
        Ok(())
    }

    #[test]
    fn parse_bbduk_reads_removed_fixture_line() -> Result<()> {
        let removed = parse_bbduk_reads_removed("Reads Removed: 137\n")?;
        assert_eq!(removed, 137);
        Ok(())
    }

    #[test]
    fn parse_bbduk_reads_removed_matches_summary_fixture() -> Result<()> {
        let removed = parse_bbduk_reads_removed(
            "#File\treads.fastq.gz\n#Total\t12282618\n#Matched\t0\t0.00000%\n",
        )?;
        assert_eq!(removed, 0);
        Ok(())
    }

    #[test]
    fn parse_fastp_metrics_fixture() -> Result<()> {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/fastp.json"
        );
        let parsed = parse_fastp_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.fastp.metrics.v1");
        assert_eq!(parsed.passed_filter_reads, 960);
        assert_eq!(parsed.too_short_reads, 12);
        Ok(())
    }

    #[test]
    fn parse_adapterremoval_metrics_fixture() {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/adapterremoval.txt"
        );
        let parsed = parse_adapterremoval_metrics(raw);
        assert_eq!(parsed.schema_version, "bijux.adapterremoval.metrics.v1");
        assert_eq!(parsed.pairs_processed, 1000);
        assert_eq!(parsed.pairs_merged, 640);
    }

    #[test]
    fn parse_seqkit_tool_metrics_fixture() -> Result<()> {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/seqkit/default/seqkit_stats_v1.txt"
        );
        let parsed = parse_seqkit_tool_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.seqkit.metrics.v1");
        assert_eq!(parsed.reads, 1000);
        Ok(())
    }

    #[test]
    fn parse_samtools_flagstat_fixture() {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/samtools_flagstat.txt"
        );
        let parsed = parse_samtools_flagstat_metrics(raw);
        assert_eq!(parsed.schema_version, "bijux.samtools.flagstat.v1");
        assert_eq!(parsed.total_reads, 1000);
        assert_eq!(parsed.mapped_reads, 900);
    }

    #[test]
    fn parse_fastqc_summary_fixture() {
        let raw = include_str!(
            "../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/fastqc_summary.txt"
        );
        let parsed = parse_fastqc_summary_metrics(raw);
        assert_eq!(parsed.schema_version, "bijux.fastqc.metrics.v1");
        assert_eq!(parsed.total_sequences, 1000);
        assert_f64_eq(parsed.gc_percent, 42.0);
    }

    #[test]
    fn parse_multiqc_general_stats_fixture() -> Result<()> {
        let raw =
            include_str!("../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/multiqc_general_stats.json");
        let parsed = parse_multiqc_general_stats_metrics(raw)?;
        assert_eq!(parsed.schema_version, "bijux.multiqc.metrics.v1");
        assert_eq!(parsed.sample_count, 2);
        assert_eq!(parsed.module_count, 2);
        Ok(())
    }

    #[test]
    fn parse_correct_errors_report_round_trips_governed_payload() -> Result<()> {
        let parsed = parse_correct_errors_report(
            r#"{
                "schema_version": "bijux.fastq.correct_errors.report.v2",
                "stage": "fastq.correct_errors",
                "stage_id": "fastq.correct_errors",
                "tool_id": "lighter",
                "paired_mode": "single_end",
                "threads": 8,
                "correction_engine": "lighter",
                "quality_encoding": "phred33",
                "kmer_size": 31,
                "musket_kmer_budget": null,
                "genome_size": 2500000,
                "max_memory_gb": null,
                "trusted_kmer_artifact": "trusted_kmers.fa",
                "conservative_mode": false,
                "input_r1": "reads.fastq.gz",
                "input_r2": null,
                "output_r1": "corrected.fastq.gz",
                "output_r2": null,
                "report_json": "correct_report.json",
                "corrected_reads": 100,
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 10000,
                "bases_out": 10000,
                "pairs_in": null,
                "pairs_out": null,
                "mean_q_before": 30.0,
                "mean_q_after": 31.0,
                "kmer_fix_rate": 0.12,
                "correction_effect": {
                    "outputs_changed": true,
                    "reads_delta": 0,
                    "bases_delta": 0,
                    "mean_q_delta": 1.0
                },
                "runtime_s": 1.8,
                "memory_mb": 96.0,
                "exit_code": 0,
                "raw_backend_report": "lighter.log",
                "raw_backend_report_format": "lighter_log",
                "backend_metrics": {
                    "trusted_kmers_loaded": true
                }
            }"#,
        )?;
        assert_eq!(parsed.tool_id, "lighter");
        assert_eq!(parsed.threads, 8);
        assert_eq!(parsed.kmer_size, Some(31));
        assert_eq!(parsed.musket_kmer_budget, None);
        assert_eq!(parsed.corrected_reads, Some(100));
        Ok(())
    }

    #[test]
    fn parse_correct_errors_report_accepts_legacy_payload() -> Result<()> {
        let parsed = parse_correct_errors_report(
            r#"{
                "schema_version": "bijux.fastq.correct_errors.report.v1",
                "stage_id": "fastq.correct_errors",
                "tool_id": "rcorrector",
                "correction_engine": "rcorrector",
                "quality_encoding": "phred33",
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "out/reads_r1.fastq.gz",
                "output_r2": "out/reads_r2.fastq.gz",
                "kmer_size": null,
                "musket_kmer_budget": null,
                "genome_size": null,
                "max_memory_gb": null,
                "trusted_kmer_artifact": null,
                "conservative_mode": false,
                "corrected_reads": 200,
                "reads_in": 200,
                "reads_out": 200,
                "bases_in": 20000,
                "bases_out": 20000,
                "pairs_in": 100,
                "pairs_out": 100,
                "mean_q_before": 28.0,
                "mean_q_after": 29.5,
                "kmer_fix_rate": 0.05,
                "correction_effect": {
                    "outputs_changed": true,
                    "reads_delta": 0,
                    "bases_delta": 0,
                    "mean_q_delta": 1.5
                },
                "runtime_s": 2.1,
                "memory_mb": 128.0,
                "exit_code": 0
            }"#,
        )?;
        assert_eq!(parsed.tool_id, "rcorrector");
        assert_eq!(parsed.stage, "fastq.correct_errors");
        assert_eq!(parsed.report_json, "correct_report.json");
        assert_eq!(parsed.threads, 1);
        assert_eq!(parsed.paired_mode, PairedMode::PairedEnd);
        Ok(())
    }

    #[test]
    fn parse_index_reference_report_parses_governed_contract() -> Result<()> {
        let parsed = parse_index_reference_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.index_reference.report.v2",
                "stage": "fastq.index_reference",
                "stage_id": "fastq.index_reference",
                "tool_id": "bowtie2_build",
                "threads": 4,
                "index_format": "bowtie2_build",
                "reference_fasta": "reference.fa",
                "reference_bytes": 4096,
                "reference_index": "reference_index/bowtie2/reference",
                "report_json": "index_reference_report.json",
                "index_prefix": "reference",
                "emitted_files": [
                    {"relative_path": "reference.1.bt2", "bytes": 1024},
                    {"relative_path": "reference.2.bt2", "bytes": 2048}
                ],
                "index_file_count": 2,
                "index_bytes": 3072,
                "runtime_s": 1.5,
                "memory_mb": 96.0,
                "exit_code": 0,
                "backend_metrics": {
                    "index_directory": "reference_index/bowtie2"
                }
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "bowtie2_build");
        assert_eq!(parsed.index_file_count, 2);
        assert_eq!(parsed.emitted_files[0].relative_path, "reference.1.bt2");
        Ok(())
    }

    #[test]
    fn parse_filter_reads_report_parses_governed_contract() -> Result<()> {
        let parsed = parse_filter_reads_report(&format!(
            r#"{{
                "schema_version": "bijux.fastq.filter_reads.report.v3",
                "stage": "fastq.filter_reads",
                "stage_id": "fastq.filter_reads",
                "tool_id": "{tool_id}",
                "paired_mode": "paired_end",
                "threads": 4,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "filtered_R1.fastq.gz",
                "output_r2": "filtered_R2.fastq.gz",
                "report_json": "filter_report.json",
                "max_n": 0,
                "max_n_fraction": null,
                "max_n_count": 0,
                "low_complexity_threshold": 20.0,
                "entropy_threshold": 20.0,
                "n_policy": "drop",
                "polyx_policy": "trim",
                "contaminant_db": "contaminants.fa",
                "reads_in": 100,
                "reads_out": 95,
                "reads_dropped": 5,
                "reads_removed_by_n": 2,
                "reads_removed_by_entropy": 1,
                "reads_removed_low_complexity": 1,
                "reads_removed_by_kmer": 1,
                "reads_removed_contaminant_kmer": 1,
                "reads_removed_by_length": 0,
                "bases_in": 10000,
                "bases_out": 9200,
                "pairs_in": 50,
                "pairs_out": 47,
                "mean_q_before": 28.0,
                "mean_q_after": 30.0,
                "runtime_s": 4.2,
                "memory_mb": 128.0,
                "exit_code": 0,
                "raw_backend_report": "fastp.json",
                "raw_backend_report_format": "fastp_json",
                "backend_metrics": {{
                    "passed_filter_reads": 95,
                    "too_many_n_reads": 2
                }}
            }}"#,
            tool_id = id_catalog::TOOL_FASTP
        ))?;
        assert_eq!(parsed.tool_id, id_catalog::TOOL_FASTP);
        assert_eq!(parsed.reads_removed_by_n, 2);
        assert_eq!(parsed.reads_out, 95);
        Ok(())
    }

    #[test]
    fn parse_infer_asvs_report_parses_governed_contract() -> Result<()> {
        let parsed = parse_infer_asvs_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.infer_asvs.report.v2",
                "stage": "fastq.infer_asvs",
                "stage_id": "fastq.infer_asvs",
                "tool_id": "dada2",
                "paired_mode": "paired_end",
                "denoising_method": "dada2",
                "pooling_mode": "independent",
                "chimera_policy": "remove_bimera_denovo",
                "requires_r_runtime": true,
                "output_table_kind": "asv_abundance_table",
                "input_reads_r1": "reads_R1.fastq.gz",
                "input_reads_r2": "reads_R2.fastq.gz",
                "asv_table_tsv": "asv_abundance.tsv",
                "asv_sequences_fasta": "asv_sequences.fasta",
                "taxonomy_ready_fasta": "taxonomy_ready.fasta",
                "taxonomy_ready_fastq": "taxonomy_ready.fastq",
                "report_json": "infer_asvs_report.json",
                "asv_count": 12,
                "sample_count": 3,
                "representative_sequence_count": 12,
                "used_fallback": false,
                "raw_backend_report": "infer_asvs_report.json",
                "raw_backend_report_format": "infer_asvs_governed_report_json",
                "runtime_s": 1.2,
                "memory_mb": 128.0,
                "exit_code": 0,
                "backend_metrics": {
                    "nonchimera_reads": 1200
                }
            })
            .to_string(),
        )?;
        assert_eq!(parsed.tool_id, "dada2");
        assert_eq!(parsed.asv_count, 12);
        assert_eq!(parsed.sample_count, 3);
        Ok(())
    }
}
