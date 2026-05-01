use super::*;

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
            "extraction_location": "read1_prefix",
            "read_name_transform": "append_to_header",
            "failed_extraction_policy": "refuse_stage",
            "grouping_policy": "pair_aware",
            "downstream_dedup_policy": "sequence_identity_recommended",
            "downstream_propagation": "header_and_report",
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
