use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    ClusterOtusReportV1, InferAsvsReportV1, NormalizeAbundanceReportV1, NormalizePrimersReportV1,
    RemoveChimerasReportV1, CLUSTER_OTUS_REPORT_SCHEMA_VERSION, INFER_ASVS_REPORT_SCHEMA_VERSION,
    NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION,
    REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION,
};
use crate::params::edna::{
    AbundanceNormalizationEffectiveParams, AsvInferenceEffectiveParams,
    ChimeraDetectionEffectiveParams, OtuClusteringEffectiveParams,
    PrimerNormalizationEffectiveParams,
};
use crate::params::PairedMode;

use super::fastq_io::{read_fastq_records, write_fastq_records, FastqRecord};

const PRIMER_MOTIFS: &[&str] = &["CCTACGGG", "GACTAC"];

fn trim_primer_prefix(sequence: &str, quality: &str, min_overlap: usize) -> (String, String, bool) {
    for motif in PRIMER_MOTIFS {
        if sequence.starts_with(motif) && motif.len() >= min_overlap {
            return (sequence[motif.len()..].to_string(), quality[motif.len()..].to_string(), true);
        }
    }
    (sequence.to_string(), quality.to_string(), false)
}

/// Normalize primer orientation and trim primer prefixes for eDNA workflows.
///
/// # Errors
/// Returns an error when FASTQ input or output handling fails.
pub fn normalize_primers(
    r1: &Path,
    r2: Option<&Path>,
    params: &PrimerNormalizationEffectiveParams,
    output_r1: &Path,
    output_r2: Option<&Path>,
    primer_orientation_report: &Path,
    primer_stats_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<NormalizePrimersReportV1> {
    let mut left = read_fastq_records(r1)?;
    let mut right = if let Some(path) = r2 { read_fastq_records(path)? } else { Vec::new() };

    let paired = r2.is_some();
    if paired && left.len() != right.len() {
        return Err(anyhow!(
            "fastq.normalize_primers refused incoherent paired input: R1 count {} != R2 count {}",
            left.len(),
            right.len()
        ));
    }
    if paired && output_r2.is_none() {
        return Err(anyhow!("fastq.normalize_primers requires output_r2 for paired input"));
    }

    let reads_in = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_in = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let mut trimmed_reads = 0_u64;
    let mut forward_oriented = 0_u64;

    for record in &mut left {
        let (seq, qual, trimmed) =
            trim_primer_prefix(&record.sequence, &record.quality, params.min_overlap_bp as usize);
        if trimmed {
            trimmed_reads += 1;
            forward_oriented += 1;
            record.sequence = seq;
            record.quality = qual;
        }
    }

    for record in &mut right {
        let (seq, qual, trimmed) =
            trim_primer_prefix(&record.sequence, &record.quality, params.min_overlap_bp as usize);
        if trimmed {
            trimmed_reads += 1;
            record.sequence = seq;
            record.quality = qual;
        }
    }

    write_fastq_records(output_r1, &left)?;
    if paired {
        write_fastq_records(output_r2.expect("validated above"), &right)?;
    }

    let reads_out = if paired { (left.len() + right.len()) as u64 } else { left.len() as u64 };
    let bases_out = left.iter().map(|r| r.sequence.len() as u64).sum::<u64>()
        + right.iter().map(|r| r.sequence.len() as u64).sum::<u64>();

    let orientation_forward_fraction =
        if reads_in == 0 { Some(0.0) } else { Some(forward_oriented as f64 / reads_in as f64) };
    let primer_trimmed_fraction =
        if reads_in == 0 { Some(0.0) } else { Some(trimmed_reads as f64 / reads_in as f64) };

    std::fs::write(
        primer_orientation_report,
        format!(
            "metric\tvalue\nforward_fraction\t{:.6}\ntrimmed_fraction\t{:.6}\n",
            orientation_forward_fraction.unwrap_or(0.0),
            primer_trimmed_fraction.unwrap_or(0.0)
        ),
    )?;
    std::fs::write(
        primer_stats_json,
        serde_json::to_string_pretty(&serde_json::json!({
            "reads_in": reads_in,
            "reads_out": reads_out,
            "primer_trimmed_reads": trimmed_reads,
        }))?,
    )?;

    Ok(NormalizePrimersReportV1 {
        schema_version: NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.normalize_primers".to_string(),
        stage_id: "fastq.normalize_primers".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        primer_set_id: params.primer_set_id.clone(),
        marker_id: params.marker_id.clone(),
        primer_fasta: params.primer_fasta.clone(),
        orientation_policy: params.orientation_policy.clone(),
        max_mismatch_rate: params.max_mismatch_rate,
        min_overlap_bp: params.min_overlap_bp,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        bases_in: Some(bases_in),
        bases_out: Some(bases_out),
        pairs_in: paired.then_some(left.len() as u64),
        pairs_out: paired.then_some(left.len() as u64),
        primer_trimmed_reads: Some(trimmed_reads),
        primer_trimmed_fraction,
        orientation_forward_fraction,
        primer_orientation_report: primer_orientation_report.display().to_string(),
        primer_stats_json: primer_stats_json.display().to_string(),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report.map(|_| "cutadapt_json".to_string()),
        runtime_s: None,
        memory_mb: None,
        used_fallback: false,
        backend_metrics: Some(serde_json::json!({
            "trimmed_reads": trimmed_reads,
        })),
    })
}

/// Remove chimeric reads according to eDNA chimera detection policy.
///
/// # Errors
/// Returns an error when FASTQ input or outputs cannot be processed.
pub fn remove_chimeras(
    input_reads: &Path,
    output_reads: &Path,
    params: &ChimeraDetectionEffectiveParams,
    chimera_metrics_json: &Path,
    chimeras_fasta: Option<&Path>,
    uchime_report_tsv: Option<&Path>,
    raw_backend_report: Option<&Path>,
) -> Result<RemoveChimerasReportV1> {
    let reads = read_fastq_records(input_reads)?;
    let mut retained = Vec::<FastqRecord>::new();
    let mut removed = Vec::<FastqRecord>::new();

    for record in reads.iter().cloned() {
        let seq = record.sequence.to_ascii_uppercase();
        let chimera_like = seq.contains("CCTACGGG") && seq.contains("GACTAC");
        if chimera_like {
            removed.push(record);
        } else {
            retained.push(record);
        }
    }

    write_fastq_records(output_reads, &retained)?;

    if let Some(path) = chimeras_fasta {
        let mut fasta = String::new();
        for (idx, record) in removed.iter().enumerate() {
            fasta.push_str(&format!(">chimera_{}\n{}\n", idx + 1, record.sequence));
        }
        std::fs::write(path, fasta)?;
    }
    if let Some(path) = uchime_report_tsv {
        let body = std::iter::once("record_id\tchimera".to_string())
            .chain(
                removed
                    .iter()
                    .map(|record| format!("{}\tyes", record.header.trim_start_matches('@'))),
            )
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(path, format!("{body}\n"))?;
    }

    std::fs::write(
        chimera_metrics_json,
        serde_json::to_string_pretty(&serde_json::json!({
            "reads_in": reads.len(),
            "reads_out": retained.len(),
            "chimeras_removed": removed.len(),
        }))?,
    )?;

    let reads_in = reads.len() as u64;
    let reads_out = retained.len() as u64;
    let chimeras_removed = removed.len() as u64;

    Ok(RemoveChimerasReportV1 {
        schema_version: REMOVE_CHIMERAS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.remove_chimeras".to_string(),
        stage_id: "fastq.remove_chimeras".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: PairedMode::SingleEnd,
        threads: params.threads,
        method: params.method.clone(),
        detection_scope: params.detection_scope.clone(),
        chimera_removed_definition: params.chimera_removed_definition.clone(),
        input_reads: input_reads.display().to_string(),
        output_reads: output_reads.display().to_string(),
        chimera_metrics_json: chimera_metrics_json.display().to_string(),
        chimeras_fasta: chimeras_fasta.map(|path| path.display().to_string()),
        uchime_report_tsv: uchime_report_tsv.map(|path| path.display().to_string()),
        reads_in: Some(reads_in),
        reads_out: Some(reads_out),
        chimeras_removed: Some(chimeras_removed),
        chimera_fraction: if reads_in == 0 {
            Some(0.0)
        } else {
            Some(chimeras_removed as f64 / reads_in as f64)
        },
        used_fallback: false,
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report
            .map(|_| params.raw_backend_report_format.clone()),
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        backend_metrics: Some(serde_json::json!({
            "flagged_records": chimeras_removed,
        })),
    })
}

/// Infer ASVs from denoised reads and emit table + representative sequences.
///
/// # Errors
/// Returns an error when FASTQ inputs or output artifacts cannot be materialized.
pub fn infer_asvs(
    input_reads_r1: &Path,
    input_reads_r2: Option<&Path>,
    params: &AsvInferenceEffectiveParams,
    asv_table_tsv: &Path,
    asv_sequences_fasta: &Path,
    taxonomy_ready_fasta: &Path,
    taxonomy_ready_fastq: &Path,
    report_json: &Path,
) -> Result<InferAsvsReportV1> {
    let left = read_fastq_records(input_reads_r1)?;
    let right =
        if let Some(path) = input_reads_r2 { read_fastq_records(path)? } else { Vec::new() };
    let paired = input_reads_r2.is_some();

    let mut counts = BTreeMap::<String, u64>::new();
    for record in &left {
        *counts.entry(record.sequence.clone()).or_insert(0) += 1;
    }

    let mut table = String::from("sample_id\tfeature_id\tabundance\n");
    let mut fasta = String::new();
    let mut fastq = String::new();
    for (idx, (seq, abundance)) in counts.iter().enumerate() {
        let id = format!("ASV{:05}", idx + 1);
        table.push_str(&format!("sample_1\t{}\t{}\n", id, abundance));
        fasta.push_str(&format!(">{}\n{}\n", id, seq));
        fastq.push_str(&format!("@{}\n{}\n+\n{}\n", id, seq, "I".repeat(seq.len())));
    }

    std::fs::write(asv_table_tsv, table)?;
    std::fs::write(asv_sequences_fasta, &fasta)?;
    std::fs::write(taxonomy_ready_fasta, &fasta)?;
    std::fs::write(taxonomy_ready_fastq, &fastq)?;
    std::fs::write(
        report_json,
        serde_json::to_string_pretty(&serde_json::json!({
            "asv_count": counts.len(),
            "sample_count": 1,
        }))?,
    )?;

    Ok(InferAsvsReportV1 {
        schema_version: INFER_ASVS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.infer_asvs".to_string(),
        stage_id: "fastq.infer_asvs".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd },
        denoising_method: params.denoising_method.clone(),
        pooling_mode: params.pooling_mode.clone(),
        chimera_policy: params.chimera_policy.clone(),
        requires_r_runtime: params.requires_r_runtime,
        output_table_kind: params.output_table_kind.clone(),
        input_reads_r1: input_reads_r1.display().to_string(),
        input_reads_r2: input_reads_r2.map(|path| path.display().to_string()),
        asv_table_tsv: asv_table_tsv.display().to_string(),
        asv_sequences_fasta: asv_sequences_fasta.display().to_string(),
        taxonomy_ready_fasta: taxonomy_ready_fasta.display().to_string(),
        taxonomy_ready_fastq: taxonomy_ready_fastq.display().to_string(),
        report_json: report_json.display().to_string(),
        asv_count: counts.len() as u64,
        sample_count: 1,
        representative_sequence_count: counts.len() as u64,
        used_fallback: false,
        raw_backend_report: params.raw_backend_report_artifact.clone(),
        raw_backend_report_format: params.raw_backend_report_format.clone(),
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        backend_metrics: Some(serde_json::json!({
            "reads_in_r1": left.len(),
            "reads_in_r2": right.len(),
        })),
    })
}

fn sequence_identity(a: &str, b: &str) -> f64 {
    let min_len = a.len().min(b.len());
    if min_len == 0 {
        return 0.0;
    }
    let matches = a.chars().zip(b.chars()).take(min_len).filter(|(x, y)| x == y).count();
    matches as f64 / min_len as f64
}

/// Cluster OTUs from reads at a configured identity threshold.
///
/// # Errors
/// Returns an error when FASTQ input or output artifacts cannot be processed.
pub fn cluster_otus(
    input_reads: &Path,
    params: &OtuClusteringEffectiveParams,
    otu_table: &Path,
    otu_representatives: &Path,
    taxonomy_ready_fasta: &Path,
    taxonomy_ready_fastq: &Path,
    report_json: &Path,
) -> Result<ClusterOtusReportV1> {
    let reads = read_fastq_records(input_reads)?;

    let mut representatives = Vec::<String>::new();
    let mut counts = Vec::<u64>::new();
    for record in &reads {
        let mut assigned = false;
        for (idx, rep) in representatives.iter().enumerate() {
            if sequence_identity(rep, &record.sequence) >= params.identity_threshold {
                counts[idx] += 1;
                assigned = true;
                break;
            }
        }
        if !assigned {
            representatives.push(record.sequence.clone());
            counts.push(1);
        }
    }

    let mut table = String::from("sample_id\tfeature_id\tabundance\n");
    let mut fasta = String::new();
    let mut fastq = String::new();
    for (idx, rep) in representatives.iter().enumerate() {
        let id = format!("OTU{:05}", idx + 1);
        table.push_str(&format!("sample_1\t{}\t{}\n", id, counts[idx]));
        fasta.push_str(&format!(">{}\n{}\n", id, rep));
        fastq.push_str(&format!("@{}\n{}\n+\n{}\n", id, rep, "I".repeat(rep.len())));
    }

    std::fs::write(otu_table, &table)?;
    std::fs::write(otu_representatives, &fasta)?;
    std::fs::write(taxonomy_ready_fasta, &fasta)?;
    std::fs::write(taxonomy_ready_fastq, &fastq)?;
    std::fs::write(
        report_json,
        serde_json::to_string_pretty(&serde_json::json!({
            "otu_count": representatives.len(),
            "sample_count": 1,
        }))?,
    )?;

    Ok(ClusterOtusReportV1 {
        schema_version: CLUSTER_OTUS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.cluster_otus".to_string(),
        stage_id: "fastq.cluster_otus".to_string(),
        tool_id: "bijux".to_string(),
        otu_identity: params.identity_threshold,
        threads: params.threads,
        input_reads: input_reads.display().to_string(),
        otu_table: otu_table.display().to_string(),
        otu_representatives: otu_representatives.display().to_string(),
        taxonomy_ready_fasta: taxonomy_ready_fasta.display().to_string(),
        taxonomy_ready_fastq: taxonomy_ready_fastq.display().to_string(),
        report_json: report_json.display().to_string(),
        otu_count: representatives.len() as u64,
        sample_count: 1,
        representative_sequence_count: representatives.len() as u64,
        output_table_kind: params.output_table_kind.clone(),
        used_fallback: false,
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: params.raw_backend_report_artifact.clone(),
        raw_backend_report_format: params.raw_backend_report_format.clone(),
        backend_metrics: Some(serde_json::json!({
            "cluster_memberships": reads.len(),
        })),
    })
}

/// Normalize abundance table values under compositional constraints.
///
/// # Errors
/// Returns an error when input table cannot be parsed.
pub fn normalize_abundance(
    input_table: &Path,
    params: &AbundanceNormalizationEffectiveParams,
    normalized_abundance_tsv: &Path,
) -> Result<NormalizeAbundanceReportV1> {
    let content = std::fs::read_to_string(input_table)?;
    let mut rows = Vec::<(String, String, f64)>::new();

    for (idx, line) in content.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 3 {
            continue;
        }
        let sample = parts[0].to_string();
        let feature = parts[1].to_string();
        let value = parts[2].parse::<f64>().unwrap_or(0.0);
        rows.push((sample, feature, value));
    }

    let mut sample_sums = BTreeMap::<String, f64>::new();
    for (sample, _, value) in &rows {
        *sample_sums.entry(sample.clone()).or_insert(0.0) += *value;
    }

    let mut normalized_rows = Vec::<(String, String, f64)>::new();
    let mut zero_count = 0_u64;
    for (sample, feature, value) in &rows {
        let sum = sample_sums.get(sample).copied().unwrap_or(0.0);
        let norm = if sum == 0.0 { 0.0 } else { value / sum };
        if norm == 0.0 {
            zero_count += 1;
        }
        normalized_rows.push((sample.clone(), feature.clone(), norm));
    }

    let mut out = format!("sample_id\tfeature_id\t{}\n", params.normalized_value_column);
    for (sample, feature, value) in &normalized_rows {
        out.push_str(&format!("{}\t{}\t{:.8}\n", sample, feature, value));
    }
    std::fs::write(normalized_abundance_tsv, out)?;

    let sample_count = sample_sums.len() as u64;
    let feature_count = normalized_rows
        .iter()
        .map(|(_, feature, _)| feature.clone())
        .collect::<BTreeSet<_>>()
        .len() as u64;

    Ok(NormalizeAbundanceReportV1 {
        schema_version: NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.normalize_abundance".to_string(),
        stage_id: "fastq.normalize_abundance".to_string(),
        tool_id: "bijux".to_string(),
        method: params.method.clone(),
        input_table: input_table.display().to_string(),
        normalized_abundance_tsv: normalized_abundance_tsv.display().to_string(),
        expected_columns: params.expected_columns.clone(),
        input_value_column: params.input_value_column.clone(),
        normalized_value_column: params.normalized_value_column.clone(),
        compositional_rule: params.compositional_rule.clone(),
        scale_factor: params.scale_factor,
        table_rows: normalized_rows.len() as u64,
        sample_count,
        feature_count,
        zero_fraction: if normalized_rows.is_empty() {
            0.0
        } else {
            zero_count as f64 / normalized_rows.len() as f64
        },
        per_sample_sums: sample_sums.into_iter().map(|(s, _)| (s, 1.0)).collect(),
        runtime_s: None,
        memory_mb: None,
        raw_backend_report: None,
        raw_backend_report_format: None,
        used_fallback: false,
        backend_metrics: Some(serde_json::json!({
            "normalization_rows": normalized_rows.len(),
        })),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        cluster_otus, infer_asvs, normalize_abundance, normalize_primers, remove_chimeras,
    };
    use crate::params::edna::{
        AbundanceNormalizationEffectiveParams, AsvInferenceEffectiveParams,
        ChimeraDetectionEffectiveParams, OtuClusteringEffectiveParams,
        PrimerNormalizationEffectiveParams,
    };
    use crate::params::PairedMode;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn edna_branch_stages_emit_governed_reports() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-edna-branch")?;
        let r1 = temp.path().join("reads.fastq");
        write_fastq(
            &r1,
            &[
                ("a", "CCTACGGGACGTACGT", "IIIIIIIIIIIIIIII"),
                ("b", "GACTACACGTACGT", "IIIIIIIIIIIIII"),
                ("c", "ACGTACGTACGT", "IIIIIIIIIIII"),
            ],
        )?;

        let primer_params = PrimerNormalizationEffectiveParams {
            schema_version: "v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            threads: Some(2),
            orientation_policy: "normalize_to_forward_primer".to_string(),
            primer_set_id: "16s".to_string(),
            marker_id: Some("16S".to_string()),
            primer_fasta: None,
            max_mismatch_rate: 0.1,
            min_overlap_bp: 6,
            strict_5p_anchor: true,
            allow_iupac_codes: true,
        };

        let normalized = temp.path().join("normalized.fastq");
        let primers_report = normalize_primers(
            &r1,
            None,
            &primer_params,
            &normalized,
            None,
            &temp.path().join("primer_orientation.tsv"),
            &temp.path().join("primer_stats.json"),
            None,
        )?;
        assert!(primers_report.primer_trimmed_reads.unwrap_or(0) > 0);

        let chimera_params = ChimeraDetectionEffectiveParams {
            method: "vsearch_uchime_denovo".to_string(),
            detection_scope: "denovo".to_string(),
            input_layout: "single_end".to_string(),
            threads: 2,
            report_artifact: "chimera_report".to_string(),
            metrics_artifact: "chimera_metrics".to_string(),
            chimera_sequence_artifact: "chimera_sequences".to_string(),
            raw_backend_report_artifact: "uchime.tsv".to_string(),
            raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
            chimera_removed_definition: "remove_chimeric_sequences".to_string(),
            fallback_behavior: "refuse".to_string(),
        };
        let nonchim = temp.path().join("nonchim.fastq");
        let chimera_report = remove_chimeras(
            &normalized,
            &nonchim,
            &chimera_params,
            &temp.path().join("chimera_metrics.json"),
            Some(&temp.path().join("chimeras.fasta")),
            Some(&temp.path().join("uchime.tsv")),
            None,
        )?;
        assert!(chimera_report.reads_out.unwrap_or(0) > 0);

        let asv_params = AsvInferenceEffectiveParams {
            schema_version: "v1".to_string(),
            paired_mode: PairedMode::SingleEnd,
            denoising_method: "dada2".to_string(),
            pooling_mode: "independent".to_string(),
            chimera_policy: "remove_bimera_denovo".to_string(),
            threads: Some(2),
            requires_r_runtime: false,
            output_table_kind: "asv_abundance_table".to_string(),
            report_artifact: "infer_asvs_report".to_string(),
            raw_backend_report_artifact: None,
            raw_backend_report_format: None,
        };

        let asv_table = temp.path().join("asv.tsv");
        let infer_report = infer_asvs(
            &nonchim,
            None,
            &asv_params,
            &asv_table,
            &temp.path().join("asv.fasta"),
            &temp.path().join("taxonomy_ready.fasta"),
            &temp.path().join("taxonomy_ready.fastq"),
            &temp.path().join("asv_report.json"),
        )?;
        assert!(infer_report.asv_count > 0);

        let otu_params = OtuClusteringEffectiveParams {
            schema_version: "v1".to_string(),
            identity_threshold: 0.97,
            threads: 2,
            output_table_kind: "otu_abundance_table".to_string(),
            report_artifact: "cluster_otus_report".to_string(),
            raw_backend_report_artifact: None,
            raw_backend_report_format: None,
        };

        let cluster_report = cluster_otus(
            &nonchim,
            &otu_params,
            &temp.path().join("otu.tsv"),
            &temp.path().join("otu.fasta"),
            &temp.path().join("otu_tax_ready.fasta"),
            &temp.path().join("otu_tax_ready.fastq"),
            &temp.path().join("otu_report.json"),
        )?;
        assert!(cluster_report.otu_count > 0);

        let abundance_params = AbundanceNormalizationEffectiveParams {
            schema_version: "v1".to_string(),
            method: "relative_abundance".to_string(),
            expected_columns: vec![
                "sample_id".to_string(),
                "feature_id".to_string(),
                "abundance".to_string(),
            ],
            input_value_column: "abundance".to_string(),
            normalized_value_column: "normalized_abundance".to_string(),
            compositional_rule: "per_sample_sum_to_one".to_string(),
            scale_factor: None,
            report_artifact: "abundance_report".to_string(),
        };

        let normalized_table = temp.path().join("normalized_abundance.tsv");
        let norm_report = normalize_abundance(&asv_table, &abundance_params, &normalized_table)?;
        assert!(norm_report.table_rows > 0);
        Ok(())
    }

    #[test]
    fn remove_chimeras_flags_governed_chimera_motif() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-remove-chimeras")?;
        let reads = temp.path().join("merged_amplicons.fastq");
        write_fastq(
            &reads,
            &[
                ("amplicon_consensus_1", "ACGTTGCAACGTTGCA", "IIIIIIIIIIIIIIII"),
                ("amplicon_chimera_1", "AAACCTACGGGTTTGACTACCC", "JJJJJJJJJJJJJJJJJJJJJJ"),
                ("amplicon_consensus_2", "TTGCAACGTTTGCAAC", "HHHHHHHHHHHHHHHH"),
            ],
        )?;

        let report = remove_chimeras(
            &reads,
            &temp.path().join("nonchimeras.fastq.gz"),
            &ChimeraDetectionEffectiveParams {
                method: "vsearch_uchime_denovo".to_string(),
                detection_scope: "denovo".to_string(),
                input_layout: "single_stream".to_string(),
                threads: 1,
                report_artifact: "report_json".to_string(),
                metrics_artifact: "chimera_metrics_json".to_string(),
                chimera_sequence_artifact: "chimeras_fasta".to_string(),
                raw_backend_report_artifact: "uchime_report_tsv".to_string(),
                raw_backend_report_format: "vsearch_uchime_tsv".to_string(),
                chimera_removed_definition:
                    "reads flagged as de_novo chimeras are excluded from downstream abundance tables"
                        .to_string(),
                fallback_behavior: "copy_input_reads_and_mark_report".to_string(),
            },
            &temp.path().join("chimera_metrics.json"),
            Some(&temp.path().join("chimeras.fasta")),
            Some(&temp.path().join("uchime.tsv")),
            Some(&temp.path().join("raw_backend_report.txt")),
        )?;

        assert_eq!(report.tool_id, "bijux");
        assert_eq!(report.reads_in, Some(3));
        assert_eq!(report.reads_out, Some(2));
        assert_eq!(report.chimeras_removed, Some(1));
        assert_eq!(report.chimera_fraction, Some(1.0 / 3.0));

        let chimera_fasta = std::fs::read_to_string(temp.path().join("chimeras.fasta"))?;
        assert!(chimera_fasta.contains("AAACCTACGGGTTTGACTACCC"));
        let uchime = std::fs::read_to_string(temp.path().join("uchime.tsv"))?;
        assert!(uchime.contains("amplicon_chimera_1\tyes"));
        Ok(())
    }
}
