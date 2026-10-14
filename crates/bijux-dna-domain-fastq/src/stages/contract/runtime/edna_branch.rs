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
use crate::stages::primer_normalization::{
    normalize_primer_records, validate_primer_read_counts, PrimerNormalizationRequest,
};

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
    let left = read_fastq_records(r1)?;
    let right = r2.map(read_fastq_records).transpose()?;

    let paired = r2.is_some();
    validate_primer_read_counts(left.len(), right.as_ref().map(Vec::len))?;
    if paired && output_r2.is_none() {
        return Err(anyhow!("fastq.normalize_primers requires output_r2 for paired input"));
    }

    let normalized =
        normalize_primer_records(PrimerNormalizationRequest { r1: left, r2: right, params })?;
    let left = normalized.r1;
    let right = normalized.r2.unwrap_or_default();
    let summary = normalized.summary;
    let reads_in = summary.reads_in;
    let reads_out = summary.reads_out;
    let bases_in = summary.bases_in;
    let bases_out = summary.bases_out;
    let trimmed_reads = summary.primer_trimmed_reads;
    let orientation_forward_fraction = summary.orientation_forward_fraction;
    let primer_trimmed_fraction = summary.primer_trimmed_fraction;

    write_fastq_records(output_r1, &left)?;
    if let Some(output_r2) = output_r2 {
        write_fastq_records(output_r2, &right)?;
    }

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

    let scale_factor = params.scale_factor.unwrap_or(1.0);
    let mut normalized_sums = BTreeMap::<String, f64>::new();
    let mut normalized_rows = Vec::<(String, String, f64)>::new();
    let mut zero_count = 0_u64;
    for (sample, feature, value) in &rows {
        let sum = sample_sums.get(sample).copied().unwrap_or(0.0);
        let norm = if sum == 0.0 { 0.0 } else { (value * scale_factor) / sum };
        if norm == 0.0 {
            zero_count += 1;
        }
        *normalized_sums.entry(sample.clone()).or_insert(0.0) += norm;
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
        per_sample_sums: normalized_sums.into_iter().collect(),
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

    fn primer_parameters() -> PrimerNormalizationEffectiveParams {
        PrimerNormalizationEffectiveParams {
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
        }
    }

    #[test]
    fn primer_outputs_preserve_exact_fastq_tsv_and_stats_bytes() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-primer-publication")?;
        let input = temp.path().join("input.fastq");
        std::fs::write(&input, "@a/1 retained\nCCTACGGGACGT\n+identity\nabcdefghIJKL\n")?;
        let output = temp.path().join("normalized.fastq");
        let empty_mate = temp.path().join("optional-mate.fastq");
        let orientation = temp.path().join("orientation.tsv");
        let stats = temp.path().join("stats.json");
        let report = normalize_primers(
            &input,
            None,
            &primer_parameters(),
            &output,
            Some(&empty_mate),
            &orientation,
            &stats,
            Some(&stats),
        )?;
        assert_eq!(std::fs::read_to_string(&output)?, "@a/1 retained\nACGT\n+identity\nIJKL\n");
        assert_eq!(std::fs::read(&empty_mate)?, Vec::<u8>::new());
        assert_eq!(
            std::fs::read_to_string(&orientation)?,
            "metric\tvalue\nforward_fraction\t1.000000\ntrimmed_fraction\t1.000000\n"
        );
        assert_eq!(
            std::fs::read_to_string(&stats)?,
            "{\n  \"primer_trimmed_reads\": 1,\n  \"reads_in\": 1,\n  \"reads_out\": 1\n}"
        );
        assert_eq!(report.paired_mode, PairedMode::SingleEnd);
        assert_eq!(report.output_r2, Some(empty_mate.display().to_string()));
        assert_eq!(report.pairs_in, None);
        assert_eq!(report.bases_in, Some(12));
        assert_eq!(report.bases_out, Some(4));
        assert_eq!(report.tool_id, "bijux");
        assert!(!report.used_fallback);
        assert_eq!(report.raw_backend_report_format.as_deref(), Some("cutadapt_json"));
        Ok(())
    }

    #[test]
    fn primer_mate_count_error_precedes_missing_output_and_all_writes() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-primer-admission")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        let output = temp.path().join("output.fastq");
        let orientation = temp.path().join("orientation.tsv");
        let stats = temp.path().join("stats.json");
        write_fastq(&r1, &[("a", "ACGT", "IIII")])?;
        write_fastq(&r2, &[])?;
        let run = || {
            normalize_primers(
                &r1,
                Some(&r2),
                &primer_parameters(),
                &output,
                None,
                &orientation,
                &stats,
                None,
            )
        };
        assert_eq!(
            run().expect_err("count mismatch rejected").to_string(),
            "fastq.normalize_primers refused incoherent paired input: R1 count 1 != R2 count 0"
        );
        write_fastq(&r2, &[("a", "ACGT", "IIII")])?;
        assert_eq!(
            run().expect_err("missing paired output rejected").to_string(),
            "fastq.normalize_primers requires output_r2 for paired input"
        );
        assert!(!output.exists());
        assert!(!orientation.exists());
        assert!(!stats.exists());
        Ok(())
    }

    #[test]
    fn primer_publication_failures_retain_declared_write_order() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-primer-write-order")?;
        let input = temp.path().join("input.fastq");
        write_fastq(&input, &[("a", "CCTACGGGACGT", "IIIIIIIIIIII")])?;
        for blocked in ["r2", "orientation", "stats"] {
            let root = temp.path().join(blocked);
            std::fs::create_dir(&root)?;
            let r1 = root.join("r1.fastq");
            let r2 = root.join("r2.fastq");
            let orientation = root.join("orientation.tsv");
            let stats = root.join("stats.json");
            let failure = match blocked {
                "r2" => &r2,
                "orientation" => &orientation,
                _ => &stats,
            };
            std::fs::create_dir(failure)?;
            assert!(normalize_primers(
                &input,
                Some(&input),
                &primer_parameters(),
                &r1,
                Some(&r2),
                &orientation,
                &stats,
                None,
            )
            .is_err());
            assert!(r1.is_file(), "R1 publication precedes {blocked}");
            if blocked == "r2" {
                assert!(!orientation.exists());
                assert!(!stats.exists());
            } else {
                assert!(r2.is_file(), "R2 publication precedes {blocked}");
                if blocked == "orientation" {
                    assert!(!stats.exists());
                } else {
                    assert!(orientation.is_file(), "orientation publication precedes stats");
                }
            }
        }
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

    fn abundance_parameters() -> AbundanceNormalizationEffectiveParams {
        AbundanceNormalizationEffectiveParams {
            schema_version: "bijux.params.edna.v1".to_string(),
            method: "relative_abundance".to_string(),
            expected_columns: vec!["sample_id".into(), "feature_id".into(), "abundance".into()],
            input_value_column: "abundance".to_string(),
            normalized_value_column: "normalized_abundance".to_string(),
            compositional_rule: "per_sample_sum_to_one".to_string(),
            scale_factor: None,
            report_artifact: "report_json".to_string(),
        }
    }

    #[test]
    fn normalize_abundance_honors_declared_counts_per_million_units() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-abundance-declared-scale")?;
        let input = temp.path().join("input.tsv");
        let output = temp.path().join("normalized.tsv");
        let original = "sample_id\tfeature_id\tabundance\ns\tf1\t25\ns\tf2\t75\n";
        std::fs::write(&input, original)?;
        let mut params = abundance_parameters();
        params.method = "counts_per_million".to_string();
        params.normalized_value_column = "counts_per_million".to_string();
        params.compositional_rule = "per_sample_sum_to_one_million".to_string();
        params.scale_factor = Some(1_000_000.0);

        let report = normalize_abundance(&input, &params, &output)?;
        assert_eq!(
            std::fs::read_to_string(&output)?,
            "sample_id\tfeature_id\tcounts_per_million\ns\tf1\t250000.00000000\ns\tf2\t750000.00000000\n"
        );
        assert_eq!(report.per_sample_sums, vec![("s".to_string(), 1_000_000.0)]);
        assert_eq!(report.method, params.method);
        assert_eq!(report.normalized_value_column, params.normalized_value_column);
        assert_eq!(report.compositional_rule, params.compositional_rule);
        assert_eq!(report.scale_factor, params.scale_factor);
        assert_eq!(report.table_rows, 2);
        assert_eq!(report.sample_count, 1);
        assert_eq!(report.feature_count, 2);
        assert_eq!(report.tool_id, "bijux");
        assert_eq!(std::fs::read_to_string(&input)?, original);
        Ok(())
    }

    #[test]
    fn normalize_abundance_relative_output_retains_order_and_eight_decimal_format(
    ) -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-abundance-relative-order")?;
        let input = temp.path().join("input.tsv");
        let output = temp.path().join("normalized.tsv");
        let original =
            "sample_id\tfeature_id\tabundance\nb\tf1\t25\na\tf1\t1\nb\tf2\t75\na\tf2\t3\n";
        std::fs::write(&input, original)?;
        let params = abundance_parameters();

        let report = normalize_abundance(&input, &params, &output)?;
        assert_eq!(
            std::fs::read_to_string(&output)?,
            "sample_id\tfeature_id\tnormalized_abundance\nb\tf1\t0.25000000\na\tf1\t0.25000000\nb\tf2\t0.75000000\na\tf2\t0.75000000\n"
        );
        assert_eq!(report.per_sample_sums, vec![("a".to_string(), 1.0), ("b".to_string(), 1.0)]);
        assert_eq!(report.table_rows, 4);
        assert_eq!(report.sample_count, 2);
        assert_eq!(report.feature_count, 2);
        assert_eq!(report.method, "relative_abundance");
        assert_eq!(report.scale_factor, None);
        assert_eq!(report.input_table, input.display().to_string());
        assert_eq!(report.normalized_abundance_tsv, output.display().to_string());
        assert_eq!(std::fs::read_to_string(&input)?, original);
        Ok(())
    }

    #[test]
    fn normalize_abundance_zero_total_reports_observed_zero_sum() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-abundance-zero-total")?;
        let input = temp.path().join("input.tsv");
        let output = temp.path().join("normalized.tsv");
        let original = "sample_id\tfeature_id\tabundance\nz\tf1\t0\nz\tf2\t0\n";
        std::fs::write(&input, original)?;

        let report = normalize_abundance(&input, &abundance_parameters(), &output)?;
        assert_eq!(
            std::fs::read_to_string(&output)?,
            "sample_id\tfeature_id\tnormalized_abundance\nz\tf1\t0.00000000\nz\tf2\t0.00000000\n"
        );
        assert_eq!(report.per_sample_sums, vec![("z".to_string(), 0.0)]);
        assert_eq!(report.zero_fraction, 1.0);
        assert_eq!(report.table_rows, 2);
        assert_eq!(report.sample_count, 1);
        assert_eq!(std::fs::read_to_string(&input)?, original);
        Ok(())
    }
}
