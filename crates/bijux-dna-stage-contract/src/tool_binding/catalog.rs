use anyhow::Result;
use bijux_dna_core::contract::ToolConstraints;
use bijux_dna_core::ids::{StageId, ToolId};

use super::{
    BindingInputV1, BindingOutputV1, OutputDiscoveryV1, ToolArtifactSourceV1, ToolBindingRegistry,
    ToolBindingV1, TOOL_BINDING_SCHEMA_V1,
};

const BAM: &str = "bijux.bam.artifact.coordinate_sorted_indexed.v1";
const FASTA: &str = "bijux.reference.artifact.fasta.v1";
const FASTQ_PAIRED: &str = "bijux.fastq.artifact.paired_end.v1";
const JSON: &str = "bijux.auxiliary.artifact.json.v1";
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
