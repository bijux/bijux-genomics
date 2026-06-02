use std::path::Path;

use anyhow::Result;

use crate::artifacts::{DetectAdaptersReportV1, DETECT_ADAPTERS_REPORT_SCHEMA_VERSION};
use crate::banks::AdapterBankV1;
use crate::params::detect_adapters::{AdapterEvidenceScope, DetectAdaptersEffectiveParams};
use crate::params::PairedMode;

use super::fastq_io::read_fastq_records;

const ADAPTER_BANK_YAML: &str =
    include_str!("../../../../../../assets/reference/adapters/bank.v1.yaml");

#[derive(Debug, Clone, PartialEq)]
struct CandidateAdapter {
    id: String,
    sequence: String,
}

#[derive(Debug, Clone, PartialEq)]
struct DetectionSummary {
    detected_adapter_ids: Vec<String>,
    candidate_adapter_count: u64,
    detection_confidence: Option<f64>,
    detection_threshold: Option<f64>,
    adapter_trimmed_fraction: Option<f64>,
    adapter_content_max: Option<f64>,
    adapter_content_mean: Option<f64>,
}

fn load_embedded_adapter_candidates() -> Result<Vec<CandidateAdapter>> {
    let bank: AdapterBankV1 =
        bijux_dna_infra::formats::parse_yaml(ADAPTER_BANK_YAML).map_err(anyhow::Error::from)?;
    Ok(bank
        .adapters
        .into_iter()
        .filter(|entry| !entry.sequence.contains('N'))
        .map(|entry| CandidateAdapter { id: entry.id, sequence: entry.sequence })
        .collect())
}

fn mean_q(qualities: impl Iterator<Item = String>) -> f64 {
    let mut total = 0_u64;
    let mut count = 0_u64;
    for qual in qualities {
        for byte in qual.bytes() {
            if byte >= 33 {
                total += (byte - 33) as u64;
                count += 1;
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

fn summarize_detected_adapters(records: &[String]) -> Result<DetectionSummary> {
    let candidates = load_embedded_adapter_candidates()?;
    let read_count = records.len() as u64;
    let detection_threshold = (read_count > 0).then_some(1.0 / (read_count as f64));
    let mut candidate_hits = vec![0_u64; candidates.len()];
    let mut per_read_hit_fraction = Vec::<f64>::new();

    for sequence in records {
        let upper = sequence.to_ascii_uppercase();
        let matched_indices = candidates
            .iter()
            .enumerate()
            .filter_map(|(idx, candidate)| upper.contains(&candidate.sequence).then_some(idx))
            .collect::<Vec<_>>();
        let accepted_indices = matched_indices
            .iter()
            .copied()
            .filter(|idx| {
                let sequence = &candidates[*idx].sequence;
                !matched_indices.iter().any(|other_idx| {
                    other_idx != idx
                        && candidates[*other_idx].sequence.len() > sequence.len()
                        && candidates[*other_idx].sequence.contains(sequence)
                })
            })
            .collect::<Vec<_>>();
        for idx in &accepted_indices {
            candidate_hits[*idx] += 1;
        }
        let denominator = candidates.len() as f64;
        per_read_hit_fraction.push(if denominator == 0.0 {
            0.0
        } else {
            accepted_indices.len() as f64 / denominator
        });
    }

    let mut detected_adapter_ids = Vec::new();
    let mut top_fraction = 0.0_f64;
    for (candidate, hits) in candidates.iter().zip(candidate_hits.iter()) {
        if *hits == 0 {
            continue;
        }
        let fraction = *hits as f64 / (read_count as f64);
        if detection_threshold.is_some_and(|threshold| fraction >= threshold) {
            detected_adapter_ids.push(candidate.id.clone());
            if fraction > top_fraction {
                top_fraction = fraction;
            }
        }
    }
    detected_adapter_ids.sort();

    let candidate_adapter_count = detected_adapter_ids.len() as u64;
    let detection_confidence = (candidate_adapter_count > 0).then_some(top_fraction);
    let adapter_trimmed_fraction = if read_count == 0 {
        Some(0.0)
    } else {
        Some(candidate_hits.iter().sum::<u64>() as f64 / (read_count as f64))
    };
    let adapter_content_max = per_read_hit_fraction
        .iter()
        .cloned()
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let adapter_content_mean = if per_read_hit_fraction.is_empty() {
        Some(0.0)
    } else {
        Some(per_read_hit_fraction.iter().sum::<f64>() / per_read_hit_fraction.len() as f64)
    };

    Ok(DetectionSummary {
        detected_adapter_ids,
        candidate_adapter_count,
        detection_confidence,
        detection_threshold,
        adapter_trimmed_fraction,
        adapter_content_max,
        adapter_content_mean,
    })
}

/// Produce backend-comparable adapter evidence from FASTQ reads.
///
/// # Errors
/// Returns an error when FASTQ input cannot be parsed.
pub fn detect_adapters(
    r1: &Path,
    r2: Option<&Path>,
    params: &DetectAdaptersEffectiveParams,
    report_json: &Path,
    adapter_evidence_dir: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<DetectAdaptersReportV1> {
    let mut left = read_fastq_records(r1)?;
    let mut right = if let Some(path) = r2 { read_fastq_records(path)? } else { Vec::new() };

    let paired = r2.is_some();
    if paired && right.len() != left.len() {
        anyhow::bail!(
            "fastq.detect_adapters refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        );
    }

    if matches!(params.evidence_scope, AdapterEvidenceScope::SampledReads) {
        let sample = params.sample_reads.unwrap_or(50_000) as usize;
        left.truncate(sample);
        if paired {
            right.truncate(sample);
        }
    }

    let reads_in = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_in = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let mut n_reads = 0_u64;
    let mut kmer_warning_count = 0_u64;
    let mut overrepresented_sequence_count = 0_u64;
    let mut sequences = Vec::<String>::new();

    for record in left.iter().chain(right.iter()) {
        n_reads += 1;
        let seq = record.sequence.to_ascii_uppercase();
        sequences.push(seq.clone());
        if seq.contains("NNNNN") {
            kmer_warning_count += 1;
        }
        if seq.chars().take(12).all(|base| base == 'A' || base == 'T') {
            overrepresented_sequence_count += 1;
        }
    }

    let detection = summarize_detected_adapters(&sequences)?;

    let duplication_rate = if n_reads == 0 {
        Some(0.0)
    } else {
        let mut seen = std::collections::BTreeSet::new();
        let mut duplicates = 0_u64;
        for record in left.iter().chain(right.iter()) {
            if !seen.insert(record.sequence.clone()) {
                duplicates += 1;
            }
        }
        Some(duplicates as f64 / n_reads as f64)
    };

    let n_rate = if bases_in == 0 {
        Some(0.0)
    } else {
        let n_bases = left
            .iter()
            .chain(right.iter())
            .map(|record| {
                record.sequence.chars().filter(|base| *base == 'N' || *base == 'n').count() as u64
            })
            .sum::<u64>();
        Some(n_bases as f64 / bases_in as f64)
    };

    let evidence_engine = params.evidence_engine.to_ascii_lowercase();
    let normalized_engine = if evidence_engine.contains("cutadapt") {
        "cutadapt"
    } else if evidence_engine.contains("adapterremoval") {
        "adapterremoval"
    } else {
        "fastp"
    };

    Ok(DetectAdaptersReportV1 {
        schema_version: DETECT_ADAPTERS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.detect_adapters".to_string(),
        stage_id: "fastq.detect_adapters".to_string(),
        tool_id: params.evidence_engine.clone(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        threads: params.threads,
        inspection_mode: params.inspection_mode.clone(),
        report_only: params.report_only,
        evidence_engine: params.evidence_engine.clone(),
        evidence_scope: params.evidence_scope.clone(),
        evidence_format: params.evidence_format.clone(),
        evidence_artifact_id: params.evidence_artifact_id.clone(),
        detected_adapter_source: format!("normalized_{normalized_engine}_evidence"),
        detected_adapter_ids: detection.detected_adapter_ids.clone(),
        detection_confidence: detection.detection_confidence,
        detection_threshold: detection.detection_threshold,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        adapter_evidence_dir: adapter_evidence_dir.display().to_string(),
        recommended_adapter_bank_id: (detection.candidate_adapter_count > 0)
            .then_some("illumina".to_string()),
        recommended_adapter_bank_hash: (detection.candidate_adapter_count > 0)
            .then_some("sha256:governed-adapter-bank".to_string()),
        recommended_adapter_preset: (detection.candidate_adapter_count > 0)
            .then_some("illumina-default".to_string()),
        reads_in,
        reads_out: reads_in,
        bases_in,
        bases_out: bases_in,
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(left.len() as u64),
        mean_q: mean_q(left.iter().chain(right.iter()).map(|record| record.quality.clone())),
        candidate_adapter_count: detection.candidate_adapter_count,
        adapter_trimmed_fraction: detection.adapter_trimmed_fraction,
        adapter_content_max: detection.adapter_content_max,
        adapter_content_mean: detection.adapter_content_mean,
        duplication_rate,
        n_rate,
        kmer_warning_count: Some(kmer_warning_count),
        overrepresented_sequence_count: Some(overrepresented_sequence_count),
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: Some(format!("{}_normalized", normalized_engine)),
    })
}

#[cfg(test)]
mod tests {
    use super::detect_adapters;
    use crate::params::detect_adapters::{
        AdapterEvidenceFormat, AdapterEvidenceScope, AdapterInspectionMode,
        DetectAdaptersEffectiveParams,
    };
    use crate::params::PairedMode;
    use crate::stages::contract::runtime::detect_adapters::summarize_detected_adapters;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn detect_adapters_normalizes_backend_evidence_fields() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-detect-adapters")?;
        let r1 = temp.path().join("r1.fastq");
        write_fastq(
            &r1,
            &[
                ("a", "ACGTAGATCGGAAGAGCTTT", "IIIIIIIIIIIIIIIIIIII"),
                ("b", "NNNNNACGTACGT", "IIIIIIIIIIIII"),
            ],
        )?;

        let params = DetectAdaptersEffectiveParams {
            schema_version: "bijux.fastq.params.detect_adapters.v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: 2,
            sample_reads: Some(100),
            inspection_mode: AdapterInspectionMode::EvidenceOnly,
            report_only: true,
            evidence_engine: "cutadapt".to_string(),
            evidence_scope: AdapterEvidenceScope::FullInput,
            evidence_format: AdapterEvidenceFormat::FastqcSummary,
            evidence_artifact_id: "adapter_evidence".to_string(),
        };

        let report = detect_adapters(
            &r1,
            None,
            &params,
            &temp.path().join("detect.json"),
            &temp.path().join("evidence"),
            Some(&temp.path().join("raw.txt")),
        )?;

        assert_eq!(report.detected_adapter_source, "normalized_cutadapt_evidence");
        assert!(report.candidate_adapter_count > 0);
        assert_eq!(report.detected_adapter_ids, vec!["truseq_universal".to_string()]);
        assert_eq!(report.detection_threshold, Some(0.5));
        assert_eq!(report.detection_confidence, Some(0.5));
        assert!(report.recommended_adapter_preset.is_some());
        Ok(())
    }

    #[test]
    fn detect_adapters_maps_sequences_to_governed_adapter_ids() -> anyhow::Result<()> {
        let detection = summarize_detected_adapters(&[
            "ACGTAGATCGGAAGAGCTTT".to_string(),
            "TTAATGATACGGCGACCACCGAGG".to_string(),
            "GGGGGGGG".to_string(),
        ])?;

        assert_eq!(
            detection.detected_adapter_ids,
            vec!["pcr_primer_fwd".to_string(), "truseq_universal".to_string()]
        );
        assert_eq!(detection.candidate_adapter_count, 2);
        assert_eq!(detection.detection_threshold, Some(1.0 / 3.0));
        assert_eq!(detection.detection_confidence, Some(1.0 / 3.0));
        Ok(())
    }
}
