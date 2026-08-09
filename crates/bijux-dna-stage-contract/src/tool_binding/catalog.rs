use anyhow::Result;
use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};

use super::{
    BindingInputV1, BindingOutputV1, BindingParameterV1, OutputDiscoveryV1, ToolArtifactSourceV1,
    ToolBindingRegistry, ToolBindingV1, TOOL_BINDING_SCHEMA_V1,
};

const BAM: &str = "bijux.bam.artifact.coordinate_sorted_indexed.v1";
const FASTA: &str = "bijux.reference.artifact.fasta.v1";
const FASTQ_PAIRED: &str = "bijux.fastq.artifact.paired_end.v1";
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
        bcftools_genotyping(),
        bedtools_coverage(),
        bowtie2_align(),
        bwa_align(),
        mosdepth_coverage(),
        samtools_coverage(),
        samtools_endogenous_content(),
    ] {
        registry.insert(binding)?;
    }
    Ok(registry)
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
    resources: ToolConstraints,
    normalizer_id: &str,
    limitations: &[&str],
) -> ToolBindingV1 {
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
    ToolConstraints { runtime: "external_artifact".to_string(), mem_gb, tmp_gb, threads }
}
