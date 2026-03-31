use super::*;

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
