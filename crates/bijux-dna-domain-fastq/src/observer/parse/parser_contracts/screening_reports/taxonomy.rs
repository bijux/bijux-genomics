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
            "interpretation_boundary": "screening_only",
            "truth_conditions": [],
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
fn parse_screen_summary_tsv_extracts_label_percent_pairs() -> Result<()> {
    let parsed = parse_screen_summary_tsv(
        "# taxonomic summary\nunclassified\t123\t23.0%\nbacteria\t410\t77.0%\n",
    )?;
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].label, "unclassified");
    assert_f64_eq(parsed[1].percent, 77.0);
    Ok(())
}
