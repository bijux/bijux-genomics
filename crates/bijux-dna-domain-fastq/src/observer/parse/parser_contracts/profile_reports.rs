use super::*;

#[test]
fn parse_seqkit_stats_parses_fixture() -> Result<()> {
    let stdout = include_str!(
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/seqkit/default/seqkit_stats_v1.txt"
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
                "length_histogram_source": "seqkit_fx2tab",
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
            "min_read_length": 90,
            "mean_read_length": 101.5,
            "median_read_length": 100.0,
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
    assert_eq!(parsed.min_read_length, 90);
    assert_eq!(parsed.median_read_length, 100.0);
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
    assert_eq!(parsed.min_read_length, 100);
    assert_eq!(parsed.median_read_length, 100.0);
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
