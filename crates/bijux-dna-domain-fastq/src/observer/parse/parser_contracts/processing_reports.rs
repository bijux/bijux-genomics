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
