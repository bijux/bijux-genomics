use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VCF_SCIENTIFIC_DRIFT_REPORT_SCHEMA_VERSION: &str = "bijux.vcf.scientific_drift.report.v1";
pub const VCF_VALIDATION_SUMMARY_SCHEMA_VERSION: &str = "bijux.vcf.validation_summary.v1";
pub const VCF_STATS_WORKFLOW_SCHEMA_VERSION: &str = "bijux.vcf.stats_workflow.v1";
pub const VCF_FILTER_CONSEQUENCE_SCHEMA_VERSION: &str = "bijux.vcf.filter_consequence.v1";
pub const VCF_NORMALIZATION_SUMMARY_SCHEMA_VERSION: &str = "bijux.vcf.normalization_summary.v1";
pub const VCF_REFERENCE_CONTEXT_SCHEMA_VERSION: &str = "bijux.vcf.reference_context_resolution.v1";
pub const VCF_DAMAGE_FILTER_SUMMARY_SCHEMA_VERSION: &str = "bijux.vcf.damage_filter_summary.v1";
pub const VCF_DIPLOID_CALLING_BOUNDARY_SCHEMA_VERSION: &str =
    "bijux.vcf.calling_boundary.diploid.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VcfScientificDriftChangeKind {
    DefaultsChange,
    BackendChange,
    FilterPolicyChange,
    NormalizationPolicyChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftSnapshotV1 {
    pub label: String,
    pub stage_id: String,
    pub tool_id: String,
    pub backend_version: Option<String>,
    pub defaults_fingerprint: Option<String>,
    pub normalization_policy_id: Option<String>,
    pub filter_policy_id: Option<String>,
    pub metrics: BTreeMap<String, f64>,
    pub artifacts: BTreeMap<String, String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftMetricDeltaV1 {
    pub metric_id: String,
    pub baseline_value: f64,
    pub candidate_value: f64,
    pub absolute_delta: f64,
    pub relative_delta: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftArtifactDeltaV1 {
    pub artifact_id: String,
    pub baseline_hash: Option<String>,
    pub candidate_hash: Option<String>,
    pub changed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfScientificDriftReportV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub baseline_label: String,
    pub candidate_label: String,
    pub baseline_tool_id: String,
    pub candidate_tool_id: String,
    pub change_kinds: Vec<VcfScientificDriftChangeKind>,
    pub metric_deltas: Vec<VcfScientificDriftMetricDeltaV1>,
    pub artifact_deltas: Vec<VcfScientificDriftArtifactDeltaV1>,
    pub downstream_risks: Vec<String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfValidationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_vcf: PathBuf,
    pub record_count: u64,
    pub sample_count: u32,
    pub header_valid: bool,
    pub sorted_records: bool,
    pub has_index: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfStatsWorkflowSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub variant_count: u64,
    pub sample_count: u32,
    pub snv_count: u64,
    pub indel_count: u64,
    #[serde(default)]
    pub ti_tv_ratio: Option<f64>,
    pub missing_genotype_calls: u64,
    #[serde(default)]
    pub filter_counts: BTreeMap<String, u64>,
    #[serde(default)]
    pub per_sample_missingness: BTreeMap<String, f64>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfFilterConsequenceV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub filter_expression: String,
    pub variants_in: u64,
    pub variants_retained: u64,
    pub variants_removed: u64,
    #[serde(default)]
    pub reason_counts: BTreeMap<String, u64>,
    pub output_subset_identity: String,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfNormalizationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub records_in: u64,
    pub records_out: u64,
    pub records_changed: u64,
    pub split_multiallelic_records: u64,
    pub duplicate_records_after_normalization: u64,
    pub raw_view_preserved: bool,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfReferenceContextResolutionV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub reference_build: String,
    pub panel_build: String,
    #[serde(default)]
    pub genetic_map_build: Option<String>,
    pub contigs_observed: u32,
    pub alias_mappings_used: u32,
    pub fasta_present: bool,
    pub fai_present: bool,
    pub panel_compatible: bool,
    pub genetic_map_compatible: bool,
    pub passes: bool,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VcfDamageFilterSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub action: String,
    pub prerequisites_passed: bool,
    pub variants_in: u64,
    pub damage_risk_sites: u64,
    pub removed_sites: u64,
    pub masked_sites: u64,
    pub annotated_sites: u64,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct VcfCallingBoundaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub mode: String,
    pub prerequisites_passed: bool,
    pub confidence: f64,
    #[serde(default)]
    pub refusal_codes: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
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
        sample_count: doc.samples.len() as u32,
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
    let keys = format.split(':').collect::<Vec<_>>();
    let values = sample_payload.split(':').collect::<Vec<_>>();
    let gt_index = keys.iter().position(|key| *key == "GT")?;
    values.get(gt_index).copied()
}

fn genotype_is_missing(gt: &str) -> bool {
    matches!(gt, "." | "./." | ".|." | "./" | ".|")
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
    let ti_tv_ratio =
        if transversions > 0 { Some(transitions as f64 / transversions as f64) } else { None };
    let per_sample_missingness = doc
        .samples
        .iter()
        .enumerate()
        .map(|(index, sample)| {
            let total = total_by_sample[index];
            let missing = missing_by_sample[index];
            let ratio = if total > 0 { missing as f64 / total as f64 } else { 0.0 };
            (sample.clone(), ratio)
        })
        .collect::<BTreeMap<_, _>>();
    Ok(VcfStatsWorkflowSummaryV1 {
        schema_version: VCF_STATS_WORKFLOW_SCHEMA_VERSION.to_string(),
        stage_id: "vcf.stats".to_string(),
        variant_count: doc.records.len() as u64,
        sample_count: doc.samples.len() as u32,
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
            let missing_fraction =
                if total_calls > 0 { missing_calls as f64 / total_calls as f64 } else { 0.0 };
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
    let output_subset_identity =
        format!("vcf.filter:{}:{}:{}", variants_in, retained, filter_expression);
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
        contigs_observed: doc.contigs.len() as u32,
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

#[must_use]
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
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bijux-vcf-{label}-{stamp}"));
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
}
