use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::artifacts::{
    OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1,
    PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
};
use crate::params::quality::stats::FastqOverrepresentedProfileParams;
use crate::params::PairedMode;

use super::fastq_io::read_fastq_records;

const OVERREPRESENTED_PAYLOAD_SCHEMA_VERSION: &str =
    "bijux.fastq.profile_overrepresented_sequences.v1";
const RAW_BACKEND_REPORT_FORMAT: &str = "bijux_profile_overrepresented_trace";

/// Profile exact overrepresented read sequences with a deterministic native implementation
/// suitable for governed local smoke execution.
///
/// # Errors
/// Returns an error when the effective parameters are incomplete, paired inputs are incoherent,
/// or the governed outputs cannot be written.
#[allow(clippy::too_many_arguments)]
pub fn profile_overrepresented_sequences(
    r1: &Path,
    r2: Option<&Path>,
    params: &FastqOverrepresentedProfileParams,
    output_tsv: &Path,
    output_json: &Path,
    report_json: &Path,
    raw_backend_report: Option<&Path>,
) -> Result<ProfileOverrepresentedReportV1> {
    let missing = params.missing_required_fields();
    if !missing.is_empty() {
        return Err(anyhow!(
            "fastq.profile_overrepresented_sequences missing required params: {}",
            missing.join(", ")
        ));
    }

    let observed_paired_mode =
        if r2.is_some() { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
    if params.paired_mode != observed_paired_mode {
        return Err(anyhow!(
            "fastq.profile_overrepresented_sequences paired_mode mismatch: params={}, inputs={}",
            paired_mode_literal(params.paired_mode),
            paired_mode_literal(observed_paired_mode)
        ));
    }

    let left = read_fastq_records(r1)?;
    let right = if let Some(path) = r2 {
        let records = read_fastq_records(path)?;
        if records.len() != left.len() {
            return Err(anyhow!(
                "fastq.profile_overrepresented_sequences refused incoherent paired input: R1 count {} != R2 count {}",
                left.len(),
                records.len()
            ));
        }
        records
    } else {
        Vec::new()
    };

    let mut counts = BTreeMap::<String, u64>::new();
    for record in &left {
        *counts.entry(record.sequence.to_ascii_uppercase()).or_insert(0) += 1;
    }
    for record in &right {
        *counts.entry(record.sequence.to_ascii_uppercase()).or_insert(0) += 1;
    }

    let total_sequences = (left.len() + right.len()) as u64;
    let rows = ranked_rows(&counts, total_sequences, params.top_k);
    let flagged_sequences = rows.iter().filter(|row| row.flag == "overrepresented").count() as u64;
    let top_fraction = rows.first().map_or(0.0, |row| row.fraction);

    ensure_parent_dir(output_tsv)?;
    ensure_parent_dir(output_json)?;
    ensure_parent_dir(report_json)?;
    if let Some(path) = raw_backend_report {
        ensure_parent_dir(path)?;
    }

    bijux_dna_infra::atomic_write_bytes(output_tsv, build_tsv(&rows).as_bytes())?;
    bijux_dna_infra::atomic_write_json(
        output_json,
        &serde_json::json!({
            "schema_version": OVERREPRESENTED_PAYLOAD_SCHEMA_VERSION,
            "top_k": params.top_k,
            "sequence_count": rows.len() as u64,
            "flagged_sequences": flagged_sequences,
            "top_fraction": top_fraction,
            "rows": rows,
        }),
    )?;

    if let Some(path) = raw_backend_report {
        std::fs::write(
            path,
            format!(
                "paired_mode\t{}\nthreads\t{}\ntop_k\t{}\ninput_sequences\t{}\nunique_sequences\t{}\nflagged_sequences\t{}\ntop_fraction\t{:.6}\n",
                paired_mode_literal(params.paired_mode),
                params.threads,
                params.top_k,
                total_sequences,
                counts.len(),
                flagged_sequences,
                top_fraction
            ),
        )?;
    }

    let report = ProfileOverrepresentedReportV1 {
        schema_version: PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.profile_overrepresented_sequences".to_string(),
        stage_id: "fastq.profile_overrepresented_sequences".to_string(),
        tool_id: "bijux".to_string(),
        paired_mode: params.paired_mode,
        threads: params.threads,
        top_k: params.top_k,
        input_r1: r1.display().to_string(),
        input_r2: r2.map(|path| path.display().to_string()),
        overrepresented_sequences_tsv: output_tsv.display().to_string(),
        overrepresented_sequences_json: output_json.display().to_string(),
        report_json: report_json.display().to_string(),
        sequence_count: rows.len() as u64,
        flagged_sequences,
        top_fraction,
        rows,
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(0),
        raw_backend_report: raw_backend_report.map(|path| path.display().to_string()),
        raw_backend_report_format: raw_backend_report
            .map(|_| RAW_BACKEND_REPORT_FORMAT.to_string()),
    };
    bijux_dna_infra::atomic_write_json(report_json, &report)?;
    Ok(report)
}

fn ranked_rows(
    counts: &BTreeMap<String, u64>,
    total_sequences: u64,
    top_k: u32,
) -> Vec<OverrepresentedSequenceRowV1> {
    let mut ranked =
        counts.iter().map(|(sequence, count)| (sequence.clone(), *count)).collect::<Vec<_>>();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    ranked
        .into_iter()
        .take(top_k as usize)
        .map(|(sequence, count)| {
            let fraction =
                if total_sequences == 0 { 0.0 } else { count as f64 / total_sequences as f64 };
            OverrepresentedSequenceRowV1 {
                sequence,
                count,
                fraction,
                flag: if count > 1 {
                    "overrepresented".to_string()
                } else {
                    "background".to_string()
                },
            }
        })
        .collect()
}

fn build_tsv(rows: &[OverrepresentedSequenceRowV1]) -> String {
    let mut tsv = String::from("sequence\tcount\tfraction\tflag\n");
    for row in rows {
        tsv.push_str(&row.sequence);
        tsv.push('\t');
        tsv.push_str(&row.count.to_string());
        tsv.push('\t');
        tsv.push_str(&format!("{:.6}", row.fraction));
        tsv.push('\t');
        tsv.push_str(&row.flag);
        tsv.push('\n');
    }
    tsv
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn paired_mode_literal(paired_mode: PairedMode) -> &'static str {
    match paired_mode {
        PairedMode::SingleEnd => "single_end",
        PairedMode::PairedEnd => "paired_end",
        PairedMode::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::profile_overrepresented_sequences;
    use crate::params::quality::stats::{
        FastqOverrepresentedProfileParams, OVERREPRESENTED_PROFILE_SCHEMA_VERSION,
    };
    use crate::params::PairedMode;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, sequence, quality) in records {
            payload.push_str(&format!("@{header}\n{sequence}\n+\n{quality}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn profile_overrepresented_sequences_counts_known_repeat() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-profile-overrepresented")?;
        let reads = temp.path().join("reads.fastq");
        write_fastq(
            &reads,
            &[
                ("repeat_a_1", "ACGTACGTACGT", "IIIIIIIIIIII"),
                ("repeat_a_2", "ACGTACGTACGT", "JJJJJJJJJJJJ"),
                ("repeat_a_3", "ACGTACGTACGT", "HHHHHHHHHHHH"),
                ("unique_c_1", "TTTTCCCCAAAA", "FFFFFFFFFFFF"),
                ("unique_g_1", "GGGGAAAATTTT", "EEEEEEEEEEEE"),
            ],
        )?;

        let report = profile_overrepresented_sequences(
            &reads,
            None,
            &FastqOverrepresentedProfileParams {
                schema_version: OVERREPRESENTED_PROFILE_SCHEMA_VERSION.to_string(),
                paired_mode: PairedMode::SingleEnd,
                threads: 1,
                top_k: 5,
            },
            &temp.path().join("overrepresented_sequences.tsv"),
            &temp.path().join("overrepresented_sequences.json"),
            &temp.path().join("overrepresented_report.json"),
            Some(&temp.path().join("raw_backend_report.txt")),
        )?;

        assert_eq!(report.tool_id, "bijux");
        assert_eq!(report.sequence_count, 3);
        assert_eq!(report.flagged_sequences, 1);
        assert!((report.top_fraction - 0.6).abs() < f64::EPSILON);
        assert_eq!(report.rows[0].sequence, "ACGTACGTACGT");
        assert_eq!(report.rows[0].count, 3);
        assert_eq!(report.rows[0].flag, "overrepresented");

        let tsv = std::fs::read_to_string(temp.path().join("overrepresented_sequences.tsv"))?;
        assert!(tsv.contains("ACGTACGTACGT\t3\t0.600000\toverrepresented"));
        Ok(())
    }
}
