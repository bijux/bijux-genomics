use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod schema;
mod types;

pub use schema::{
    VCF_ADMIXTURE_OUTPUT_TRUTH_SCHEMA_VERSION, VCF_DAMAGE_FILTER_SUMMARY_SCHEMA_VERSION,
    VCF_DEMOGRAPHY_OUTPUT_TRUTH_SCHEMA_VERSION, VCF_DIPLOID_CALLING_BOUNDARY_SCHEMA_VERSION,
    VCF_FILTER_CONSEQUENCE_SCHEMA_VERSION, VCF_FILTER_OUTPUT_TRUTH_SCHEMA_VERSION,
    VCF_GENOTYPE_TRUTH_SCHEMA_VERSION, VCF_GL_WORKFLOW_BOUNDARY_SCHEMA_VERSION,
    VCF_IBD_OUTPUT_TRUTH_SCHEMA_VERSION, VCF_IMPUTATION_OUTPUT_TRUTH_SCHEMA_VERSION,
    VCF_NORMALIZATION_SUMMARY_SCHEMA_VERSION, VCF_PCA_OUTPUT_TRUTH_SCHEMA_VERSION,
    VCF_PHASING_OUTPUT_TRUTH_SCHEMA_VERSION, VCF_PHASING_WORKFLOW_BOUNDARY_SCHEMA_VERSION,
    VCF_POPULATION_STRUCTURE_OUTPUT_TRUTH_SCHEMA_VERSION,
    VCF_PSEUDOHAPLOID_CALLING_BOUNDARY_SCHEMA_VERSION, VCF_REFERENCE_CONTEXT_SCHEMA_VERSION,
    VCF_ROH_OUTPUT_TRUTH_SCHEMA_VERSION, VCF_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION,
    VCF_STATS_WORKFLOW_SCHEMA_VERSION, VCF_VALIDATION_SUMMARY_SCHEMA_VERSION,
};
pub use types::{
    VcfAdmixtureOutputTruthRowV1, VcfAdmixtureOutputTruthSummaryV1, VcfCallingBoundaryV1,
    VcfDamageFilterSummaryV1, VcfDemographyEstimateTruthRowV1,
    VcfDemographyInsufficientDataTruthV1, VcfDemographyOutputTruthSummaryV1,
    VcfFilterConsequenceV1, VcfFilterOutputTruthSummaryV1, VcfGenotypeTruthSummaryV1,
    VcfIbdInsufficientOverlapTruthV1, VcfIbdOutputTruthSummaryV1, VcfIbdPairTruthRowV1,
    VcfImputationOutputTruthSummaryV1, VcfLikelihoodWorkflowBoundaryV1, VcfNormalizationSummaryV1,
    VcfPcaOutputTruthSummaryV1, VcfPhasingOutputTruthSummaryV1, VcfPhasingWorkflowBoundaryV1,
    VcfPopulationCoordinateTruthRowV1, VcfPopulationDistanceTruthRowV1,
    VcfPopulationStructureOutputTruthRowV1, VcfPopulationStructureOutputTruthSummaryV1,
    VcfReferenceContextResolutionV1, VcfRohOutputTruthSummaryV1, VcfRohSampleTruthRowV1,
    VcfRohSegmentTruthRowV1, VcfScientificDriftArtifactDeltaV1, VcfScientificDriftChangeKind,
    VcfScientificDriftMetricDeltaV1, VcfScientificDriftReportV1, VcfScientificDriftSnapshotV1,
    VcfStatsWorkflowSummaryV1, VcfValidationSummaryV1,
};
#[cfg(test)]
const VCF_IMPUTATION_WORKFLOW_BOUNDARY_SCHEMA_VERSION: &str =
    "bijux.vcf.calling_boundary.imputation.v1";
#[cfg(test)]
const VCF_COHORT_QC_WORKFLOW_SCHEMA_VERSION: &str = "bijux.vcf.cohort_qc.v1";
#[cfg(test)]
const VCF_PCA_ADMIXTURE_GUARDRAIL_SCHEMA_VERSION: &str = "bijux.vcf.pca_admixture.v1";
#[cfg(test)]
const VCF_ROH_IBD_WORKFLOW_BOUNDARY_SCHEMA_VERSION: &str = "bijux.vcf.roh_ibd_boundary.v1";
#[cfg(test)]
const VCF_DEMOGRAPHY_REFUSAL_BOUNDARY_SCHEMA_VERSION: &str = "bijux.vcf.demography_refusal.v1";
#[cfg(test)]
const VCF_PANEL_REFERENCE_DRIFT_REPORT_SCHEMA_VERSION: &str = "bijux.vcf.panel_reference_drift.v1";
#[cfg(test)]
const VCF_STRUCTURAL_VARIANT_BOUNDARY_SCHEMA_VERSION: &str = "bijux.vcf.structural_variant.v1";
#[cfg(test)]
const VCF_ANNOTATION_PROVENANCE_WORKFLOW_SCHEMA_VERSION: &str =
    "bijux.vcf.annotation_provenance.v1";
#[cfg(test)]
const VCF_POPULATION_HANDOFF_BOUNDARY_SCHEMA_VERSION: &str = "bijux.vcf.population_handoff.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
struct VcfImputationWorkflowBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub backend: String,
    pub panel_id: Option<String>,
    pub map_id: Option<String>,
    pub prerequisites_passed: bool,
    pub panel_compatible: bool,
    pub map_compatible: bool,
    pub confidence: f64,
    pub simulation_mode: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfCohortQcSampleCaveatV1 {
    pub sample_id: String,
    pub missingness: f64,
    pub heterozygosity: Option<f64>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfCohortQcWorkflowSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub prerequisites_passed: bool,
    pub sample_count: u32,
    pub minimum_sample_count: u32,
    pub missingness_threshold: f64,
    pub heterozygosity_lower: f64,
    pub heterozygosity_upper: f64,
    pub high_missingness_samples: u32,
    pub heterozygosity_outlier_samples: u32,
    pub relatedness_flagged_pairs: u32,
    pub variants_in: u64,
    pub variants_after_filter: u64,
    pub variants_removed_by_filter: u64,
    #[serde(default)]
    pub per_sample: Vec<VcfCohortQcSampleCaveatV1>,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfPcaAdmixtureGuardrailV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub prerequisites_passed: bool,
    pub ld_pruned: bool,
    pub sample_inclusion_defined: bool,
    pub marker_count: u64,
    pub minimum_marker_count: u64,
    pub missingness_rate: f64,
    pub maximum_missingness_rate: f64,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfRohIbdWorkflowBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub prerequisites_passed: bool,
    pub marker_density_per_mb: f64,
    pub minimum_marker_density_per_mb: f64,
    pub cohort_size: u32,
    pub minimum_cohort_size: u32,
    pub missingness_rate: f64,
    pub maximum_missingness_rate: f64,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfDemographyRefusalBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub requested_model: String,
    pub prerequisites_passed: bool,
    pub cohort_size: u32,
    pub minimum_cohort_size: u32,
    #[serde(default)]
    pub missing_assumptions: Vec<String>,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct VcfPanelReferenceSnapshotV1 {
    pub label: String,
    pub reference_build: String,
    pub reference_fasta_sha256: String,
    pub panel_id: String,
    pub map_id: String,
    pub contig_alias_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct VcfPanelReferenceDriftReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub baseline_label: String,
    pub candidate_label: String,
    #[serde(default)]
    pub changed_surfaces: Vec<String>,
    #[serde(default)]
    pub invalidated_outputs: Vec<String>,
    pub requires_reprocessing: bool,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
struct VcfStructuralVariantBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub has_structural_variants: bool,
    pub explicit_sv_mode: bool,
    pub backend: Option<String>,
    pub requested_operation: String,
    pub support_declared: bool,
    #[serde(default)]
    pub supported_sv_types: Vec<String>,
    pub prerequisites_passed: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfAnnotationProvenanceWorkflowSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub annotation_source: String,
    pub annotation_version: String,
    #[serde(default)]
    pub requested_fields: Vec<String>,
    #[serde(default)]
    pub covered_fields: Vec<String>,
    pub field_coverage: f64,
    pub gene_mapping: Option<String>,
    pub transcript_mapping: Option<String>,
    pub prerequisites_passed: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
struct VcfPopulationAnalysisHandoffV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub target_analysis: String,
    pub filtered: bool,
    pub normalized: bool,
    pub sample_identity_match: bool,
    pub reference_identity_match: bool,
    pub trust_class_compatible: bool,
    pub caveats_attached: bool,
    pub prerequisites_passed: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
}

#[derive(Debug, Clone)]
struct TinyVcfRecord {
    chrom: String,
    pos: u64,
    ref_allele: String,
    alt_alleles: Vec<String>,
    qual: Option<f64>,
    filter: String,
    info: String,
    format: Option<String>,
    samples: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct TinyVcfDocument {
    contigs: BTreeSet<String>,
    info_definitions: BTreeSet<String>,
    format_definitions: BTreeSet<String>,
    samples: Vec<String>,
    records: Vec<TinyVcfRecord>,
}

fn parse_id_from_header_meta(line: &str, section: &str) -> Option<String> {
    let prefix = format!("##{section}=<ID=");
    let payload = line.strip_prefix(&prefix)?;
    payload.split([',', '>']).next().map(ToOwned::to_owned)
}

fn parse_contig_from_header_meta(line: &str) -> Option<String> {
    let payload = line.strip_prefix("##contig=<ID=")?;
    payload.split([',', '>']).next().map(ToOwned::to_owned)
}

fn parse_tiny_vcf(path: &Path) -> Result<TinyVcfDocument> {
    let raw = std::fs::read_to_string(path)?;
    let mut doc = TinyVcfDocument::default();
    let mut saw_header = false;
    for (line_index, raw_line) in raw.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() {
            continue;
        }
        if line.starts_with("##") {
            if let Some(contig) = parse_contig_from_header_meta(line) {
                doc.contigs.insert(contig);
            }
            if let Some(info_id) = parse_id_from_header_meta(line, "INFO") {
                doc.info_definitions.insert(info_id);
            }
            if let Some(format_id) = parse_id_from_header_meta(line, "FORMAT") {
                doc.format_definitions.insert(format_id);
            }
            continue;
        }
        if let Some(payload) = line.strip_prefix("#CHROM\t") {
            saw_header = true;
            let header_fields = payload.split('\t').collect::<Vec<_>>();
            if header_fields.len() >= 9 {
                doc.samples = header_fields[8..].iter().map(|value| (*value).to_string()).collect();
            }
            continue;
        }

        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 8 {
            return Err(anyhow!(
                "malformed VCF record at line {}: expected at least 8 fields",
                line_index + 1
            ));
        }
        let pos = fields[1].parse::<u64>().map_err(|error| {
            anyhow!(
                "malformed VCF record at line {}: invalid position {} ({error})",
                line_index + 1,
                fields[1]
            )
        })?;
        let qual = if fields[5] == "." {
            None
        } else {
            Some(fields[5].parse::<f64>().map_err(|error| {
                anyhow!(
                    "malformed VCF record at line {}: invalid QUAL {} ({error})",
                    line_index + 1,
                    fields[5]
                )
            })?)
        };
        let alt_alleles = fields[4]
            .split(',')
            .filter(|allele| !allele.is_empty())
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        let format =
            if fields.len() >= 9 && fields[8] != "." { Some(fields[8].to_string()) } else { None };
        let samples = if fields.len() >= 10 {
            fields[9..].iter().map(|value| (*value).to_string()).collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        doc.records.push(TinyVcfRecord {
            chrom: fields[0].to_string(),
            pos,
            ref_allele: fields[3].to_string(),
            alt_alleles,
            qual,
            filter: fields[6].to_string(),
            info: fields[7].to_string(),
            format,
            samples,
        });
    }
    if !saw_header {
        return Err(anyhow!("missing #CHROM header line"));
    }
    Ok(doc)
}

/// Execute fixture-safe VCF validation and return refusal reasons for malformed content.
///
/// # Errors
/// Returns an error if the VCF payload cannot be read or parsed at all.
pub fn execute_vcf_validation(
    input_vcf: &Path,
    expected_contigs: &[&str],
    require_index: bool,
    has_index: bool,
    declared_reference_build: Option<&str>,
    expected_reference_build: Option<&str>,
) -> Result<VcfValidationSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut refusal_codes = Vec::<String>::new();

    let mut duplicate_sample_names = BTreeSet::<String>::new();
    let mut observed_sample_names = BTreeSet::<String>::new();
    for sample in &doc.samples {
        if !observed_sample_names.insert(sample.clone()) {
            duplicate_sample_names.insert(sample.clone());
        }
    }
    if !duplicate_sample_names.is_empty() {
        refusal_codes.push("duplicate_sample_names".to_string());
    }
    let expected_contig_set =
        expected_contigs.iter().map(|value| (*value).to_string()).collect::<BTreeSet<_>>();
    if !expected_contig_set.is_empty() && !expected_contig_set.is_subset(&doc.contigs) {
        refusal_codes.push("missing_contig_header".to_string());
    }

    let mut sorted_records = true;
    let contig_rank = expected_contigs
        .iter()
        .enumerate()
        .map(|(index, contig)| ((*contig).to_string(), index))
        .collect::<HashMap<_, _>>();
    let mut previous_rank = 0_usize;
    let mut previous_pos = 0_u64;
    for (index, record) in doc.records.iter().enumerate() {
        if !doc.contigs.contains(&record.chrom) {
            refusal_codes.push("record_contig_missing_from_header".to_string());
        }
        if record.ref_allele.is_empty() || record.alt_alleles.is_empty() {
            refusal_codes.push("bad_ref_or_alt_allele".to_string());
            continue;
        }
        let record_rank = contig_rank
            .get(&record.chrom)
            .copied()
            .unwrap_or_else(|| expected_contigs.len() + index);
        if index > 0
            && (record_rank < previous_rank
                || (record_rank == previous_rank && record.pos < previous_pos))
        {
            sorted_records = false;
        }
        previous_rank = record_rank;
        previous_pos = record.pos;

        if record.info != "." {
            for token in record.info.split(';').filter(|token| !token.is_empty()) {
                let key = token.split('=').next().unwrap_or_default();
                if !key.is_empty()
                    && !doc.info_definitions.contains(key)
                    && !matches!(key, "END" | "DP" | "AC" | "AN")
                {
                    refusal_codes.push("bad_info_field_definition".to_string());
                    break;
                }
            }
        }
        if let Some(format) = &record.format {
            for key in format.split(':').filter(|token| !token.is_empty()) {
                if !doc.format_definitions.contains(key)
                    && !matches!(key, "GT" | "GQ" | "DP" | "PL")
                {
                    refusal_codes.push("bad_format_field_definition".to_string());
                    break;
                }
            }
        }
    }
    if !sorted_records {
        refusal_codes.push("unsorted_records".to_string());
    }
    if require_index && !has_index {
        refusal_codes.push("missing_index".to_string());
    }
    if let (Some(declared), Some(expected)) = (declared_reference_build, expected_reference_build) {
        if declared != expected {
            refusal_codes.push("reference_build_mismatch".to_string());
        }
    }

    refusal_codes.sort();
    refusal_codes.dedup();

    Ok(VcfValidationSummaryV1 {
        schema_version: VCF_VALIDATION_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.qc".to_string(),
        input_vcf: input_vcf.to_path_buf(),
        record_count: doc.records.len() as u64,
        sample_count: usize_to_u32_saturating(doc.samples.len()),
        header_valid: !doc.samples.iter().any(|value| value.trim().is_empty()),
        sorted_records,
        has_index,
        refusal_codes,
        notes: vec![
            "validation checks malformed headers, contigs, ordering, and INFO/FORMAT definitions"
                .to_string(),
        ],
    })
}

fn is_transition(ref_allele: &str, alt_allele: &str) -> bool {
    matches!((ref_allele, alt_allele), ("A", "G") | ("G", "A") | ("C", "T") | ("T", "C"))
}

fn parse_gt_from_sample<'a>(format: &'a str, sample_payload: &'a str) -> Option<&'a str> {
    parse_format_value_from_sample(format, sample_payload, "GT")
}

fn parse_format_value_from_sample<'a>(
    format: &'a str,
    sample_payload: &'a str,
    field_name: &str,
) -> Option<&'a str> {
    let keys = format.split(':').collect::<Vec<_>>();
    let values = sample_payload.split(':').collect::<Vec<_>>();
    let field_index = keys.iter().position(|key| *key == field_name)?;
    values.get(field_index).copied()
}

fn genotype_is_missing(gt: &str) -> bool {
    matches!(gt, "." | "./." | ".|." | "./" | ".|")
}

fn parse_called_genotype(gt: &str) -> Option<(bool, Vec<u32>)> {
    if genotype_is_missing(gt) {
        return None;
    }
    let (phased, alleles) = if gt.contains('|') {
        (true, gt.split('|').collect::<Vec<_>>())
    } else if gt.contains('/') {
        (false, gt.split('/').collect::<Vec<_>>())
    } else {
        (false, vec![gt])
    };
    if alleles.iter().any(|allele| allele.is_empty() || *allele == ".") {
        return None;
    }
    let parsed = alleles
        .into_iter()
        .map(str::parse::<u32>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .ok()?;
    Some((phased, parsed))
}

fn likelihood_value_present(value: &str) -> bool {
    !value.is_empty() && value.split(',').any(|token| token != ".")
}

fn usize_to_u32_saturating(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn u64_ratio_saturating(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    let numerator = u32::try_from(numerator).unwrap_or(u32::MAX);
    let denominator = u32::try_from(denominator).unwrap_or(u32::MAX);
    f64::from(numerator) / f64::from(denominator)
}

/// Build a bcftools-style stats summary for tiny fixture-safe VCF records.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
pub fn execute_vcf_stats_workflow(input_vcf: &Path) -> Result<VcfStatsWorkflowSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut snv_count = 0_u64;
    let mut indel_count = 0_u64;
    let mut transitions = 0_u64;
    let mut transversions = 0_u64;
    let mut missing_genotype_calls = 0_u64;
    let mut filter_counts = BTreeMap::<String, u64>::new();
    let mut missing_by_sample = vec![0_u64; doc.samples.len()];
    let mut total_by_sample = vec![0_u64; doc.samples.len()];
    for record in &doc.records {
        *filter_counts.entry(record.filter.clone()).or_insert(0) += 1;
        let is_snv_site =
            record.alt_alleles.iter().all(|alt| record.ref_allele.len() == 1 && alt.len() == 1);
        if is_snv_site {
            snv_count += 1;
            for alt in &record.alt_alleles {
                if is_transition(&record.ref_allele, alt) {
                    transitions += 1;
                } else {
                    transversions += 1;
                }
            }
        } else {
            indel_count += 1;
        }
        if let Some(format) = &record.format {
            for (sample_index, payload) in record.samples.iter().enumerate() {
                total_by_sample[sample_index] += 1;
                if let Some(gt) = parse_gt_from_sample(format, payload) {
                    if genotype_is_missing(gt) {
                        missing_genotype_calls += 1;
                        missing_by_sample[sample_index] += 1;
                    }
                }
            }
        }
    }
    let ti_tv_ratio = if transversions > 0 {
        Some(u64_ratio_saturating(transitions, transversions))
    } else {
        None
    };
    let per_sample_missingness = doc
        .samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let total = total_by_sample[index];
            let missing = missing_by_sample[index];
            let ratio = u64_ratio_saturating(missing, total);
            (sample.clone(), ratio)
        })
        .collect::<BTreeMap<_, _>>();
    Ok(VcfStatsWorkflowSummaryV1 {
        schema_version: VCF_STATS_WORKFLOW_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.stats".to_string(),
        variant_count: doc.records.len() as u64,
        sample_count: usize_to_u32_saturating(doc.samples.len()),
        snv_count,
        indel_count,
        ti_tv_ratio,
        missing_genotype_calls,
        filter_counts,
        per_sample_missingness,
        caveats: vec![
            "Ti/Tv is reported only when both transitions and transversions are observed"
                .to_string(),
            "missingness is derived from GT fields in fixture-safe records".to_string(),
        ],
    })
}

/// Summarize genotype-state, ploidy-width, missingness, and likelihood-field truth for a VCF.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
pub fn summarize_vcf_genotype_truth(
    input_vcf: &Path,
    stage_id: &str,
    tool_id: &str,
) -> Result<VcfGenotypeTruthSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let stats = execute_vcf_stats_workflow(input_vcf)?;
    let mut observed_ploidy_widths = BTreeSet::<u32>::new();
    let mut called_calls = 0_u64;
    let mut missing_calls = 0_u64;
    let mut reference_only_calls = 0_u64;
    let mut mixed_allele_calls = 0_u64;
    let mut alternate_only_calls = 0_u64;
    let mut phased_calls = 0_u64;
    let mut unphased_calls = 0_u64;
    let mut likelihood_fields_present = BTreeSet::<String>::new();
    let mut sites_with_likelihood_values = BTreeMap::<String, u64>::new();
    let mut sites_missing_likelihood_values = BTreeMap::<String, u64>::new();

    for record in &doc.records {
        let Some(format) = &record.format else {
            continue;
        };
        for field_name in ["GL", "GP", "PL"] {
            if !format.split(':').any(|field| field == field_name) {
                continue;
            }
            likelihood_fields_present.insert(field_name.to_string());
            let has_value = record.samples.iter().any(|sample_payload| {
                parse_format_value_from_sample(format, sample_payload, field_name)
                    .is_some_and(likelihood_value_present)
            });
            if has_value {
                *sites_with_likelihood_values.entry(field_name.to_string()).or_insert(0) += 1;
            } else {
                *sites_missing_likelihood_values.entry(field_name.to_string()).or_insert(0) += 1;
            }
        }
        for sample_payload in &record.samples {
            let Some(gt) = parse_gt_from_sample(format, sample_payload) else {
                continue;
            };
            let Some((phased, alleles)) = parse_called_genotype(gt) else {
                missing_calls += 1;
                continue;
            };
            called_calls += 1;
            observed_ploidy_widths.insert(usize_to_u32_saturating(alleles.len()));
            if phased {
                phased_calls += 1;
            } else {
                unphased_calls += 1;
            }
            if alleles.iter().all(|allele| *allele == 0) {
                reference_only_calls += 1;
            } else if alleles.iter().all(|allele| *allele > 0) {
                alternate_only_calls += 1;
            } else {
                mixed_allele_calls += 1;
            }
        }
    }

    Ok(VcfGenotypeTruthSummaryV1 {
        schema_version: VCF_GENOTYPE_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        input_vcf: input_vcf.to_path_buf(),
        sample_count: usize_to_u32_saturating(doc.samples.len()),
        variant_count: doc.records.len() as u64,
        observed_ploidy_widths: observed_ploidy_widths.into_iter().collect(),
        called_calls,
        missing_calls,
        reference_only_calls,
        mixed_allele_calls,
        alternate_only_calls,
        phased_calls,
        unphased_calls,
        per_sample_missingness: stats.per_sample_missingness,
        likelihood_fields_present: likelihood_fields_present.into_iter().collect(),
        sites_with_likelihood_values,
        sites_missing_likelihood_values,
    })
}

/// Summarize the labeled filter output observed in a VCF after filtering or damage-aware removal.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
pub fn summarize_vcf_filter_output_truth(
    input_vcf: &Path,
    stage_id: &str,
    tool_id: &str,
) -> Result<VcfFilterOutputTruthSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut pass_variant_count = 0_u64;
    let mut failed_variant_count = 0_u64;
    let mut observed_filter_ids = BTreeSet::<String>::new();
    let mut per_filter_variant_count = BTreeMap::<String, u64>::new();
    let mut per_filter_sites = BTreeMap::<String, Vec<String>>::new();
    let mut pass_sites = Vec::<String>::new();

    for record in &doc.records {
        let site_id = format!("{}:{}", record.chrom, record.pos);
        if matches!(record.filter.as_str(), "PASS" | ".") {
            pass_variant_count += 1;
            pass_sites.push(site_id);
            continue;
        }
        failed_variant_count += 1;
        for filter_id in record.filter.split(';').filter(|value| !value.is_empty()) {
            observed_filter_ids.insert(filter_id.to_string());
            *per_filter_variant_count.entry(filter_id.to_string()).or_insert(0) += 1;
            per_filter_sites.entry(filter_id.to_string()).or_default().push(site_id.clone());
        }
    }

    Ok(VcfFilterOutputTruthSummaryV1 {
        schema_version: VCF_FILTER_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        input_vcf: input_vcf.to_path_buf(),
        sample_count: usize_to_u32_saturating(doc.samples.len()),
        variant_count: doc.records.len() as u64,
        pass_variant_count,
        failed_variant_count,
        observed_filter_ids: observed_filter_ids.into_iter().collect(),
        per_filter_variant_count,
        per_filter_sites,
        pass_sites,
    })
}

/// Summarize phased genotype counts and phase-set evidence from a phasing-output VCF.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
pub fn summarize_vcf_phasing_output_truth(
    input_vcf: &Path,
    stage_id: &str,
    tool_id: &str,
) -> Result<VcfPhasingOutputTruthSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut called_genotype_count = 0_u64;
    let mut phased_genotype_count = 0_u64;
    let mut unphased_genotype_count = 0_u64;
    let mut phase_sets_by_sample = doc
        .samples
        .iter()
        .cloned()
        .map(|sample_id| (sample_id, BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();

    for record in &doc.records {
        let Some(format) = &record.format else {
            continue;
        };
        let ps_in_format = format.split(':').any(|field| field == "PS");
        for (sample_index, sample_payload) in record.samples.iter().enumerate() {
            let Some(gt) = parse_gt_from_sample(format, sample_payload) else {
                continue;
            };
            let Some((phased, _alleles)) = parse_called_genotype(gt) else {
                continue;
            };
            called_genotype_count += 1;
            if phased {
                phased_genotype_count += 1;
                if ps_in_format {
                    if let Some(ps) = parse_format_value_from_sample(format, sample_payload, "PS") {
                        let sample_id = &doc.samples[sample_index];
                        if !ps.is_empty() && ps != "." {
                            phase_sets_by_sample
                                .entry(sample_id.clone())
                                .or_default()
                                .insert(ps.to_string());
                        }
                    }
                }
            } else {
                unphased_genotype_count += 1;
            }
        }
    }

    let phase_sets_by_sample = phase_sets_by_sample
        .into_iter()
        .map(|(sample_id, phase_sets)| (sample_id, phase_sets.into_iter().collect::<Vec<_>>()))
        .collect::<BTreeMap<_, _>>();
    let phase_set_count = phase_sets_by_sample
        .values()
        .map(|phase_sets| u64::try_from(phase_sets.len()).unwrap_or(0))
        .sum();

    Ok(VcfPhasingOutputTruthSummaryV1 {
        schema_version: VCF_PHASING_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        input_vcf: input_vcf.to_path_buf(),
        sample_count: usize_to_u32_saturating(doc.samples.len()),
        sample_ids: doc.samples,
        variant_count: doc.records.len() as u64,
        called_genotype_count,
        phased_genotype_count,
        unphased_genotype_count,
        phase_set_count,
        phase_sets_by_sample,
    })
}

/// Summarize imputation-output truth from an imputed VCF plus optional masked-site truth VCF.
///
/// # Errors
/// Returns an error when the VCF or truth VCF cannot be parsed.
pub fn summarize_vcf_imputation_output_truth(
    input_vcf: &Path,
    truth_vcf: Option<&Path>,
    stage_id: &str,
    tool_id: &str,
) -> Result<VcfImputationOutputTruthSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut info_sum = 0.0_f64;
    let mut r2_sum = 0.0_f64;
    let mut sites_with_info_score = 0_u64;
    let mut sites_with_r2_score = 0_u64;

    for record in &doc.records {
        if let Some(info_score) = parse_record_info_metric(&record.info, "INFO") {
            info_sum += info_score;
            sites_with_info_score += 1;
        }
        if let Some(r2_score) = parse_record_info_metric(&record.info, "R2") {
            r2_sum += r2_score;
            sites_with_r2_score += 1;
        }
    }

    let truth_comparison = match truth_vcf {
        Some(path) => Some(compare_imputed_truth(&doc, &parse_tiny_vcf(path)?)?),
        None => None,
    };
    let mean_info_score = if sites_with_info_score > 0 {
        Some(info_sum / sites_with_info_score as f64)
    } else {
        None
    };
    let mean_r2_score =
        if sites_with_r2_score > 0 { Some(r2_sum / sites_with_r2_score as f64) } else { None };
    let genotype_concordance = truth_comparison.as_ref().and_then(|truth| {
        let resolved = truth.masked_truth_match_count + truth.masked_truth_mismatch_count;
        if resolved == 0 {
            None
        } else {
            Some(truth.masked_truth_match_count as f64 / resolved as f64)
        }
    });

    Ok(VcfImputationOutputTruthSummaryV1 {
        schema_version: VCF_IMPUTATION_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        input_vcf: input_vcf.to_path_buf(),
        truth_vcf: truth_vcf.map(Path::to_path_buf),
        sample_count: usize_to_u32_saturating(doc.samples.len()),
        sample_ids: doc.samples,
        variant_count: doc.records.len() as u64,
        sites_with_info_score,
        sites_with_r2_score,
        mean_info_score,
        mean_r2_score,
        masked_truth_site_count: truth_comparison
            .as_ref()
            .map_or(0, |truth| truth.masked_truth_site_count),
        masked_truth_match_count: truth_comparison
            .as_ref()
            .map_or(0, |truth| truth.masked_truth_match_count),
        masked_truth_mismatch_count: truth_comparison
            .as_ref()
            .map_or(0, |truth| truth.masked_truth_mismatch_count),
        unresolved_count: truth_comparison.as_ref().map_or(0, |truth| truth.unresolved_count),
        genotype_concordance,
    })
}

/// Summarize normalized ROH metrics into governed interval truth.
///
/// # Errors
/// Returns an error when normalized metrics drift away from their interval summaries.
pub fn summarize_vcf_roh_output_truth(
    metrics: &serde_json::Value,
) -> Result<VcfRohOutputTruthSummaryV1> {
    let stage_id = json_required_string(metrics, "stage_id")?;
    let tool_id = json_required_string(metrics, "tool_id")?;
    let status = json_required_string(metrics, "status")?;
    let sample_count = json_required_u64(metrics, "sample_count")?;
    let segment_count = json_required_u64(metrics, "segment_count")?;
    let total_length = json_required_u64(metrics, "total_length")?;
    let segments = json_required_array(metrics, "segments")?
        .iter()
        .map(|row| {
            Ok(VcfRohSegmentTruthRowV1 {
                sample_id: json_required_string(row, "sample_id")?,
                contig: json_required_string(row, "contig")?,
                start: json_required_u64(row, "start")?,
                end: json_required_u64(row, "end")?,
                length: json_required_u64(row, "length")?,
                variant_count: json_required_u64(row, "variant_count")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let per_sample_summary = json_required_array(metrics, "per_sample_summary")?
        .iter()
        .map(|row| {
            Ok(VcfRohSampleTruthRowV1 {
                sample_id: json_required_string(row, "sample_id")?,
                segment_count: json_required_u64(row, "segment_count")?,
                total_length: json_required_u64(row, "total_length")?,
                mean_length: json_required_f64(row, "mean_length")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    if segment_count != segments.len() as u64 {
        return Err(anyhow!(
            "ROH segment_count drifted: metrics=`{segment_count}`, rows=`{}`",
            segments.len()
        ));
    }
    if sample_count != per_sample_summary.len() as u64 {
        return Err(anyhow!(
            "ROH sample_count drifted: metrics=`{sample_count}`, per-sample rows=`{}`",
            per_sample_summary.len()
        ));
    }

    let mut observed_per_sample = BTreeMap::<String, (u64, u64)>::new();
    let mut observed_total_length = 0_u64;
    for segment in &segments {
        observed_total_length += segment.length;
        let entry = observed_per_sample.entry(segment.sample_id.clone()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += segment.length;
    }
    if total_length != observed_total_length {
        return Err(anyhow!(
            "ROH total_length drifted: metrics=`{total_length}`, intervals=`{observed_total_length}`"
        ));
    }
    for row in &per_sample_summary {
        let (observed_segment_count, observed_sample_total_length) =
            observed_per_sample.get(&row.sample_id).copied().ok_or_else(|| {
                anyhow!("ROH per-sample summary contains unknown sample `{}`", row.sample_id)
            })?;
        if row.segment_count != observed_segment_count
            || row.total_length != observed_sample_total_length
        {
            return Err(anyhow!(
                "ROH per-sample summary drifted for `{}`: summary=({}, {}), intervals=({}, {})",
                row.sample_id,
                row.segment_count,
                row.total_length,
                observed_segment_count,
                observed_sample_total_length
            ));
        }
        let observed_mean_length = if observed_segment_count == 0 {
            0.0
        } else {
            observed_sample_total_length as f64 / observed_segment_count as f64
        };
        if !f64s_match(row.mean_length, observed_mean_length) {
            return Err(anyhow!(
                "ROH mean_length drifted for `{}`: summary=`{}`, intervals=`{}`",
                row.sample_id,
                row.mean_length,
                observed_mean_length
            ));
        }
    }

    let sample_ids =
        collect_sorted_unique_strings(segments.iter().map(|row| row.sample_id.as_str()));
    if sample_count != sample_ids.len() as u64 {
        return Err(anyhow!(
            "ROH sample_count drifted from interval sample IDs: metrics=`{sample_count}`, intervals=`{}`",
            sample_ids.len()
        ));
    }

    Ok(VcfRohOutputTruthSummaryV1 {
        schema_version: VCF_ROH_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id,
        tool_id,
        status,
        sample_count: usize_to_u32_saturating(sample_ids.len()),
        segment_count,
        total_length,
        sample_ids,
        segments,
        per_sample_summary,
    })
}

/// Summarize normalized IBD metrics into governed pair truth.
///
/// # Errors
/// Returns an error when normalized metrics drift away from their insufficiency probe or pair rows.
pub fn summarize_vcf_ibd_output_truth(
    metrics: &serde_json::Value,
) -> Result<VcfIbdOutputTruthSummaryV1> {
    let stage_id = json_required_string(metrics, "stage_id")?;
    let tool_id = json_required_string(metrics, "tool_id")?;
    let status = json_required_string(metrics, "status")?;
    let insufficient_reason = json_optional_string(metrics, "insufficient_reason")?;
    let pair_count = json_required_u64(metrics, "pair_count")?;
    let rows = json_required_array(metrics, "rows")?
        .iter()
        .map(|row| {
            Ok(VcfIbdPairTruthRowV1 {
                sample_a: json_required_string(row, "sample_a")?,
                sample_b: json_required_string(row, "sample_b")?,
                segment_count: json_required_u64(row, "segment_count")?,
                total_length: json_required_f64(row, "total_length")?,
                overlap_marker_count: json_required_u64(row, "overlap_marker_count")?,
                status: json_required_string(row, "status")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if pair_count != rows.len() as u64 {
        return Err(anyhow!(
            "IBD pair_count drifted: metrics=`{pair_count}`, rows=`{}`",
            rows.len()
        ));
    }

    let insufficient_overlap_probe_value =
        metrics.get("insufficient_overlap_probe").ok_or_else(|| {
            anyhow!("normalized metrics are missing object field `insufficient_overlap_probe`")
        })?;
    let insufficient_overlap_probe = VcfIbdInsufficientOverlapTruthV1 {
        status: json_required_string(insufficient_overlap_probe_value, "status")?,
        insufficient_reason: json_optional_string(
            insufficient_overlap_probe_value,
            "insufficient_reason",
        )?,
        filtered_segment_count: json_required_u64(
            insufficient_overlap_probe_value,
            "filtered_segment_count",
        )?,
    };

    match status.as_str() {
        "complete" => {
            if insufficient_reason.is_some() {
                return Err(anyhow!(
                    "IBD complete metrics must not declare an insufficient_reason"
                ));
            }
            if insufficient_overlap_probe.status != "not_run"
                || insufficient_overlap_probe.insufficient_reason.is_some()
                || insufficient_overlap_probe.filtered_segment_count != 0
            {
                return Err(anyhow!(
                    "IBD complete metrics must keep the insufficient-overlap probe in `not_run` state"
                ));
            }
        }
        "insufficient_marker_overlap" => {
            if insufficient_overlap_probe.status != status
                || insufficient_overlap_probe.insufficient_reason != insufficient_reason
            {
                return Err(anyhow!(
                    "IBD insufficient-overlap probe drifted from top-level status"
                ));
            }
        }
        other => {
            return Err(anyhow!("unsupported IBD output truth status `{other}`"));
        }
    }

    let retained_segment_count = rows.iter().map(|row| row.segment_count).sum::<u64>();
    let total_length = rows.iter().map(|row| row.total_length).sum::<f64>();
    let overlap_marker_total = rows.iter().map(|row| row.overlap_marker_count).sum::<u64>();
    let sample_ids = collect_sorted_unique_strings(
        rows.iter().flat_map(|row| [row.sample_a.as_str(), row.sample_b.as_str()].into_iter()),
    );

    Ok(VcfIbdOutputTruthSummaryV1 {
        schema_version: VCF_IBD_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id,
        tool_id,
        status,
        insufficient_reason,
        pair_count,
        retained_segment_count,
        total_length,
        overlap_marker_total,
        sample_ids,
        rows,
        insufficient_overlap_probe,
    })
}

/// Summarize normalized demography metrics into governed estimate truth.
///
/// # Errors
/// Returns an error when normalized metrics drift away from their time-bin or insufficiency contract.
pub fn summarize_vcf_demography_output_truth(
    metrics: &serde_json::Value,
) -> Result<VcfDemographyOutputTruthSummaryV1> {
    let stage_id = json_required_string(metrics, "stage_id")?;
    let tool_id = json_required_string(metrics, "tool_id")?;
    let method = json_required_string(metrics, "method")?;
    let inference_status = json_required_string(metrics, "inference_status")?;
    let status = json_required_string(metrics, "status")?;
    let insufficient_reason = json_optional_string(metrics, "insufficient_reason")?;
    let time_bins = json_required_array(metrics, "time_bins")?
        .iter()
        .map(|value| {
            value.as_u64().ok_or_else(|| anyhow!("demography time_bins must contain only integers"))
        })
        .collect::<Result<Vec<_>>>()?;
    let ne_estimates = json_required_array(metrics, "ne_estimates")?
        .iter()
        .map(|row| {
            Ok(VcfDemographyEstimateTruthRowV1 {
                generation: json_required_u64(row, "generation")?,
                ne: json_required_f64(row, "ne")?,
                ci_low: json_required_f64(row, "ci_low")?,
                ci_high: json_required_f64(row, "ci_high")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if time_bins.len() != ne_estimates.len() {
        return Err(anyhow!(
            "demography time_bins drifted from ne_estimates: bins=`{}`, estimates=`{}`",
            time_bins.len(),
            ne_estimates.len()
        ));
    }
    for (time_bin, estimate) in time_bins.iter().zip(&ne_estimates) {
        if *time_bin != estimate.generation {
            return Err(anyhow!(
                "demography estimate generation drifted from time_bins: bin=`{time_bin}`, generation=`{}`",
                estimate.generation
            ));
        }
    }

    let insufficient_data_probe_value =
        metrics.get("insufficient_data_probe").ok_or_else(|| {
            anyhow!("normalized metrics are missing object field `insufficient_data_probe`")
        })?;
    let insufficient_probe_time_bins =
        json_required_array(insufficient_data_probe_value, "time_bins")?
            .iter()
            .map(|value| {
                value.as_u64().ok_or_else(|| {
                    anyhow!(
                        "demography insufficient_data_probe time_bins must contain only integers"
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
    let insufficient_probe_estimates =
        json_required_array(insufficient_data_probe_value, "ne_estimates")?
            .iter()
            .map(|row| {
                Ok(VcfDemographyEstimateTruthRowV1 {
                    generation: json_required_u64(row, "generation")?,
                    ne: json_required_f64(row, "ne")?,
                    ci_low: json_required_f64(row, "ci_low")?,
                    ci_high: json_required_f64(row, "ci_high")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
    let insufficient_data_probe = VcfDemographyInsufficientDataTruthV1 {
        status: json_required_string(insufficient_data_probe_value, "status")?,
        insufficient_reason: json_optional_string(
            insufficient_data_probe_value,
            "insufficient_reason",
        )?,
        time_bins: insufficient_probe_time_bins,
        ne_estimates: insufficient_probe_estimates,
    };

    match status.as_str() {
        "complete" => {
            if insufficient_reason.is_some() {
                return Err(anyhow!(
                    "demography complete metrics must not declare an insufficient_reason"
                ));
            }
            if insufficient_data_probe.status != "not_run"
                || insufficient_data_probe.insufficient_reason.is_some()
                || !insufficient_data_probe.time_bins.is_empty()
                || !insufficient_data_probe.ne_estimates.is_empty()
            {
                return Err(anyhow!(
                    "demography complete metrics must keep the insufficient-data probe in `not_run` state"
                ));
            }
        }
        "insufficient_data" => {
            if insufficient_reason.is_none() {
                return Err(anyhow!(
                    "demography insufficient_data metrics must declare an insufficient_reason"
                ));
            }
            if !time_bins.is_empty() || !ne_estimates.is_empty() {
                return Err(anyhow!(
                    "demography insufficient_data metrics must not retain estimate rows"
                ));
            }
            if insufficient_data_probe.status != status
                || insufficient_data_probe.insufficient_reason != insufficient_reason
                || !insufficient_data_probe.time_bins.is_empty()
                || !insufficient_data_probe.ne_estimates.is_empty()
            {
                return Err(anyhow!(
                    "demography insufficient_data probe drifted from top-level status"
                ));
            }
        }
        other => {
            return Err(anyhow!("unsupported demography output truth status `{other}`"));
        }
    }

    Ok(VcfDemographyOutputTruthSummaryV1 {
        schema_version: VCF_DEMOGRAPHY_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id,
        tool_id,
        method,
        inference_status,
        status,
        insufficient_reason,
        estimate_count: ne_estimates.len() as u64,
        time_bins,
        ne_estimates,
        insufficient_data_probe,
    })
}

#[derive(Debug, Clone)]
struct PopulationMetadataRow {
    sample_id: String,
    population_id: String,
    sex: String,
    role: String,
}

#[derive(Debug, Clone)]
struct CoordinateRow {
    sample_id: String,
    population_id: String,
    pc1: f64,
    pc2: f64,
}

/// Summarize normalized PCA metrics against governed sample metadata.
///
/// # Errors
/// Returns an error when normalized metrics cannot be joined to sample metadata.
pub fn summarize_vcf_pca_output_truth(
    metrics: &serde_json::Value,
    sample_metadata_path: &Path,
) -> Result<VcfPcaOutputTruthSummaryV1> {
    let stage_id = json_required_string(metrics, "stage_id")?;
    let tool_id = json_required_string(metrics, "tool_id")?;
    let metadata_rows = parse_population_metadata_rows(sample_metadata_path)?;
    let metadata_by_sample =
        metadata_rows.iter().map(|row| (row.sample_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let coordinates = parse_coordinate_rows(json_required_array(metrics, "rows")?)?;

    let rows = coordinates
        .iter()
        .map(|coordinate| {
            let metadata =
                metadata_by_sample.get(coordinate.sample_id.as_str()).ok_or_else(|| {
                    anyhow!(
                        "PCA metrics sample `{}` is missing from metadata `{}`",
                        coordinate.sample_id,
                        sample_metadata_path.display()
                    )
                })?;
            if coordinate.population_id != metadata.population_id {
                return Err(anyhow!(
                    "PCA metrics sample `{}` population drifted: metrics=`{}`, metadata=`{}`",
                    coordinate.sample_id,
                    coordinate.population_id,
                    metadata.population_id
                ));
            }
            Ok(VcfPopulationCoordinateTruthRowV1 {
                sample_id: coordinate.sample_id.clone(),
                population_id: coordinate.population_id.clone(),
                sex: metadata.sex.clone(),
                role: metadata.role.clone(),
                pc1: coordinate.pc1,
                pc2: coordinate.pc2,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let pairwise_distances = build_population_distance_rows(&rows)?;
    let population_ids = collect_population_ids(rows.iter().map(|row| row.population_id.as_str()));

    Ok(VcfPcaOutputTruthSummaryV1 {
        schema_version: VCF_PCA_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id,
        tool_id,
        sample_metadata_path: sample_metadata_path.to_path_buf(),
        sample_count: usize_to_u32_saturating(rows.len()),
        joined_sample_count: usize_to_u32_saturating(rows.len()),
        pair_count: pairwise_distances.len() as u64,
        population_ids,
        rows,
        pairwise_distances,
    })
}

/// Summarize normalized admixture metrics against governed sample metadata.
///
/// # Errors
/// Returns an error when normalized metrics cannot be joined to sample metadata.
pub fn summarize_vcf_admixture_output_truth(
    metrics: &serde_json::Value,
    sample_metadata_path: &Path,
) -> Result<VcfAdmixtureOutputTruthSummaryV1> {
    let stage_id = json_required_string(metrics, "stage_id")?;
    let tool_id = json_required_string(metrics, "tool_id")?;
    let selected_k = json_required_u64(metrics, "selected_k")?;
    let population_count = json_required_u64(metrics, "population_count")?;
    let cluster_headers = json_required_array(metrics, "cluster_headers")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("admixture cluster_headers must contain only strings"))
        })
        .collect::<Result<Vec<_>>>()?;
    let metadata_rows = parse_population_metadata_rows(sample_metadata_path)?;
    let metadata_by_sample =
        metadata_rows.iter().map(|row| (row.sample_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let metric_rows = json_required_array(metrics, "rows")?;
    let rows = metric_rows
        .iter()
        .map(|row| {
            let sample_id = json_required_string(row, "sample_id")?;
            let population_id = json_required_string(row, "population_id")?;
            let metadata = metadata_by_sample.get(sample_id.as_str()).ok_or_else(|| {
                anyhow!(
                    "admixture metrics sample `{sample_id}` is missing from metadata `{}`",
                    sample_metadata_path.display()
                )
            })?;
            if population_id != metadata.population_id {
                return Err(anyhow!(
                    "admixture metrics sample `{sample_id}` population drifted: metrics=`{population_id}`, metadata=`{}`",
                    metadata.population_id
                ));
            }
            let cluster_fractions = cluster_headers
                .iter()
                .map(|header| Ok((header.clone(), json_required_f64(row, header)?)))
                .collect::<Result<BTreeMap<_, _>>>()?;
            let (dominant_cluster, dominant_fraction) =
                dominant_cluster_fraction(&cluster_fractions, &sample_id)?;
            Ok(VcfAdmixtureOutputTruthRowV1 {
                sample_id,
                population_id,
                sex: metadata.sex.clone(),
                role: metadata.role.clone(),
                status: json_required_string(row, "status")?,
                dominant_cluster,
                dominant_fraction,
                cluster_fractions,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let population_ids = collect_population_ids(rows.iter().map(|row| row.population_id.as_str()));

    Ok(VcfAdmixtureOutputTruthSummaryV1 {
        schema_version: VCF_ADMIXTURE_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id,
        tool_id,
        sample_metadata_path: sample_metadata_path.to_path_buf(),
        selected_k,
        sample_count: usize_to_u32_saturating(rows.len()),
        joined_sample_count: usize_to_u32_saturating(rows.len()),
        population_count,
        cluster_headers,
        population_ids,
        rows,
    })
}

/// Summarize normalized population-structure metrics against governed sample metadata.
///
/// # Errors
/// Returns an error when normalized metrics cannot be joined to sample metadata.
pub fn summarize_vcf_population_structure_output_truth(
    metrics: &serde_json::Value,
    sample_metadata_path: &Path,
) -> Result<VcfPopulationStructureOutputTruthSummaryV1> {
    let stage_id = json_required_string(metrics, "stage_id")?;
    let tool_id = json_required_string(metrics, "tool_id")?;
    let metadata_rows = parse_population_metadata_rows(sample_metadata_path)?;
    let metadata_by_sample =
        metadata_rows.iter().map(|row| (row.sample_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let metric_rows = json_required_array(metrics, "sample_groups")?;
    let sample_groups = metric_rows
        .iter()
        .map(|row| {
            let sample_id = json_required_string(row, "sample_id")?;
            let population_id = json_required_string(row, "population_id")?;
            let metadata = metadata_by_sample.get(sample_id.as_str()).ok_or_else(|| {
                anyhow!(
                    "population-structure metrics sample `{sample_id}` is missing from metadata `{}`",
                    sample_metadata_path.display()
                )
            })?;
            if population_id != metadata.population_id {
                return Err(anyhow!(
                    "population-structure metrics sample `{sample_id}` population drifted: metrics=`{population_id}`, metadata=`{}`",
                    metadata.population_id
                ));
            }
            Ok(VcfPopulationStructureOutputTruthRowV1 {
                sample_id,
                population_id,
                sex: metadata.sex.clone(),
                role: metadata.role.clone(),
                dominant_cluster: json_required_string(row, "dominant_cluster")?,
                dominant_fraction: json_required_f64(row, "dominant_fraction")?,
                pc1: json_required_f64(row, "pc1")?,
                pc2: json_required_f64(row, "pc2")?,
                status: json_required_string(row, "status")?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let pairwise_distances = build_population_distance_rows_from_structure(&sample_groups)?;
    let (
        within_population_pair_count,
        cross_population_pair_count,
        min_pc_distance,
        max_pc_distance,
        mean_pc_distance,
    ) = summarize_population_distance_rows(&pairwise_distances);
    let population_ids =
        collect_population_ids(sample_groups.iter().map(|row| row.population_id.as_str()));

    Ok(VcfPopulationStructureOutputTruthSummaryV1 {
        schema_version: VCF_POPULATION_STRUCTURE_OUTPUT_TRUTH_SCHEMA_VERSION.to_string(),
        stage_id,
        tool_id,
        sample_metadata_path: sample_metadata_path.to_path_buf(),
        sample_count: usize_to_u32_saturating(sample_groups.len()),
        joined_sample_count: usize_to_u32_saturating(sample_groups.len()),
        pair_count: pairwise_distances.len() as u64,
        within_population_pair_count,
        cross_population_pair_count,
        min_pc_distance,
        max_pc_distance,
        mean_pc_distance,
        population_ids,
        sample_groups,
        pairwise_distances,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImputedTruthComparison {
    masked_truth_site_count: u64,
    masked_truth_match_count: u64,
    masked_truth_mismatch_count: u64,
    unresolved_count: u64,
}

fn compare_imputed_truth(
    doc: &TinyVcfDocument,
    truth: &TinyVcfDocument,
) -> Result<ImputedTruthComparison> {
    let output_sample_index = doc
        .samples
        .iter()
        .enumerate()
        .map(|(idx, sample_id)| (sample_id.clone(), idx))
        .collect::<BTreeMap<_, _>>();
    let truth_sample_index = truth
        .samples
        .iter()
        .enumerate()
        .map(|(idx, sample_id)| (sample_id.clone(), idx))
        .collect::<BTreeMap<_, _>>();
    let output_records = doc
        .records
        .iter()
        .map(|record| (format!("{}:{}", record.chrom, record.pos), record))
        .collect::<BTreeMap<_, _>>();

    let mut masked_truth_site_count = 0_u64;
    let mut masked_truth_match_count = 0_u64;
    let mut masked_truth_mismatch_count = 0_u64;
    let mut unresolved_count = 0_u64;

    for truth_record in &truth.records {
        let site_id = format!("{}:{}", truth_record.chrom, truth_record.pos);
        let Some(output_record) = output_records.get(&site_id) else {
            for truth_payload in &truth_record.samples {
                let gt = truth_record
                    .format
                    .as_deref()
                    .and_then(|format| parse_gt_from_sample(format, truth_payload));
                if gt.is_some_and(|value| !genotype_is_missing(value)) {
                    unresolved_count += 1;
                }
            }
            continue;
        };
        let Some(truth_format) = truth_record.format.as_deref() else {
            continue;
        };
        let Some(output_format) = output_record.format.as_deref() else {
            continue;
        };
        for (sample_id, truth_index) in &truth_sample_index {
            let truth_payload = truth_record.samples.get(*truth_index).ok_or_else(|| {
                anyhow!("truth VCF row `{site_id}` is missing genotype for `{sample_id}`")
            })?;
            let Some(truth_gt) = parse_gt_from_sample(truth_format, truth_payload) else {
                continue;
            };
            if genotype_is_missing(truth_gt) {
                continue;
            }
            masked_truth_site_count += 1;
            let Some(output_index) = output_sample_index.get(sample_id) else {
                unresolved_count += 1;
                continue;
            };
            let output_payload = output_record.samples.get(*output_index).ok_or_else(|| {
                anyhow!("imputed VCF row `{site_id}` is missing genotype for `{sample_id}`")
            })?;
            let Some(output_gt) = parse_gt_from_sample(output_format, output_payload) else {
                unresolved_count += 1;
                continue;
            };
            if genotype_is_missing(output_gt) {
                unresolved_count += 1;
                continue;
            }
            if canonicalize_genotype_for_truth(output_gt)
                == canonicalize_genotype_for_truth(truth_gt)
            {
                masked_truth_match_count += 1;
            } else {
                masked_truth_mismatch_count += 1;
            }
        }
    }

    Ok(ImputedTruthComparison {
        masked_truth_site_count,
        masked_truth_match_count,
        masked_truth_mismatch_count,
        unresolved_count,
    })
}

fn canonicalize_genotype_for_truth(gt: &str) -> String {
    gt.replace('|', "/")
}

fn parse_record_info_metric(info: &str, key: &str) -> Option<f64> {
    info.split(';').find_map(|entry| {
        let (entry_key, value) = entry.split_once('=')?;
        if entry_key == key {
            value.parse::<f64>().ok()
        } else {
            None
        }
    })
}

fn parse_population_metadata_rows(path: &Path) -> Result<Vec<PopulationMetadataRow>> {
    let raw = fs::read_to_string(path)?;
    let mut lines = raw.lines();
    let header =
        lines.next().ok_or_else(|| anyhow!("population metadata `{}` is empty", path.display()))?;
    let expected = ["sample_id", "population_id", "sex", "role"];
    for column in expected {
        if !header.split('\t').any(|value| value == column) {
            return Err(anyhow!(
                "population metadata `{}` is missing required column `{column}`",
                path.display()
            ));
        }
    }
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let columns = line.split('\t').collect::<Vec<_>>();
            if columns.len() < 4 {
                return Err(anyhow!(
                    "population metadata `{}` contains malformed row `{line}`",
                    path.display()
                ));
            }
            Ok(PopulationMetadataRow {
                sample_id: columns[0].to_string(),
                population_id: columns[1].to_string(),
                sex: columns[2].to_string(),
                role: columns[3].to_string(),
            })
        })
        .collect()
}

fn parse_coordinate_rows(rows: &[serde_json::Value]) -> Result<Vec<CoordinateRow>> {
    rows.iter()
        .map(|row| {
            Ok(CoordinateRow {
                sample_id: json_required_string(row, "sample_id")?,
                population_id: json_required_string(row, "population_id")?,
                pc1: json_required_f64(row, "pc1")?,
                pc2: json_required_f64(row, "pc2")?,
            })
        })
        .collect()
}

fn build_population_distance_rows(
    rows: &[VcfPopulationCoordinateTruthRowV1],
) -> Result<Vec<VcfPopulationDistanceTruthRowV1>> {
    let coordinates = rows
        .iter()
        .map(|row| CoordinateRow {
            sample_id: row.sample_id.clone(),
            population_id: row.population_id.clone(),
            pc1: row.pc1,
            pc2: row.pc2,
        })
        .collect::<Vec<_>>();
    build_population_distance_rows_from_coordinates(&coordinates)
}

fn build_population_distance_rows_from_structure(
    rows: &[VcfPopulationStructureOutputTruthRowV1],
) -> Result<Vec<VcfPopulationDistanceTruthRowV1>> {
    let coordinates = rows
        .iter()
        .map(|row| CoordinateRow {
            sample_id: row.sample_id.clone(),
            population_id: row.population_id.clone(),
            pc1: row.pc1,
            pc2: row.pc2,
        })
        .collect::<Vec<_>>();
    build_population_distance_rows_from_coordinates(&coordinates)
}

fn build_population_distance_rows_from_coordinates(
    rows: &[CoordinateRow],
) -> Result<Vec<VcfPopulationDistanceTruthRowV1>> {
    let mut distances = Vec::new();
    for left_index in 0..rows.len() {
        for right_index in (left_index + 1)..rows.len() {
            let left = &rows[left_index];
            let right = &rows[right_index];
            let delta_pc1 = left.pc1 - right.pc1;
            let delta_pc2 = left.pc2 - right.pc2;
            distances.push(VcfPopulationDistanceTruthRowV1 {
                left_sample_id: left.sample_id.clone(),
                right_sample_id: right.sample_id.clone(),
                left_population_id: left.population_id.clone(),
                right_population_id: right.population_id.clone(),
                distance: (delta_pc1.powi(2) + delta_pc2.powi(2)).sqrt(),
            });
        }
    }
    Ok(distances)
}

fn summarize_population_distance_rows(
    rows: &[VcfPopulationDistanceTruthRowV1],
) -> (u64, u64, f64, f64, f64) {
    let within_population_pair_count =
        rows.iter().filter(|row| row.left_population_id == row.right_population_id).count() as u64;
    let cross_population_pair_count = rows.len() as u64 - within_population_pair_count;
    let min_pc_distance = rows.iter().map(|row| row.distance).reduce(f64::min).unwrap_or(0.0);
    let max_pc_distance = rows.iter().map(|row| row.distance).reduce(f64::max).unwrap_or(0.0);
    let mean_pc_distance = if rows.is_empty() {
        0.0
    } else {
        rows.iter().map(|row| row.distance).sum::<f64>() / rows.len() as f64
    };
    (
        within_population_pair_count,
        cross_population_pair_count,
        min_pc_distance,
        max_pc_distance,
        mean_pc_distance,
    )
}

fn dominant_cluster_fraction(
    cluster_fractions: &BTreeMap<String, f64>,
    sample_id: &str,
) -> Result<(String, f64)> {
    cluster_fractions
        .iter()
        .max_by(|left, right| left.1.partial_cmp(right.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(cluster_id, fraction)| (cluster_id.clone(), *fraction))
        .ok_or_else(|| {
            anyhow!("admixture metrics sample `{sample_id}` is missing cluster fractions")
        })
}

fn collect_population_ids<'a>(population_ids: impl Iterator<Item = &'a str>) -> Vec<String> {
    collect_sorted_unique_strings(population_ids)
}

fn collect_sorted_unique_strings<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut values = values.map(str::to_string).collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn json_required_array<'a>(
    value: &'a serde_json::Value,
    field: &str,
) -> Result<&'a Vec<serde_json::Value>> {
    value
        .get(field)
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("normalized metrics are missing array field `{field}`"))
}

fn json_required_string(value: &serde_json::Value, field: &str) -> Result<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| anyhow!("normalized metrics are missing string field `{field}`"))
}

fn json_optional_string(value: &serde_json::Value, field: &str) -> Result<Option<String>> {
    match value.get(field) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(raw)) => Ok(Some(raw.clone())),
        Some(_) => Err(anyhow!("normalized metrics field `{field}` must be a string or null")),
    }
}

fn json_required_u64(value: &serde_json::Value, field: &str) -> Result<u64> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("normalized metrics are missing integer field `{field}`"))
}

fn json_required_f64(value: &serde_json::Value, field: &str) -> Result<f64> {
    value
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("normalized metrics are missing numeric field `{field}`"))
}

fn f64s_match(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-12
}

/// Apply fixture-safe VCF filtering and report explainable retained/removed consequences.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
pub fn execute_vcf_filter_with_explainable_consequences(
    input_vcf: &Path,
    min_qual: Option<f64>,
    max_missing_genotype_fraction: Option<f64>,
) -> Result<VcfFilterConsequenceV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut reason_counts = BTreeMap::<String, u64>::new();
    let mut retained = 0_u64;
    for record in &doc.records {
        let mut reasons = Vec::<String>::new();
        if let Some(threshold) = min_qual {
            if record.qual.is_some_and(|qual| qual < threshold) {
                reasons.push("low_qual".to_string());
            }
        }
        if let (Some(max_fraction), Some(format)) = (max_missing_genotype_fraction, &record.format)
        {
            let mut missing_calls = 0_u64;
            let mut total_calls = 0_u64;
            for payload in &record.samples {
                total_calls += 1;
                if parse_gt_from_sample(format, payload).is_some_and(genotype_is_missing) {
                    missing_calls += 1;
                }
            }
            let missing_fraction = u64_ratio_saturating(missing_calls, total_calls);
            if missing_fraction > max_fraction {
                reasons.push("missingness_above_threshold".to_string());
            }
        }
        if reasons.is_empty() {
            retained += 1;
        } else {
            for reason in reasons {
                *reason_counts.entry(reason).or_insert(0) += 1;
            }
        }
    }
    let variants_in = doc.records.len() as u64;
    let variants_removed = variants_in.saturating_sub(retained);
    let filter_expression = format!(
        "qual>={};missing_fraction<={}",
        min_qual.map_or_else(|| "none".to_string(), |value| value.to_string()),
        max_missing_genotype_fraction.map_or_else(|| "none".to_string(), |value| value.to_string())
    );
    let output_subset_identity = format!("vcf.filter:{variants_in}:{retained}:{filter_expression}");
    Ok(VcfFilterConsequenceV1 {
        schema_version: VCF_FILTER_CONSEQUENCE_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.filter".to_string(),
        filter_expression,
        variants_in,
        variants_retained: retained,
        variants_removed,
        reason_counts,
        output_subset_identity,
        caveats: vec![
            "reason counts are per-rule and a single site can contribute to multiple reasons"
                .to_string(),
        ],
    })
}

fn normalize_alleles(mut pos: u64, ref_allele: &str, alt_allele: &str) -> (u64, String, String) {
    let mut ref_norm = ref_allele.to_string();
    let mut alt_norm = alt_allele.to_string();
    while ref_norm.len() > 1
        && alt_norm.len() > 1
        && ref_norm.chars().next() == alt_norm.chars().next()
    {
        ref_norm.remove(0);
        alt_norm.remove(0);
        pos += 1;
    }
    while ref_norm.len() > 1
        && alt_norm.len() > 1
        && ref_norm.chars().last() == alt_norm.chars().last()
    {
        ref_norm.pop();
        alt_norm.pop();
    }
    if alt_norm.is_empty() {
        alt_norm = "-".to_string();
    }
    if ref_norm.is_empty() {
        ref_norm = "-".to_string();
    }
    (pos, ref_norm, alt_norm)
}

/// Execute fixture-safe normalization/decomposition accounting for VCF records.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
pub fn execute_vcf_normalization_and_decomposition(
    input_vcf: &Path,
) -> Result<VcfNormalizationSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut records_out = 0_u64;
    let mut records_changed = 0_u64;
    let mut split_multiallelic_records = 0_u64;
    let mut normalized_keys = BTreeSet::<String>::new();
    let mut duplicate_records_after_normalization = 0_u64;
    for record in &doc.records {
        if record.alt_alleles.len() > 1 {
            split_multiallelic_records += 1;
        }
        for alt in &record.alt_alleles {
            records_out += 1;
            let (norm_pos, norm_ref, norm_alt) =
                normalize_alleles(record.pos, &record.ref_allele, alt);
            if norm_pos != record.pos || norm_ref != record.ref_allele || norm_alt != *alt {
                records_changed += 1;
            }
            let key = format!("{}:{}:{}>{}", record.chrom, norm_pos, norm_ref, norm_alt);
            if !normalized_keys.insert(key) {
                duplicate_records_after_normalization += 1;
            }
        }
    }
    Ok(VcfNormalizationSummaryV1 {
        schema_version: VCF_NORMALIZATION_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.postprocess".to_string(),
        records_in: doc.records.len() as u64,
        records_out,
        records_changed,
        split_multiallelic_records,
        duplicate_records_after_normalization,
        raw_view_preserved: true,
        caveats: vec![
            "raw input view must remain inspectable beside normalized outputs".to_string(),
            "decomposition can increase record count for multiallelic sites".to_string(),
        ],
    })
}

/// Resolve VCF reference context and build compatibility before downstream planning.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
#[allow(clippy::too_many_arguments)]
pub fn resolve_vcf_reference_context(
    input_vcf: &Path,
    reference_build: &str,
    panel_build: &str,
    genetic_map_build: Option<&str>,
    alias_map: &BTreeMap<String, String>,
    has_fasta: bool,
    has_fai: bool,
    known_reference_contigs: &BTreeSet<String>,
) -> Result<VcfReferenceContextResolutionV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut refusal_codes = Vec::<String>::new();
    let mut alias_mappings_used = 0_u32;
    for contig in &doc.contigs {
        let canonical = alias_map.get(contig).map_or(contig.as_str(), String::as_str);
        if canonical != contig {
            alias_mappings_used += 1;
        }
        if !known_reference_contigs.is_empty() && !known_reference_contigs.contains(canonical) {
            refusal_codes.push("reference_contig_mismatch".to_string());
        }
    }
    if !has_fasta {
        refusal_codes.push("reference_fasta_missing".to_string());
    }
    if !has_fai {
        refusal_codes.push("reference_fai_missing".to_string());
    }
    let panel_compatible = reference_build == panel_build;
    if !panel_compatible {
        refusal_codes.push("panel_build_mismatch".to_string());
    }
    let genetic_map_compatible = genetic_map_build.is_none_or(|build| build == reference_build);
    if !genetic_map_compatible {
        refusal_codes.push("genetic_map_build_mismatch".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    Ok(VcfReferenceContextResolutionV1 {
        schema_version: VCF_REFERENCE_CONTEXT_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.prepare_reference_panel".to_string(),
        reference_build: reference_build.to_string(),
        panel_build: panel_build.to_string(),
        genetic_map_build: genetic_map_build.map(ToOwned::to_owned),
        contigs_observed: usize_to_u32_saturating(doc.contigs.len()),
        alias_mappings_used,
        fasta_present: has_fasta,
        fai_present: has_fai,
        panel_compatible,
        genetic_map_compatible,
        passes: refusal_codes.is_empty(),
        refusal_codes,
        caveats: vec![
            "contig aliases are resolved before comparing against reference expectations"
                .to_string(),
            "panel and genetic map compatibility are required for downstream phasing/imputation"
                .to_string(),
        ],
    })
}

fn is_damage_transition(ref_allele: &str, alt_allele: &str) -> bool {
    matches!((ref_allele, alt_allele), ("C", "T") | ("G", "A"))
}

/// Execute damage-aware VCF filtering semantics with explicit action mode.
///
/// # Errors
/// Returns an error when the VCF cannot be parsed.
pub fn execute_damage_aware_vcf_filter(
    input_vcf: &Path,
    has_damage_context: bool,
    action: &str,
    risk_info_keys: &[&str],
) -> Result<VcfDamageFilterSummaryV1> {
    let doc = parse_tiny_vcf(input_vcf)?;
    let mut refusal_codes = Vec::<String>::new();
    if !has_damage_context {
        refusal_codes.push("damage_context_required".to_string());
    }
    if !matches!(action, "remove" | "mask" | "annotate") {
        refusal_codes.push("invalid_damage_action".to_string());
    }

    let mut damage_risk_sites = 0_u64;
    let mut removed_sites = 0_u64;
    let mut masked_sites = 0_u64;
    let mut annotated_sites = 0_u64;
    for record in &doc.records {
        let risk_from_transition = record.alt_alleles.iter().any(|alt| {
            record.ref_allele.len() == 1
                && alt.len() == 1
                && is_damage_transition(&record.ref_allele, alt)
        });
        let risk_from_info = risk_info_keys
            .iter()
            .any(|key| record.info.split(';').any(|token| token.split('=').next() == Some(*key)));
        if risk_from_transition || risk_from_info {
            damage_risk_sites += 1;
            match action {
                "remove" => removed_sites += 1,
                "mask" => masked_sites += 1,
                "annotate" => annotated_sites += 1,
                _ => {}
            }
        }
    }
    refusal_codes.sort();
    refusal_codes.dedup();

    Ok(VcfDamageFilterSummaryV1 {
        schema_version: VCF_DAMAGE_FILTER_SUMMARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.damage_filter".to_string(),
        action: action.to_string(),
        prerequisites_passed: refusal_codes.is_empty(),
        variants_in: doc.records.len() as u64,
        damage_risk_sites,
        removed_sites,
        masked_sites,
        annotated_sites,
        refusal_codes,
        caveats: vec![
            "damage-aware filtering is evidence-scoped and should not hide uncertainty".to_string(),
            "action mode must be explicit to avoid silent semantic drift".to_string(),
        ],
    })
}

/// Evaluate diploid calling prerequisites and return explicit refusal reasons when unmet.
#[must_use]
pub fn evaluate_diploid_calling_boundary(
    has_input_bam: bool,
    has_reference_context: bool,
    declared_ploidy: Option<&str>,
    mean_coverage: f64,
    minimum_mean_coverage: f64,
) -> VcfCallingBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !has_input_bam {
        refusal_codes.push("input_bam_required".to_string());
    }
    if !has_reference_context {
        refusal_codes.push("reference_context_required".to_string());
    }
    if declared_ploidy != Some("diploid") {
        refusal_codes.push("diploid_ploidy_required".to_string());
    }
    if mean_coverage < minimum_mean_coverage {
        refusal_codes.push("coverage_below_diploid_minimum".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    let prerequisites_passed = refusal_codes.is_empty();
    let confidence = if prerequisites_passed {
        (mean_coverage / minimum_mean_coverage).clamp(0.0, 1.0)
    } else {
        0.0
    };
    VcfCallingBoundaryV1 {
        schema_version: VCF_DIPLOID_CALLING_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.call_diploid".to_string(),
        mode: "diploid".to_string(),
        prerequisites_passed,
        confidence,
        refusal_codes,
        assumptions: vec![
            "diploid genotype fields and reference context are required".to_string(),
            "coverage threshold must be met for stable diploid inference".to_string(),
        ],
        caveats: vec![
            "diploid calling boundaries do not certify downstream population compatibility"
                .to_string(),
        ],
    }
}

/// Evaluate pseudohaploid calling prerequisites with explicit uncertainty requirements.
#[must_use]
pub fn evaluate_pseudohaploid_calling_boundary(
    has_input_bam: bool,
    low_coverage_expected: bool,
    sampling_strategy: Option<&str>,
    declared_ploidy: Option<&str>,
    uncertainty_reported: bool,
) -> VcfCallingBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !has_input_bam {
        refusal_codes.push("input_bam_required".to_string());
    }
    if !low_coverage_expected {
        refusal_codes.push("low_coverage_context_required".to_string());
    }
    if sampling_strategy.is_none_or(str::is_empty) {
        refusal_codes.push("sampling_strategy_required".to_string());
    }
    if !matches!(declared_ploidy, Some("haploid" | "pseudohaploid")) {
        refusal_codes.push("pseudohaploid_ploidy_required".to_string());
    }
    if !uncertainty_reported {
        refusal_codes.push("uncertainty_reporting_required".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    let prerequisites_passed = refusal_codes.is_empty();
    VcfCallingBoundaryV1 {
        schema_version: VCF_PSEUDOHAPLOID_CALLING_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.call_pseudohaploid".to_string(),
        mode: "pseudohaploid".to_string(),
        prerequisites_passed,
        confidence: if prerequisites_passed { 0.75 } else { 0.0 },
        refusal_codes,
        assumptions: vec![
            "pseudohaploid mode is intended for low-coverage contexts".to_string(),
            "sampling strategy and uncertainty disclosure are required".to_string(),
        ],
        caveats: vec!["pseudo-haploid outputs are not diploid genotype replacements".to_string()],
    }
}

/// Evaluate genotype-likelihood workflow boundaries and uncertainty propagation requirements.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub fn evaluate_genotype_likelihood_workflow_boundary(
    has_gl_fields: bool,
    has_gp_or_pl_fields: bool,
    low_coverage_model_declared: bool,
    downstream_gl_compatible: bool,
    uncertainty_propagated: bool,
) -> VcfLikelihoodWorkflowBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !has_gl_fields {
        refusal_codes.push("gl_fields_required".to_string());
    }
    if !has_gp_or_pl_fields {
        refusal_codes.push("gp_or_pl_fields_required".to_string());
    }
    if !low_coverage_model_declared {
        refusal_codes.push("low_coverage_model_required".to_string());
    }
    if !downstream_gl_compatible {
        refusal_codes.push("downstream_gl_compatibility_required".to_string());
    }
    if !uncertainty_propagated {
        refusal_codes.push("uncertainty_propagation_required".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    VcfLikelihoodWorkflowBoundaryV1 {
        schema_version: VCF_GL_WORKFLOW_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.call_gl".to_string(),
        prerequisites_passed: refusal_codes.is_empty(),
        refusal_codes,
        assumptions: vec![
            "GL workflows require explicit uncertainty-aware data fields".to_string(),
            "downstream tools must accept likelihood-bearing semantics".to_string(),
        ],
        caveats: vec![
            "GL-bearing outputs should not be silently coerced into hard diploid genotypes"
                .to_string(),
        ],
    }
}

/// Evaluate phasing workflow boundaries and sample/reference prerequisites.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub fn evaluate_phasing_workflow_boundary(
    has_reference_context: bool,
    has_reference_panel: bool,
    has_genetic_map: bool,
    builds_compatible: bool,
    sample_count: u32,
    minimum_samples: u32,
    sample_metadata_complete: bool,
) -> VcfPhasingWorkflowBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !has_reference_context {
        refusal_codes.push("reference_context_required".to_string());
    }
    if !has_reference_panel {
        refusal_codes.push("reference_panel_required".to_string());
    }
    if !has_genetic_map {
        refusal_codes.push("genetic_map_required".to_string());
    }
    if !builds_compatible {
        refusal_codes.push("build_compatibility_required".to_string());
    }
    if sample_count < minimum_samples {
        refusal_codes.push("sample_count_below_phasing_minimum".to_string());
    }
    if !sample_metadata_complete {
        refusal_codes.push("sample_metadata_required".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    let prerequisites_passed = refusal_codes.is_empty();
    let confidence = if prerequisites_passed {
        (f64::from(sample_count) / f64::from(minimum_samples.max(1))).clamp(0.0, 1.0)
    } else {
        0.0
    };
    VcfPhasingWorkflowBoundaryV1 {
        schema_version: VCF_PHASING_WORKFLOW_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.phasing".to_string(),
        prerequisites_passed,
        panel_compatible: has_reference_panel && builds_compatible,
        genetic_map_compatible: has_genetic_map && builds_compatible,
        confidence,
        sample_count,
        minimum_samples,
        refusal_codes,
        assumptions: vec![
            "phasing requires panel, map, and reference-build compatibility".to_string(),
            "insufficient sample support reduces phasing confidence".to_string(),
        ],
        caveats: vec![
            "phasing confidence does not certify downstream imputation accuracy".to_string(),
            "sample metadata gaps can bias switch-error interpretation".to_string(),
        ],
    }
}

/// Evaluate imputation workflow boundaries and enforce panel/map provenance identity.
#[must_use]
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn evaluate_imputation_workflow_boundary(
    backend: &str,
    panel_id: Option<&str>,
    map_id: Option<&str>,
    build_compatible: bool,
    input_calling_mode: Option<&str>,
    info_score_mean: Option<f64>,
    simulation_mode: bool,
    simulation_labeled: bool,
) -> VcfImputationWorkflowBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !matches!(backend, "beagle" | "glimpse" | "impute5") {
        refusal_codes.push("unsupported_imputation_backend".to_string());
    }
    if panel_id.is_none_or(str::is_empty) {
        refusal_codes.push("panel_identity_required".to_string());
    }
    if map_id.is_none_or(str::is_empty) {
        refusal_codes.push("map_identity_required".to_string());
    }
    if !build_compatible {
        refusal_codes.push("build_compatibility_required".to_string());
    }
    if input_calling_mode != Some("diploid") {
        refusal_codes.push("diploid_input_required".to_string());
    }
    if simulation_mode && !simulation_labeled {
        refusal_codes.push("simulation_label_required".to_string());
    }
    let info_threshold = 0.30_f64;
    if info_score_mean.is_some_and(|score| score < info_threshold) {
        refusal_codes.push("imputation_info_below_threshold".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    let prerequisites_passed = refusal_codes.is_empty();
    let confidence =
        if prerequisites_passed { info_score_mean.unwrap_or(0.80).clamp(0.0, 1.0) } else { 0.0 };
    VcfImputationWorkflowBoundaryV1 {
        schema_version: VCF_IMPUTATION_WORKFLOW_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.imputation_metrics".to_string(),
        backend: backend.to_string(),
        panel_id: panel_id.map(ToOwned::to_owned),
        map_id: map_id.map(ToOwned::to_owned),
        prerequisites_passed,
        panel_compatible: panel_id.is_some_and(|id| !id.is_empty()) && build_compatible,
        map_compatible: map_id.is_some_and(|id| !id.is_empty()) && build_compatible,
        confidence,
        simulation_mode,
        refusal_codes,
        assumptions: vec![
            "imputation requires explicit panel and map identity with build compatibility"
                .to_string(),
            "diploid-compatible inputs are required for production imputation".to_string(),
        ],
        caveats: vec![
            "simulation-mode outputs must stay explicitly labeled".to_string(),
            "INFO-like summary metrics are cohort-sensitive and do not replace QC".to_string(),
        ],
    }
}

/// Build a cohort QC summary with explicit per-sample caveats and cohort-level readiness flags.
#[must_use]
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn execute_cohort_qc_workflow(
    sample_missingness: &BTreeMap<String, f64>,
    sample_heterozygosity: &BTreeMap<String, f64>,
    related_pairs: &[(String, String, f64)],
    minimum_sample_count: u32,
    missingness_threshold: f64,
    heterozygosity_lower: f64,
    heterozygosity_upper: f64,
    variants_in: u64,
    variants_after_filter: u64,
) -> VcfCohortQcWorkflowSummaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    let sample_count = usize_to_u32_saturating(sample_missingness.len());
    if sample_count < minimum_sample_count {
        refusal_codes.push("cohort_sample_count_below_minimum".to_string());
    }
    if sample_missingness.is_empty() {
        refusal_codes.push("missingness_metrics_required".to_string());
    }
    if sample_heterozygosity.is_empty() {
        refusal_codes.push("heterozygosity_metrics_required".to_string());
    }
    if variants_after_filter > variants_in {
        refusal_codes.push("filter_variant_counts_incoherent".to_string());
    }

    let mut high_missingness_samples = 0_u32;
    let mut heterozygosity_outlier_samples = 0_u32;
    let mut per_sample = Vec::<VcfCohortQcSampleCaveatV1>::new();
    for (sample_id, missingness) in sample_missingness {
        let heterozygosity = sample_heterozygosity.get(sample_id).copied();
        let mut caveats = Vec::<String>::new();
        if *missingness > missingness_threshold {
            high_missingness_samples += 1;
            caveats.push("missingness_above_threshold".to_string());
        }
        if let Some(value) = heterozygosity {
            if !(heterozygosity_lower..=heterozygosity_upper).contains(&value) {
                heterozygosity_outlier_samples += 1;
                caveats.push("heterozygosity_outlier".to_string());
            }
        } else {
            caveats.push("heterozygosity_missing".to_string());
        }
        per_sample.push(VcfCohortQcSampleCaveatV1 {
            sample_id: sample_id.clone(),
            missingness: *missingness,
            heterozygosity,
            caveats,
        });
    }
    per_sample.sort_by(|left, right| left.sample_id.cmp(&right.sample_id));
    let relatedness_flagged_pairs = usize_to_u32_saturating(
        related_pairs.iter().filter(|(_, _, kinship)| *kinship >= 0.0884).count(),
    );
    let variants_removed_by_filter = variants_in.saturating_sub(variants_after_filter);
    refusal_codes.sort();
    refusal_codes.dedup();
    VcfCohortQcWorkflowSummaryV1 {
        schema_version: VCF_COHORT_QC_WORKFLOW_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.qc".to_string(),
        prerequisites_passed: refusal_codes.is_empty(),
        sample_count,
        minimum_sample_count,
        missingness_threshold,
        heterozygosity_lower,
        heterozygosity_upper,
        high_missingness_samples,
        heterozygosity_outlier_samples,
        relatedness_flagged_pairs,
        variants_in,
        variants_after_filter,
        variants_removed_by_filter,
        per_sample,
        refusal_codes,
        caveats: vec![
            "relatedness flags are triage signals and require context-aware follow-up".to_string(),
            "cohort QC summaries should not be interpreted as ancestry conclusions".to_string(),
        ],
    }
}

/// Evaluate guardrails for PCA/admixture analyses.
#[must_use]
#[cfg(test)]
fn evaluate_pca_admixture_guardrail(
    ld_pruned: bool,
    sample_inclusion_defined: bool,
    marker_count: u64,
    minimum_marker_count: u64,
    missingness_rate: f64,
    maximum_missingness_rate: f64,
    interpretation_caveats_attached: bool,
) -> VcfPcaAdmixtureGuardrailV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !ld_pruned {
        refusal_codes.push("ld_pruning_required".to_string());
    }
    if !sample_inclusion_defined {
        refusal_codes.push("sample_inclusion_policy_required".to_string());
    }
    if marker_count < minimum_marker_count {
        refusal_codes.push("marker_count_below_minimum".to_string());
    }
    if missingness_rate > maximum_missingness_rate {
        refusal_codes.push("missingness_above_maximum".to_string());
    }
    if !interpretation_caveats_attached {
        refusal_codes.push("interpretation_caveats_required".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    VcfPcaAdmixtureGuardrailV1 {
        schema_version: VCF_PCA_ADMIXTURE_GUARDRAIL_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.population_structure".to_string(),
        prerequisites_passed: refusal_codes.is_empty(),
        ld_pruned,
        sample_inclusion_defined,
        marker_count,
        minimum_marker_count,
        missingness_rate,
        maximum_missingness_rate,
        refusal_codes,
        assumptions: vec![
            "PCA/admixture analyses require LD-pruned marker sets".to_string(),
            "sample inclusion policy must be explicit for reproducible population summaries"
                .to_string(),
        ],
        caveats: vec![
            "population-structure plots are descriptive and not deterministic ancestry labels"
                .to_string(),
            "batch composition can shift principal-component orientation".to_string(),
        ],
    }
}

/// Evaluate ROH/IBD workflow boundaries before running cohort-level analyses.
#[must_use]
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
fn evaluate_roh_ibd_workflow_boundary(
    method: &str,
    marker_density_per_mb: f64,
    minimum_marker_density_per_mb: f64,
    cohort_size: u32,
    minimum_cohort_size: u32,
    missingness_rate: f64,
    maximum_missingness_rate: f64,
    assumptions_documented: bool,
) -> VcfRohIbdWorkflowBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !matches!(method, "roh" | "ibd") {
        refusal_codes.push("unsupported_roh_ibd_method".to_string());
    }
    if marker_density_per_mb < minimum_marker_density_per_mb {
        refusal_codes.push("marker_density_below_minimum".to_string());
    }
    if cohort_size < minimum_cohort_size {
        refusal_codes.push("cohort_size_below_minimum".to_string());
    }
    if missingness_rate > maximum_missingness_rate {
        refusal_codes.push("missingness_above_maximum".to_string());
    }
    if !assumptions_documented {
        refusal_codes.push("method_assumptions_required".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    VcfRohIbdWorkflowBoundaryV1 {
        schema_version: VCF_ROH_IBD_WORKFLOW_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: if method == "roh" { "vcf.roh" } else { "vcf.ibd" }.to_string(),
        method: method.to_string(),
        prerequisites_passed: refusal_codes.is_empty(),
        marker_density_per_mb,
        minimum_marker_density_per_mb,
        cohort_size,
        minimum_cohort_size,
        missingness_rate,
        maximum_missingness_rate,
        refusal_codes,
        assumptions: vec![
            "ROH/IBD analyses require adequate marker density and cohort support".to_string(),
            "method assumptions must be documented before interpretation".to_string(),
        ],
        caveats: vec![
            "low coverage can destabilize segment boundaries".to_string(),
            "relatedness interpretation depends on cohort ascertainment and phasing quality"
                .to_string(),
        ],
    }
}

/// Evaluate demography-analysis refusal boundaries for underpowered or incompatible requests.
#[must_use]
#[cfg(test)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::fn_params_excessive_bools)]
fn evaluate_demography_refusal_boundary(
    requested_model: &str,
    cohort_size: u32,
    minimum_cohort_size: u32,
    has_ibd_segments: bool,
    marker_density_passed: bool,
    missingness_passed: bool,
    method_assumptions_documented: bool,
    temporal_metadata_available: bool,
) -> VcfDemographyRefusalBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    let mut missing_assumptions = Vec::<String>::new();
    if !matches!(requested_model, "ibdne" | "smc++" | "dadi" | "psmc") {
        refusal_codes.push("unsupported_demography_model".to_string());
    }
    if cohort_size < minimum_cohort_size {
        refusal_codes.push("cohort_size_below_demography_minimum".to_string());
        missing_assumptions.push("adequate_cohort_size".to_string());
    }
    if !has_ibd_segments {
        refusal_codes.push("ibd_segments_required".to_string());
        missing_assumptions.push("ibd_segments".to_string());
    }
    if !marker_density_passed {
        refusal_codes.push("marker_density_gate_failed".to_string());
        missing_assumptions.push("marker_density_pass".to_string());
    }
    if !missingness_passed {
        refusal_codes.push("missingness_gate_failed".to_string());
        missing_assumptions.push("missingness_pass".to_string());
    }
    if !method_assumptions_documented {
        refusal_codes.push("method_assumptions_required".to_string());
        missing_assumptions.push("method_assumptions_documented".to_string());
    }
    if !temporal_metadata_available {
        refusal_codes.push("temporal_metadata_required".to_string());
        missing_assumptions.push("temporal_metadata".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    missing_assumptions.sort();
    missing_assumptions.dedup();
    VcfDemographyRefusalBoundaryV1 {
        schema_version: VCF_DEMOGRAPHY_REFUSAL_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.demography".to_string(),
        requested_model: requested_model.to_string(),
        prerequisites_passed: refusal_codes.is_empty(),
        cohort_size,
        minimum_cohort_size,
        missing_assumptions,
        refusal_codes,
        caveats: vec![
            "demography outputs are model-based and should be interpreted alongside uncertainty"
                .to_string(),
            "boundary refusal protects against underpowered historical inference".to_string(),
        ],
    }
}

/// Build a panel/reference drift report with explicit invalidation of downstream artifacts.
#[must_use]
#[cfg(test)]
fn build_panel_reference_drift_report(
    baseline: &VcfPanelReferenceSnapshotV1,
    candidate: &VcfPanelReferenceSnapshotV1,
    downstream_outputs: &[&str],
) -> VcfPanelReferenceDriftReportV1 {
    let mut changed_surfaces = Vec::<String>::new();
    if baseline.reference_build != candidate.reference_build {
        changed_surfaces.push("reference_build".to_string());
    }
    if baseline.reference_fasta_sha256 != candidate.reference_fasta_sha256 {
        changed_surfaces.push("reference_fasta".to_string());
    }
    if baseline.panel_id != candidate.panel_id {
        changed_surfaces.push("panel_id".to_string());
    }
    if baseline.map_id != candidate.map_id {
        changed_surfaces.push("map_id".to_string());
    }
    if baseline.contig_alias_digest != candidate.contig_alias_digest {
        changed_surfaces.push("contig_alias_digest".to_string());
    }
    changed_surfaces.sort();
    changed_surfaces.dedup();
    let requires_reprocessing = !changed_surfaces.is_empty();
    let mut invalidated_outputs = if requires_reprocessing {
        downstream_outputs.iter().map(|output| (*output).to_string()).collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    invalidated_outputs.sort();
    invalidated_outputs.dedup();
    VcfPanelReferenceDriftReportV1 {
        schema_version: VCF_PANEL_REFERENCE_DRIFT_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.prepare_reference_panel".to_string(),
        baseline_label: baseline.label.clone(),
        candidate_label: candidate.label.clone(),
        changed_surfaces,
        invalidated_outputs,
        requires_reprocessing,
        caveats: vec![
            "panel/reference drift requires explicit downstream artifact invalidation".to_string(),
            "drift classification does not infer biological correctness".to_string(),
        ],
    }
}

/// Evaluate structural-variant support boundaries and refuse unsafe coercion into small-variant semantics.
#[must_use]
#[cfg(test)]
fn evaluate_structural_variant_support_boundary(
    has_structural_variants: bool,
    explicit_sv_mode: bool,
    backend: Option<&str>,
    supported_sv_types: &[&str],
    requested_operation: &str,
    treat_as_small_variant: bool,
) -> VcfStructuralVariantBoundaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    let support_declared = !supported_sv_types.is_empty();
    if has_structural_variants && !explicit_sv_mode {
        refusal_codes.push("explicit_sv_mode_required".to_string());
    }
    if has_structural_variants && backend.is_none_or(str::is_empty) {
        refusal_codes.push("sv_backend_required".to_string());
    }
    if has_structural_variants && !support_declared {
        refusal_codes.push("supported_sv_types_required".to_string());
    }
    if !matches!(requested_operation, "call" | "filter" | "annotate" | "summarize") {
        refusal_codes.push("unsupported_sv_operation".to_string());
    }
    if has_structural_variants && treat_as_small_variant {
        refusal_codes.push("small_variant_coercion_forbidden".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    VcfStructuralVariantBoundaryV1 {
        schema_version: VCF_STRUCTURAL_VARIANT_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.postprocess".to_string(),
        has_structural_variants,
        explicit_sv_mode,
        backend: backend.map(ToOwned::to_owned),
        requested_operation: requested_operation.to_string(),
        support_declared,
        supported_sv_types: supported_sv_types.iter().map(|kind| (*kind).to_string()).collect(),
        prerequisites_passed: refusal_codes.is_empty(),
        refusal_codes,
        caveats: vec![
            "SV handling boundaries prevent accidental interpretation as small-variant VCF"
                .to_string(),
            "SV tool semantics differ by caller and must be surfaced explicitly".to_string(),
        ],
    }
}

/// Build annotation provenance summary with explicit source/version and field-coverage accounting.
#[must_use]
#[cfg(test)]
fn execute_annotation_provenance_workflow(
    annotation_source: &str,
    annotation_version: &str,
    requested_fields: &[&str],
    covered_fields: &[&str],
    gene_mapping: Option<&str>,
    transcript_mapping: Option<&str>,
    minimum_field_coverage: f64,
) -> VcfAnnotationProvenanceWorkflowSummaryV1 {
    let mut refusal_codes = Vec::<String>::new();
    if annotation_source.is_empty() {
        refusal_codes.push("annotation_source_required".to_string());
    }
    if annotation_version.is_empty() {
        refusal_codes.push("annotation_version_required".to_string());
    }
    if requested_fields.is_empty() {
        refusal_codes.push("requested_annotation_fields_required".to_string());
    }
    if gene_mapping.is_none_or(str::is_empty) {
        refusal_codes.push("gene_mapping_required".to_string());
    }
    if transcript_mapping.is_none_or(str::is_empty) {
        refusal_codes.push("transcript_mapping_required".to_string());
    }
    let requested = requested_fields.iter().map(|field| (*field).to_string()).collect::<Vec<_>>();
    let mut covered = covered_fields.iter().map(|field| (*field).to_string()).collect::<Vec<_>>();
    covered.sort();
    covered.dedup();
    let covered_count = requested
        .iter()
        .filter(|field| covered.iter().any(|covered_field| covered_field == *field))
        .count();
    let requested_count = requested.len();
    let field_coverage = if requested_count > 0 {
        let covered_count_f64 = f64::from(u32::try_from(covered_count).unwrap_or(u32::MAX));
        let requested_count_f64 = f64::from(u32::try_from(requested_count).unwrap_or(u32::MAX));
        covered_count_f64 / requested_count_f64
    } else {
        0.0
    };
    if field_coverage < minimum_field_coverage {
        refusal_codes.push("annotation_field_coverage_below_minimum".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    VcfAnnotationProvenanceWorkflowSummaryV1 {
        schema_version: VCF_ANNOTATION_PROVENANCE_WORKFLOW_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.postprocess".to_string(),
        annotation_source: annotation_source.to_string(),
        annotation_version: annotation_version.to_string(),
        requested_fields: requested,
        covered_fields: covered,
        field_coverage,
        gene_mapping: gene_mapping.map(ToOwned::to_owned),
        transcript_mapping: transcript_mapping.map(ToOwned::to_owned),
        prerequisites_passed: refusal_codes.is_empty(),
        refusal_codes,
        caveats: vec![
            "annotation completeness depends on source release and transcript model".to_string(),
            "field-level coverage should be reviewed before downstream interpretation".to_string(),
        ],
    }
}

/// Evaluate typed handoff boundaries from filtered/normalized VCF artifacts into population analyses.
#[must_use]
#[cfg(test)]
#[allow(clippy::fn_params_excessive_bools)]
fn evaluate_vcf_population_analysis_handoff(
    target_analysis: &str,
    filtered: bool,
    normalized: bool,
    sample_identity_match: bool,
    reference_identity_match: bool,
    trust_class_compatible: bool,
    caveats_attached: bool,
) -> VcfPopulationAnalysisHandoffV1 {
    let mut refusal_codes = Vec::<String>::new();
    if !matches!(target_analysis, "pca" | "roh" | "ibd" | "demography") {
        refusal_codes.push("unsupported_population_analysis_target".to_string());
    }
    if !filtered {
        refusal_codes.push("filtered_vcf_required".to_string());
    }
    if !normalized {
        refusal_codes.push("normalized_vcf_required".to_string());
    }
    if !sample_identity_match {
        refusal_codes.push("sample_identity_mismatch".to_string());
    }
    if !reference_identity_match {
        refusal_codes.push("reference_identity_mismatch".to_string());
    }
    if !trust_class_compatible {
        refusal_codes.push("trust_class_incompatible".to_string());
    }
    if !caveats_attached {
        refusal_codes.push("caveats_required_for_handoff".to_string());
    }
    refusal_codes.sort();
    refusal_codes.dedup();
    VcfPopulationAnalysisHandoffV1 {
        schema_version: VCF_POPULATION_HANDOFF_BOUNDARY_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.population_structure".to_string(),
        target_analysis: target_analysis.to_string(),
        filtered,
        normalized,
        sample_identity_match,
        reference_identity_match,
        trust_class_compatible,
        caveats_attached,
        prerequisites_passed: refusal_codes.is_empty(),
        refusal_codes,
    }
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_vcf_scientific_drift_report(
    baseline: &VcfScientificDriftSnapshotV1,
    candidate: &VcfScientificDriftSnapshotV1,
) -> VcfScientificDriftReportV1 {
    let mut metric_ids =
        baseline.metrics.keys().chain(candidate.metrics.keys()).cloned().collect::<Vec<_>>();
    metric_ids.sort();
    metric_ids.dedup();
    let metric_deltas = metric_ids
        .into_iter()
        .filter_map(|metric_id| {
            let baseline_value = *baseline.metrics.get(&metric_id)?;
            let candidate_value = *candidate.metrics.get(&metric_id)?;
            let absolute_delta = candidate_value - baseline_value;
            let relative_delta = if baseline_value.abs() > f64::EPSILON {
                Some(absolute_delta / baseline_value)
            } else {
                None
            };
            Some(VcfScientificDriftMetricDeltaV1 {
                metric_id,
                baseline_value,
                candidate_value,
                absolute_delta,
                relative_delta,
            })
        })
        .collect::<Vec<_>>();

    let mut artifact_ids =
        baseline.artifacts.keys().chain(candidate.artifacts.keys()).cloned().collect::<Vec<_>>();
    artifact_ids.sort();
    artifact_ids.dedup();
    let artifact_deltas = artifact_ids
        .into_iter()
        .map(|artifact_id| {
            let baseline_hash = baseline.artifacts.get(&artifact_id).cloned();
            let candidate_hash = candidate.artifacts.get(&artifact_id).cloned();
            VcfScientificDriftArtifactDeltaV1 {
                artifact_id,
                changed: baseline_hash != candidate_hash,
                baseline_hash,
                candidate_hash,
            }
        })
        .collect::<Vec<_>>();

    let mut change_kinds = Vec::new();
    if baseline.defaults_fingerprint != candidate.defaults_fingerprint {
        change_kinds.push(VcfScientificDriftChangeKind::DefaultsChange);
    }
    if baseline.backend_version != candidate.backend_version
        || baseline.tool_id != candidate.tool_id
    {
        change_kinds.push(VcfScientificDriftChangeKind::BackendChange);
    }
    if baseline.filter_policy_id != candidate.filter_policy_id {
        change_kinds.push(VcfScientificDriftChangeKind::FilterPolicyChange);
    }
    if baseline.normalization_policy_id != candidate.normalization_policy_id {
        change_kinds.push(VcfScientificDriftChangeKind::NormalizationPolicyChange);
    }

    let mut downstream_risks = Vec::new();
    if metric_deltas
        .iter()
        .any(|delta| delta.metric_id == "variants_total" && delta.absolute_delta != 0.0)
    {
        downstream_risks.push("variant_count_shift".to_string());
    }
    if metric_deltas
        .iter()
        .any(|delta| delta.metric_id == "annotation_coverage" && delta.absolute_delta != 0.0)
    {
        downstream_risks.push("annotation_coverage_shift".to_string());
    }
    if metric_deltas
        .iter()
        .any(|delta| delta.metric_id == "missingness_post" && delta.absolute_delta != 0.0)
    {
        downstream_risks.push("cohort_readiness_shift".to_string());
    }
    if artifact_deltas.iter().any(|delta| delta.changed) {
        downstream_risks.push("artifact_identity_shift".to_string());
    }
    downstream_risks.sort();
    downstream_risks.dedup();

    let mut caveats = baseline.caveats.clone();
    caveats.extend(candidate.caveats.clone());
    if change_kinds.contains(&VcfScientificDriftChangeKind::DefaultsChange) {
        caveats.push(
            "default-setting drift detected; compare downstream cohort and variant summaries before promotion"
                .to_string(),
        );
    }
    if change_kinds.contains(&VcfScientificDriftChangeKind::BackendChange) {
        caveats.push(
            "backend drift detected; metric shifts may combine scientific and implementation causes"
                .to_string(),
        );
    }
    caveats.sort();
    caveats.dedup();

    VcfScientificDriftReportV1 {
        schema_version: VCF_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION.to_string(),
        stage_id: baseline.stage_id.clone(),
        baseline_label: baseline.label.clone(),
        candidate_label: candidate.label.clone(),
        baseline_tool_id: baseline.tool_id.clone(),
        candidate_tool_id: candidate.tool_id.clone(),
        change_kinds,
        metric_deltas,
        artifact_deltas,
        downstream_risks,
        caveats,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    fn unique_temp_dir(label: &str) -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("bijux-vcf-{label}-{seq}"));
        std::fs::create_dir_all(&path).expect("create temporary directory");
        path
    }

    #[test]
    fn execute_vcf_validation_accepts_valid_fixture() {
        let temp = unique_temp_dir("vcf-validate-ok");
        let input = temp.join("valid.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.3\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n\
chr1\t10\t.\tA\tG\t42\tPASS\tDP=8\tGT\t0/1\n\
chr1\t20\t.\tC\tT\t55\tPASS\tDP=9\tGT\t1/1\n",
        )
        .expect("write valid VCF fixture");

        let summary =
            execute_vcf_validation(&input, &["chr1"], true, true, Some("GRCh38"), Some("GRCh38"))
                .expect("validate fixture");
        assert_eq!(summary.record_count, 2);
        assert_eq!(summary.sample_count, 1);
        assert!(summary.refusal_codes.is_empty());
        assert!(summary.sorted_records);
    }

    #[test]
    fn execute_vcf_validation_rejects_malformed_unsorted_and_mismatched_inputs() {
        let temp = unique_temp_dir("vcf-validate-refuse");
        let input = temp.join("invalid.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.3\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts1\n\
chr2\t20\t.\tA\tG\t42\tPASS\tBADKEY=8\tGT:XX\t0/1:7\t0/0:9\n\
chr1\t10\t.\tC\tT\t55\tPASS\tDP=9\tGT\t1/1\t0/1\n",
        )
        .expect("write invalid VCF fixture");

        let summary = execute_vcf_validation(
            &input,
            &["chr1", "chr2", "chrX"],
            true,
            false,
            Some("GRCh37"),
            Some("GRCh38"),
        )
        .expect("validate fixture");
        assert!(summary.refusal_codes.contains(&"duplicate_sample_names".to_string()));
        assert!(summary.refusal_codes.contains(&"missing_contig_header".to_string()));
        assert!(summary.refusal_codes.contains(&"bad_info_field_definition".to_string()));
        assert!(summary.refusal_codes.contains(&"bad_format_field_definition".to_string()));
        assert!(summary.refusal_codes.contains(&"unsorted_records".to_string()));
        assert!(summary.refusal_codes.contains(&"missing_index".to_string()));
        assert!(summary.refusal_codes.contains(&"reference_build_mismatch".to_string()));
    }

    #[test]
    fn execute_vcf_stats_workflow_reports_variant_filters_and_missingness() {
        let temp = unique_temp_dir("vcf-stats");
        let input = temp.join("stats.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.3\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\n\
chr1\t10\t.\tA\tG\t42\tPASS\tDP=8\tGT\t0/1\t./.\n\
chr1\t20\t.\tC\tA\t55\tq10\tDP=9\tGT\t1/1\t0/1\n\
chr1\t30\t.\tAT\tA\t60\tPASS\tDP=7\tGT\t0/1\t0/0\n",
        )
        .expect("write stats VCF fixture");

        let summary = execute_vcf_stats_workflow(&input).expect("build stats summary");
        assert_eq!(summary.variant_count, 3);
        assert_eq!(summary.sample_count, 2);
        assert_eq!(summary.snv_count, 2);
        assert_eq!(summary.indel_count, 1);
        assert_eq!(summary.missing_genotype_calls, 1);
        assert_eq!(summary.filter_counts.get("PASS"), Some(&2));
        assert_eq!(summary.filter_counts.get("q10"), Some(&1));
        assert_eq!(summary.per_sample_missingness.get("s1"), Some(&0.0));
        assert_eq!(summary.per_sample_missingness.get("s2"), Some(&(1.0 / 3.0)));
        assert_eq!(summary.ti_tv_ratio, Some(1.0));
    }

    #[test]
    fn execute_vcf_filter_with_explainable_consequences_reports_reason_breakdown() {
        let temp = unique_temp_dir("vcf-filter-consequences");
        let input = temp.join("filter.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.3\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\ts2\n\
chr1\t10\t.\tA\tG\t20\tPASS\tDP=8\tGT\t0/1\t0/1\n\
chr1\t20\t.\tC\tT\t60\tPASS\tDP=8\tGT\t./.\t./.\n\
chr1\t30\t.\tG\tA\t65\tPASS\tDP=8\tGT\t0/0\t0/1\n",
        )
        .expect("write filter VCF fixture");

        let summary =
            execute_vcf_filter_with_explainable_consequences(&input, Some(30.0), Some(0.5))
                .expect("filter with explanations");
        assert_eq!(summary.variants_in, 3);
        assert_eq!(summary.variants_retained, 1);
        assert_eq!(summary.variants_removed, 2);
        assert_eq!(summary.reason_counts.get("low_qual"), Some(&1));
        assert_eq!(summary.reason_counts.get("missingness_above_threshold"), Some(&1));
        assert!(summary.output_subset_identity.starts_with("vcf.filter:3:1:"));
    }

    #[test]
    fn execute_vcf_normalization_and_decomposition_tracks_changes_and_splits() {
        let temp = unique_temp_dir("vcf-normalize");
        let input = temp.join("normalize.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.3\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n\
chr1\t10\t.\tACG\tATG\t55\tPASS\tDP=9\tGT\t0/1\n\
chr1\t20\t.\tA\tC,G\t60\tPASS\tDP=7\tGT\t0/1\n",
        )
        .expect("write normalization fixture");

        let summary =
            execute_vcf_normalization_and_decomposition(&input).expect("normalize/decompose");
        assert_eq!(summary.records_in, 2);
        assert_eq!(summary.records_out, 3);
        assert_eq!(summary.split_multiallelic_records, 1);
        assert!(summary.records_changed >= 1);
        assert!(summary.raw_view_preserved);
    }

    #[test]
    fn resolve_vcf_reference_context_checks_alias_build_and_asset_compatibility() {
        let temp = unique_temp_dir("vcf-reference-context");
        let input = temp.join("context.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.3\n\
##contig=<ID=1,length=1000>\n\
##INFO=<ID=DP,Number=1,Type=Integer,Description=\"Depth\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n\
1\t10\t.\tA\tG\t55\tPASS\tDP=9\tGT\t0/1\n",
        )
        .expect("write context fixture");

        let alias_map = BTreeMap::from([(String::from("1"), String::from("chr1"))]);
        let known_reference_contigs = BTreeSet::from([String::from("chr1")]);
        let ready = resolve_vcf_reference_context(
            &input,
            "GRCh38",
            "GRCh38",
            Some("GRCh38"),
            &alias_map,
            true,
            true,
            &known_reference_contigs,
        )
        .expect("resolve reference context");
        assert!(ready.passes);
        assert_eq!(ready.alias_mappings_used, 1);

        let refused = resolve_vcf_reference_context(
            &input,
            "GRCh38",
            "GRCh37",
            Some("GRCh37"),
            &BTreeMap::new(),
            false,
            false,
            &BTreeSet::from([String::from("chr2")]),
        )
        .expect("resolve refusal context");
        assert!(!refused.passes);
        assert!(refused.refusal_codes.contains(&"reference_contig_mismatch".to_string()));
        assert!(refused.refusal_codes.contains(&"reference_fasta_missing".to_string()));
        assert!(refused.refusal_codes.contains(&"reference_fai_missing".to_string()));
        assert!(refused.refusal_codes.contains(&"panel_build_mismatch".to_string()));
        assert!(refused.refusal_codes.contains(&"genetic_map_build_mismatch".to_string()));
    }

    #[test]
    fn execute_damage_aware_vcf_filter_tracks_risk_sites_by_action_mode() {
        let temp = unique_temp_dir("vcf-damage-filter");
        let input = temp.join("damage.vcf");
        std::fs::write(
            &input,
            "##fileformat=VCFv4.3\n\
##contig=<ID=chr1,length=1000>\n\
##INFO=<ID=PMD,Number=1,Type=String,Description=\"Damage tag\">\n\
##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">\n\
#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\ts1\n\
chr1\t10\t.\tC\tT\t55\tPASS\tPMD=high\tGT\t0/1\n\
chr1\t20\t.\tA\tG\t55\tPASS\tDP=8\tGT\t0/1\n\
chr1\t30\t.\tG\tA\t55\tPASS\tDP=8\tGT\t0/1\n",
        )
        .expect("write damage fixture");

        let remove = execute_damage_aware_vcf_filter(&input, true, "remove", &["PMD"])
            .expect("damage remove action");
        assert!(remove.prerequisites_passed);
        assert_eq!(remove.damage_risk_sites, 2);
        assert_eq!(remove.removed_sites, 2);
        assert_eq!(remove.masked_sites, 0);

        let refused = execute_damage_aware_vcf_filter(&input, false, "unknown", &["PMD"])
            .expect("damage refusal");
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"damage_context_required".to_string()));
        assert!(refused.refusal_codes.contains(&"invalid_damage_action".to_string()));
    }

    #[test]
    fn evaluate_diploid_calling_boundary_enforces_ploidy_and_coverage_requirements() {
        let ready = evaluate_diploid_calling_boundary(true, true, Some("diploid"), 12.0, 6.0);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.call_diploid");
        assert!(ready.confidence > 0.0);
        assert!(ready.refusal_codes.is_empty());

        let refused = evaluate_diploid_calling_boundary(false, false, Some("haploid"), 2.0, 6.0);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"input_bam_required".to_string()));
        assert!(refused.refusal_codes.contains(&"reference_context_required".to_string()));
        assert!(refused.refusal_codes.contains(&"diploid_ploidy_required".to_string()));
        assert!(refused.refusal_codes.contains(&"coverage_below_diploid_minimum".to_string()));
    }

    #[test]
    fn evaluate_pseudohaploid_calling_boundary_requires_low_coverage_context_and_uncertainty() {
        let ready = evaluate_pseudohaploid_calling_boundary(
            true,
            true,
            Some("single_read_sampling"),
            Some("pseudohaploid"),
            true,
        );
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.call_pseudohaploid");
        assert_eq!(ready.mode, "pseudohaploid");

        let refused =
            evaluate_pseudohaploid_calling_boundary(false, false, None, Some("diploid"), false);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"input_bam_required".to_string()));
        assert!(refused.refusal_codes.contains(&"low_coverage_context_required".to_string()));
        assert!(refused.refusal_codes.contains(&"sampling_strategy_required".to_string()));
        assert!(refused.refusal_codes.contains(&"pseudohaploid_ploidy_required".to_string()));
        assert!(refused.refusal_codes.contains(&"uncertainty_reporting_required".to_string()));
    }

    #[test]
    fn evaluate_genotype_likelihood_workflow_boundary_requires_gl_semantics() {
        let ready = evaluate_genotype_likelihood_workflow_boundary(true, true, true, true, true);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.call_gl");
        assert!(ready.refusal_codes.is_empty());

        let refused =
            evaluate_genotype_likelihood_workflow_boundary(false, false, false, false, false);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"gl_fields_required".to_string()));
        assert!(refused.refusal_codes.contains(&"gp_or_pl_fields_required".to_string()));
        assert!(refused.refusal_codes.contains(&"low_coverage_model_required".to_string()));
        assert!(refused
            .refusal_codes
            .contains(&"downstream_gl_compatibility_required".to_string()));
        assert!(refused.refusal_codes.contains(&"uncertainty_propagation_required".to_string()));
    }

    #[test]
    fn evaluate_phasing_workflow_boundary_requires_panel_map_and_sample_prerequisites() {
        let ready = evaluate_phasing_workflow_boundary(true, true, true, true, 24, 12, true);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.phasing");
        assert!(ready.panel_compatible);
        assert!(ready.genetic_map_compatible);
        assert!((ready.confidence - 1.0).abs() < f64::EPSILON);
        assert!(ready.refusal_codes.is_empty());

        let refused = evaluate_phasing_workflow_boundary(false, false, false, false, 3, 12, false);
        assert!(!refused.prerequisites_passed);
        assert!(refused.confidence.abs() < f64::EPSILON);
        assert!(refused.refusal_codes.contains(&"reference_context_required".to_string()));
        assert!(refused.refusal_codes.contains(&"reference_panel_required".to_string()));
        assert!(refused.refusal_codes.contains(&"genetic_map_required".to_string()));
        assert!(refused.refusal_codes.contains(&"build_compatibility_required".to_string()));
        assert!(refused.refusal_codes.contains(&"sample_count_below_phasing_minimum".to_string()));
        assert!(refused.refusal_codes.contains(&"sample_metadata_required".to_string()));
    }

    #[test]
    fn evaluate_imputation_workflow_boundary_requires_backend_and_panel_map_identity() {
        let ready = evaluate_imputation_workflow_boundary(
            "glimpse",
            Some("1000g_phase3"),
            Some("hapmap_genetic_map"),
            true,
            Some("diploid"),
            Some(0.92),
            false,
            false,
        );
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.imputation_metrics");
        assert!(ready.panel_compatible);
        assert!(ready.map_compatible);
        assert!(ready.refusal_codes.is_empty());

        let refused = evaluate_imputation_workflow_boundary(
            "unknown",
            None,
            None,
            false,
            Some("pseudohaploid"),
            Some(0.12),
            true,
            false,
        );
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"unsupported_imputation_backend".to_string()));
        assert!(refused.refusal_codes.contains(&"panel_identity_required".to_string()));
        assert!(refused.refusal_codes.contains(&"map_identity_required".to_string()));
        assert!(refused.refusal_codes.contains(&"build_compatibility_required".to_string()));
        assert!(refused.refusal_codes.contains(&"diploid_input_required".to_string()));
        assert!(refused.refusal_codes.contains(&"simulation_label_required".to_string()));
        assert!(refused.refusal_codes.contains(&"imputation_info_below_threshold".to_string()));
    }

    #[test]
    fn execute_cohort_qc_workflow_reports_missingness_heterozygosity_relatedness_and_filter_impact()
    {
        let sample_missingness = BTreeMap::from([
            (String::from("s1"), 0.01_f64),
            (String::from("s2"), 0.12_f64),
            (String::from("s3"), 0.03_f64),
        ]);
        let sample_heterozygosity =
            BTreeMap::from([(String::from("s1"), 0.22_f64), (String::from("s2"), 0.48_f64)]);
        let related_pairs = vec![
            (String::from("s1"), String::from("s2"), 0.10_f64),
            (String::from("s2"), String::from("s3"), 0.02_f64),
        ];

        let summary = execute_cohort_qc_workflow(
            &sample_missingness,
            &sample_heterozygosity,
            &related_pairs,
            2,
            0.05,
            0.15,
            0.35,
            1200,
            980,
        );
        assert!(summary.prerequisites_passed);
        assert_eq!(summary.sample_count, 3);
        assert_eq!(summary.high_missingness_samples, 1);
        assert_eq!(summary.heterozygosity_outlier_samples, 1);
        assert_eq!(summary.relatedness_flagged_pairs, 1);
        assert_eq!(summary.variants_removed_by_filter, 220);
        let s3 =
            summary.per_sample.iter().find(|sample| sample.sample_id == "s3").expect("sample s3");
        assert!(s3.caveats.contains(&"heterozygosity_missing".to_string()));

        let refused = execute_cohort_qc_workflow(
            &BTreeMap::new(),
            &BTreeMap::new(),
            &[],
            2,
            0.05,
            0.15,
            0.35,
            10,
            12,
        );
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"cohort_sample_count_below_minimum".to_string()));
        assert!(refused.refusal_codes.contains(&"missingness_metrics_required".to_string()));
        assert!(refused.refusal_codes.contains(&"heterozygosity_metrics_required".to_string()));
        assert!(refused.refusal_codes.contains(&"filter_variant_counts_incoherent".to_string()));
    }

    #[test]
    fn evaluate_pca_admixture_guardrail_requires_ld_pruning_marker_support_and_caveats() {
        let ready = evaluate_pca_admixture_guardrail(true, true, 150_000, 50_000, 0.01, 0.05, true);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.population_structure");
        assert!(ready.refusal_codes.is_empty());

        let refused =
            evaluate_pca_admixture_guardrail(false, false, 10_000, 50_000, 0.11, 0.05, false);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"ld_pruning_required".to_string()));
        assert!(refused.refusal_codes.contains(&"sample_inclusion_policy_required".to_string()));
        assert!(refused.refusal_codes.contains(&"marker_count_below_minimum".to_string()));
        assert!(refused.refusal_codes.contains(&"missingness_above_maximum".to_string()));
        assert!(refused.refusal_codes.contains(&"interpretation_caveats_required".to_string()));
    }

    #[test]
    fn evaluate_roh_ibd_workflow_boundary_requires_density_cohort_missingness_and_assumptions() {
        let ready = evaluate_roh_ibd_workflow_boundary("roh", 35.0, 10.0, 28, 10, 0.02, 0.05, true);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.roh");
        assert!(ready.refusal_codes.is_empty());

        let refused =
            evaluate_roh_ibd_workflow_boundary("unknown", 1.0, 10.0, 4, 10, 0.12, 0.05, false);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"unsupported_roh_ibd_method".to_string()));
        assert!(refused.refusal_codes.contains(&"marker_density_below_minimum".to_string()));
        assert!(refused.refusal_codes.contains(&"cohort_size_below_minimum".to_string()));
        assert!(refused.refusal_codes.contains(&"missingness_above_maximum".to_string()));
        assert!(refused.refusal_codes.contains(&"method_assumptions_required".to_string()));
    }

    #[test]
    fn evaluate_demography_refusal_boundary_reports_precise_missing_assumptions() {
        let ready =
            evaluate_demography_refusal_boundary("ibdne", 40, 20, true, true, true, true, true);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.stage_id, "vcf.demography");
        assert!(ready.refusal_codes.is_empty());
        assert!(ready.missing_assumptions.is_empty());

        let refused = evaluate_demography_refusal_boundary(
            "unknown", 3, 20, false, false, false, false, false,
        );
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"unsupported_demography_model".to_string()));
        assert!(refused
            .refusal_codes
            .contains(&"cohort_size_below_demography_minimum".to_string()));
        assert!(refused.refusal_codes.contains(&"ibd_segments_required".to_string()));
        assert!(refused.refusal_codes.contains(&"marker_density_gate_failed".to_string()));
        assert!(refused.refusal_codes.contains(&"missingness_gate_failed".to_string()));
        assert!(refused.refusal_codes.contains(&"method_assumptions_required".to_string()));
        assert!(refused.refusal_codes.contains(&"temporal_metadata_required".to_string()));
        assert!(refused.missing_assumptions.contains(&"adequate_cohort_size".to_string()));
        assert!(refused.missing_assumptions.contains(&"method_assumptions_documented".to_string()));
    }

    #[test]
    fn build_panel_reference_drift_report_flags_changed_surfaces_and_invalidated_outputs() {
        let baseline = VcfPanelReferenceSnapshotV1 {
            label: "baseline".to_string(),
            reference_build: "GRCh38".to_string(),
            reference_fasta_sha256: "aaa".to_string(),
            panel_id: "panel_v1".to_string(),
            map_id: "map_v1".to_string(),
            contig_alias_digest: "digest_a".to_string(),
        };
        let candidate = VcfPanelReferenceSnapshotV1 {
            label: "candidate".to_string(),
            reference_build: "GRCh38".to_string(),
            reference_fasta_sha256: "bbb".to_string(),
            panel_id: "panel_v2".to_string(),
            map_id: "map_v1".to_string(),
            contig_alias_digest: "digest_b".to_string(),
        };

        let report = build_panel_reference_drift_report(
            &baseline,
            &candidate,
            &["vcf.impute.vcf.gz", "vcf.phasing.vcf.gz", "vcf.impute.vcf.gz"],
        );
        assert!(report.requires_reprocessing);
        assert!(report.changed_surfaces.contains(&"reference_fasta".to_string()));
        assert!(report.changed_surfaces.contains(&"panel_id".to_string()));
        assert!(report.changed_surfaces.contains(&"contig_alias_digest".to_string()));
        assert_eq!(report.invalidated_outputs.len(), 2);

        let stable = build_panel_reference_drift_report(&baseline, &baseline, &["noop"]);
        assert!(!stable.requires_reprocessing);
        assert!(stable.changed_surfaces.is_empty());
        assert!(stable.invalidated_outputs.is_empty());
    }

    #[test]
    fn evaluate_structural_variant_support_boundary_requires_explicit_sv_support() {
        let ready = evaluate_structural_variant_support_boundary(
            true,
            true,
            Some("sniffles2"),
            &["DEL", "INS", "INV"],
            "call",
            false,
        );
        assert!(ready.prerequisites_passed);
        assert!(ready.support_declared);
        assert_eq!(ready.stage_id, "vcf.postprocess");

        let refused =
            evaluate_structural_variant_support_boundary(true, false, None, &[], "unknown", true);
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"explicit_sv_mode_required".to_string()));
        assert!(refused.refusal_codes.contains(&"sv_backend_required".to_string()));
        assert!(refused.refusal_codes.contains(&"supported_sv_types_required".to_string()));
        assert!(refused.refusal_codes.contains(&"unsupported_sv_operation".to_string()));
        assert!(refused.refusal_codes.contains(&"small_variant_coercion_forbidden".to_string()));
    }

    #[test]
    fn execute_annotation_provenance_workflow_tracks_source_version_mapping_and_field_coverage() {
        let ready = execute_annotation_provenance_workflow(
            "vep",
            "110",
            &["gene", "transcript", "impact"],
            &["gene", "transcript", "impact"],
            Some("ensembl_gene"),
            Some("ensembl_transcript"),
            0.90,
        );
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.annotation_source, "vep");
        assert!((ready.field_coverage - 1.0).abs() < f64::EPSILON);
        assert!(ready.refusal_codes.is_empty());

        let refused = execute_annotation_provenance_workflow(
            "",
            "",
            &["gene", "impact", "clinvar"],
            &["gene"],
            None,
            None,
            0.80,
        );
        assert!(!refused.prerequisites_passed);
        assert!(refused.refusal_codes.contains(&"annotation_source_required".to_string()));
        assert!(refused.refusal_codes.contains(&"annotation_version_required".to_string()));
        assert!(refused.refusal_codes.contains(&"gene_mapping_required".to_string()));
        assert!(refused.refusal_codes.contains(&"transcript_mapping_required".to_string()));
        assert!(refused
            .refusal_codes
            .contains(&"annotation_field_coverage_below_minimum".to_string()));
    }

    #[test]
    fn evaluate_vcf_population_analysis_handoff_enforces_typed_and_caveated_transfer() {
        let ready =
            evaluate_vcf_population_analysis_handoff("pca", true, true, true, true, true, true);
        assert!(ready.prerequisites_passed);
        assert_eq!(ready.target_analysis, "pca");
        assert!(ready.refusal_codes.is_empty());

        let refused = evaluate_vcf_population_analysis_handoff(
            "unknown", false, false, false, false, false, false,
        );
        assert!(!refused.prerequisites_passed);
        assert!(refused
            .refusal_codes
            .contains(&"unsupported_population_analysis_target".to_string()));
        assert!(refused.refusal_codes.contains(&"filtered_vcf_required".to_string()));
        assert!(refused.refusal_codes.contains(&"normalized_vcf_required".to_string()));
        assert!(refused.refusal_codes.contains(&"sample_identity_mismatch".to_string()));
        assert!(refused.refusal_codes.contains(&"reference_identity_mismatch".to_string()));
        assert!(refused.refusal_codes.contains(&"trust_class_incompatible".to_string()));
        assert!(refused.refusal_codes.contains(&"caveats_required_for_handoff".to_string()));
    }

    fn repo_fixture_path(relative_path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../").join(relative_path)
    }

    #[test]
    fn summarize_vcf_genotype_truth_reports_diploid_counts_from_governed_fixture() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_diploid/raw.diploid.vcf",
        );

        let summary =
            summarize_vcf_genotype_truth(&input, "vcf.call_diploid", "bcftools").expect("summary");
        assert_eq!(summary.schema_version, VCF_GENOTYPE_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.sample_count, 1);
        assert_eq!(summary.variant_count, 3);
        assert_eq!(summary.observed_ploidy_widths, vec![2]);
        assert_eq!(summary.called_calls, 3);
        assert_eq!(summary.missing_calls, 0);
        assert_eq!(summary.reference_only_calls, 1);
        assert_eq!(summary.mixed_allele_calls, 1);
        assert_eq!(summary.alternate_only_calls, 1);
        assert_eq!(summary.phased_calls, 0);
        assert_eq!(summary.unphased_calls, 3);
        assert_eq!(summary.per_sample_missingness.get("sample_a"), Some(&0.0));
        assert!(summary.likelihood_fields_present.is_empty());
    }

    #[test]
    fn summarize_vcf_genotype_truth_reports_haploid_missingness_from_governed_fixture() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/angsd/vcf.call_pseudohaploid/raw.pseudohaploid.vcf",
        );

        let summary = summarize_vcf_genotype_truth(&input, "vcf.call_pseudohaploid", "angsd")
            .expect("summary");
        assert_eq!(summary.observed_ploidy_widths, vec![1]);
        assert_eq!(summary.called_calls, 2);
        assert_eq!(summary.missing_calls, 1);
        assert_eq!(summary.reference_only_calls, 1);
        assert_eq!(summary.mixed_allele_calls, 0);
        assert_eq!(summary.alternate_only_calls, 1);
        assert_eq!(summary.unphased_calls, 2);
        assert_eq!(summary.per_sample_missingness.get("sample_lowcov"), Some(&(1.0 / 3.0)));
    }

    #[test]
    fn summarize_vcf_genotype_truth_reports_single_likelihood_field_truth() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call_gl/raw.gl.vcf",
        );

        let summary =
            summarize_vcf_genotype_truth(&input, "vcf.call_gl", "bcftools").expect("summary");
        assert_eq!(summary.observed_ploidy_widths, vec![2]);
        assert_eq!(summary.likelihood_fields_present, vec!["PL".to_string()]);
        assert_eq!(summary.sites_with_likelihood_values.get("PL"), Some(&3));
        assert!(summary.sites_missing_likelihood_values.is_empty());
    }

    #[test]
    fn summarize_vcf_genotype_truth_reports_gl_gp_pl_field_sets() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.gl_propagation/raw.propagated.vcf",
        );

        let summary = summarize_vcf_genotype_truth(&input, "vcf.gl_propagation", "bcftools")
            .expect("summary");
        assert_eq!(
            summary.likelihood_fields_present,
            vec!["GL".to_string(), "GP".to_string(), "PL".to_string()]
        );
        assert_eq!(summary.sites_with_likelihood_values.get("GL"), Some(&3));
        assert_eq!(summary.sites_with_likelihood_values.get("GP"), Some(&3));
        assert_eq!(summary.sites_with_likelihood_values.get("PL"), Some(&3));
        assert!(summary.sites_missing_likelihood_values.is_empty());
    }

    #[test]
    fn summarize_vcf_filter_output_truth_reports_known_filter_labels_from_governed_fixture() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.filter/raw.filtered.vcf",
        );

        let summary =
            summarize_vcf_filter_output_truth(&input, "vcf.filter", "bcftools").expect("summary");
        assert_eq!(summary.schema_version, VCF_FILTER_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.sample_count, 1);
        assert_eq!(summary.variant_count, 5);
        assert_eq!(summary.pass_variant_count, 1);
        assert_eq!(summary.failed_variant_count, 4);
        assert_eq!(
            summary.observed_filter_ids,
            vec![
                "HIGH_MISSING".to_string(),
                "LOWQUAL".to_string(),
                "LOW_DP".to_string(),
                "LOW_MQ".to_string()
            ]
        );
        assert_eq!(summary.per_filter_variant_count.get("LOWQUAL"), Some(&1));
        assert_eq!(summary.per_filter_variant_count.get("LOW_DP"), Some(&1));
        assert_eq!(summary.per_filter_variant_count.get("HIGH_MISSING"), Some(&1));
        assert_eq!(summary.per_filter_sites.get("LOWQUAL"), Some(&vec!["chr1:20".to_string()]));
        assert_eq!(summary.pass_sites, vec!["chr1:10".to_string()]);
    }

    #[test]
    fn summarize_vcf_filter_output_truth_reports_retained_damage_filter_sites() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.damage_filter/raw.damage_filtered.vcf",
        );

        let summary = summarize_vcf_filter_output_truth(&input, "vcf.damage_filter", "bcftools")
            .expect("summary");
        assert_eq!(summary.variant_count, 2);
        assert_eq!(summary.pass_variant_count, 2);
        assert_eq!(summary.failed_variant_count, 0);
        assert!(summary.observed_filter_ids.is_empty());
        assert_eq!(summary.pass_sites, vec!["chr1:10".to_string(), "chr1:40".to_string()]);
    }

    #[test]
    fn summarize_vcf_phasing_output_truth_reports_phase_sets_from_governed_fixture() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/phasing/shapeit5/raw.phased.vcf",
        );

        let summary =
            summarize_vcf_phasing_output_truth(&input, "vcf.phasing", "shapeit5").expect("summary");
        assert_eq!(summary.schema_version, VCF_PHASING_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.sample_count, 2);
        assert_eq!(summary.sample_ids, vec!["cohort_alpha".to_string(), "cohort_beta".to_string()]);
        assert_eq!(summary.variant_count, 4);
        assert_eq!(summary.called_genotype_count, 8);
        assert_eq!(summary.phased_genotype_count, 8);
        assert_eq!(summary.unphased_genotype_count, 0);
        assert_eq!(summary.phase_set_count, 4);
        assert_eq!(
            summary.phase_sets_by_sample.get("cohort_alpha"),
            Some(&vec!["101".to_string(), "145".to_string()])
        );
        assert_eq!(
            summary.phase_sets_by_sample.get("cohort_beta"),
            Some(&vec!["101".to_string(), "145".to_string()])
        );
    }

    #[test]
    fn summarize_vcf_imputation_output_truth_reports_concordance_and_info_scores() {
        let input = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.impute/raw.imputed.vcf",
        );
        let truth = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.impute/raw.truth.vcf",
        );

        let summary =
            summarize_vcf_imputation_output_truth(&input, Some(&truth), "vcf.impute", "beagle")
                .expect("summary");
        assert_eq!(summary.schema_version, VCF_IMPUTATION_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.sample_count, 2);
        assert_eq!(
            summary.sample_ids,
            vec!["masked_sample".to_string(), "donor_sample".to_string()]
        );
        assert_eq!(summary.variant_count, 2);
        assert_eq!(summary.sites_with_info_score, 2);
        assert_eq!(summary.sites_with_r2_score, 2);
        assert_eq!(summary.mean_info_score, Some(0.825));
        assert_eq!(summary.mean_r2_score, Some(0.775));
        assert_eq!(summary.masked_truth_site_count, 1);
        assert_eq!(summary.masked_truth_match_count, 1);
        assert_eq!(summary.masked_truth_mismatch_count, 0);
        assert_eq!(summary.unresolved_count, 0);
        assert_eq!(summary.genotype_concordance, Some(1.0));
    }

    #[test]
    fn summarize_vcf_pca_output_truth_joins_coordinates_to_metadata() {
        let artifact_root =
            repo_fixture_path("benchmarks/tests/fixtures/bench/parsers/vcf/eigensoft/pca");
        let metadata = repo_fixture_path(
            "benchmarks/tests/fixtures/corpora/vcf-mini/metadata/sample_metadata.tsv",
        );
        let metrics =
            crate::parse_eigensoft_stage_metrics(crate::VcfDomainStage::Pca, &artifact_root)
                .expect("parse eigensoft pca metrics");

        let summary = summarize_vcf_pca_output_truth(&metrics, &metadata).expect("summary");
        assert_eq!(summary.schema_version, VCF_PCA_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.stage_id, "vcf.pca");
        assert_eq!(summary.tool_id, "eigensoft");
        assert_eq!(summary.sample_count, 4);
        assert_eq!(summary.joined_sample_count, 4);
        assert_eq!(summary.pair_count, 6);
        assert_eq!(
            summary.population_ids,
            vec!["cohort_alpha".to_string(), "cohort_beta".to_string()]
        );
        assert_eq!(summary.rows[0].sample_id, "sample_a");
        assert_eq!(summary.rows[0].sex, "female");
        assert_eq!(summary.rows[0].role, "cohort");
        assert_eq!(summary.rows[0].pc1, 0.01);
        assert_eq!(summary.rows[0].pc2, 0.02);
        assert_eq!(summary.pairwise_distances[0].left_sample_id, "sample_a");
        assert_eq!(summary.pairwise_distances[0].right_sample_id, "sample_b");
        assert_eq!(summary.pairwise_distances[0].distance, 0.01414213562373095);
    }

    #[test]
    fn summarize_vcf_admixture_output_truth_tracks_cluster_fractions_with_metadata() {
        let artifact_root =
            repo_fixture_path("benchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.admixture");
        let metadata = repo_fixture_path(
            "benchmarks/tests/fixtures/corpora/vcf-mini/metadata/sample_metadata.tsv",
        );
        let metrics =
            crate::parse_plink2_stage_metrics(crate::VcfDomainStage::Admixture, &artifact_root)
                .expect("parse plink2 admixture metrics");

        let summary = summarize_vcf_admixture_output_truth(&metrics, &metadata).expect("summary");
        assert_eq!(summary.schema_version, VCF_ADMIXTURE_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.stage_id, "vcf.admixture");
        assert_eq!(summary.tool_id, "plink2");
        assert_eq!(summary.selected_k, 2);
        assert_eq!(summary.sample_count, 4);
        assert_eq!(summary.joined_sample_count, 4);
        assert_eq!(summary.population_count, 2);
        assert_eq!(summary.cluster_headers, vec!["cluster_1".to_string(), "cluster_2".to_string()]);
        assert_eq!(summary.rows[0].sample_id, "sample_a");
        assert_eq!(summary.rows[0].dominant_cluster, "cluster_1");
        assert_eq!(summary.rows[0].dominant_fraction, 1.0);
        assert_eq!(summary.rows[0].cluster_fractions.get("cluster_2"), Some(&0.0));
        assert_eq!(summary.rows[2].sample_id, "sample_c");
        assert_eq!(summary.rows[2].dominant_cluster, "cluster_2");
        assert_eq!(summary.rows[2].sex, "female");
    }

    #[test]
    fn summarize_vcf_population_structure_output_truth_reports_distance_summary() {
        let artifact_root = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/plink2/vcf.population_structure",
        );
        let metadata = repo_fixture_path(
            "benchmarks/tests/fixtures/corpora/vcf-mini/metadata/sample_metadata.tsv",
        );
        let metrics = crate::parse_plink2_stage_metrics(
            crate::VcfDomainStage::PopulationStructure,
            &artifact_root,
        )
        .expect("parse plink2 population structure metrics");

        let summary =
            summarize_vcf_population_structure_output_truth(&metrics, &metadata).expect("summary");
        assert_eq!(summary.schema_version, VCF_POPULATION_STRUCTURE_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.stage_id, "vcf.population_structure");
        assert_eq!(summary.tool_id, "plink2");
        assert_eq!(summary.sample_count, 4);
        assert_eq!(summary.joined_sample_count, 4);
        assert_eq!(summary.pair_count, 6);
        assert_eq!(summary.within_population_pair_count, 2);
        assert_eq!(summary.cross_population_pair_count, 4);
        assert_eq!(summary.min_pc_distance, 0.01414213562373095);
        assert_eq!(summary.max_pc_distance, 0.042426406871192854);
        assert_eq!(summary.mean_pc_distance, 0.023570226039551587);
        assert_eq!(summary.sample_groups[0].dominant_cluster, "cluster_1");
        assert_eq!(summary.sample_groups[2].dominant_cluster, "cluster_2");
        assert_eq!(summary.sample_groups[1].role, "cohort");
    }

    #[test]
    fn summarize_vcf_roh_output_truth_reports_interval_rows() {
        let artifact_root = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/segments/plink2/vcf.roh/complete",
        );
        let metrics = crate::parse_segment_stage_metrics(
            "plink2",
            crate::VcfDomainStage::Roh,
            &artifact_root,
        )
        .expect("parse plink2 roh metrics");

        let summary = summarize_vcf_roh_output_truth(&metrics).expect("summary");
        assert_eq!(summary.schema_version, VCF_ROH_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.stage_id, "vcf.roh");
        assert_eq!(summary.tool_id, "plink2");
        assert_eq!(summary.status, "complete");
        assert_eq!(summary.sample_count, 4);
        assert_eq!(summary.segment_count, 8);
        assert_eq!(summary.total_length, 8);
        assert_eq!(
            summary.sample_ids,
            vec![
                "sample_a".to_string(),
                "sample_b".to_string(),
                "sample_c".to_string(),
                "sample_d".to_string()
            ]
        );
        assert_eq!(summary.segments[0].sample_id, "sample_a");
        assert_eq!(summary.segments[0].start, 3);
        assert_eq!(summary.segments[7].sample_id, "sample_d");
        assert_eq!(summary.segments[7].end, 8);
        assert_eq!(summary.per_sample_summary[0].segment_count, 2);
        assert_eq!(summary.per_sample_summary[0].mean_length, 1.0);
    }

    #[test]
    fn summarize_vcf_ibd_output_truth_tracks_pair_lengths_and_marker_overlap() {
        let artifact_root = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/segments/germline/vcf.ibd/complete",
        );
        let metrics = crate::parse_segment_stage_metrics(
            "germline",
            crate::VcfDomainStage::Ibd,
            &artifact_root,
        )
        .expect("parse germline ibd metrics");

        let summary = summarize_vcf_ibd_output_truth(&metrics).expect("summary");
        assert_eq!(summary.schema_version, VCF_IBD_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(summary.stage_id, "vcf.ibd");
        assert_eq!(summary.tool_id, "germline");
        assert_eq!(summary.status, "complete");
        assert_eq!(summary.pair_count, 1);
        assert_eq!(summary.retained_segment_count, 2);
        assert_eq!(summary.total_length, 9.5);
        assert_eq!(summary.overlap_marker_total, 41);
        assert_eq!(summary.sample_ids, vec!["sample_a".to_string(), "sample_b".to_string()]);
        assert_eq!(summary.rows[0].sample_a, "sample_a");
        assert_eq!(summary.rows[0].sample_b, "sample_b");
        assert_eq!(summary.rows[0].segment_count, 2);
        assert_eq!(summary.insufficient_overlap_probe.status, "not_run");
    }

    #[test]
    fn summarize_vcf_ibd_output_truth_preserves_marker_overlap_insufficiency() {
        let artifact_root = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/segments/ibdseq/vcf.ibd/insufficient_marker_overlap",
        );
        let metrics = crate::parse_segment_stage_metrics(
            "ibdseq",
            crate::VcfDomainStage::Ibd,
            &artifact_root,
        )
        .expect("parse ibdseq insufficiency metrics");

        let summary = summarize_vcf_ibd_output_truth(&metrics).expect("summary");
        assert_eq!(summary.status, "insufficient_marker_overlap");
        assert_eq!(
            summary.insufficient_reason.as_deref(),
            Some("no_pairs_met_min_marker_or_length_threshold")
        );
        assert_eq!(summary.pair_count, 0);
        assert_eq!(summary.retained_segment_count, 0);
        assert_eq!(summary.total_length, 0.0);
        assert_eq!(summary.overlap_marker_total, 0);
        assert!(summary.sample_ids.is_empty());
        assert_eq!(summary.insufficient_overlap_probe.status, "insufficient_marker_overlap");
    }

    #[test]
    fn summarize_vcf_demography_output_truth_preserves_complete_and_insufficient_states() {
        let complete_root = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/segments/ibdne/vcf.demography/complete",
        );
        let complete_metrics = crate::parse_segment_stage_metrics(
            "ibdne",
            crate::VcfDomainStage::Demography,
            &complete_root,
        )
        .expect("parse complete demography metrics");
        let complete_summary =
            summarize_vcf_demography_output_truth(&complete_metrics).expect("complete summary");
        assert_eq!(complete_summary.schema_version, VCF_DEMOGRAPHY_OUTPUT_TRUTH_SCHEMA_VERSION);
        assert_eq!(complete_summary.status, "complete");
        assert_eq!(complete_summary.estimate_count, 3);
        assert_eq!(complete_summary.time_bins, vec![5, 10, 20]);
        assert_eq!(complete_summary.ne_estimates[0].ne, 12000.0);
        assert_eq!(complete_summary.insufficient_data_probe.status, "not_run");

        let insufficient_root = repo_fixture_path(
            "benchmarks/tests/fixtures/bench/parsers/vcf/segments/ibdne/vcf.demography/insufficient_data",
        );
        let insufficient_metrics = crate::parse_segment_stage_metrics(
            "ibdne",
            crate::VcfDomainStage::Demography,
            &insufficient_root,
        )
        .expect("parse insufficient demography metrics");
        let insufficient_summary = summarize_vcf_demography_output_truth(&insufficient_metrics)
            .expect("insufficient summary");
        assert_eq!(insufficient_summary.status, "insufficient_data");
        assert_eq!(
            insufficient_summary.insufficient_reason.as_deref(),
            Some("not_enough_ibd_segments")
        );
        assert_eq!(insufficient_summary.estimate_count, 0);
        assert!(insufficient_summary.time_bins.is_empty());
        assert!(insufficient_summary.ne_estimates.is_empty());
        assert_eq!(insufficient_summary.insufficient_data_probe.status, "insufficient_data");
    }
}
