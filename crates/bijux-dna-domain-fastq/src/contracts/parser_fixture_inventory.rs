use std::sync::OnceLock;

use crate::observer::observer_specialization_contracts;
use crate::{
    CLUSTER_OTUS_REPORT_SCHEMA_VERSION, CORRECT_ERRORS_REPORT_SCHEMA_VERSION,
    DEPLETE_HOST_REPORT_SCHEMA_VERSION, DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION,
    DEPLETE_RRNA_REPORT_SCHEMA_VERSION, DETECT_ADAPTERS_REPORT_SCHEMA_VERSION,
    DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION,
    ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION, EXTRACT_UMIS_REPORT_SCHEMA_VERSION,
    FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION, INDEX_REFERENCE_REPORT_SCHEMA_VERSION,
    INFER_ASVS_REPORT_SCHEMA_VERSION, MERGE_PAIRS_REPORT_SCHEMA_VERSION,
    NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION, NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION,
    PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION, PROFILE_READS_REPORT_SCHEMA_VERSION,
    PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION, REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
    REPORT_QC_REPORT_SCHEMA_VERSION, SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
    TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION, TRIM_POLYG_REPORT_SCHEMA_VERSION,
    TRIM_READS_REPORT_SCHEMA_VERSION, VALIDATION_REPORT_SCHEMA_VERSION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FastqParserFixtureBinding {
    pub tool_id: &'static str,
    pub stage_id: &'static str,
    pub parser_id: &'static str,
    pub parser_schema_id: &'static str,
    pub fixture_case_id: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FastqParserFixtureCase {
    pub fixture_case_id: &'static str,
    pub stage_id: &'static str,
    pub semantic_surface: &'static str,
    pub canonical_tool_id: &'static str,
    pub raw_fixture: &'static str,
}

#[must_use]
pub fn fastq_parser_fixture_bindings() -> &'static [FastqParserFixtureBinding] {
    fixture_bindings().as_slice()
}

#[must_use]
pub fn fastq_parser_fixture_cases() -> &'static [FastqParserFixtureCase] {
    FASTQ_PARSER_FIXTURE_CASES
}

#[must_use]
pub fn find_fastq_parser_fixture_binding(
    stage_id: &str,
    tool_id: &str,
) -> Option<FastqParserFixtureBinding> {
    fastq_parser_fixture_bindings()
        .iter()
        .copied()
        .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
}

#[must_use]
pub fn find_fastq_parser_fixture_case(fixture_case_id: &str) -> Option<FastqParserFixtureCase> {
    fastq_parser_fixture_cases().iter().copied().find(|row| row.fixture_case_id == fixture_case_id)
}

fn fixture_bindings() -> &'static Vec<FastqParserFixtureBinding> {
    static BINDINGS: OnceLock<Vec<FastqParserFixtureBinding>> = OnceLock::new();
    BINDINGS.get_or_init(|| {
        let mut rows = observer_specialization_contracts()
            .iter()
            .map(|contract| {
                binding_for_stage_surface(
                    contract.stage_id,
                    contract.tool_id,
                    contract.semantic_surface,
                )
            })
            .collect::<Vec<_>>();
        rows.push(binding_for_stage_surface(
            "fastq.detect_duplicates_premerge",
            "bijux_dna",
            "report_json",
        ));
        rows.push(binding_for_stage_surface(
            "fastq.estimate_library_complexity_prealign",
            "bijux_dna",
            "report_json",
        ));
        rows.sort_by(|left, right| {
            left.stage_id.cmp(right.stage_id).then_with(|| left.tool_id.cmp(right.tool_id))
        });
        rows
    })
}

fn binding_for_stage_surface(
    stage_id: &'static str,
    tool_id: &'static str,
    semantic_surface: &'static str,
) -> FastqParserFixtureBinding {
    let (parser_id, parser_schema_id, fixture_case_id) = match (stage_id, semantic_surface) {
        ("fastq.cluster_otus", "report_json") => (
            "parse_cluster_otus_report",
            CLUSTER_OTUS_REPORT_SCHEMA_VERSION,
            "fastq.cluster_otus.report_json",
        ),
        ("fastq.correct_errors", "report_json") => (
            "parse_correct_errors_report",
            CORRECT_ERRORS_REPORT_SCHEMA_VERSION,
            "fastq.correct_errors.report_json",
        ),
        ("fastq.deplete_host", "host_depletion_report_json") => (
            "parse_deplete_host_report",
            DEPLETE_HOST_REPORT_SCHEMA_VERSION,
            "fastq.deplete_host.host_depletion_report_json",
        ),
        ("fastq.deplete_reference_contaminants", "contaminant_screen_report_json") => (
            "parse_deplete_reference_contaminants_report",
            DEPLETE_REFERENCE_CONTAMINANTS_REPORT_SCHEMA_VERSION,
            "fastq.deplete_reference_contaminants.contaminant_screen_report_json",
        ),
        ("fastq.deplete_rrna", "rrna_report_json") => (
            "parse_deplete_rrna_report",
            DEPLETE_RRNA_REPORT_SCHEMA_VERSION,
            "fastq.deplete_rrna.rrna_report_json",
        ),
        ("fastq.detect_adapters", "report_json") => (
            "parse_detect_adapters_report",
            DETECT_ADAPTERS_REPORT_SCHEMA_VERSION,
            "fastq.detect_adapters.report_json",
        ),
        ("fastq.detect_duplicates_premerge", "report_json") => (
            "parse_detect_duplicates_premerge_report",
            DETECT_DUPLICATES_PREMERGE_REPORT_SCHEMA_VERSION,
            "fastq.detect_duplicates_premerge.report_json",
        ),
        ("fastq.estimate_library_complexity_prealign", "report_json") => (
            "parse_estimate_library_complexity_prealign_report",
            ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN_REPORT_SCHEMA_VERSION,
            "fastq.estimate_library_complexity_prealign.report_json",
        ),
        ("fastq.extract_umis", "report_json") => (
            "parse_extract_umis_report",
            EXTRACT_UMIS_REPORT_SCHEMA_VERSION,
            "fastq.extract_umis.report_json",
        ),
        ("fastq.filter_low_complexity", "filter_report_json") => (
            "parse_filter_low_complexity_report",
            FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION,
            "fastq.filter_low_complexity.filter_report_json",
        ),
        ("fastq.filter_reads", "report_json") => (
            "parse_filter_reads_report",
            "bijux.fastq.filter_reads.report.v3",
            "fastq.filter_reads.report_json",
        ),
        ("fastq.index_reference", "report_json") => (
            "parse_index_reference_report",
            INDEX_REFERENCE_REPORT_SCHEMA_VERSION,
            "fastq.index_reference.report_json",
        ),
        ("fastq.infer_asvs", "report_json") => (
            "parse_infer_asvs_report",
            INFER_ASVS_REPORT_SCHEMA_VERSION,
            "fastq.infer_asvs.report_json",
        ),
        ("fastq.merge_pairs", "report_json") => (
            "parse_merge_pairs_report",
            MERGE_PAIRS_REPORT_SCHEMA_VERSION,
            "fastq.merge_pairs.report_json",
        ),
        ("fastq.normalize_abundance", "report_json") => (
            "parse_normalize_abundance_report",
            NORMALIZE_ABUNDANCE_REPORT_SCHEMA_VERSION,
            "fastq.normalize_abundance.report_json",
        ),
        ("fastq.normalize_primers", "report_json") => (
            "parse_normalize_primers_report",
            NORMALIZE_PRIMERS_REPORT_SCHEMA_VERSION,
            "fastq.normalize_primers.report_json",
        ),
        ("fastq.profile_overrepresented_sequences", "report_json") => (
            "parse_profile_overrepresented_report",
            PROFILE_OVERREPRESENTED_REPORT_SCHEMA_VERSION,
            "fastq.profile_overrepresented_sequences.report_json",
        ),
        ("fastq.profile_read_lengths", "report_json") => (
            "parse_profile_read_lengths_report",
            PROFILE_READ_LENGTHS_REPORT_SCHEMA_VERSION,
            "fastq.profile_read_lengths.report_json",
        ),
        ("fastq.profile_reads", "qc_json") => (
            "parse_profile_reads_report",
            PROFILE_READS_REPORT_SCHEMA_VERSION,
            "fastq.profile_reads.qc_json",
        ),
        ("fastq.report_qc", "multiqc_data") => (
            "parse_report_qc_report",
            REPORT_QC_REPORT_SCHEMA_VERSION,
            "fastq.report_qc.multiqc_data",
        ),
        ("fastq.remove_chimeras", "report_json") => (
            "parse_remove_chimeras_report",
            "bijux.fastq.remove_chimeras.report.v2",
            "fastq.remove_chimeras.report_json",
        ),
        ("fastq.remove_duplicates", "report_json") => (
            "parse_remove_duplicates_report",
            REMOVE_DUPLICATES_REPORT_SCHEMA_VERSION,
            "fastq.remove_duplicates.report_json",
        ),
        ("fastq.screen_taxonomy", "classification_report_json") => (
            "parse_screen_taxonomy_report",
            SCREEN_TAXONOMY_REPORT_SCHEMA_VERSION,
            "fastq.screen_taxonomy.classification_report_json",
        ),
        ("fastq.trim_polyg_tails", "report_json") => (
            "parse_trim_polyg_report",
            TRIM_POLYG_REPORT_SCHEMA_VERSION,
            "fastq.trim_polyg_tails.report_json",
        ),
        ("fastq.trim_reads", "report_json") => (
            "parse_trim_reads_report",
            TRIM_READS_REPORT_SCHEMA_VERSION,
            "fastq.trim_reads.report_json",
        ),
        ("fastq.trim_terminal_damage", "report_json") => (
            "parse_terminal_damage_report",
            TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION,
            "fastq.trim_terminal_damage.report_json",
        ),
        ("fastq.validate_reads", "validation_report") => (
            "parse_validation_report",
            VALIDATION_REPORT_SCHEMA_VERSION,
            "fastq.validate_reads.validation_report",
        ),
        _ => panic!(
            "missing FASTQ parser fixture binding for stage `{stage_id}` surface `{semantic_surface}`"
        ),
    };

    FastqParserFixtureBinding { tool_id, stage_id, parser_id, parser_schema_id, fixture_case_id }
}

const FASTQ_PARSER_FIXTURE_CASES: &[FastqParserFixtureCase] = &[
    FastqParserFixtureCase {
        fixture_case_id: "fastq.cluster_otus.report_json",
        stage_id: "fastq.cluster_otus",
        semantic_surface: "report_json",
        canonical_tool_id: "vsearch",
        raw_fixture: r#"{"schema_version":"bijux.fastq.cluster_otus.report.v2","stage":"fastq.cluster_otus","stage_id":"fastq.cluster_otus","tool_id":"vsearch","otu_identity":0.97,"threads":4,"input_reads":"merged.fastq.gz","otu_table":"otu_abundance.tsv","otu_representatives":"otu_representatives.fasta","taxonomy_ready_fasta":"taxonomy_ready.fasta","taxonomy_ready_fastq":"taxonomy_ready.fastq","report_json":"cluster_otus_report.json","otu_count":18,"sample_count":4,"representative_sequence_count":18,"output_table_kind":"otu_abundance_table","used_fallback":false,"runtime_s":3.4,"memory_mb":96.0,"exit_code":0,"raw_backend_report":"otu_clusters.uc","raw_backend_report_format":"vsearch_uc","backend_metrics":{"cluster_memberships":18}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.correct_errors.report_json",
        stage_id: "fastq.correct_errors",
        semantic_surface: "report_json",
        canonical_tool_id: "lighter",
        raw_fixture: r#"{"schema_version":"bijux.fastq.correct_errors.report.v2","stage":"fastq.correct_errors","stage_id":"fastq.correct_errors","tool_id":"lighter","paired_mode":"single_end","threads":8,"correction_engine":"lighter","quality_encoding":"phred33","kmer_size":31,"musket_kmer_budget":null,"genome_size":2500000,"max_memory_gb":null,"trusted_kmer_artifact":"trusted_kmers.fa","conservative_mode":false,"input_r1":"reads.fastq.gz","input_r2":null,"output_r1":"corrected.fastq.gz","output_r2":null,"report_json":"correct_report.json","corrected_reads":100,"reads_in":100,"reads_out":100,"bases_in":10000,"bases_out":10000,"pairs_in":null,"pairs_out":null,"mean_q_before":30.0,"mean_q_after":31.0,"kmer_fix_rate":0.12,"correction_effect":{"outputs_changed":true,"reads_delta":0,"bases_delta":0,"mean_q_delta":1.0},"runtime_s":1.8,"memory_mb":96.0,"exit_code":0,"raw_backend_report":"lighter.log","raw_backend_report_format":"lighter_log","backend_metrics":{"trusted_kmers_loaded":true}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.deplete_host.host_depletion_report_json",
        stage_id: "fastq.deplete_host",
        semantic_surface: "host_depletion_report_json",
        canonical_tool_id: "bowtie2",
        raw_fixture: r#"{"schema_version":"bijux.fastq.deplete_host.report.v2","stage":"fastq.deplete_host","stage_id":"fastq.deplete_host","tool_id":"bowtie2","paired_mode":"paired_end","threads":6,"reference_scope":"host","reference_catalog_id":"host_reference","reference_index_artifact_id":"reference_index","reference_index_backend":"bowtie2_build","reference_build_id":"2026.03","reference_digest":"sha256:host","masking_policy":"unmasked","decoy_policy":"none","decoy_catalog_id":null,"identity_threshold":0.95,"retained_read_policy":"keep_non_host_reads","emit_removed_reads":true,"report_format":"bowtie2_metrics_file","retain_unmapped_pairs":true,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"host_depleted_R1.fastq.gz","output_r2":"host_depleted_R2.fastq.gz","removed_host_r1":"removed_host_R1.fastq.gz","removed_host_r2":"removed_host_R2.fastq.gz","report_json":"host_depletion_report.json","reads_in":200,"reads_out":150,"reads_removed":50,"bases_in":20000,"bases_out":15000,"bases_removed":5000,"pairs_in":100,"pairs_out":75,"host_fraction_removed":0.25,"runtime_s":10.5,"memory_mb":512.0,"exit_code":0,"raw_backend_report":"bowtie2.host.metrics.txt","raw_backend_report_format":"bowtie2_met_file","backend_metrics":{"reads_removed":50}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.deplete_reference_contaminants.contaminant_screen_report_json",
        stage_id: "fastq.deplete_reference_contaminants",
        semantic_surface: "contaminant_screen_report_json",
        canonical_tool_id: "bowtie2",
        raw_fixture: r#"{"schema_version":"bijux.fastq.deplete_reference_contaminants.report.v2","stage":"fastq.deplete_reference_contaminants","stage_id":"fastq.deplete_reference_contaminants","tool_id":"bowtie2","paired_mode":"paired_end","threads":6,"reference_catalog_id":"contaminant_reference","contaminant_reference":"phix_and_spikeins","reference_index_artifact_id":"reference_index","reference_index_backend":"bowtie2_build","reference_build_id":"2026.03","reference_digest":"sha256:contaminant","match_threshold":0.95,"retained_read_role":"contaminant_screened_reads","rejected_read_role":"removed_contaminant_reads","retain_unmapped_pairs":true,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"contaminant_screened_R1.fastq.gz","output_r2":"contaminant_screened_R2.fastq.gz","removed_reads_r1":"removed_contaminant_R1.fastq.gz","removed_reads_r2":"removed_contaminant_R2.fastq.gz","report_json":"contaminant_screen_report.json","reads_in":200,"reads_out":160,"reads_removed":40,"bases_in":20000,"bases_out":15600,"bases_removed":4400,"pairs_in":100,"pairs_out":80,"contaminant_fraction_removed":0.2,"runtime_s":9.8,"memory_mb":512.0,"exit_code":0,"raw_backend_report":"bowtie2.contaminant.metrics.txt","raw_backend_report_format":"bowtie2_met_file","backend_metrics":{"reads_removed":40}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.deplete_rrna.rrna_report_json",
        stage_id: "fastq.deplete_rrna",
        semantic_surface: "rrna_report_json",
        canonical_tool_id: "sortmerna",
        raw_fixture: r#"{"schema_version":"bijux.fastq.deplete_rrna.report.v2","stage":"fastq.deplete_rrna","stage_id":"fastq.deplete_rrna","tool_id":"sortmerna","paired_mode":"paired_end","threads":6,"rrna_db":"/refs/silva","database_artifact_id":"silva_nr99","database_build_id":"2026.03","database_digest":"sha256:silva","screening_engine":"sortmerna","report_format":"summary_tsv_and_json","emit_removed_reads":false,"min_identity":0.95,"retained_read_role":"rrna_filtered_reads","rejected_read_role":"removed_rrna_reads","input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"rrna_filtered_R1.fastq.gz","output_r2":"rrna_filtered_R2.fastq.gz","removed_reads_r1":"removed_rrna_R1.fastq.gz","removed_reads_r2":"removed_rrna_R2.fastq.gz","rrna_report_tsv":"rrna_report.tsv","rrna_report_json":"rrna_report.json","reads_in":200,"reads_out":150,"reads_removed":50,"bases_in":20000,"bases_out":15000,"bases_removed":5000,"pairs_in":100,"pairs_out":75,"rrna_fraction_removed":0.25,"runtime_s":12.3,"memory_mb":256.0,"exit_code":0,"raw_backend_report":"sortmerna.log","raw_backend_report_format":"sortmerna_log","backend_metrics":{"reads_removed":50}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.detect_adapters.report_json",
        stage_id: "fastq.detect_adapters",
        semantic_surface: "report_json",
        canonical_tool_id: "fastqc",
        raw_fixture: r#"{"schema_version":"bijux.fastq.detect_adapters.report.v3","stage":"fastq.detect_adapters","stage_id":"fastq.detect_adapters","tool_id":"fastqc","paired_mode":"paired_end","threads":4,"inspection_mode":"evidence_only","report_only":true,"evidence_engine":"fastqc","evidence_scope":"full_input","evidence_format":"fastqc_summary","evidence_artifact_id":"report_json","detected_adapter_source":"normalized_fastqc_evidence","detected_adapter_ids":["truseq_universal","nextera_transposase"],"detection_confidence":0.22,"detection_threshold":0.01,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","report_json":"adapter_report.json","adapter_evidence_dir":"fastqc","reads_in":200,"reads_out":200,"bases_in":20000,"bases_out":20000,"pairs_in":100,"pairs_out":100,"mean_q":31.2,"candidate_adapter_count":2,"adapter_trimmed_fraction":0.08,"adapter_content_max":12.5,"adapter_content_mean":3.2,"duplication_rate":0.15,"n_rate":0.001,"kmer_warning_count":4,"overrepresented_sequence_count":3,"runtime_s":4.0,"memory_mb":64.0,"exit_code":0,"raw_backend_report":"fastqc/fastqc_data.txt","raw_backend_report_format":"fastqc_data_txt"}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.detect_duplicates_premerge.report_json",
        stage_id: "fastq.detect_duplicates_premerge",
        semantic_surface: "report_json",
        canonical_tool_id: "bijux_dna",
        raw_fixture: r#"{"schema_version":"bijux.fastq.detect_duplicates_premerge.report.v1","stage":"fastq.detect_duplicates_premerge","stage_id":"fastq.detect_duplicates_premerge","tool_id":"bijux_dna","paired_mode":"paired_end","duplicate_detection_policy":"report_only","measurement_scope":"premerge_sequence_signature","modifies_reads":false,"advisory_only":true,"reads_in":12,"duplicate_signal_reads":4,"duplicate_signal_fraction":0.3333333333333333,"compared_read_pairs":6}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.estimate_library_complexity_prealign.report_json",
        stage_id: "fastq.estimate_library_complexity_prealign",
        semantic_surface: "report_json",
        canonical_tool_id: "bijux_dna",
        raw_fixture: r#"{"schema_version":"bijux.fastq.estimate_library_complexity_prealign.report.v1","stage":"fastq.estimate_library_complexity_prealign","stage_id":"fastq.estimate_library_complexity_prealign","tool_id":"bijux_dna","paired_mode":"single_end","complexity_policy":"prealign_kmer","estimate_method":"kmer_redundancy","modifies_reads":false,"advisory_only":true,"reads_in":0,"estimated_unique_fraction":0.0,"estimated_duplicate_fraction":0.0,"insufficient_data_reason":"insufficient_reads_for_prealign_complexity_estimation","kmer_size":31}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.extract_umis.report_json",
        stage_id: "fastq.extract_umis",
        semantic_surface: "report_json",
        canonical_tool_id: "umi_tools",
        raw_fixture: r#"{"schema_version":"bijux.fastq.extract_umis.report.v2","stage":"fastq.extract_umis","stage_id":"fastq.extract_umis","tool_id":"umi_tools","paired_mode":"paired_end","threads":2,"umi_pattern":"NNNNNNNN","extraction_location":"read1_prefix","read_name_transform":"append_to_header","failed_extraction_policy":"refuse_stage","grouping_policy":"pair_aware","downstream_dedup_policy":"sequence_identity_recommended","downstream_propagation":"header_and_report","input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"umi_reads_R1.fastq.gz","output_r2":"umi_reads_R2.fastq.gz","report_json":"umi_report.json","reads_in":200,"reads_out":200,"bases_in":20000,"bases_out":20000,"pairs_in":100,"pairs_out":100,"reads_with_umi":200,"mean_q_before":30.0,"mean_q_after":30.0,"runtime_s":1.4,"memory_mb":64.0,"exit_code":0,"raw_backend_report":"umi_tools.extract.log","raw_backend_report_format":"umi_tools_log","backend_metrics":{"reads_with_umi_fraction":1.0}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.filter_low_complexity.filter_report_json",
        stage_id: "fastq.filter_low_complexity",
        semantic_surface: "filter_report_json",
        canonical_tool_id: "bbduk",
        raw_fixture: r#"{"schema_version":"bijux.fastq.filter_low_complexity.report.v2","stage":"fastq.filter_low_complexity","stage_id":"fastq.filter_low_complexity","tool_id":"bbduk","paired_mode":"single_end","threads":8,"input_r1":"reads.fastq.gz","input_r2":null,"output_r1":"filtered.fastq.gz","output_r2":null,"report_json":"low_complexity_report.json","entropy_threshold":0.5,"polyx_threshold":20,"reads_in":100,"reads_out":92,"reads_removed_low_complexity":8,"bases_in":1000,"bases_out":910,"pairs_in":null,"pairs_out":null,"mean_q_before":28.0,"mean_q_after":29.0,"runtime_s":1.1,"memory_mb":64.0,"exit_code":0,"raw_backend_report":"bbduk.low_complexity.stats","raw_backend_report_format":"bbduk_stats","backend_metrics":{"reads_removed_reported":8}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.filter_reads.report_json",
        stage_id: "fastq.filter_reads",
        semantic_surface: "report_json",
        canonical_tool_id: "fastp",
        raw_fixture: r#"{"schema_version":"bijux.fastq.filter_reads.report.v3","stage":"fastq.filter_reads","stage_id":"fastq.filter_reads","tool_id":"fastp","paired_mode":"paired_end","threads":4,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"filtered_R1.fastq.gz","output_r2":"filtered_R2.fastq.gz","report_json":"filter_report.json","max_n":0,"max_n_fraction":null,"max_n_count":0,"low_complexity_threshold":20.0,"entropy_threshold":20.0,"n_policy":"drop","polyx_policy":"trim","contaminant_db":"contaminants.fa","reads_in":100,"reads_out":95,"reads_dropped":5,"reads_removed_by_n":2,"reads_removed_by_entropy":1,"reads_removed_low_complexity":1,"reads_removed_by_kmer":1,"reads_removed_contaminant_kmer":1,"reads_removed_by_length":0,"bases_in":10000,"bases_out":9200,"pairs_in":50,"pairs_out":47,"mean_q_before":28.0,"mean_q_after":30.0,"runtime_s":4.2,"memory_mb":128.0,"exit_code":0,"raw_backend_report":"fastp.json","raw_backend_report_format":"fastp_json","backend_metrics":{"passed_filter_reads":95,"too_many_n_reads":2}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.index_reference.report_json",
        stage_id: "fastq.index_reference",
        semantic_surface: "report_json",
        canonical_tool_id: "bowtie2_build",
        raw_fixture: r#"{"schema_version":"bijux.fastq.index_reference.report.v2","stage":"fastq.index_reference","stage_id":"fastq.index_reference","tool_id":"bowtie2_build","threads":4,"index_format":"bowtie2_build","reference_fasta":"reference.fa","reference_bytes":4096,"reference_index":"reference_index/bowtie2/reference","report_json":"index_reference_report.json","index_prefix":"reference","emitted_files":[{"relative_path":"reference.1.bt2","bytes":1024},{"relative_path":"reference.2.bt2","bytes":2048}],"index_file_count":2,"index_bytes":3072,"runtime_s":1.5,"memory_mb":96.0,"exit_code":0,"backend_metrics":{"index_directory":"reference_index/bowtie2"}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.infer_asvs.report_json",
        stage_id: "fastq.infer_asvs",
        semantic_surface: "report_json",
        canonical_tool_id: "dada2",
        raw_fixture: r#"{"schema_version":"bijux.fastq.infer_asvs.report.v2","stage":"fastq.infer_asvs","stage_id":"fastq.infer_asvs","tool_id":"dada2","paired_mode":"paired_end","denoising_method":"dada2","pooling_mode":"independent","chimera_policy":"remove_bimera_denovo","requires_r_runtime":true,"output_table_kind":"asv_abundance_table","input_reads_r1":"reads_R1.fastq.gz","input_reads_r2":"reads_R2.fastq.gz","asv_table_tsv":"asv_abundance.tsv","asv_sequences_fasta":"asv_sequences.fasta","taxonomy_ready_fasta":"taxonomy_ready.fasta","taxonomy_ready_fastq":"taxonomy_ready.fastq","report_json":"infer_asvs_report.json","asv_count":12,"sample_count":3,"representative_sequence_count":12,"used_fallback":false,"raw_backend_report":"infer_asvs_report.json","raw_backend_report_format":"infer_asvs_governed_report_json","runtime_s":1.2,"memory_mb":128.0,"exit_code":0,"backend_metrics":{"nonchimera_reads":1200}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.merge_pairs.report_json",
        stage_id: "fastq.merge_pairs",
        semantic_surface: "report_json",
        canonical_tool_id: "pear",
        raw_fixture: r#"{"schema_version":"bijux.fastq.merge_pairs.report.v2","stage":"fastq.merge_pairs","stage_id":"fastq.merge_pairs","tool_id":"pear","paired_mode":"paired_end","merge_engine":"pear","threads":4,"merge_overlap":20,"min_len":80,"unmerged_read_policy":"emit_unmerged_pairs","input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","merged_reads":"merged.fastq.gz","unmerged_reads_r1":"unmerged_R1.fastq.gz","unmerged_reads_r2":"unmerged_R2.fastq.gz","reads_r1":100,"reads_r2":96,"reads_merged":88,"reads_unmerged":6,"merge_rate":0.9166666667}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.normalize_abundance.report_json",
        stage_id: "fastq.normalize_abundance",
        semantic_surface: "report_json",
        canonical_tool_id: "seqkit",
        raw_fixture: r#"{"schema_version":"bijux.fastq.normalize_abundance.report.v2","stage":"fastq.normalize_abundance","stage_id":"fastq.normalize_abundance","tool_id":"seqkit","method":"relative_abundance","input_table":"otu_abundance.tsv","normalized_abundance_tsv":"abundance_normalized.tsv","expected_columns":["sample_id","feature_id","abundance"],"input_value_column":"abundance","normalized_value_column":"normalized_abundance","compositional_rule":"per_sample_sum_to_one","scale_factor":null,"table_rows":12,"sample_count":3,"feature_count":4,"zero_fraction":0.25,"per_sample_sums":[["sample_a",1.0],["sample_b",1.0]],"runtime_s":1.2,"memory_mb":32.0,"raw_backend_report":null,"raw_backend_report_format":null,"used_fallback":false,"backend_metrics":{"normalization_rows":12}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.normalize_primers.report_json",
        stage_id: "fastq.normalize_primers",
        semantic_surface: "report_json",
        canonical_tool_id: "cutadapt",
        raw_fixture: r#"{"schema_version":"bijux.fastq.normalize_primers.report.v2","stage":"fastq.normalize_primers","stage_id":"fastq.normalize_primers","tool_id":"cutadapt","paired_mode":"paired_end","primer_set_id":"16S_universal_v1","marker_id":"16S","primer_fasta":"assets/reference/primers/16S_universal_v1.fasta","orientation_policy":"normalize_to_forward_primer","max_mismatch_rate":0.1,"min_overlap_bp":10,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"normalized_R1.fastq.gz","output_r2":"normalized_R2.fastq.gz","reads_in":200,"reads_out":200,"bases_in":10000,"bases_out":9600,"pairs_in":100,"pairs_out":100,"primer_trimmed_reads":190,"primer_trimmed_fraction":0.95,"orientation_forward_fraction":0.94,"primer_orientation_report":"primer_orientation.tsv","primer_stats_json":"primer_stats.json","raw_backend_report":"primer_stats.json","raw_backend_report_format":"cutadapt_json","runtime_s":3.1,"memory_mb":96.0,"used_fallback":false,"backend_metrics":{"tool":"cutadapt","trimmed_reads":190}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.profile_overrepresented_sequences.report_json",
        stage_id: "fastq.profile_overrepresented_sequences",
        semantic_surface: "report_json",
        canonical_tool_id: "fastqc",
        raw_fixture: r#"{"schema_version":"bijux.fastq.profile_overrepresented.report.v2","stage":"fastq.profile_overrepresented_sequences","stage_id":"fastq.profile_overrepresented_sequences","tool_id":"fastqc","paired_mode":"paired_end","threads":4,"top_k":25,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","overrepresented_sequences_tsv":"overrepresented_sequences.tsv","overrepresented_sequences_json":"overrepresented_sequences.json","report_json":"overrepresented_report.json","sequence_count":25,"flagged_sequences":3,"top_fraction":0.12,"rows":[{"sequence":"ACGT","count":12,"fraction":0.12,"flag":"overrepresented"}],"runtime_s":1.4,"memory_mb":48.0,"exit_code":0,"raw_backend_report":"fastqc_data.txt","raw_backend_report_format":"fastqc_module_txt"}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.profile_read_lengths.report_json",
        stage_id: "fastq.profile_read_lengths",
        semantic_surface: "report_json",
        canonical_tool_id: "seqkit_stats",
        raw_fixture: r#"{"schema_version":"bijux.fastq.profile_read_lengths.report.v2","stage":"fastq.profile_read_lengths","stage_id":"fastq.profile_read_lengths","tool_id":"seqkit_stats","paired_mode":"paired_end","threads":2,"histogram_bins":64,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","length_distribution_tsv":"length_distribution.tsv","length_distribution_json":"length_distribution.json","report_json":"profile_read_lengths_report.json","read_count":200,"min_read_length":90,"mean_read_length":101.5,"median_read_length":100.0,"max_read_length":150,"distinct_lengths":12,"histogram":[{"read_length":100,"count":180}],"runtime_s":1.1,"memory_mb":16.0,"exit_code":0,"raw_backend_report":"length_distribution.tsv","raw_backend_report_format":"seqkit_fx2tab_tsv"}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.profile_reads.qc_json",
        stage_id: "fastq.profile_reads",
        semantic_surface: "qc_json",
        canonical_tool_id: "seqkit_stats",
        raw_fixture: r#"{"schema_version":"bijux.fastq.profile_reads.report.v2","stage":"fastq.profile_reads","stage_id":"fastq.profile_reads","tool_id":"seqkit_stats","paired_mode":"paired_end","threads":2,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","qc_json":"qc.json","qc_tsv":"qc.tsv","qc_plots_dir":"plots","length_histogram_source":"seqkit_fx2tab","reads_total":200,"bases_total":20000,"mean_q":31.2,"gc_percent":42.0,"length_histogram":[{"length":100,"count":200}],"mate_summaries":[{"label":"reads_r1","reads":100,"bases":10000,"mean_q":31.0,"gc_percent":41.0},{"label":"reads_r2","reads":100,"bases":10000,"mean_q":31.4,"gc_percent":43.0}],"runtime_s":1.2,"memory_mb":20.0,"exit_code":0,"raw_backend_report":"qc.tsv","raw_backend_report_format":"seqkit_stats_tsv","backend_metrics":[{"schema_version":"bijux.seqkit.metrics.v1","reads":100,"bases":10000,"mean_q":31.0,"gc_percent":41.0}]}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.report_qc.multiqc_data",
        stage_id: "fastq.report_qc",
        semantic_surface: "multiqc_data",
        canonical_tool_id: "multiqc",
        raw_fixture: r#"{"schema_version":"bijux.fastq.report_qc.report.v2","stage":"fastq.report_qc","stage_id":"fastq.report_qc","tool_id":"multiqc","paired_mode":"paired_end","aggregation_engine":"multiqc","aggregation_scope":"governed_qc_artifacts","reads_in":200,"reads_out":200,"bases_in":20000,"bases_out":20000,"pairs_in":100,"pairs_out":100,"mean_q":31.0,"contamination_rate":0.04,"adapter_content_max":0.1,"adapter_content_mean":0.03,"duplication_rate":0.08,"n_rate":0.001,"kmer_warning_count":2,"overrepresented_sequence_count":1,"multiqc_sample_count":2,"multiqc_module_count":5,"raw_fastqc_dir":"raw_fastqc","trimmed_fastqc_dir":"trimmed_fastqc","multiqc_report":"multiqc_report.html","multiqc_data":"multiqc_data","governed_qc_input_count":3,"governed_qc_contributor_stage_ids":["fastq.trim_reads"],"governed_qc_contributor_tool_ids":["fastp"],"governed_qc_contributors":[{"contributor_id":"fastq.trim_reads.fastp","stage_id":"fastq.trim_reads","tool_id":"fastp","artifact_id":"report_json","artifact_role":"report_json","path":"trim/report.json"}],"governed_qc_lineage_hash":"lineage","governed_qc_inputs_manifest":"governed_qc_inputs_manifest.json","runtime_s":3.0,"memory_mb":128.0,"exit_code":0}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.remove_chimeras.report_json",
        stage_id: "fastq.remove_chimeras",
        semantic_surface: "report_json",
        canonical_tool_id: "vsearch",
        raw_fixture: r#"{"schema_version":"bijux.fastq.remove_chimeras.report.v2","stage":"fastq.remove_chimeras","stage_id":"fastq.remove_chimeras","tool_id":"vsearch","paired_mode":"single_end","threads":2,"method":"vsearch_uchime_denovo","detection_scope":"denovo","chimera_removed_definition":"reads flagged as de_novo chimeras are excluded from downstream abundance tables","input_reads":"merged.fastq.gz","output_reads":"nonchimeras.fastq.gz","chimera_metrics_json":"chimera_metrics.json","chimeras_fasta":"chimeras.fasta","uchime_report_tsv":"uchime.tsv","reads_in":100,"reads_out":92,"chimeras_removed":8,"chimera_fraction":0.08,"used_fallback":false,"raw_backend_report":"uchime.tsv","raw_backend_report_format":"vsearch_uchime_tsv","runtime_s":1.7,"memory_mb":32.0,"exit_code":0,"backend_metrics":{"parsed_records":100,"flagged_records":8}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.remove_duplicates.report_json",
        stage_id: "fastq.remove_duplicates",
        semantic_surface: "report_json",
        canonical_tool_id: "clumpify",
        raw_fixture: r#"{"schema_version":"bijux.fastq.remove_duplicates.report.v2","stage":"fastq.remove_duplicates","stage_id":"fastq.remove_duplicates","tool_id":"clumpify","paired_mode":"single_end","threads":4,"dedup_mode":"optical_aware","keep_order":false,"input_r1":"reads.fastq.gz","input_r2":null,"output_r1":"dedup.fastq.gz","output_r2":null,"reads_in":100,"reads_out":85,"reads_in_r2":null,"reads_out_r2":null,"pairs_in":null,"pairs_out":null,"pair_count_match":null,"duplicates_removed":15,"dedup_rate":0.15,"duplicate_classes_tsv":"duplicate_classes.tsv","duplicate_provenance_json":"duplicate_provenance.json","duplicate_classes":[{"class":"duplicate","reads_removed":11,"paired_mode":"single_end"},{"class":"optical_duplicate","reads_removed":4,"paired_mode":"single_end"}],"raw_backend_report":"clumpify.log","raw_backend_report_format":"clumpify_log","runtime_s":2.2,"memory_mb":64.0}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.screen_taxonomy.classification_report_json",
        stage_id: "fastq.screen_taxonomy",
        semantic_surface: "classification_report_json",
        canonical_tool_id: "kraken2",
        raw_fixture: r#"{"schema_version":"bijux.fastq.screen_taxonomy.report.v2","stage":"fastq.screen_taxonomy","stage_id":"fastq.screen_taxonomy","tool_id":"kraken2","paired_mode":"paired_end","threads":8,"classifier":"kraken2","report_format":"kraken_report","assignment_format":"kraken_assignments","database_catalog_id":"taxonomy_reference","database_artifact_id":"taxonomy_db","database_build_id":"build-2026-03","database_digest":"sha256:taxonomy","database_namespace":"read_screening","database_scope":"read_screening","minimum_confidence":0.1,"emit_unclassified":true,"interpretation_boundary":"screening_only","truth_conditions":[],"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","screen_report_tsv":"kraken2.report.tsv","classification_report_json":"kraken2.classifications.json","reads_in":200,"reads_out":200,"bases_in":20000,"bases_out":20000,"pairs_in":100,"pairs_out":100,"contamination_rate":0.23,"classified_fraction":0.77,"unclassified_fraction":0.23,"summary_entries":[{"label":"unclassified","percent":23.0},{"label":"bacteria","percent":77.0}],"top_taxa":[{"label":"bacteria","percent":77.0}],"runtime_s":12.5,"memory_mb":512.0}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.trim_polyg_tails.report_json",
        stage_id: "fastq.trim_polyg_tails",
        semantic_surface: "report_json",
        canonical_tool_id: "fastp",
        raw_fixture: r#"{"schema_version":"bijux.fastq.trim_polyg_tails.report.v2","stage":"fastq.trim_polyg_tails","stage_id":"fastq.trim_polyg_tails","tool_id":"fastp","paired_mode":"paired_end","threads":4,"trim_polyg":true,"min_polyg_run":10,"input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"trimmed_R1.fastq.gz","output_r2":"trimmed_R2.fastq.gz","reads_in":100,"reads_out":98,"bases_in":10000,"bases_out":9820,"pairs_in":50,"pairs_out":49,"mean_q_before":27.9,"mean_q_after":28.4,"trimmed_tail_count":12,"bases_trimmed_polyg":180,"polyx_bank_id":"polyx","polyx_bank_hash":"sha256:polyx","polyx_preset":"illumina_twocolor","runtime_s":4.2,"memory_mb":96.0,"raw_backend_report":"trim_polyg.fastp.json","raw_backend_report_format":"fastp_json","backend_metrics":{"schema_version":"bijux.fastp.metrics.v1","passed_filter_reads":98}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.trim_reads.report_json",
        stage_id: "fastq.trim_reads",
        semantic_surface: "report_json",
        canonical_tool_id: "fastp",
        raw_fixture: r#"{"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","tool_id":"fastp","paired_mode":"paired_end","threads":4,"trimming_backend":"fastp","backend_mode":"enforced","input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"trimmed_R1.fastq.gz","output_r2":"trimmed_R2.fastq.gz","min_length":30,"quality_cutoff":20,"adapter_policy":"bank","polyx_policy":"trim","n_policy":"drop","contaminant_policy":"none","adapter_bank_id":"illumina","adapter_bank_hash":"sha256:adapter","adapter_preset":"default","detected_adapter_source":"governed_pattern_scan","adapter_overrides":{"enable":["AGATCGGAAGAGC"],"disable":["polyA"]},"prepared_adapter_bank":null,"polyx_bank_id":"polyx","polyx_bank_hash":"sha256:polyx","polyx_preset":"illumina_twocolor","contaminant_bank_id":"contaminants","contaminant_bank_hash":"sha256:contaminants","contaminant_preset":"illumina_default","reads_in":100,"reads_out":90,"bases_in":1000,"bases_out":820,"pairs_in":50,"pairs_out":45,"mean_q_before":28.0,"mean_q_after":31.0,"effective_trim_params":{"adapter_policy":"bank","min_length":30,"quality_cutoff":20},"runtime_s":8.4,"memory_mb":128.0,"raw_backend_report":"trim.fastp.json","raw_backend_report_format":"fastp_json"}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.trim_terminal_damage.report_json",
        stage_id: "fastq.trim_terminal_damage",
        semantic_surface: "report_json",
        canonical_tool_id: "cutadapt",
        raw_fixture: r#"{"schema_version":"bijux.fastq.trim_terminal_damage.report.v2","stage":"fastq.trim_terminal_damage","stage_id":"fastq.trim_terminal_damage","tool_id":"cutadapt","paired_mode":"paired_end","threads":4,"damage_mode":"ancient","execution_policy":"explicit_terminal_trim","trim_5p_bases":2,"trim_3p_bases":1,"requested_trim_5p_bases":2,"requested_trim_3p_bases":1,"udg_classification":"non_udg","input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","output_r1":"trimmed_R1.fastq.gz","output_r2":"trimmed_R2.fastq.gz","reads_in":200,"reads_out":198,"bases_in":20000,"bases_out":19100,"mean_q_before":28.0,"mean_q_after":28.5,"ct_ga_asymmetry_pre":0.45,"ct_ga_asymmetry_post":0.12,"ct_ga_asymmetry_pre_r1":0.5,"ct_ga_asymmetry_post_r1":0.15,"ct_ga_asymmetry_pre_r2":0.4,"ct_ga_asymmetry_post_r2":0.09,"terminal_base_composition_pre_r1":{"C":80},"terminal_base_composition_post_r1":{"C":30},"terminal_base_composition_pre_r2":{"G":75},"terminal_base_composition_post_r2":{"G":28},"raw_backend_report":"cutadapt.damage.json","raw_backend_report_format":"cutadapt_json","runtime_s":12.4,"memory_mb":256.0,"used_fallback":false,"backend_metrics":{"reads_profiled_r1":200}}"#,
    },
    FastqParserFixtureCase {
        fixture_case_id: "fastq.validate_reads.validation_report",
        stage_id: "fastq.validate_reads",
        semantic_surface: "validation_report",
        canonical_tool_id: "fastqvalidator",
        raw_fixture: r#"{"schema_version":"bijux.fastq.validate.report.v1","stage":"fastq.validate_reads","stage_id":"fastq.validate_reads","tool_id":"fastqvalidator","validation_mode":"strict","pair_sync_policy":"require_header_sync","input_r1":"reads_R1.fastq.gz","input_r2":"reads_R2.fastq.gz","validation_log_r1":"validation_r1.log","validation_log_r2":"validation_r2.log","validated_inputs":2,"validated_reads_r1":101,"validated_reads_r2":100,"validated_pairs":100,"status_r1":0,"status_r2":0,"pair_sync_checked":true,"pair_sync_pass":false,"pair_count_match":false,"failure_class":"pair_count_mismatch","strict_pass":false,"exit_code":96}"#,
    },
];

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        fastq_parser_fixture_bindings, fastq_parser_fixture_cases,
        find_fastq_parser_fixture_binding, find_fastq_parser_fixture_case,
    };

    #[test]
    fn fastq_parser_fixture_inventory_covers_governed_active_bindings() {
        let rows = fastq_parser_fixture_bindings();
        assert_eq!(rows.len(), 69);

        let unique_rows = rows
            .iter()
            .map(|row| format!("{}:{}", row.stage_id, row.tool_id))
            .collect::<BTreeSet<_>>();
        assert_eq!(unique_rows.len(), rows.len());

        assert!(rows.iter().all(|row| !row.parser_id.is_empty()));
        assert!(rows.iter().all(|row| !row.parser_schema_id.is_empty()));
        assert!(rows
            .iter()
            .all(|row| find_fastq_parser_fixture_case(row.fixture_case_id).is_some()));

        assert!(rows.iter().any(|row| {
            row.stage_id == "fastq.trim_reads"
                && row.tool_id == "trimmomatic"
                && row.parser_id == "parse_trim_reads_report"
                && row.fixture_case_id == "fastq.trim_reads.report_json"
        }));
        assert!(rows.iter().any(|row| {
            row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.parser_id == "parse_screen_taxonomy_report"
        }));
        assert!(rows.iter().any(|row| {
            row.stage_id == "fastq.detect_duplicates_premerge"
                && row.tool_id == "bijux_dna"
                && row.parser_id == "parse_detect_duplicates_premerge_report"
        }));
    }

    #[test]
    fn fastq_parser_fixture_cases_remain_unique_and_queryable() {
        let cases = fastq_parser_fixture_cases();
        assert_eq!(cases.len(), 27);

        let unique_rows = cases.iter().map(|row| row.fixture_case_id).collect::<BTreeSet<_>>();
        assert_eq!(unique_rows.len(), cases.len());
        assert!(cases.iter().all(|row| !row.raw_fixture.trim().is_empty()));

        let trim_reads = find_fastq_parser_fixture_case("fastq.trim_reads.report_json")
            .expect("trim reads fixture case");
        assert_eq!(trim_reads.stage_id, "fastq.trim_reads");
        assert_eq!(trim_reads.canonical_tool_id, "fastp");

        let trim_reads_binding = find_fastq_parser_fixture_binding("fastq.trim_reads", "fastp")
            .expect("trim reads binding");
        assert_eq!(trim_reads_binding.fixture_case_id, "fastq.trim_reads.report_json");
    }
}
