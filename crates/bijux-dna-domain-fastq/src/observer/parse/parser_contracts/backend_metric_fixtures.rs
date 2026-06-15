use super::*;

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
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/fastp.json"
    );
    let parsed = parse_fastp_metrics(raw)?;
    assert_eq!(parsed.schema_version, "bijux.fastp.metrics.v1");
    assert_eq!(parsed.passed_filter_reads, 960);
    assert_eq!(parsed.too_short_reads, 12);
    Ok(())
}

#[test]
fn parse_adapterremoval_metrics_fixture() -> Result<()> {
    let raw = include_str!(
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/adapterremoval.txt"
    );
    let parsed = parse_adapterremoval_metrics(raw)?;
    assert_eq!(parsed.schema_version, "bijux.adapterremoval.metrics.v1");
    assert_eq!(parsed.pairs_processed, 1000);
    assert_eq!(parsed.pairs_merged, 640);
    Ok(())
}

#[test]
fn parse_seqkit_tool_metrics_fixture() -> Result<()> {
    let raw = include_str!(
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/seqkit/default/seqkit_stats_v1.txt"
    );
    let parsed = parse_seqkit_tool_metrics(raw)?;
    assert_eq!(parsed.schema_version, "bijux.seqkit.metrics.v1");
    assert_eq!(parsed.reads, 1000);
    Ok(())
}

#[test]
fn parse_samtools_flagstat_fixture() -> Result<()> {
    let raw = include_str!(
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/samtools_flagstat.txt"
    );
    let parsed = parse_samtools_flagstat_metrics(raw)?;
    assert_eq!(parsed.schema_version, "bijux.samtools.flagstat.v1");
    assert_eq!(parsed.total_reads, 1000);
    assert_eq!(parsed.mapped_reads, 900);
    Ok(())
}

#[test]
fn parse_fastqc_summary_fixture() -> Result<()> {
    let raw = include_str!(
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/fastqc_summary.txt"
    );
    let parsed = parse_fastqc_summary_metrics(raw)?;
    assert_eq!(parsed.schema_version, "bijux.fastqc.metrics.v1");
    assert_eq!(parsed.total_sequences, 1000);
    assert_f64_eq(parsed.gc_percent, 42.0);
    Ok(())
}

#[test]
fn parse_multiqc_general_stats_fixture() -> Result<()> {
    let raw = include_str!(
        "../../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/multiqc_general_stats.json"
    );
    let parsed = parse_multiqc_general_stats_metrics(raw)?;
    assert_eq!(parsed.schema_version, "bijux.multiqc.metrics.v1");
    assert_eq!(parsed.sample_count, 2);
    assert_eq!(parsed.module_count, 2);
    Ok(())
}
