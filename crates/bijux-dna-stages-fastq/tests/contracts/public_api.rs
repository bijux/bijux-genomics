use std::path::Path;

#[test]
fn public_api_docs_list_stable_root_exports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api =
        std::fs::read_to_string(root.join("docs/PUBLIC_API.md")).expect("read docs/PUBLIC_API.md");
    let readme = std::fs::read_to_string(root.join("README.md")).expect("read README.md");
    let stable_exports = [
        "FastqStagePlugin",
        "StagePlanJson",
        "RuntimeInterpretationLevel",
        "contract_stage_ids",
        "closed_execution_stage_ids",
        "implemented_stages",
        "observer_specialized_stage_ids",
        "observer_stage_ids",
        "observer_stage_tool_bindings",
        "runtime_interpretation_for_stage",
        "runtime_interpretation_for_stage_tool",
        "runtime_interpretation_stage_ids",
    ];

    for export in stable_exports {
        assert!(public_api.contains(export), "docs/PUBLIC_API.md must list `{export}`");
        assert!(readme.contains(export), "README.md must list `{export}`");
    }
}

#[test]
fn public_api_exports_remain_usable_from_crate_root() {
    let plugin = bijux_dna_stages_fastq::FastqStagePlugin;
    let plan_json: Option<bijux_dna_stages_fastq::StagePlanJson> = None;
    let level = bijux_dna_stages_fastq::RuntimeInterpretationLevel::GenericEnvelope;
    let _ = (plugin, plan_json, level);

    assert!(!bijux_dna_stages_fastq::contract_stage_ids().is_empty());
    assert!(!bijux_dna_stages_fastq::implemented_stages().is_empty());
    assert!(!bijux_dna_stages_fastq::closed_execution_stage_ids().is_empty());
    assert!(!bijux_dna_stages_fastq::observer_stage_ids().is_empty());
    assert!(!bijux_dna_stages_fastq::observer_specialized_stage_ids().is_empty());
    assert!(!bijux_dna_stages_fastq::observer_stage_tool_bindings().is_empty());
}

#[test]
fn public_modules_remain_available() {
    let contract_lookup = bijux_dna_stages_fastq::contracts::contract_for_stage;
    let validator_parser = bijux_dna_stages_fastq::observer::parse_fastqvalidator_count;
    let stage_specs = bijux_dna_stages_fastq::stage_specs::STAGES.len();
    let _ = (contract_lookup, validator_parser, stage_specs);
}

#[test]
fn observer_parser_exports_remain_available() {
    let _ = bijux_dna_stages_fastq::observer::parse_adapterremoval_metrics;
    let _ = bijux_dna_stages_fastq::observer::parse_fastp_metrics;
    let _ = bijux_dna_stages_fastq::observer::parse_fastqc_summary_metrics;
    let _ = bijux_dna_stages_fastq::observer::parse_multiqc_general_stats_metrics;
    let _ = bijux_dna_stages_fastq::observer::parse_samtools_flagstat_metrics;
    let _ = bijux_dna_stages_fastq::observer::parse_seqkit_tool_metrics;
    let _ = bijux_dna_stages_fastq::observer::parse_seqkit_stats;
    let _ = bijux_dna_stages_fastq::observer::parse_length_histogram;
    let _ = bijux_dna_stages_fastq::observer::parse_trim_reads_report;
    let _ = bijux_dna_stages_fastq::observer::parse_filter_reads_report;
    let _ = bijux_dna_stages_fastq::observer::parse_filter_low_complexity_report;
    let _ = bijux_dna_stages_fastq::observer::parse_merge_pairs_report;
    let _ = bijux_dna_stages_fastq::observer::parse_deduplicate_report;
    let _ = bijux_dna_stages_fastq::observer::parse_remove_duplicates_report;
    let _ = bijux_dna_stages_fastq::observer::parse_report_qc_report;
    let _ = bijux_dna_stages_fastq::observer::parse_screen_taxonomy_report;
}
