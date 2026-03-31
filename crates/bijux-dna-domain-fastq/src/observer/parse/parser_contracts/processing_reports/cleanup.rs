use super::*;

#[test]
fn parse_low_complexity_report_parses_key_value_fixture() -> Result<()> {
    let raw = include_str!(
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/stage_output_bank/default/fastq.filter_low_complexity.bbduk.txt"
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
