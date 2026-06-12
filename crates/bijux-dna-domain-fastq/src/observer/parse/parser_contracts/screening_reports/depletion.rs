use super::*;

#[test]
fn parse_deplete_rrna_report_round_trips_governed_payload() -> Result<()> {
    let parsed = parse_deplete_rrna_report(
        &serde_json::json!({
            "schema_version": DEPLETE_RRNA_REPORT_SCHEMA_VERSION,
            "stage": "fastq.deplete_rrna",
            "stage_id": "fastq.deplete_rrna",
            "tool_id": "sortmerna",
            "paired_mode": "paired_end",
            "threads": 6,
            "rrna_db": "/refs/silva",
            "database_artifact_id": "silva_nr99",
            "database_build_id": "2026.03",
            "database_digest": "sha256:silva",
            "screening_engine": "sortmerna",
            "report_format": "summary_tsv_and_json",
            "emit_removed_reads": false,
            "min_identity": 0.95,
            "retained_read_role": "rrna_filtered_reads",
            "rejected_read_role": "removed_rrna_reads",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "rrna_filtered_R1.fastq.gz",
            "output_r2": "rrna_filtered_R2.fastq.gz",
            "removed_reads_r1": "removed_rrna_R1.fastq.gz",
            "removed_reads_r2": "removed_rrna_R2.fastq.gz",
            "rrna_report_tsv": "rrna_report.tsv",
            "rrna_report_json": "rrna_report.json",
            "reads_in": 200,
            "reads_out": 150,
            "reads_removed": 50,
            "bases_in": 20000,
            "bases_out": 15000,
            "bases_removed": 5000,
            "pairs_in": 100,
            "pairs_out": 75,
            "rrna_fraction_removed": 0.25,
            "runtime_s": 12.3,
            "memory_mb": 256.0,
            "exit_code": 0,
            "raw_backend_report": "sortmerna.log",
            "raw_backend_report_format": "sortmerna_log",
            "backend_metrics": {
                "reads_removed": 50
            }
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "sortmerna");
    assert_eq!(parsed.database_artifact_id, "silva_nr99");
    assert_eq!(parsed.database_digest.as_deref(), Some("sha256:silva"));
    assert_eq!(parsed.retained_read_role, "rrna_filtered_reads");
    assert_eq!(parsed.removed_reads_r1, "removed_rrna_R1.fastq.gz");
    assert_eq!(parsed.reads_removed, 50);
    Ok(())
}

#[test]
fn parse_deplete_rrna_report_accepts_legacy_payload() -> Result<()> {
    let parsed = parse_deplete_rrna_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.deplete_rrna.report.v1",
            "stage_id": "fastq.deplete_rrna",
            "tool_id": "sortmerna",
            "rrna_fraction_removed": 0.4,
            "reads_in": 100,
            "reads_out": 60,
            "bases_in": 1000,
            "bases_out": 600,
            "runtime_s": 4.0,
            "memory_mb": 32.0
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "sortmerna");
    assert_eq!(parsed.reads_removed, 40);
    assert_eq!(parsed.retained_read_role, "rrna_filtered_reads");
    assert_eq!(parsed.rejected_read_role, "removed_rrna_reads");
    assert_f64_eq(parsed.rrna_fraction_removed, 0.4);
    Ok(())
}

#[test]
fn parse_deplete_reference_contaminants_report_round_trips_governed_payload() -> Result<()> {
    let parsed = parse_deplete_reference_contaminants_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.deplete_reference_contaminants.report.v2",
            "stage": "fastq.deplete_reference_contaminants",
            "stage_id": "fastq.deplete_reference_contaminants",
            "tool_id": "bowtie2",
            "paired_mode": "paired_end",
            "threads": 6,
            "reference_catalog_id": "contaminant_reference",
            "contaminant_reference": "phix_and_spikeins",
            "reference_index_artifact_id": "reference_index",
            "reference_index_backend": "bowtie2_build",
            "reference_build_id": "2026.03",
            "reference_digest": "sha256:contaminant",
            "match_threshold": 0.95,
            "retained_read_role": "contaminant_screened_reads",
            "rejected_read_role": "removed_contaminant_reads",
            "retain_unmapped_pairs": true,
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "contaminant_screened_R1.fastq.gz",
            "output_r2": "contaminant_screened_R2.fastq.gz",
            "removed_reads_r1": "removed_contaminant_R1.fastq.gz",
            "removed_reads_r2": "removed_contaminant_R2.fastq.gz",
            "report_json": "contaminant_screen_report.json",
            "reads_in": 200,
            "reads_out": 160,
            "reads_removed": 40,
            "bases_in": 20000,
            "bases_out": 15600,
            "bases_removed": 4400,
            "pairs_in": 100,
            "pairs_out": 80,
            "contaminant_fraction_removed": 0.2,
            "runtime_s": 9.8,
            "memory_mb": 512.0,
            "exit_code": 0,
            "raw_backend_report": "bowtie2.contaminant.metrics.txt",
            "raw_backend_report_format": "bowtie2_met_file",
            "backend_metrics": {
                "reads_removed": 40
            }
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "bowtie2");
    assert_eq!(parsed.reads_removed, 40);
    assert_eq!(parsed.reference_index_artifact_id, "reference_index");
    assert_eq!(parsed.match_threshold, Some(0.95));
    assert_eq!(parsed.retained_read_role, "contaminant_screened_reads");
    assert_eq!(parsed.removed_reads_r1, "removed_contaminant_R1.fastq.gz");
    assert_eq!(parsed.raw_backend_report_format.as_deref(), Some("bowtie2_met_file"));
    Ok(())
}

#[test]
fn parse_deplete_reference_contaminants_report_accepts_legacy_payload() -> Result<()> {
    let parsed = parse_deplete_reference_contaminants_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.deplete_reference_contaminants.report.v1",
            "stage_id": "fastq.deplete_reference_contaminants",
            "tool_id": "bowtie2",
            "contaminant_fraction_removed": 0.35,
            "reads_in": 100,
            "reads_out": 65,
            "bases_in": 1000,
            "bases_out": 650,
            "runtime_s": 4.0,
            "memory_mb": 32.0
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "bowtie2");
    assert_eq!(parsed.reads_removed, 35);
    assert_eq!(parsed.reference_index_artifact_id, "reference_index");
    assert_eq!(parsed.retained_read_role, "contaminant_screened_reads");
    assert_eq!(parsed.rejected_read_role, "removed_contaminant_reads");
    assert_f64_eq(parsed.contaminant_fraction_removed, 0.35);
    Ok(())
}

#[test]
fn parse_deplete_host_report_round_trips_governed_payload() -> Result<()> {
    let parsed = parse_deplete_host_report(
        r#"{
                "schema_version": "bijux.fastq.deplete_host.report.v2",
                "stage": "fastq.deplete_host",
                "stage_id": "fastq.deplete_host",
                "tool_id": "bowtie2",
                "paired_mode": "paired_end",
                "threads": 6,
                "reference_scope": "host",
                "reference_catalog_id": "host_reference",
                "reference_index_artifact_id": "reference_index",
                "reference_index_backend": "bowtie2_build",
                "reference_build_id": "2026.03",
                "reference_digest": "sha256:host",
                "masking_policy": "unmasked",
                "decoy_policy": "none",
                "decoy_catalog_id": null,
                "identity_threshold": 0.95,
                "retained_read_policy": "keep_non_host_reads",
                "emit_removed_reads": true,
                "report_format": "bowtie2_metrics_file",
                "retain_unmapped_pairs": true,
                "input_r1": "reads_R1.fastq.gz",
                "input_r2": "reads_R2.fastq.gz",
                "output_r1": "host_depleted_R1.fastq.gz",
                "output_r2": "host_depleted_R2.fastq.gz",
                "removed_host_r1": "removed_host_R1.fastq.gz",
                "removed_host_r2": "removed_host_R2.fastq.gz",
                "report_json": "host_depletion_report.json",
                "reads_in": 200,
                "reads_out": 150,
                "reads_removed": 50,
                "bases_in": 20000,
                "bases_out": 15000,
                "bases_removed": 5000,
                "pairs_in": 100,
                "pairs_out": 75,
                "host_fraction_removed": 0.25,
                "runtime_s": 10.5,
                "memory_mb": 512.0,
                "exit_code": 0,
                "raw_backend_report": "bowtie2.host.metrics.txt",
                "raw_backend_report_format": "bowtie2_met_file",
                "backend_metrics": {
                    "reads_removed": 50
                }
            }"#,
    )?;
    assert_eq!(parsed.tool_id, "bowtie2");
    assert_eq!(parsed.reads_removed, 50);
    assert_eq!(parsed.raw_backend_report_format.as_deref(), Some("bowtie2_met_file"));
    Ok(())
}

#[test]
fn parse_deplete_host_report_accepts_legacy_payload() -> Result<()> {
    let parsed = parse_deplete_host_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.deplete_host.report.v1",
            "stage_id": "fastq.deplete_host",
            "tool_id": "bowtie2",
            "host_fraction_removed": 0.4,
            "reads_in": 100,
            "reads_out": 60,
            "bases_in": 1000,
            "bases_out": 600,
            "runtime_s": 4.0,
            "memory_mb": 32.0
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "bowtie2");
    assert_eq!(parsed.reads_removed, 40);
    assert_f64_eq(parsed.host_fraction_removed, 0.4);
    Ok(())
}
