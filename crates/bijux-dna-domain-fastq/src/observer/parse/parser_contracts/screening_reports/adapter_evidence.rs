use super::*;

#[test]
fn parse_detect_adapters_report_round_trips_governed_payload() -> Result<()> {
    let parsed = parse_detect_adapters_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.detect_adapters.report.v2",
            "stage": "fastq.detect_adapters",
            "stage_id": "fastq.detect_adapters",
            "tool_id": "fastqc",
            "paired_mode": "paired_end",
            "threads": 4,
            "inspection_mode": "evidence_only",
            "report_only": true,
            "evidence_engine": "fastqc",
            "evidence_scope": "full_input",
            "evidence_format": "fastqc_summary",
            "evidence_artifact_id": "report_json",
            "detected_adapter_source": "normalized_fastqc_evidence",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "report_json": "adapter_report.json",
            "adapter_evidence_dir": "fastqc",
            "reads_in": 200_u64,
            "reads_out": 200_u64,
            "bases_in": 20_000_u64,
            "bases_out": 20_000_u64,
            "pairs_in": 100_u64,
            "pairs_out": 100_u64,
            "mean_q": 31.2,
            "candidate_adapter_count": 2_u64,
            "adapter_trimmed_fraction": 0.08,
            "adapter_content_max": 12.5,
            "adapter_content_mean": 3.2,
            "duplication_rate": 0.15,
            "n_rate": 0.001,
            "kmer_warning_count": 4_u64,
            "overrepresented_sequence_count": 3_u64,
            "runtime_s": 4.0,
            "memory_mb": 64.0,
            "exit_code": 0,
            "raw_backend_report": "fastqc/fastqc_data.txt",
            "raw_backend_report_format": "fastqc_data_txt"
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "fastqc");
    assert_eq!(parsed.candidate_adapter_count, 2);
    assert_eq!(parsed.threads, 4);
    Ok(())
}
