use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::{
    AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode, DetectAdaptersReportV1,
    PairedMode, TaxonomyScreenSummaryEntryV1,
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
        return Err(anyhow!("unsupported detect adapters report schema {}", legacy.schema_version));
    }
    let inspection_mode = match legacy.inspection_mode.as_str() {
        "evidence_only" => AdapterInspectionMode::EvidenceOnly,
        other => return Err(anyhow!("unsupported detect adapters inspection mode {other}")),
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
        detected_adapter_source: "legacy_fastqc_summary".to_string(),
        input_r1: legacy.input_fastq,
        input_r2: None,
        report_json: "adapter_report.json".to_string(),
        adapter_evidence_dir: legacy.fastqc_dir,
        recommended_adapter_bank_id: None,
        recommended_adapter_bank_hash: None,
        recommended_adapter_preset: None,
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
            return Err(anyhow!("screen report line {} has {} columns", idx + 1, parts.len()));
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
