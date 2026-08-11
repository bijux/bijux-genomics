use anyhow::Result;
use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};

use super::{
    BindingInputV1, BindingOutputV1, BindingParameterV1, OutputDiscoveryV1, ToolArtifactSourceV1,
    ToolBindingRegistry, ToolBindingV1, TOOL_BINDING_SCHEMA_V1,
};

const BAM: &str = "bijux.bam.artifact.coordinate_sorted_indexed.v1";
const DIRECTORY: &str = "bijux.auxiliary.artifact.directory.v1";
const FILE: &str = "bijux.auxiliary.artifact.file.v1";
const FASTA: &str = "bijux.reference.artifact.fasta.v1";
const FASTQ_PAIRED: &str = "bijux.fastq.artifact.paired_end.v1";
const FASTQ_SINGLE: &str = "bijux.fastq.artifact.single_end.v1";
const JSON: &str = "bijux.auxiliary.artifact.json.v1";
const SAM: &str = "bijux.sam.artifact.unsorted.v1";
const TABLE: &str = "bijux.auxiliary.artifact.table.v1";
const VCF: &str = "bijux.vcf.artifact.coordinate_sorted_indexed.v1";
const VCF_PANEL: &str = "bijux.vcf.artifact.reference_panel_indexed.v1";

/// Returns every governed binding for a licensing-restricted external artifact.
///
/// The returned registry describes product integration independently of site admission. Runtime
/// owners must still verify an immutable external-artifact admission record before rendering or
/// executing any command.
///
/// # Errors
///
/// Returns an error if a canonical binding violates the binding schema or duplicates a stage/tool
/// pair.
pub fn canonical_external_artifact_tool_bindings() -> Result<ToolBindingRegistry> {
    let mut registry = ToolBindingRegistry::default();
    for binding in [
        angsd_kinship(),
        angsd_sex(),
        angsd_call_gl(),
        angsd_call_pseudohaploid(),
        fastuniq_remove_duplicates(),
        impute5_imputation_metrics(),
        impute5_impute(),
        ngsbriggs_damage(),
    ] {
        registry.insert(binding)?;
    }
    Ok(registry)
}

/// Returns the canonical retained distributable bindings currently closed by this crate.
///
/// # Errors
///
/// Returns an error if a binding violates the schema or duplicates a stage/tool pair.
pub fn canonical_retained_tool_bindings() -> Result<ToolBindingRegistry> {
    let mut registry = ToolBindingRegistry::default();
    for binding in [
        bbduk_filter_low_complexity(),
        bcftools_genotyping(),
        bedtools_coverage(),
        bowtie2_build_index_reference(),
        bowtie2_align(),
        bwa_align(),
        centrifuge_screen_taxonomy(),
        dada2_infer_asvs(),
        fastp_filter_low_complexity(),
        fastqc_detect_adapters(),
        fastqvalidator_validate_reads(),
        fqtools_validate_reads(),
        kaiju_screen_taxonomy(),
        kraken2_screen_taxonomy(),
        krakenuniq_screen_taxonomy(),
        mosdepth_coverage(),
        picard_gc_bias(),
        picard_insert_size(),
        picard_duplication_metrics(),
        picard_mapping_summary(),
        prinseq_filter_low_complexity(),
        rcorrector_correct_errors(),
        rxy_sex(),
        samtools_coverage(),
        samtools_duplication_metrics(),
        samtools_endogenous_content(),
        samtools_mapping_summary(),
        seqfu_profile_read_lengths(),
        seqfu_profile_reads(),
        seqkit_filter_reads(),
        seqkit_profile_read_lengths(),
        seqkit_profile_reads(),
        seqkit_stats_profile_reads(),
        seqtk_validate_reads(),
        star_index_reference(),
        umi_tools_extract_umis(),
        vsearch_cluster_otus(),
    ] {
        registry.insert(binding)?;
    }
    Ok(registry)
}

fn vsearch_cluster_otus() -> ToolBindingV1 {
    retained_binding(
        "fastq.cluster_otus",
        "vsearch",
        &[
            "single_end_or_merged_amplicon_reads",
            "identity_threshold_clustering",
            "native_uc_and_otu_tables",
        ],
        &[input("reads", FASTQ_SINGLE)],
        &["identity_threshold", "threads"],
        &[
            output("otu_representatives", FASTA),
            output("otu_clusters", TABLE),
            output("otu_table", TABLE),
        ],
        &[
            "${executable}",
            "--cluster_fast",
            "${input:reads}",
            "--id",
            "${parameter:identity_threshold}",
            "--sizein",
            "--sizeout",
            "--relabel",
            "OTU_",
            "--threads",
            "${parameter:threads}",
            "--centroids",
            "${output:otu_representatives}",
            "--uc",
            "${output:otu_clusters}",
            "--otutabout",
            "${output:otu_table}",
        ],
        "bijux.benchmark.fastq.cluster_otus.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::cluster_otus",
        "benchmark.fastq.cluster_otus.consume",
        resources(4, 8, 4),
        &[
            "qualification_binding_requires_single_end_or_premerged_amplicon_reads",
            "qualification_identity_threshold_is_0.97",
            "taxonomy_ready_derivatives_and_governed_reports_are_normalizer_outputs_not_native_vsearch_outputs",
            "comparative_case_admission_requires_the_closed_cluster_otus_stage_interface",
        ],
    )
}

fn dada2_infer_asvs() -> ToolBindingV1 {
    retained_binding(
        "fastq.infer_asvs",
        "dada2",
        &[
            "paired_end_amplicon_reads",
            "asv_inference",
            "native_asv_result_directory",
        ],
        &[input("reads_r1", FASTQ_PAIRED), input("reads_r2", FASTQ_PAIRED)],
        &["sample_id", "threads"],
        &[output("asv_result_directory", DIRECTORY)],
        &[
            "${executable}",
            "infer-asvs",
            "--input",
            "${input:reads_r1}",
            "--input-r2",
            "${input:reads_r2}",
            "--output-dir",
            "${output:asv_result_directory}",
            "--sample-id",
            "${parameter:sample_id}",
            "--threads",
            "${parameter:threads}",
        ],
        "bijux.benchmark.fastq.infer_asvs.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::infer_asvs",
        "benchmark.fastq.infer_asvs.consume",
        resources(4, 16, 8),
        &[
            "the_native_directory_must_contain_asv_table_sequences_taxonomy_ready_sequences_and_report",
            "the_governed_wrapper_applies_maxee_2_truncq_2_phix_removal_and_consensus_chimera_removal",
            "qualification_binding_uses_the_governed_paired_end_amplicon_profile",
            "comparative_case_admission_requires_the_closed_infer_asvs_stage_interface",
        ],
    )
}

fn bowtie2_build_index_reference() -> ToolBindingV1 {
    retained_binding(
        "fastq.index_reference",
        "bowtie2_build",
        &["reference_fasta", "fm_index_build", "multi_file_index_directory"],
        &[input("reference_fasta", FASTA)],
        &["threads"],
        &[output("reference_index", DIRECTORY)],
        &[
            "sh",
            "-lc",
            "set -eu\nmkdir -p '${output:reference_index}'\n'${executable}' --threads '${parameter:threads}' '${input:reference_fasta}' '${output:reference_index}/reference'",
        ],
        "bijux.benchmark.fastq.index_reference.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::index_reference",
        "benchmark.fastq.index_reference.consume",
        resources(4, 16, 16),
        &[
            "the_governed_output_is_a_directory_containing_the_complete_bowtie2_prefix_family",
            "the_downstream_probe_must_project_the_reference_prefix_inside_the_governed_directory",
            "index_formats_are_tool_specific_and_require_a_compatible_alignment_consumer",
            "comparative_case_admission_requires_the_closed_index_reference_stage_interface",
        ],
    )
}

fn star_index_reference() -> ToolBindingV1 {
    retained_binding(
        "fastq.index_reference",
        "star",
        &["reference_fasta", "splice_aware_index_build", "multi_file_index_directory"],
        &[input("reference_fasta", FASTA)],
        &["threads"],
        &[output("reference_index", DIRECTORY)],
        &[
            "sh",
            "-lc",
            "set -eu\nmkdir -p '${output:reference_index}'\n'${executable}' --runMode genomeGenerate --runThreadN '${parameter:threads}' --genomeDir '${output:reference_index}' --genomeFastaFiles '${input:reference_fasta}'",
        ],
        "bijux.benchmark.fastq.index_reference.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::index_reference",
        "benchmark.fastq.index_reference.consume",
        resources(4, 16, 16),
        &[
            "the_governed_output_is_the_complete_star_genome_directory",
            "index_formats_are_tool_specific_and_require_a_compatible_alignment_consumer",
            "comparative_case_admission_requires_the_closed_index_reference_stage_interface",
        ],
    )
}

fn bbduk_filter_low_complexity() -> ToolBindingV1 {
    retained_binding(
        "fastq.filter_low_complexity",
        "bbduk",
        &["single_end_reads", "entropy_filter", "native_filter_statistics"],
        &[input("reads_r1", FASTQ_SINGLE)],
        &["threads"],
        &[output("filtered_reads_r1", FASTQ_SINGLE), output("native_filter_statistics", TABLE)],
        &[
            "${executable}",
            "in=${input:reads_r1}",
            "out=${output:filtered_reads_r1}",
            "stats=${output:native_filter_statistics}",
            "entropy=0.5",
            "threads=${parameter:threads}",
            "overwrite=t",
        ],
        "bijux.benchmark.fastq.filter_low_complexity.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::filter_low_complexity",
        "benchmark.fastq.filter_low_complexity.compare",
        resources(4, 8, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "qualification_entropy_threshold_is_the_canonical_fraction_0.5",
            "comparative_case_admission_requires_the_closed_filter_low_complexity_stage_interface",
            "paired_end_execution_requires_the_stage_planner_pair_synchronization_wrapper",
        ],
    )
}

fn fastp_filter_low_complexity() -> ToolBindingV1 {
    retained_binding(
        "fastq.filter_low_complexity",
        "fastp",
        &["single_end_reads", "low_complexity_filter", "native_json_report"],
        &[input("reads_r1", FASTQ_SINGLE)],
        &["threads"],
        &[
            output("filtered_reads_r1", FASTQ_SINGLE),
            output("native_filter_report", JSON),
        ],
        &[
            "${executable}",
            "--in1",
            "${input:reads_r1}",
            "--out1",
            "${output:filtered_reads_r1}",
            "--thread",
            "${parameter:threads}",
            "--disable_adapter_trimming",
            "--disable_quality_filtering",
            "--low_complexity_filter",
            "--complexity_threshold",
            "50",
            "--json",
            "${output:native_filter_report}",
            "--html",
            "/dev/null",
        ],
        "bijux.benchmark.fastq.filter_low_complexity.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::filter_low_complexity",
        "benchmark.fastq.filter_low_complexity.compare",
        resources(4, 8, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "qualification_complexity_threshold_50_is_the_fastp_numeric_projection_of_policy_fraction_0.5",
            "fastp_adjacent_base_change_complexity_is_not_shannon_entropy",
            "adapter_and_quality_filtering_are_disabled_to_isolate_low_complexity_filtering",
            "comparative_case_admission_requires_the_closed_filter_low_complexity_stage_interface",
            "paired_end_execution_requires_the_stage_planner_pair_synchronization_wrapper",
        ],
    )
}

fn prinseq_filter_low_complexity() -> ToolBindingV1 {
    retained_binding(
        "fastq.filter_low_complexity",
        "prinseq",
        &["single_end_reads", "entropy_filter", "filtered_fastq_output"],
        &[input("reads_r1", FASTQ_SINGLE)],
        &["threads"],
        &[output("filtered_reads_r1", FASTQ_SINGLE)],
        &[
            "${executable}",
            "-threads",
            "${parameter:threads}",
            "-fastq",
            "${input:reads_r1}",
            "-out_good",
            "${output:filtered_reads_r1}",
            "-out_bad",
            "/dev/null",
            "-lc_entropy",
            "50",
        ],
        "bijux.benchmark.fastq.filter_low_complexity.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::filter_low_complexity",
        "benchmark.fastq.filter_low_complexity.compare",
        resources(4, 8, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "qualification_entropy_threshold_50_is_the_prinseq_integer_percent_projection_of_policy_fraction_0.5",
            "comparative_case_admission_requires_the_closed_filter_low_complexity_stage_interface",
            "paired_end_execution_requires_the_stage_planner_pair_synchronization_wrapper",
        ],
    )
}

fn seqkit_filter_reads() -> ToolBindingV1 {
    retained_binding(
        "fastq.filter_reads",
        "seqkit",
        &["single_end_reads", "minimum_length_filter", "streaming_fastq_transform"],
        &[input("reads_r1", FASTQ_SINGLE)],
        &["threads"],
        &[output("filtered_reads_r1", FASTQ_SINGLE)],
        &[
            "${executable}",
            "seq",
            "-j",
            "${parameter:threads}",
            "-m",
            "1",
            "-o",
            "${output:filtered_reads_r1}",
            "${input:reads_r1}",
        ],
        "bijux.benchmark.fastq.filter_reads.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::filter_reads",
        "benchmark.fastq.filter_reads.compare",
        resources(4, 8, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "comparative_case_admission_requires_the_closed_filter_reads_stage_interface",
            "paired_end_execution_requires_the_stage_planner_pair_synchronization_wrapper",
        ],
    )
}

fn fastqc_detect_adapters() -> ToolBindingV1 {
    retained_binding(
        "fastq.detect_adapters",
        "fastqc",
        &["single_end_reads", "adapter_content_evidence", "full_input_qc"],
        &[input("reads_r1", FASTQ_PAIRED)],
        &["threads"],
        &[output("adapter_evidence_dir", DIRECTORY)],
        &[
            "/bin/sh",
            "-c",
            "mkdir -p '${output:adapter_evidence_dir}' && '${executable}' --outdir '${output:adapter_evidence_dir}' --threads '${parameter:threads}' '${input:reads_r1}'",
        ],
        "bijux.benchmark.fastq.detect_adapters.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::fastq::detect_adapters",
        "benchmark.fastq.detect_adapters.compare",
        resources(4, 8, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "native_fastqc_directory_is_preserved_as_raw_attempt_evidence",
            "adapter_report_json_is_emitted_by_the_shared_result_normalizer",
        ],
    )
}

fn fastqvalidator_validate_reads() -> ToolBindingV1 {
    validate_reads_binding(
        "fastqvalidator",
        "'${executable}' --file '${input:reads_r1}'",
        &["single_end_reads", "structural_fastq_validation"],
    )
}

fn fqtools_validate_reads() -> ToolBindingV1 {
    validate_reads_binding(
        "fqtools",
        "'${executable}' validate '${input:reads_r1}'",
        &["single_end_reads", "fqtools_fastq_validation"],
    )
}

fn seqtk_validate_reads() -> ToolBindingV1 {
    validate_reads_binding(
        "seqtk",
        "'${executable}' seq '${input:reads_r1}'",
        &["single_end_reads", "seqtk_fastq_parsing"],
    )
}

fn validate_reads_binding(
    tool_id: &str,
    backend_command: &str,
    backend_capabilities: &[&str],
) -> ToolBindingV1 {
    let command = format!(
        "set +e; {backend_command} > \"$TMPDIR/bijux-{tool_id}-validation.log\" 2>&1; status=$?; if [ \"$status\" -eq 0 ]; then strict_pass=true; else strict_pass=false; fi; printf '{{\"schema_version\":\"bijux.fastq.validate.report.v1\",\"stage_id\":\"fastq.validate_reads\",\"tool_id\":\"{tool_id}\",\"validated_inputs\":1,\"strict_pass\":%s,\"exit_code\":%s}}\\n' \"$strict_pass\" \"$status\" > '${{output:validation_report}}'; printf '{{\"schema_version\":\"bijux.fastq.validate.lineage.v1\",\"stage_id\":\"fastq.validate_reads\",\"tool_id\":\"{tool_id}\",\"input_r1\":\"reads_r1\",\"validation_report\":\"validation_report\",\"paired_mode\":\"single_end\",\"validated_stream_ids\":[\"reads_r1\"]}}\\n' > '${{output:validated_reads_manifest}}'; exit \"$status\""
    );
    let mut capabilities = vec!["report_only_validation"];
    capabilities.extend_from_slice(backend_capabilities);
    retained_binding(
        "fastq.validate_reads",
        tool_id,
        &capabilities,
        &[input("reads_r1", FASTQ_PAIRED)],
        &[],
        &[output("validation_report", JSON), output("validated_reads_manifest", JSON)],
        &["/bin/sh", "-c", &command],
        "bijux.benchmark.fastq.validate_reads.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::pre::validate_reads",
        "benchmark.fastq.validate_reads.compare",
        resources(4, 8, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "paired_end_execution_requires_the_stage_planner_pair_synchronization_wrapper",
            "native_validation_log_remains_attempt_evidence_only",
        ],
    )
}

fn seqfu_profile_read_lengths() -> ToolBindingV1 {
    retained_binding(
        "fastq.profile_read_lengths",
        "seqfu",
        &["single_end_reads", "per_record_length_observations", "report_only_profile"],
        &[input("reads_r1", FASTQ_PAIRED)],
        &[],
        &[output("length_distribution_tsv", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' tab '${input:reads_r1}' | awk -F '\\t' '{print $1 \"\\t\" length($3)}' > '${output:length_distribution_tsv}'",
        ],
        "bijux.benchmark.fastq.profile_read_lengths.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::pre::profile_read_lengths",
        "benchmark.fastq.profile_read_lengths.compare",
        resources(2, 4, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "histogram_bins_are_applied_by_the_shared_result_normalizer",
            "native_tsv_contains_record_identity_and_exact_sequence_length",
        ],
    )
}

fn seqfu_profile_reads() -> ToolBindingV1 {
    retained_binding(
        "fastq.profile_reads",
        "seqfu",
        &["single_end_reads", "read_statistics", "gc_statistics"],
        &[input("reads_r1", FASTQ_PAIRED)],
        &["threads"],
        &[output("qc_tsv", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' stats --threads '${parameter:threads}' --gc '${input:reads_r1}' > '${output:qc_tsv}'",
        ],
        "bijux.benchmark.fastq.profile_reads.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::qc::profile_reads",
        "benchmark.fastq.profile_reads.compare",
        resources(2, 4, 4),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "qc_json_and_histograms_are_emitted_by_the_shared_result_normalizer",
        ],
    )
}

fn seqkit_profile_read_lengths() -> ToolBindingV1 {
    retained_binding(
        "fastq.profile_read_lengths",
        "seqkit",
        &["single_end_reads", "per_record_length_observations", "report_only_profile"],
        &[input("reads_r1", FASTQ_PAIRED)],
        &["threads"],
        &[output("length_distribution_tsv", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' fx2tab --name --only-id --length --threads '${parameter:threads}' '${input:reads_r1}' | awk -F '\\t' 'NF < 2 { exit 2 } { print $1 \"\\t\" $NF }' > '${output:length_distribution_tsv}'",
        ],
        "bijux.benchmark.fastq.profile_read_lengths.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::pre::profile_read_lengths",
        "benchmark.fastq.profile_read_lengths.compare",
        resources(2, 4, 4),
        &[
            "qualification_uses_the_governed_single_end_profile_when_selected_for_this_stage",
            "histogram_bins_are_applied_by_the_shared_result_normalizer",
            "native_tsv_contains_record_identity_and_exact_sequence_length",
        ],
    )
}

fn seqkit_profile_reads() -> ToolBindingV1 {
    retained_binding(
        "fastq.profile_reads",
        "seqkit",
        &["single_end_reads", "read_statistics", "tabular_statistics"],
        &[input("reads_r1", FASTQ_PAIRED)],
        &["threads"],
        &[output("qc_tsv", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' stats -a -T -j '${parameter:threads}' '${input:reads_r1}' > '${output:qc_tsv}'",
        ],
        "bijux.benchmark.fastq.profile_reads.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::qc::profile_reads",
        "benchmark.fastq.profile_reads.compare",
        resources(2, 4, 4),
        &[
            "qualification_uses_the_governed_single_end_profile_when_selected_for_this_stage",
            "qc_json_and_histograms_are_emitted_by_the_shared_result_normalizer",
        ],
    )
}

fn seqkit_stats_profile_reads() -> ToolBindingV1 {
    retained_binding(
        "fastq.profile_reads",
        "seqkit_stats",
        &["single_end_reads", "read_statistics", "tabular_statistics"],
        &[input("reads_r1", FASTQ_PAIRED)],
        &["threads"],
        &[output("qc_tsv", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' -a -T -j '${parameter:threads}' '${input:reads_r1}' > '${output:qc_tsv}'",
        ],
        "bijux.benchmark.fastq.profile_reads.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::qc::profile_reads",
        "benchmark.fastq.profile_reads.compare",
        resources(2, 2, 2),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "qc_json_and_histograms_are_emitted_by_the_shared_result_normalizer",
        ],
    )
}

fn centrifuge_screen_taxonomy() -> ToolBindingV1 {
    retained_binding(
        "fastq.screen_taxonomy",
        "centrifuge",
        &["single_end_reads", "taxonomy_database", "read_level_taxonomy_assignments"],
        &[input("reads_r1", FASTQ_PAIRED), input("taxonomy_database", "bijux.auxiliary.artifact.database.v1")],
        &["threads"],
        &[output("screen_report_tsv", TABLE), output("native_assignments", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' -x '${input:taxonomy_database}/reference' -q -U '${input:reads_r1}' -S '${output:native_assignments}' --report-file '${output:screen_report_tsv}' -p '${parameter:threads}'",
        ],
        "bijux.benchmark.fastq.screen_taxonomy.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::qc::screen_taxonomy",
        "benchmark.fastq.screen_taxonomy.compare",
        resources(4, 16, 8),
        &["qualification_binding_uses_the_governed_single_end_profile"],
    )
}

fn kaiju_screen_taxonomy() -> ToolBindingV1 {
    retained_binding(
        "fastq.screen_taxonomy",
        "kaiju",
        &["single_end_reads", "protein_level_taxonomy", "taxonomy_database"],
        &[input("reads_r1", FASTQ_PAIRED), input("taxonomy_database", "bijux.auxiliary.artifact.database.v1")],
        &["threads"],
        &[output("screen_report_tsv", TABLE), output("native_assignments", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' -t '${input:taxonomy_database}/taxonomy/nodes.dmp' -f '${input:taxonomy_database}/kaiju/kaiju_db.fmi' -i '${input:reads_r1}' -o '${output:native_assignments}' -z '${parameter:threads}' && kaiju2table -t '${input:taxonomy_database}/taxonomy/nodes.dmp' -n '${input:taxonomy_database}/taxonomy/names.dmp' -r genus -o '${output:screen_report_tsv}' '${output:native_assignments}'",
        ],
        "bijux.benchmark.fastq.screen_taxonomy.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::qc::screen_taxonomy",
        "benchmark.fastq.screen_taxonomy.compare",
        resources(4, 16, 8),
        &[
            "qualification_binding_uses_the_governed_single_end_profile",
            "kaiju2table_must_be_present_in_the_same_admitted_image",
        ],
    )
}

fn kraken2_screen_taxonomy() -> ToolBindingV1 {
    retained_binding(
        "fastq.screen_taxonomy",
        "kraken2",
        &["single_end_reads", "exact_kmer_taxonomy", "taxonomy_database"],
        &[
            input("reads_r1", FASTQ_PAIRED),
            input("taxonomy_database", "bijux.auxiliary.artifact.database.v1"),
        ],
        &["threads"],
        &[output("screen_report_tsv", TABLE), output("native_assignments", TABLE)],
        &[
            "${executable}",
            "--db",
            "${input:taxonomy_database}",
            "--threads",
            "${parameter:threads}",
            "--report",
            "${output:screen_report_tsv}",
            "--output",
            "${output:native_assignments}",
            "${input:reads_r1}",
        ],
        "bijux.benchmark.fastq.screen_taxonomy.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::qc::screen_taxonomy",
        "benchmark.fastq.screen_taxonomy.compare",
        resources(4, 16, 8),
        &["qualification_binding_uses_the_governed_single_end_profile"],
    )
}

fn krakenuniq_screen_taxonomy() -> ToolBindingV1 {
    retained_binding(
        "fastq.screen_taxonomy",
        "krakenuniq",
        &["single_end_reads", "unique_kmer_taxonomy", "taxonomy_database"],
        &[
            input("reads_r1", FASTQ_PAIRED),
            input("taxonomy_database", "bijux.auxiliary.artifact.database.v1"),
        ],
        &["threads"],
        &[output("screen_report_tsv", TABLE), output("native_assignments", TABLE)],
        &[
            "${executable}",
            "--db",
            "${input:taxonomy_database}",
            "--threads",
            "${parameter:threads}",
            "--fastq-input",
            "--report-file",
            "${output:screen_report_tsv}",
            "--output",
            "${output:native_assignments}",
            "${input:reads_r1}",
        ],
        "bijux.benchmark.fastq.screen_taxonomy.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::qc::screen_taxonomy",
        "benchmark.fastq.screen_taxonomy.compare",
        resources(4, 16, 8),
        &["qualification_binding_uses_the_governed_single_end_profile"],
    )
}

fn bedtools_coverage() -> ToolBindingV1 {
    retained_binding(
        "bam.coverage",
        "bedtools",
        &["bam_input", "genome_coverage", "native_bedgraph_output"],
        &[input("bam", BAM)],
        &[],
        &[output("coverage_bedgraph", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' genomecov -ibam '${input:bam}' -bga > '${output:coverage_bedgraph}'",
        ],
        "bijux.benchmark.bam.coverage.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::coverage",
        "benchmark.bam.coverage.compare",
        resources(1, 8, 8),
        &["bedgraph_is_preserved_before_tool_independent_coverage_normalization"],
    )
}

fn mosdepth_coverage() -> ToolBindingV1 {
    retained_binding(
        "bam.coverage",
        "mosdepth",
        &["bam_input", "parallel_coverage", "native_summary_and_distribution_outputs"],
        &[input("bam", BAM)],
        &[],
        &[
            output("coverage_summary", TABLE),
            output("coverage_distribution", TABLE),
        ],
        &[
            "/bin/sh",
            "-c",
            "prefix='${output:coverage_summary}.native'; '${executable}' --threads 4 --no-per-base \"$prefix\" '${input:bam}'; mv \"$prefix.mosdepth.summary.txt\" '${output:coverage_summary}'; mv \"$prefix.mosdepth.global.dist.txt\" '${output:coverage_distribution}'",
        ],
        "bijux.benchmark.bam.coverage.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::coverage",
        "benchmark.bam.coverage.compare",
        resources(4, 8, 8),
        &["native_summary_and_global_distribution_are_both_required_for_normalization"],
    )
}

fn samtools_coverage() -> ToolBindingV1 {
    retained_binding(
        "bam.coverage",
        "samtools",
        &["bam_input", "per_position_depth", "header_reference_span"],
        &[input("bam", BAM)],
        &[],
        &[
            output("depth_table", TABLE),
            output("reference_span", TABLE),
        ],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' depth '${input:bam}' > '${output:depth_table}'; '${executable}' view -H '${input:bam}' | awk -F '\\t' 'BEGIN {OFS=\"\\t\"} $1 == \"@SQ\" {for (i=2; i<=NF; i++) if ($i ~ /^LN:/) total += substr($i, 4)} END {if (total <= 0) exit 2; print \"reference_bases\", total}' > '${output:reference_span}'",
        ],
        "bijux.benchmark.bam.coverage.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::coverage",
        "benchmark.bam.coverage.compare",
        resources(1, 8, 8),
        &["zero_depth_positions_are_absent_and_reference_span_is_recovered_from_the_bam_header"],
    )
}

fn samtools_endogenous_content() -> ToolBindingV1 {
    retained_binding(
        "bam.endogenous_content",
        "samtools",
        &[
            "bam_input",
            "primary_read_counting",
            "mapped_primary_read_counting",
            "mapped_nonduplicate_primary_read_counting",
        ],
        &[input("bam", BAM)],
        &[],
        &[
            output("total_primary_reads", TABLE),
            output("mapped_primary_reads", TABLE),
            output("mapped_nonduplicate_primary_reads", TABLE),
        ],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' view -@ 4 -c -F 0x900 '${input:bam}' > '${output:total_primary_reads}' && '${executable}' view -@ 4 -c -F 0x904 '${input:bam}' > '${output:mapped_primary_reads}' && '${executable}' view -@ 4 -c -F 0xD04 '${input:bam}' > '${output:mapped_nonduplicate_primary_reads}'",
        ],
        "bijux.benchmark.bam.endogenous_content.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::endogenous_content",
        "benchmark.bam.endogenous_content.consume",
        resources(4, 8, 8),
        &[
            "primary_counts_exclude_secondary_and_supplementary_records",
            "mapped_primary_counts_also_exclude_unmapped_records",
            "mapped_nonduplicate_primary_counts_also_exclude_unmapped_and_duplicate_records",
        ],
    )
}

fn picard_mapping_summary() -> ToolBindingV1 {
    retained_binding(
        "bam.mapping_summary",
        "picard",
        &["bam_input", "alignment_summary_metrics", "pair_and_unpaired_categories"],
        &[input("bam", BAM)],
        &[],
        &[output("alignment_summary_metrics", TABLE)],
        &[
            "${executable}",
            "CollectAlignmentSummaryMetrics",
            "I=${input:bam}",
            "O=${output:alignment_summary_metrics}",
        ],
        "bijux.benchmark.bam.mapping_summary.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::mapping_summary",
        "benchmark.bam.mapping_summary.compare",
        resources(1, 16, 8),
        &["pair_and_unpaired_categories_are_aggregated_without_first_or_second_of_pair_rows"],
    )
}

fn picard_insert_size() -> ToolBindingV1 {
    retained_binding(
        "bam.insert_size",
        "picard",
        &["bam_input", "paired_template_distribution", "native_histogram_output"],
        &[input("bam", BAM)],
        &[],
        &[output("insert_size_metrics", TABLE), output("insert_size_histogram", FILE)],
        &[
            "${executable}",
            "CollectInsertSizeMetrics",
            "I=${input:bam}",
            "O=${output:insert_size_metrics}",
            "H=${output:insert_size_histogram}",
            "M=0.5",
        ],
        "bijux.benchmark.bam.insert_size.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::insert_size",
        "benchmark.bam.insert_size.consume",
        resources(1, 16, 16),
        &["paired_end_alignment_metadata_is_required_for_insert_size_inference"],
    )
}

fn picard_gc_bias() -> ToolBindingV1 {
    retained_binding(
        "bam.gc_bias",
        "picard",
        &["bam_input", "reference_indexing", "native_gc_bias_outputs"],
        &[input("bam", BAM), input("reference", FASTA)],
        &[],
        &[
            output("gc_bias_metrics", TABLE),
            output("gc_bias_summary", JSON),
            output("gc_bias_chart", FILE),
        ],
        &[
            "${executable}",
            "CollectGcBiasMetrics",
            "I=${input:bam}",
            "R=${input:reference}",
            "O=${output:gc_bias_metrics}",
            "S=${output:gc_bias_summary}",
            "CHART=${output:gc_bias_chart}",
        ],
        "bijux.benchmark.bam.gc_bias.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::gc_bias",
        "benchmark.bam.gc_bias.consume",
        resources(1, 16, 16),
        &["reference_bytes_and_gc_context_are_immutable_campaign_inputs"],
    )
}

fn picard_duplication_metrics() -> ToolBindingV1 {
    retained_binding(
        "bam.duplication_metrics",
        "picard",
        &["bam_input", "duplicate_detection", "native_duplication_metrics"],
        &[input("bam", BAM)],
        &[],
        &[output("duplication_metrics", TABLE), output("marked_bam", BAM)],
        &[
            "${executable}",
            "MarkDuplicates",
            "I=${input:bam}",
            "O=${output:marked_bam}",
            "M=${output:duplication_metrics}",
            "REMOVE_DUPLICATES=false",
            "ASSUME_SORT_ORDER=coordinate",
            "CREATE_INDEX=true",
            "VALIDATION_STRINGENCY=SILENT",
        ],
        "bijux.benchmark.bam.duplication_metrics.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::duplication_metrics",
        "benchmark.bam.duplication_metrics.compare",
        resources(1, 16, 16),
        &["picard_native_metrics_are_normalized_from_examined_and_duplicate_read_categories"],
    )
}

fn samtools_duplication_metrics() -> ToolBindingV1 {
    retained_binding(
        "bam.duplication_metrics",
        "samtools",
        &["bam_input", "duplicate_detection", "native_markdup_statistics"],
        &[input("bam", BAM)],
        &[],
        &[output("markdup_statistics", TABLE), output("marked_bam", BAM)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' markdup -@ 4 -s '${input:bam}' '${output:marked_bam}' 2> '${output:markdup_statistics}' && '${executable}' index -@ 4 '${output:marked_bam}'",
        ],
        "bijux.benchmark.bam.duplication_metrics.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::duplication_metrics",
        "benchmark.bam.duplication_metrics.compare",
        resources(4, 16, 16),
        &["input_bam_must_carry_the_mate_tags_required_by_samtools_markdup"],
    )
}

fn samtools_mapping_summary() -> ToolBindingV1 {
    retained_binding(
        "bam.mapping_summary",
        "samtools",
        &["bam_input", "primary_mapping_counts", "native_flagstat_output"],
        &[input("bam", BAM)],
        &[],
        &[output("flagstat", TABLE)],
        &["/bin/sh", "-c", "'${executable}' flagstat -@ 1 '${input:bam}' > '${output:flagstat}'"],
        "bijux.benchmark.bam.mapping_summary.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::mapping_summary",
        "benchmark.bam.mapping_summary.compare",
        resources(1, 8, 8),
        &["primary_counts_are_read_from_the_native_flagstat_categories"],
    )
}

fn rxy_sex() -> ToolBindingV1 {
    retained_binding(
        "bam.sex",
        "rxy",
        &["bam_input", "xy_chromosome_sex_inference", "native_json_report"],
        &[input("bam", BAM)],
        &["classification_profile", "minimum_sex_reads"],
        &[output("sex_report", JSON), output("sex_summary", JSON)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' --input '${input:bam}' --classification-profile '${parameter:classification_profile}' --minimum-sex-reads '${parameter:minimum_sex_reads}' > '${output:sex_report}' && cp '${output:sex_report}' '${output:sex_summary}'",
        ],
        "bijux.benchmark.bam.sex.bundle.v1",
        "parse_sex_json",
        "bijux.stage_probe.bam.sex.v1",
        resources(1, 8, 8),
        &[
            "classification_profile_must_match_the_governed_reference_and_interpretation_scope",
            "input_bam_must_be_indexed_and_expose_declared_x_and_y_contigs",
        ],
    )
}

fn rcorrector_correct_errors() -> ToolBindingV1 {
    retained_binding(
        "fastq.correct_errors",
        "rcorrector",
        &["paired_end_reads", "kmer_error_correction", "pair_preserving_outputs"],
        &[input("reads_r1", FASTQ_PAIRED), input("reads_r2", FASTQ_PAIRED)],
        &[],
        &[
            output("corrected_reads_r1", FASTQ_PAIRED),
            output("corrected_reads_r2", FASTQ_PAIRED),
        ],
        &[
            "/bin/sh",
            "-c",
            "work=\"$TMPDIR/bijux-rcorrector-work\"; rm -rf \"$work\"; mkdir -p \"$work\"; '${executable}' -t 4 -tmpd \"$work\" -od \"$work\" -1 '${input:reads_r1}' -2 '${input:reads_r2}'; find \"$work\" -type f \\( -name '*.cor.fq' -o -name '*.cor.fastq' -o -name '*.cor.fq.gz' -o -name '*.cor.fastq.gz' \\) | LC_ALL=C sort > \"$work/outputs.list\"; test \"$(wc -l < \"$work/outputs.list\" | tr -d '[:space:]')\" -eq 2; normalize() { src=$1; dest=$2; case \"$src\" in *.gz) cp -- \"$src\" \"$dest\" ;; *) gzip -c -- \"$src\" > \"$dest\" ;; esac; }; normalize \"$(sed -n '1p' \"$work/outputs.list\")\" '${output:corrected_reads_r1}'; normalize \"$(sed -n '2p' \"$work/outputs.list\")\" '${output:corrected_reads_r2}'",
        ],
        "bijux.benchmark.fastq.correct_errors.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::transform::correct_errors",
        "bijux.stage_probe.fastq.correct_errors.v1",
        resources(4, 8, 4),
        &[
            "paired_end_binding",
            "exactly_two_native_corrected_outputs_are_required",
            "lexical_native_output_order_must_preserve_read_one_before_read_two",
        ],
    )
}

fn umi_tools_extract_umis() -> ToolBindingV1 {
    retained_binding(
        "fastq.extract_umis",
        "umi_tools",
        &["paired_end_reads", "umi_extraction", "native_extraction_log"],
        &[input("reads_r1", FASTQ_PAIRED), input("reads_r2", FASTQ_PAIRED)],
        &["umi_pattern"],
        &[
            output("umi_reads_r1", FASTQ_PAIRED),
            output("umi_reads_r2", FASTQ_PAIRED),
            output("extraction_log", FILE),
        ],
        &[
            "${executable}",
            "extract",
            "--stdin",
            "${input:reads_r1}",
            "--stdout",
            "${output:umi_reads_r1}",
            "--read2-in",
            "${input:reads_r2}",
            "--read2-out",
            "${output:umi_reads_r2}",
            "--bc-pattern",
            "${parameter:umi_pattern}",
            "--log",
            "${output:extraction_log}",
        ],
        "bijux.benchmark.fastq.extract_umis.bundle.v1",
        "bijux_dna_planner_fastq::tool_adapters::stages::transform::extract_umis",
        "bijux.stage_probe.fastq.extract_umis.v1",
        resources(2, 4, 4),
        &[
            "paired_end_binding",
            "binding_extracts_the_umi_from_the_read1_prefix",
            "umi_pattern_must_match_the_library_preparation",
        ],
    )
}

/// Returns every canonical tool binding regardless of artifact distribution policy.
///
/// # Errors
///
/// Returns an error if any canonical binding violates conformance or duplicates ownership.
pub fn canonical_tool_bindings() -> Result<ToolBindingRegistry> {
    let mut registry = canonical_external_artifact_tool_bindings()?;
    for binding in canonical_retained_tool_bindings()?.bindings() {
        registry.insert(binding.clone())?;
    }
    Ok(registry)
}

fn bowtie2_align() -> ToolBindingV1 {
    retained_binding(
        "bam.align",
        "bowtie2",
        &["paired_end_reads", "fm_index_alignment", "native_sam_output"],
        &[
            input("reads_r1", FASTQ_PAIRED),
            input("reads_r2", FASTQ_PAIRED),
            input("reference_index", "bijux.auxiliary.artifact.index.v1"),
        ],
        &["threads", "read_group_id", "read_group_sample"],
        &[output("raw_sam", SAM)],
        &[
            "${executable}",
            "-x",
            "${input:reference_index}",
            "-1",
            "${input:reads_r1}",
            "-2",
            "${input:reads_r2}",
            "--very-sensitive",
            "--rg-id",
            "${parameter:read_group_id}",
            "--rg",
            "SM:${parameter:read_group_sample}",
            "-p",
            "${parameter:threads}",
            "-S",
            "${output:raw_sam}",
        ],
        "bijux.benchmark.bam.align.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::align",
        "benchmark.bam.align.compare",
        resources(8, 16, 16),
        &[
            "paired_end_binding",
            "canonical_preset_is_very_sensitive",
            "reference_index_must_match_the_admitted_reference_fasta",
        ],
    )
}

fn bwa_align() -> ToolBindingV1 {
    retained_binding(
        "bam.align",
        "bwa",
        &["paired_end_reads", "bwa_mem_alignment", "native_sam_output"],
        &[
            input("reads_r1", FASTQ_PAIRED),
            input("reads_r2", FASTQ_PAIRED),
            input("reference", FASTA),
        ],
        &["threads", "read_group"],
        &[output("raw_sam", SAM)],
        &[
            "${executable}",
            "mem",
            "-t",
            "${parameter:threads}",
            "-R",
            "${parameter:read_group}",
            "-o",
            "${output:raw_sam}",
            "${input:reference}",
            "${input:reads_r1}",
            "${input:reads_r2}",
        ],
        "bijux.benchmark.bam.align.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::align",
        "benchmark.bam.align.compare",
        resources(8, 16, 16),
        &["paired_end_binding", "canonical_preset_is_bwa_mem_default"],
    )
}

fn bcftools_genotyping() -> ToolBindingV1 {
    retained_binding(
        "bam.genotyping",
        "bcftools",
        &["bam_input", "candidate_site_genotyping", "indexed_vcf_output"],
        &[
            input("bam", BAM),
            input("reference", FASTA),
            input("candidate_regions", TABLE),
            input("candidate_alleles", TABLE),
        ],
        &[],
        &[output("called_vcf", VCF)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' mpileup -f '${input:reference}' -R '${input:candidate_regions}' -q 20 -Q 20 -a FORMAT/DP,FORMAT/AD -Ou '${input:bam}' | '${executable}' call --threads 2 -A -i -m -C alleles -T '${input:candidate_alleles}' -Oz -o '${output:called_vcf}' && '${executable}' index -f '${output:called_vcf}'",
        ],
        "bijux.benchmark.bam.genotyping.bundle.v1",
        "bijux_dna_planner_bam::tool_adapters::bam::genotyping",
        "benchmark.bam.genotyping.consume",
        resources(4, 16, 16),
        &[
            "candidate_sites_are_independent_evidence_not_sample_genotype_truth",
            "scientific_thresholds_are_applied_by_the_result_normalizer",
        ],
    )
}

#[allow(clippy::too_many_arguments)]
fn retained_binding(
    stage_id: &str,
    tool_id: &str,
    capabilities: &[&str],
    inputs: &[BindingInputV1],
    parameters: &[&str],
    outputs: &[BindingOutputV1],
    command_template: &[&str],
    normalized_output_contract_id: &str,
    normalizer_id: &str,
    downstream_probe_id: &str,
    resources: ToolConstraints,
    limitations: &[&str],
) -> ToolBindingV1 {
    ToolBindingV1 {
        schema_version: TOOL_BINDING_SCHEMA_V1.to_string(),
        binding_id: format!("{stage_id}__{tool_id}"),
        stage_id: StageId::new(stage_id),
        tool_id: ToolId::new(tool_id),
        artifact_source: ToolArtifactSourceV1::CatalogImage {
            catalog_tool_id: ToolId::new(tool_id),
        },
        capabilities: capabilities.iter().map(ToString::to_string).collect(),
        inputs: inputs.to_vec(),
        parameters: parameters
            .iter()
            .map(|name| BindingParameterV1 { name: (*name).to_string(), required: true })
            .collect(),
        outputs: outputs.to_vec(),
        normalized_output_contract_id: normalized_output_contract_id.to_string(),
        command_template: command_template.iter().map(ToString::to_string).collect(),
        resources,
        normalizer_id: normalizer_id.to_string(),
        downstream_probe_id: downstream_probe_id.to_string(),
        limitations: limitations.iter().map(ToString::to_string).collect(),
    }
}

fn angsd_kinship() -> ToolBindingV1 {
    external_binding(
        "bam.kinship",
        "angsd",
        &["bam_input", "pairwise_ibs"],
        &[input("bam", BAM)],
        &[output("kinship_prefix", TABLE)],
        &["${executable}", "-i", "${input:bam}", "-doIBS", "1", "-out", "${output:kinship_prefix}"],
        resources(8, 16, 8),
        "bijux_dna_domain_bam::metrics::normalize_kinship",
        &["native_angsd_prefix_requires_governed_kinship_normalization"],
    )
}

fn angsd_sex() -> ToolBindingV1 {
    external_binding(
        "bam.sex",
        "angsd",
        &["bam_input", "chromosome_counts"],
        &[input("bam", BAM)],
        &[output("counts_prefix", TABLE)],
        &[
            "${executable}",
            "-i",
            "${input:bam}",
            "-doCounts",
            "1",
            "-dumpCounts",
            "2",
            "-out",
            "${output:counts_prefix}",
        ],
        resources(8, 16, 8),
        "parse_sex_json",
        &["native_angsd_counts_require_governed_sex_normalization"],
    )
}

fn angsd_call_gl() -> ToolBindingV1 {
    external_binding(
        "vcf.call_gl",
        "angsd",
        &["bam_input", "genotype_likelihoods"],
        &[input("bam", BAM), input("reference", FASTA)],
        &[output("gl_sites_vcf", VCF)],
        &[
            "${executable}",
            "-i",
            "${input:bam}",
            "-ref",
            "${input:reference}",
            "-GL",
            "2",
            "-doGlf",
            "2",
            "-doMajorMinor",
            "1",
            "-doMaf",
            "1",
            "-out",
            "${output:gl_sites_vcf}",
        ],
        resources(8, 24, 16),
        "parse_angsd_call_gl_metrics",
        &["native_angsd_prefix_requires_governed_vcf_materialization"],
    )
}

fn angsd_call_pseudohaploid() -> ToolBindingV1 {
    external_binding(
        "vcf.call_pseudohaploid",
        "angsd",
        &["bam_input", "seeded_pseudohaploid_calling"],
        &[input("bam", BAM), input("reference", FASTA)],
        &[output("pseudohaploid_vcf", VCF)],
        &[
            "${executable}",
            "-i",
            "${input:bam}",
            "-ref",
            "${input:reference}",
            "-doHaploCall",
            "1",
            "-doCounts",
            "1",
            "-seed",
            "42",
            "-out",
            "${output:pseudohaploid_vcf}",
        ],
        resources(8, 24, 16),
        "parse_angsd_call_pseudohaploid_metrics",
        &["native_angsd_prefix_requires_governed_vcf_materialization"],
    )
}

fn fastuniq_remove_duplicates() -> ToolBindingV1 {
    external_binding(
        "fastq.remove_duplicates",
        "fastuniq",
        &["paired_end_reads", "pair_preserving_deduplication"],
        &[input("reads_r1", FASTQ_PAIRED), input("reads_r2", FASTQ_PAIRED)],
        &[output("dedup_reads_r1", FASTQ_PAIRED), output("dedup_reads_r2", FASTQ_PAIRED)],
        &[
            "/bin/bash",
            "-lc",
            "pair_list=$(mktemp); trap 'rm -f \"$pair_list\"' EXIT; printf '%s\\n%s\\n' '${input:reads_r1}' '${input:reads_r2}' > \"$pair_list\"; '${executable}' -i \"$pair_list\" -t q -o '${output:dedup_reads_r1}' -p '${output:dedup_reads_r2}'",
        ],
        resources(4, 8, 4),
        "parse_deduplicate_report",
        &["paired_end_only", "gzip_materialization_requires_the_governed_fastuniq_io_adapter"],
    )
}

fn impute5_imputation_metrics() -> ToolBindingV1 {
    external_binding(
        "vcf.imputation_metrics",
        "impute5",
        &["vcf_input", "imputation_quality_metrics"],
        &[input("imputed_vcf", VCF)],
        &[output("imputation_metrics_json", JSON)],
        &[
            "${executable}",
            "--g",
            "${input:imputed_vcf}",
            "--out-ap-field",
            "${output:imputation_metrics_json}",
        ],
        resources(8, 16, 8),
        "parse_impute5_imputation_metrics",
        &["metrics_projection_requires_governed_impute5_quality_adapter"],
    )
}

fn impute5_impute() -> ToolBindingV1 {
    external_binding(
        "vcf.impute",
        "impute5",
        &["phased_vcf_input", "reference_panel_imputation"],
        &[
            input("phased_vcf", VCF),
            input("reference_panel_vcf", VCF_PANEL),
            input("genetic_map", TABLE),
        ],
        &[output("imputed_vcf", VCF)],
        &[
            "${executable}",
            "--g",
            "${input:phased_vcf}",
            "--h",
            "${input:reference_panel_vcf}",
            "--m",
            "${input:genetic_map}",
            "--r",
            "1:1-1000000",
            "--o",
            "${output:imputed_vcf}",
            "--threads",
            "8",
            "--seed",
            "42",
        ],
        resources(8, 32, 32),
        "parse_impute5_impute_metrics",
        &["output_indexing_requires_the_governed_vcf_postprocessing_adapter"],
    )
}

fn ngsbriggs_damage() -> ToolBindingV1 {
    external_binding(
        "bam.damage",
        "ngsbriggs",
        &["bam_input", "briggs_damage_model"],
        &[input("bam", BAM), input("reference", FASTA)],
        &[output("damage_report", JSON)],
        &[
            "${executable}",
            "analyze",
            "--bam",
            "${input:bam}",
            "--reference",
            "${input:reference}",
            "--json-out",
            "${output:damage_report}",
            "--model",
            "nb",
        ],
        resources(8, 16, 8),
        "parse_ngsbriggs_json",
        &[],
    )
}

#[allow(clippy::too_many_arguments)]
fn external_binding(
    stage_id: &str,
    tool_id: &str,
    capabilities: &[&str],
    inputs: &[BindingInputV1],
    outputs: &[BindingOutputV1],
    command_template: &[&str],
    mut resources: ToolConstraints,
    normalizer_id: &str,
    limitations: &[&str],
) -> ToolBindingV1 {
    resources.runtime = "external_artifact".to_string();
    let mut binding_limitations = vec![
        "external_artifact_admission_required".to_string(),
        "artifact_redistribution_prohibited".to_string(),
        "benchmark_stage_interface_must_be_closed_before_case_admission".to_string(),
    ];
    binding_limitations.extend(limitations.iter().map(ToString::to_string));
    ToolBindingV1 {
        schema_version: TOOL_BINDING_SCHEMA_V1.to_string(),
        binding_id: format!("{stage_id}__{tool_id}"),
        stage_id: StageId::new(stage_id),
        tool_id: ToolId::new(tool_id),
        artifact_source: ToolArtifactSourceV1::ExternalArtifact {
            artifact_kind: "operator_provided_binary_or_apptainer_sif".to_string(),
            version_probe: version_probe(tool_id),
            checksum_required: true,
        },
        capabilities: capabilities.iter().map(ToString::to_string).collect(),
        inputs: inputs.to_vec(),
        parameters: vec![],
        outputs: outputs.to_vec(),
        normalized_output_contract_id: outputs
            .first()
            .map_or_else(String::new, |output| output.artifact_contract_id.clone()),
        command_template: command_template.iter().map(ToString::to_string).collect(),
        resources,
        normalizer_id: normalizer_id.to_string(),
        downstream_probe_id: format!("bijux.stage_probe.{stage_id}.v1"),
        limitations: binding_limitations,
    }
}

fn version_probe(tool_id: &str) -> Vec<String> {
    let flag = if tool_id == "fastuniq" { "-h" } else { "--version" };
    vec!["${executable}".to_string(), flag.to_string()]
}

fn input(name: &str, artifact_contract_id: &str) -> BindingInputV1 {
    BindingInputV1 {
        name: name.to_string(),
        artifact_contract_id: artifact_contract_id.to_string(),
        required: true,
    }
}

fn output(name: &str, artifact_contract_id: &str) -> BindingOutputV1 {
    BindingOutputV1 {
        name: name.to_string(),
        artifact_contract_id: artifact_contract_id.to_string(),
        discovery: OutputDiscoveryV1::ExactPath,
    }
}

fn resources(threads: u32, mem_gb: u32, tmp_gb: u32) -> ToolConstraints {
    ToolConstraints { runtime: "apptainer".to_string(), mem_gb, tmp_gb, threads }
}
