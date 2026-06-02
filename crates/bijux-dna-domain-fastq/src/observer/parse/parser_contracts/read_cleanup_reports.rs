use super::*;

#[test]
fn parse_detect_duplicates_premerge_report_parses_governed_json() -> Result<()> {
    let parsed = parse_detect_duplicates_premerge_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.detect_duplicates_premerge.report.v1",
            "stage": "fastq.detect_duplicates_premerge",
            "stage_id": "fastq.detect_duplicates_premerge",
            "tool_id": "bijux",
            "paired_mode": "paired_end",
            "duplicate_detection_policy": "report_only",
            "measurement_scope": "premerge_sequence_signature",
            "modifies_reads": false,
            "advisory_only": true,
            "reads_in": 12,
            "duplicate_signal_reads": 4,
            "duplicate_signal_fraction": 0.3333333333333333,
            "compared_read_pairs": 6
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "bijux");
    assert_eq!(parsed.reads_in, 12);
    assert_eq!(parsed.duplicate_signal_reads, 4);
    assert_eq!(parsed.compared_read_pairs, Some(6));
    assert_f64_eq(parsed.duplicate_signal_fraction, 0.3333333333333333);
    Ok(())
}

#[test]
fn parse_deduplicate_report_parses_fixture() -> Result<()> {
    let raw =
            include_str!("../../../../../bijux-dna-stages-fastq/tests/fixtures/deduplicate/default/deduplicate_report_v1.json");
    let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
    assert_eq!(reads_in, 1000);
    assert_eq!(reads_out, 820);
    Ok(())
}

#[test]
fn parse_remove_duplicates_report_parses_governed_json() -> Result<()> {
    let parsed = parse_remove_duplicates_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.remove_duplicates.report.v2",
            "stage": "fastq.remove_duplicates",
            "stage_id": "fastq.remove_duplicates",
            "tool_id": "clumpify",
            "paired_mode": "single_end",
            "threads": 4,
            "dedup_mode": "optical_aware",
            "keep_order": false,
            "input_r1": "reads.fastq.gz",
            "input_r2": null,
            "output_r1": "dedup.fastq.gz",
            "output_r2": null,
            "reads_in": 100,
            "reads_out": 85,
            "reads_in_r2": null,
            "reads_out_r2": null,
            "pairs_in": null,
            "pairs_out": null,
            "pair_count_match": null,
            "duplicates_removed": 15,
            "dedup_rate": 0.15,
            "duplicate_classes_tsv": "duplicate_classes.tsv",
            "duplicate_provenance_json": "duplicate_provenance.json",
            "duplicate_classes": [
                {"class": "duplicate", "reads_removed": 11, "paired_mode": "single_end"},
                {"class": "optical_duplicate", "reads_removed": 4, "paired_mode": "single_end"}
            ],
            "raw_backend_report": "clumpify.log",
            "raw_backend_report_format": "clumpify_log",
            "runtime_s": 2.2,
            "memory_mb": 64.0
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "clumpify");
    assert_eq!(parsed.threads, 4);
    assert_eq!(parsed.duplicate_classes.len(), 2);
    assert_f64_eq(parsed.dedup_rate, 0.15);
    Ok(())
}

#[test]
fn parse_remove_duplicates_report_rejects_incomplete_governed_json() {
    let result = parse_remove_duplicates_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.remove_duplicates.report.v2",
            "stage": "fastq.remove_duplicates",
            "stage_id": "fastq.remove_duplicates",
            "tool_id": "clumpify",
            "paired_mode": "single_end",
            "dedup_mode": "optical_aware",
            "keep_order": false,
            "reads_in": 100,
            "reads_out": 85,
            "duplicates_removed": 15,
            "dedup_rate": 0.15
        })
        .to_string(),
    );
    let Err(error) = result else {
        panic!(
            "incomplete governed remove-duplicates reports must not fall back to legacy parsing"
        );
    };

    assert!(error.to_string().contains("parse remove duplicates report"));
}

#[test]
fn parse_remove_duplicates_provenance_parses_governed_json() -> Result<()> {
    let parsed = parse_remove_duplicates_provenance(
        &serde_json::json!({
            "schema_version": "bijux.fastq.remove_duplicates.provenance.v2",
            "stage_id": "fastq.remove_duplicates",
            "tool_id": "fastuniq",
            "paired_mode": "paired_end",
            "threads": 4,
            "dedup_mode": "exact",
            "keep_order": true,
            "duplicates_removed": 18,
            "dedup_rate": 0.09,
            "backend_log": "fastuniq.log",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "dedup_R1.fastq.gz",
            "output_r2": "dedup_R2.fastq.gz",
            "raw_backend_report": "fastuniq.log",
            "raw_backend_report_format": "fastuniq_log"
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "fastuniq");
    assert_eq!(parsed.duplicates_removed, 18);
    Ok(())
}

#[test]
fn parse_low_complexity_report_parses_fixture() -> Result<()> {
    let raw = include_str!(
            "../../../../../bijux-dna-stages-fastq/tests/fixtures/low_complexity/default/low_complexity_report_v1.json"
        );
    let removed = parse_low_complexity_report(raw)?;
    assert_eq!(removed, 137);
    Ok(())
}

#[test]
fn parse_deduplicate_report_parses_key_value_fixture() -> Result<()> {
    let raw = include_str!(
            "../../../../../bijux-dna-stages-fastq/tests/fixtures/stage_output_bank/default/fastq.remove_duplicates.fastuniq.txt"
        );
    let (reads_in, reads_out) = parse_deduplicate_report(raw)?;
    assert_eq!(reads_in, 1000);
    assert_eq!(reads_out, 820);
    Ok(())
}

#[test]
fn parse_duplicate_classes_tsv_parses_fixture_lines() -> Result<()> {
    let parsed = parse_duplicate_classes_tsv(
            "class\treads_removed\tpaired_mode\nduplicate\t180\tpaired_end\noptical_duplicate\t12\tpaired_end\n",
        )?;
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].class, "duplicate");
    assert_eq!(parsed[1].reads_removed, 12);
    Ok(())
}

#[test]
fn parse_remove_chimeras_report_parses_governed_contract() -> Result<()> {
    let parsed = parse_remove_chimeras_report(
            &serde_json::json!({
                "schema_version": "bijux.fastq.remove_chimeras.report.v2",
                "stage": "fastq.remove_chimeras",
                "stage_id": "fastq.remove_chimeras",
                "tool_id": "vsearch",
                "paired_mode": "single_end",
                "threads": 2,
                "method": "vsearch_uchime_denovo",
                "detection_scope": "denovo",
                "chimera_removed_definition": "reads flagged as de_novo chimeras are excluded from downstream abundance tables",
                "input_reads": "merged.fastq.gz",
                "output_reads": "nonchimeras.fastq.gz",
                "chimera_metrics_json": "chimera_metrics.json",
                "chimeras_fasta": "chimeras.fasta",
                "uchime_report_tsv": "uchime.tsv",
                "reads_in": 100,
                "reads_out": 92,
                "chimeras_removed": 8,
                "chimera_fraction": 0.08,
                "used_fallback": false,
                "raw_backend_report": "uchime.tsv",
                "raw_backend_report_format": "vsearch_uchime_tsv",
                "runtime_s": 1.7,
                "memory_mb": 32.0,
                "exit_code": 0,
                "backend_metrics": {
                    "parsed_records": 100,
                    "flagged_records": 8
                }
            })
            .to_string(),
        )?;
    assert_eq!(parsed.tool_id, "vsearch");
    assert_eq!(parsed.threads, 2);
    assert_eq!(parsed.chimera_fraction, Some(0.08));
    assert_eq!(parsed.uchime_report_tsv.as_deref(), Some("uchime.tsv"));
    Ok(())
}

#[test]
fn parse_remove_chimeras_report_accepts_legacy_metrics_payload() -> Result<()> {
    let parsed = parse_remove_chimeras_report(
        &serde_json::json!({
            "schema_version": "bijux.fastq.remove_chimeras.v2",
            "chimera_fraction": 0.12,
            "chimeras_removed": 12,
            "non_chimera_reads": 88,
            "tool": "vsearch",
            "used_fallback": true
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "vsearch");
    assert_eq!(parsed.chimeras_removed, Some(12));
    assert!(parsed.used_fallback);
    Ok(())
}
