use super::tool_metrics::{
    parse_adapterremoval_metrics, parse_fastp_metrics, parse_fastqc_summary_metrics,
    parse_multiqc_general_stats_metrics, parse_samtools_flagstat_metrics,
    parse_seqkit_tool_metrics,
};
use super::{
    parse_bbduk_reads_removed, parse_correct_errors_report, parse_deduplicate_report,
    parse_deplete_host_report, parse_deplete_reference_contaminants_report,
    parse_deplete_rrna_report, parse_detect_adapters_report, parse_duplicate_classes_tsv,
    parse_extract_umis_report, parse_fastqvalidator_count, parse_filter_low_complexity_report,
    parse_filter_reads_report, parse_index_reference_report, parse_infer_asvs_report,
    parse_length_histogram, parse_low_complexity_report, parse_profile_overrepresented_report,
    parse_profile_read_lengths_report, parse_profile_reads_report, parse_remove_chimeras_report,
    parse_remove_duplicates_provenance, parse_remove_duplicates_report, parse_report_qc_report,
    parse_screen_summary_tsv, parse_screen_taxonomy_report, parse_seqkit_stats,
    parse_terminal_damage_report, parse_trim_reads_report, parse_validated_reads_manifest,
    parse_validation_report,
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

mod read_cleanup_reports;
mod screening_reports;
mod validation_reports;

fn assert_f64_eq(left: f64, right: f64) {
    assert!((left - right).abs() < f64::EPSILON);
}

#[test]
fn parse_seqkit_stats_parses_fixture() -> Result<()> {
    let stdout = include_str!(
        "../../../../bijux-dna-stages-fastq/tests/fixtures/seqkit/default/seqkit_stats_v1.txt"
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
            "../../../../bijux-dna-stages-fastq/tests/fixtures/stage_output_bank/default/fastq.filter_low_complexity.bbduk.txt"
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
        "../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/fastp.json"
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
        "../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/adapterremoval.txt"
    );
    let parsed = parse_adapterremoval_metrics(raw);
    assert_eq!(parsed.schema_version, "bijux.adapterremoval.metrics.v1");
    assert_eq!(parsed.pairs_processed, 1000);
    assert_eq!(parsed.pairs_merged, 640);
}

#[test]
fn parse_seqkit_tool_metrics_fixture() -> Result<()> {
    let raw = include_str!(
        "../../../../bijux-dna-stages-fastq/tests/fixtures/seqkit/default/seqkit_stats_v1.txt"
    );
    let parsed = parse_seqkit_tool_metrics(raw)?;
    assert_eq!(parsed.schema_version, "bijux.seqkit.metrics.v1");
    assert_eq!(parsed.reads, 1000);
    Ok(())
}

#[test]
fn parse_samtools_flagstat_fixture() {
    let raw = include_str!(
            "../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/samtools_flagstat.txt"
        );
    let parsed = parse_samtools_flagstat_metrics(raw);
    assert_eq!(parsed.schema_version, "bijux.samtools.flagstat.v1");
    assert_eq!(parsed.total_reads, 1000);
    assert_eq!(parsed.mapped_reads, 900);
}

#[test]
fn parse_fastqc_summary_fixture() {
    let raw = include_str!(
        "../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/fastqc_summary.txt"
    );
    let parsed = parse_fastqc_summary_metrics(raw);
    assert_eq!(parsed.schema_version, "bijux.fastqc.metrics.v1");
    assert_eq!(parsed.total_sequences, 1000);
    assert_f64_eq(parsed.gc_percent, 42.0);
}

#[test]
fn parse_multiqc_general_stats_fixture() -> Result<()> {
    let raw =
            include_str!("../../../../bijux-dna-stages-fastq/tests/fixtures/tool_metrics/default/multiqc_general_stats.json");
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
