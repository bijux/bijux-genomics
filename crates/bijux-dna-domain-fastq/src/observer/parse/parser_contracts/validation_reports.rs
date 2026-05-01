use super::*;

#[test]
fn parse_fastqvalidator_count_parses_fixture() -> Result<()> {
    let stdout =
            include_str!("../../../../../bijux-dna-stages-fastq/tests/fixtures/fastqvalidator/default/fastqvalidator_v1.txt");
    let count = parse_fastqvalidator_count(stdout)?;
    assert_eq!(count, 12345);
    Ok(())
}

#[test]
fn parse_fastqvalidator_count_rejects_missing_marker() {
    let stdout = "fastqvalidator output without total reads";
    assert!(parse_fastqvalidator_count(stdout).is_err());
}

#[test]
fn parse_validation_report_parses_governed_validate_json() -> Result<()> {
    let parsed = parse_validation_report(
        &serde_json::json!({
            "schema_version": VALIDATION_REPORT_SCHEMA_VERSION,
            "stage": "fastq.validate_reads",
            "stage_id": "fastq.validate_reads",
            "tool_id": "fastqvalidator",
            "validation_mode": "strict",
            "pair_sync_policy": "require_header_sync",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "validation_log_r1": "validation_r1.log",
            "validation_log_r2": "validation_r2.log",
            "validated_inputs": 2,
            "validated_reads_r1": 101,
            "validated_reads_r2": 100,
            "validated_pairs": 100,
            "status_r1": 0,
            "status_r2": 0,
            "pair_sync_checked": true,
            "pair_sync_pass": false,
            "pair_count_match": false,
            "failure_class": "pair_count_mismatch",
            "strict_pass": false,
            "exit_code": 96
        })
        .to_string(),
    )?;
    assert_eq!(parsed.failure_class, ValidateFailureClass::PairCountMismatch);
    assert_eq!(parsed.validated_reads_r2, Some(100));
    Ok(())
}

#[test]
fn parse_validation_report_accepts_structural_failure_taxonomy() -> Result<()> {
    let parsed = parse_validation_report(
        &serde_json::json!({
            "schema_version": VALIDATION_REPORT_SCHEMA_VERSION,
            "stage": "fastq.validate_reads",
            "stage_id": "fastq.validate_reads",
            "tool_id": "fastqvalidator",
            "validation_mode": "strict",
            "pair_sync_policy": "not_applicable",
            "input_r1": "reads.fastq.zst",
            "input_r2": null,
            "validation_log_r1": "validation_r1.log",
            "validation_log_r2": null,
            "validated_inputs": 1,
            "validated_reads_r1": 0,
            "validated_reads_r2": null,
            "validated_pairs": null,
            "status_r1": 0,
            "status_r2": 0,
            "pair_sync_checked": false,
            "pair_sync_pass": null,
            "pair_count_match": null,
            "failure_class": "unsupported_compression",
            "strict_pass": false,
            "exit_code": 90
        })
        .to_string(),
    )?;
    assert_eq!(parsed.failure_class, ValidateFailureClass::UnsupportedCompression);
    assert_eq!(parsed.validated_reads_r1, 0);
    Ok(())
}

#[test]
fn parse_validated_reads_manifest_parses_governed_lineage_json() -> Result<()> {
    let parsed = parse_validated_reads_manifest(
        &serde_json::json!({
            "schema_version": VALIDATED_READS_MANIFEST_SCHEMA_VERSION,
            "stage_id": "fastq.validate_reads",
            "tool_id": "seqtk",
            "validation_mode": "report_only",
            "pair_sync_policy": "skip_header_sync",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "validation_report": "validation.json",
            "paired_mode": "paired_end",
            "validated_stream_ids": ["reads_r1", "reads_r2"],
            "pair_sync_checked": false,
            "pair_sync_pass": null,
            "validated_pairs": 120
        })
        .to_string(),
    )?;
    assert_eq!(parsed.paired_mode, PairedMode::PairedEnd);
    assert_eq!(parsed.validated_stream_ids, vec!["reads_r1", "reads_r2"]);
    Ok(())
}

#[test]
fn parse_terminal_damage_report_parses_governed_json() -> Result<()> {
    let parsed = parse_terminal_damage_report(
        &serde_json::json!({
            "schema_version": TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
            "stage": "fastq.trim_terminal_damage",
            "stage_id": "fastq.trim_terminal_damage",
            "tool_id": id_catalog::TOOL_CUTADAPT,
            "paired_mode": "paired_end",
            "threads": 4,
            "damage_mode": "ancient",
            "execution_policy": "explicit_terminal_trim",
            "trim_5p_bases": 2,
            "trim_3p_bases": 1,
            "requested_trim_5p_bases": 2,
            "requested_trim_3p_bases": 1,
            "udg_classification": "non_udg",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "trimmed_R1.fastq.gz",
            "output_r2": "trimmed_R2.fastq.gz",
            "reads_in": 200,
            "reads_out": 198,
            "bases_in": 20000,
            "bases_out": 19100,
            "mean_q_before": 28.0,
            "mean_q_after": 28.5,
            "ct_ga_asymmetry_pre": 0.45,
            "ct_ga_asymmetry_post": 0.12,
            "ct_ga_asymmetry_pre_r1": 0.50,
            "ct_ga_asymmetry_post_r1": 0.15,
            "ct_ga_asymmetry_pre_r2": 0.40,
            "ct_ga_asymmetry_post_r2": 0.09,
            "terminal_base_composition_pre_r1": {"C": 80},
            "terminal_base_composition_post_r1": {"C": 30},
            "terminal_base_composition_pre_r2": {"G": 75},
            "terminal_base_composition_post_r2": {"G": 28},
            "raw_backend_report": "cutadapt.damage.json",
            "raw_backend_report_format": "cutadapt_json",
            "runtime_s": 12.4,
            "memory_mb": 256.0,
            "used_fallback": false,
            "backend_metrics": {"reads_profiled_r1": 200}
        })
        .to_string(),
    )?;
    assert_eq!(parsed.damage_mode, DamageMode::Ancient);
    assert_eq!(parsed.threads, 4);
    assert_eq!(parsed.execution_policy, TerminalDamageExecutionPolicy::ExplicitTerminalTrim);
    assert_eq!(parsed.raw_backend_report_format.as_deref(), Some("cutadapt_json"));
    assert_eq!(parsed.reads_in, Some(200));
    assert!(!parsed.used_fallback);
    Ok(())
}

#[test]
fn parse_trim_reads_report_parses_governed_json() -> Result<()> {
    let parsed = parse_trim_reads_report(
        r#"{
            "schema_version": "bijux.fastq.trim_reads.report.v2",
            "stage": "fastq.trim_reads",
            "stage_id": "fastq.trim_reads",
            "tool_id": "fastp",
            "paired_mode": "paired_end",
            "threads": 4,
            "trimming_backend": "fastp",
            "backend_mode": "enforced",
            "input_r1": "reads_R1.fastq.gz",
            "input_r2": "reads_R2.fastq.gz",
            "output_r1": "trimmed_R1.fastq.gz",
            "output_r2": "trimmed_R2.fastq.gz",
            "min_length": 30,
            "quality_cutoff": 20,
            "adapter_policy": "bank",
            "polyx_policy": "trim",
            "n_policy": "drop",
            "contaminant_policy": "none",
            "adapter_bank_id": "illumina",
            "adapter_bank_hash": "sha256:adapter",
            "adapter_preset": "default",
            "detected_adapter_source": "governed_pattern_scan",
            "adapter_overrides": {
                "enable": ["AGATCGGAAGAGC"],
                "disable": ["polyA"]
            },
            "prepared_adapter_bank": null,
            "polyx_bank_id": "polyx",
            "polyx_bank_hash": "sha256:polyx",
            "polyx_preset": "illumina_twocolor",
            "contaminant_bank_id": "contaminants",
            "contaminant_bank_hash": "sha256:contaminants",
            "contaminant_preset": "illumina_default",
            "reads_in": 100,
            "reads_out": 90,
            "bases_in": 1000,
            "bases_out": 820,
            "pairs_in": 50,
            "pairs_out": 45,
            "mean_q_before": 28.0,
            "mean_q_after": 31.0,
            "effective_trim_params": {
                "adapter_policy": "bank",
                "min_length": 30,
                "quality_cutoff": 20
            },
            "runtime_s": 8.4,
            "memory_mb": 128.0,
            "raw_backend_report": "trim.fastp.json",
            "raw_backend_report_format": "fastp_json"
        }"#,
    )?;

    assert_eq!(parsed.tool_id, id_catalog::TOOL_FASTP);
    assert_eq!(parsed.threads, 4);
    assert_eq!(parsed.paired_mode, PairedMode::PairedEnd);
    assert_eq!(parsed.adapter_policy, "bank");
    assert_eq!(
        parsed.adapter_overrides,
        Some(serde_json::json!({
            "enable": ["AGATCGGAAGAGC"],
            "disable": ["polyA"]
        }))
    );
    assert_eq!(parsed.raw_backend_report_format.as_deref(), Some("fastp_json"));
    Ok(())
}

#[test]
fn parse_report_qc_report_parses_governed_json() -> Result<()> {
    let parsed = parse_report_qc_report(
        &serde_json::json!({
            "schema_version": REPORT_QC_REPORT_SCHEMA_VERSION,
            "stage": "fastq.report_qc",
            "stage_id": "fastq.report_qc",
            "tool_id": "multiqc",
            "paired_mode": "paired_end",
            "aggregation_engine": "multiqc",
            "aggregation_scope": "governed_qc_artifacts",
            "reads_in": 200,
            "reads_out": 200,
            "bases_in": 20000,
            "bases_out": 20000,
            "pairs_in": 100,
            "pairs_out": 100,
            "mean_q": 31.0,
            "contamination_rate": 0.04,
            "adapter_content_max": 0.1,
            "adapter_content_mean": 0.03,
            "duplication_rate": 0.08,
            "n_rate": 0.001,
            "kmer_warning_count": 2,
            "overrepresented_sequence_count": 1,
            "multiqc_sample_count": 2,
            "multiqc_module_count": 5,
            "raw_fastqc_dir": "raw_fastqc",
            "trimmed_fastqc_dir": "trimmed_fastqc",
            "multiqc_report": "multiqc_report.html",
            "multiqc_data": "multiqc_data",
            "governed_qc_input_count": 3,
            "governed_qc_contributor_stage_ids": ["fastq.trim_reads"],
            "governed_qc_contributor_tool_ids": [id_catalog::TOOL_FASTP],
            "governed_qc_contributors": [{
                "contributor_id": "fastq.trim_reads.fastp",
                "stage_id": "fastq.trim_reads",
                "tool_id": id_catalog::TOOL_FASTP,
                "artifact_id": "report_json",
                "artifact_role": "report_json",
                "path": "trim/report.json"
            }],
            "governed_qc_lineage_hash": "lineage",
            "governed_qc_inputs_manifest": "governed_qc_inputs_manifest.json",
            "runtime_s": 3.0,
            "memory_mb": 128.0,
            "exit_code": 0
        })
        .to_string(),
    )?;
    assert_eq!(parsed.tool_id, "multiqc");
    assert_eq!(parsed.governed_qc_input_count, 3);
    assert_eq!(parsed.multiqc_module_count, Some(5));
    Ok(())
}
