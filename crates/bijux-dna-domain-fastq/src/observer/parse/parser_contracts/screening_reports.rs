use super::*;

#[test]
fn parse_screen_taxonomy_report_parses_governed_json() -> Result<()> {
    let parsed = parse_screen_taxonomy_report(
        &serde_json::json!({
            "schema_version": SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
            "stage": "fastq.screen_taxonomy",
            "stage_id": "fastq.screen_taxonomy",
            "tool_id": id_catalog::TOOL_KRAKEN2,
            "paired_mode": "paired_end",
            "threads": 8,
            "classifier": id_catalog::TOOL_KRAKEN2,
            "report_format": "kraken_report",
            "assignment_format": "kraken_assignments",
            "database_catalog_id": "taxonomy_reference",
            "database_artifact_id": "taxonomy_db",
            "database_build_id": "build-2026-03",
            "database_digest": "sha256:taxonomy",
            "database_namespace": "read_screening",
            "database_scope": "read_screening",
            "minimum_confidence": 0.1,
            "emit_unclassified": true,
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "screen_report_tsv": "kraken2.report.tsv",
            "classification_report_json": "kraken2.classifications.json",
            "reads_in": 200,
            "reads_out": 200,
            "bases_in": 20000,
            "bases_out": 20000,
            "pairs_in": 100,
            "pairs_out": 100,
            "contamination_rate": 0.23,
            "classified_fraction": 0.77,
            "unclassified_fraction": 0.23,
            "summary_entries": [
                {"label": "unclassified", "percent": 23.0},
                {"label": "bacteria", "percent": 77.0}
            ],
            "top_taxa": [
                {"label": "bacteria", "percent": 77.0}
            ],
            "runtime_s": 12.5,
            "memory_mb": 512.0
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, id_catalog::TOOL_KRAKEN2);
    assert_eq!(parsed.threads, 8);
    assert_eq!(parsed.top_taxa.len(), 1);
    assert_eq!(parsed.top_taxa[0].label, "bacteria");
    Ok(())
}

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
            "screening_engine": "sortmerna",
            "report_format": "summary_tsv_and_json",
            "emit_removed_reads": false,
            "min_identity": 0.95,
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "rrna_filtered_R1.fastq.gz",
            "output_r2": "rrna_filtered_R2.fastq.gz",
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
            "index_artifact": "reference_index",
            "reference_index_backend": "bowtie2_build",
            "reference_build_id": "2026.03",
            "reference_digest": "sha256:contaminant",
            "retain_unmapped_pairs": true,
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "contaminant_screened_R1.fastq.gz",
            "output_r2": "contaminant_screened_R2.fastq.gz",
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
    assert_eq!(
        parsed.raw_backend_report_format.as_deref(),
        Some("bowtie2_met_file")
    );
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
    assert_eq!(
        parsed.raw_backend_report_format.as_deref(),
        Some("bowtie2_met_file")
    );
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

#[test]
fn parse_screen_summary_tsv_extracts_label_percent_pairs() -> Result<()> {
    let parsed = parse_screen_summary_tsv(
        "# taxonomic summary\nunclassified\t123\t23.0%\nbacteria\t410\t77.0%\n",
    )?;
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].label, "unclassified");
    assert_f64_eq(parsed[1].percent, 77.0);
    Ok(())
}
