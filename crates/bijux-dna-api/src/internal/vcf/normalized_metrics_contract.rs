use anyhow::{anyhow, bail, Result};
use bijux_dna_stages_vcf::stage_specs::{vcf_stage_catalog, VcfStageSpec};
use serde::{Deserialize, Serialize};

const JSON_SCHEMA_DRAFT_2020_12: &str = "https://json-schema.org/draft/2020-12/schema";
pub(crate) const VCF_NORMALIZED_METRICS_SCHEMA_ID: &str =
    "bijux.schemas.bench.vcf-normalized-metrics.v1";

const BASE_REQUIRED_KEYS: &[&str] = &["schema_version", "stage_id", "tool_id"];

const CALL_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("variant_count", FieldKind::Integer),
    field("snp_count", FieldKind::Integer),
    field("indel_count", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
];
const CALL_DIPLOID_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("ploidy", FieldKind::String),
    field("called_genotypes", FieldKind::Integer),
    field("heterozygous_count", FieldKind::Integer),
    field("homozygous_ref_count", FieldKind::Integer),
    field("homozygous_alt_count", FieldKind::Integer),
    field("missing_count", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
];
const CALL_PSEUDOHAPLOID_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("target_sites", FieldKind::Integer),
    field("covered_sites", FieldKind::Integer),
    field("called_sites", FieldKind::Integer),
    field("missing_sites", FieldKind::Integer),
    field("sampling_policy", FieldKind::String),
    field("random_seed", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
];
const CALL_GL_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("likelihood_field", FieldKind::String),
    field("sites_with_likelihoods", FieldKind::Integer),
    field("samples_with_likelihoods", FieldKind::Integer),
    field("missing_likelihoods", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
];
const DAMAGE_FILTER_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("input_variants", FieldKind::Integer),
    field("removed_variants", FieldKind::Integer),
    field("retained_variants", FieldKind::Integer),
    field("low_quality_filtered_variants", FieldKind::Integer),
    field("damage_ratio_filtered_variants", FieldKind::Integer),
    field("terminal_damage_filtered_variants", FieldKind::Integer),
    field("damage_context_rule", FieldKind::String),
    field("terminal_context_count", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
];
const GL_PROPAGATION_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("input_likelihood_fields", FieldKind::StringArray),
    field("output_likelihood_fields", FieldKind::StringArray),
    field("lost_fields", FieldKind::StringArray),
    field("site_count_before", FieldKind::Integer),
    field("site_count_after", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
];
const QC_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("sample_missingness", FieldKind::ObjectArray),
    field("variant_missingness", FieldKind::ObjectArray),
    field("maf_summary", FieldKind::Object),
    field("heterozygosity", FieldKind::Object),
    field("excluded_samples", FieldKind::ObjectArray),
    field("excluded_variants", FieldKind::ObjectArray),
    field("sample_missingness_exclusion_threshold", FieldKind::Number),
    field("variant_missingness_exclusion_threshold", FieldKind::Number),
];
const PCA_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("variant_count", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
    field("excluded_samples", FieldKind::StringArray),
    field("unexpected_samples", FieldKind::StringArray),
    field("eigenvalues", FieldKind::NumberArray),
    field("rows", FieldKind::ObjectArray),
];
const POPULATION_STRUCTURE_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("consumed_pca", FieldKind::Object),
    field("consumed_admixture", FieldKind::Object),
    field("sample_groups", FieldKind::ObjectArray),
    field("distance_summary", FieldKind::Object),
    field("status", FieldKind::String),
];
const ADMIXTURE_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("selected_k", FieldKind::Integer),
    field("status", FieldKind::String),
    field("sample_count", FieldKind::Integer),
    field("population_count", FieldKind::Integer),
    field("cluster_headers", FieldKind::StringArray),
    field("rows", FieldKind::ObjectArray),
];
const ROH_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("status", FieldKind::String),
    field("sample_count", FieldKind::Integer),
    field("segment_count", FieldKind::Integer),
    field("total_length", FieldKind::Integer),
    field("segments", FieldKind::ObjectArray),
    field("per_sample_summary", FieldKind::ObjectArray),
];
const IBD_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("status", FieldKind::String),
    field("pair_count", FieldKind::Integer),
    field("rows", FieldKind::ObjectArray),
    field("insufficient_overlap_probe", FieldKind::Object),
];
const IMPUTATION_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("status", FieldKind::String),
    field("mean_info_score", FieldKind::Number),
    field("r2_available", FieldKind::Boolean),
    field("low_confidence_sites", FieldKind::Integer),
    field("masked_truth_sites", FieldKind::Integer),
    field("missing_quality_fields", FieldKind::StringArray),
];
const IMPUTE_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("variant_count", FieldKind::Integer),
    field("missing_before", FieldKind::Integer),
    field("missing_after", FieldKind::Integer),
    field("imputed_genotypes", FieldKind::Integer),
    field("low_confidence_count", FieldKind::Integer),
    field("masked_truth_site_count", FieldKind::Integer),
    field("masked_truth_match_count", FieldKind::Integer),
    field("unresolved_count", FieldKind::Integer),
    field("not_imputable_reasons", FieldKind::Object),
    field("sample_count", FieldKind::Integer),
    field("sample_ids", FieldKind::StringArray),
];
const PHASING_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("input_genotypes", FieldKind::Integer),
    field("phased_genotypes", FieldKind::Integer),
    field("unphased_genotypes", FieldKind::Integer),
    field("phase_set_count", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
    field("sample_ids", FieldKind::StringArray),
];
const POSTPROCESS_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("readable_vcf", FieldKind::Boolean),
    field("tabix_present", FieldKind::Boolean),
    field("contigs_consistent_with_species_context", FieldKind::Boolean),
    field("left_align_applied", FieldKind::Boolean),
    field("multiallelic_records_split", FieldKind::Integer),
    field("indels_normalized", FieldKind::Integer),
    field("variant_ids_normalized", FieldKind::Integer),
    field("invalid_records_removed", FieldKind::Integer),
    field("filter_standardized_to_pass", FieldKind::Integer),
];
const PREPARE_REFERENCE_PANEL_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("input_variants", FieldKind::Integer),
    field("output_variants", FieldKind::Integer),
    field("sample_count", FieldKind::Integer),
    field("sample_ids", FieldKind::StringArray),
    field("sample_consistent", FieldKind::Boolean),
    field("duplicate_sites_removed", FieldKind::Integer),
    field("normalization_status", FieldKind::String),
    field("parseable", FieldKind::Boolean),
];
const DEMOGRAPHY_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("method", FieldKind::String),
    field("inference_status", FieldKind::String),
    field("status", FieldKind::String),
    field("time_bins", FieldKind::IntegerArray),
    field("ne_estimates", FieldKind::ObjectArray),
    field("insufficient_data_probe", FieldKind::Object),
];
const FILTER_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("input_variants", FieldKind::Integer),
    field("pass_variants", FieldKind::Integer),
    field("failed_variants", FieldKind::Integer),
    field("filter_ids", FieldKind::StringArray),
    field("depth_threshold", FieldKind::Number),
    field("quality_threshold", FieldKind::Number),
    field("missingness_threshold", FieldKind::Number),
    field("sample_count", FieldKind::Integer),
];
const STATS_FIELDS: &[VcfNormalizedMetricsFieldContract] = &[
    field("variant_count", FieldKind::Integer),
    field("snp_count", FieldKind::Integer),
    field("indel_count", FieldKind::Integer),
    field("transition_count", FieldKind::Integer),
    field("transversion_count", FieldKind::Integer),
    field("ti_tv", FieldKind::Number),
    field("sample_count", FieldKind::Integer),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldKind {
    String,
    Integer,
    Number,
    Boolean,
    StringArray,
    IntegerArray,
    NumberArray,
    Object,
    ObjectArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VcfNormalizedMetricsFieldContract {
    pub(crate) key: &'static str,
    pub(crate) kind: FieldKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VcfNormalizedMetricsStageSchemaDescriptor {
    pub stage_id: String,
    pub schema_version: String,
    pub schema_id: String,
    pub file_name: String,
    pub extension_id: String,
    pub required_key_count: usize,
}

const fn field(key: &'static str, kind: FieldKind) -> VcfNormalizedMetricsFieldContract {
    VcfNormalizedMetricsFieldContract { key, kind }
}

pub(crate) fn vcf_normalized_metrics_stage_descriptors(
) -> Result<Vec<VcfNormalizedMetricsStageSchemaDescriptor>> {
    let mut descriptors = vcf_stage_catalog()
        .iter()
        .filter(|spec| spec.parser_supported)
        .map(stage_descriptor_from_spec)
        .collect::<Result<Vec<_>>>()?;
    descriptors.sort_by(|left, right| left.stage_id.cmp(&right.stage_id));
    Ok(descriptors)
}

pub(crate) fn render_vcf_normalized_metrics_schema() -> Result<serde_json::Value> {
    let mut stage_defs = serde_json::Map::new();
    let mut one_of = Vec::new();
    let mut stage_ids = Vec::new();

    for spec in parser_supported_stage_specs()? {
        let descriptor = stage_descriptor_from_spec(spec)?;
        let stage_schema = render_stage_extension(spec)?;
        stage_ids.push(spec.stage_id);
        stage_defs.insert(spec.stage_id.to_string(), stage_schema);
        one_of.push(serde_json::json!({
            "$ref": format!("#/$defs/stages/{}", spec.stage_id)
        }));
        if descriptor.required_key_count == 0 {
            bail!(
                "VCF normalized metrics descriptor for `{}` cannot have zero required keys",
                descriptor.stage_id
            );
        }
    }

    Ok(serde_json::json!({
        "$schema": JSON_SCHEMA_DRAFT_2020_12,
        "$id": VCF_NORMALIZED_METRICS_SCHEMA_ID,
        "title": "VCF normalized benchmark metrics",
        "description": "Governed normalized parser outputs for VCF benchmark stages. Each payload must satisfy the shared envelope and exactly one stage-specific extension.",
        "type": "object",
        "oneOf": one_of,
        "$defs": {
            "base": {
                "type": "object",
                "required": BASE_REQUIRED_KEYS,
                "properties": {
                    "schema_version": { "type": "string" },
                    "stage_id": { "type": "string", "enum": stage_ids },
                    "tool_id": { "type": "string" }
                },
                "additionalProperties": true
            },
            "stages": stage_defs
        }
    }))
}

pub(crate) fn render_vcf_normalized_metrics_stage_schema(
    stage_id: &str,
) -> Result<serde_json::Value> {
    let spec = parser_supported_stage_specs()?
        .into_iter()
        .find(|spec| spec.stage_id == stage_id)
        .ok_or_else(|| anyhow!("unknown parser-supported VCF stage `{stage_id}`"))?;
    let descriptor = stage_descriptor_from_spec(spec)?;
    Ok(serde_json::json!({
        "$schema": JSON_SCHEMA_DRAFT_2020_12,
        "$id": descriptor.schema_id,
        "title": format!("VCF normalized benchmark metrics: {}", spec.stage_id),
        "description": format!("Governed normalized parser output for `{}`.", spec.stage_id),
        "allOf": [
            { "$ref": "../vcf-normalized-metrics.v1.json#/$defs/base" },
            render_stage_extension(spec)?
        ]
    }))
}

pub(crate) fn validate_vcf_normalized_metrics(
    metrics: &serde_json::Value,
) -> Result<VcfNormalizedMetricsStageSchemaDescriptor> {
    let object = metrics
        .as_object()
        .ok_or_else(|| anyhow!("VCF normalized metrics payload must be a JSON object"))?;
    let schema_version =
        object.get("schema_version").and_then(serde_json::Value::as_str).ok_or_else(|| {
            anyhow!("VCF normalized metrics payload is missing string `schema_version`")
        })?;
    let stage_id = object
        .get("stage_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("VCF normalized metrics payload is missing string `stage_id`"))?;
    let descriptor = vcf_normalized_metrics_stage_descriptors()?
        .into_iter()
        .find(|descriptor| descriptor.stage_id == stage_id)
        .ok_or_else(|| {
            anyhow!("VCF normalized metrics payload declares unknown stage `{stage_id}`")
        })?;
    if schema_version != descriptor.schema_version {
        bail!(
            "VCF normalized metrics payload for `{stage_id}` uses unsupported schema_version `{schema_version}`"
        );
    }
    if !object.get("tool_id").is_some_and(serde_json::Value::is_string) {
        bail!("VCF normalized metrics payload for `{stage_id}` is missing string `tool_id`");
    }

    for field in fields_for_stage(stage_id)? {
        let Some(value) = object.get(field.key) else {
            bail!("VCF normalized metrics payload for `{stage_id}` is missing key `{}`", field.key);
        };
        validate_field_type(stage_id, field, value)?;
    }

    Ok(descriptor)
}

fn parser_supported_stage_specs() -> Result<Vec<&'static VcfStageSpec>> {
    let specs = vcf_stage_catalog().iter().filter(|spec| spec.parser_supported).collect::<Vec<_>>();
    if specs.is_empty() {
        bail!("VCF normalized metrics contract requires at least one parser-supported stage");
    }
    Ok(specs)
}

fn stage_descriptor_from_spec(
    spec: &VcfStageSpec,
) -> Result<VcfNormalizedMetricsStageSchemaDescriptor> {
    let stage_suffix = stage_suffix(spec.stage_id)?;
    let required_key_count = BASE_REQUIRED_KEYS.len() + fields_for_stage(spec.stage_id)?.len();
    Ok(VcfNormalizedMetricsStageSchemaDescriptor {
        stage_id: spec.stage_id.to_string(),
        schema_version: spec.metrics_schema.to_string(),
        schema_id: format!("bijux.schemas.bench.vcf-normalized-metrics.{stage_suffix}.v1"),
        file_name: format!("{stage_suffix}.v1.json"),
        extension_id: format!("vcf_{}_normalized_v1", stage_suffix.replace('-', "_")),
        required_key_count,
    })
}

fn render_stage_extension(spec: &VcfStageSpec) -> Result<serde_json::Value> {
    let descriptor = stage_descriptor_from_spec(spec)?;
    let mut properties = serde_json::Map::new();
    properties
        .insert("schema_version".to_string(), serde_json::json!({ "const": spec.metrics_schema }));
    properties.insert("stage_id".to_string(), serde_json::json!({ "const": spec.stage_id }));
    properties.insert("tool_id".to_string(), serde_json::json!({ "type": "string" }));
    let mut required = BASE_REQUIRED_KEYS.iter().map(|key| (*key).to_string()).collect::<Vec<_>>();
    for field in fields_for_stage(spec.stage_id)? {
        properties.insert(field.key.to_string(), field_kind_schema(field.kind));
        required.push(field.key.to_string());
    }

    Ok(serde_json::json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": true,
        "x-bijux-extension-id": descriptor.extension_id
    }))
}

fn fields_for_stage(stage_id: &str) -> Result<&'static [VcfNormalizedMetricsFieldContract]> {
    match stage_id {
        "vcf.call" => Ok(CALL_FIELDS),
        "vcf.call_diploid" => Ok(CALL_DIPLOID_FIELDS),
        "vcf.call_gl" => Ok(CALL_GL_FIELDS),
        "vcf.call_pseudohaploid" => Ok(CALL_PSEUDOHAPLOID_FIELDS),
        "vcf.damage_filter" => Ok(DAMAGE_FILTER_FIELDS),
        "vcf.demography" => Ok(DEMOGRAPHY_FIELDS),
        "vcf.filter" => Ok(FILTER_FIELDS),
        "vcf.gl_propagation" => Ok(GL_PROPAGATION_FIELDS),
        "vcf.ibd" => Ok(IBD_FIELDS),
        "vcf.imputation" => Ok(IMPUTATION_FIELDS),
        "vcf.impute" => Ok(IMPUTE_FIELDS),
        "vcf.pca" => Ok(PCA_FIELDS),
        "vcf.phasing" => Ok(PHASING_FIELDS),
        "vcf.population_structure" => Ok(POPULATION_STRUCTURE_FIELDS),
        "vcf.postprocess" => Ok(POSTPROCESS_FIELDS),
        "vcf.prepare_reference_panel" => Ok(PREPARE_REFERENCE_PANEL_FIELDS),
        "vcf.qc" => Ok(QC_FIELDS),
        "vcf.roh" => Ok(ROH_FIELDS),
        "vcf.stats" => Ok(STATS_FIELDS),
        "vcf.admixture" => Ok(ADMIXTURE_FIELDS),
        other => Err(anyhow!(
            "VCF normalized metrics contract is missing stage field mapping for `{other}`"
        )),
    }
}

fn field_kind_schema(kind: FieldKind) -> serde_json::Value {
    match kind {
        FieldKind::String => serde_json::json!({ "type": "string" }),
        FieldKind::Integer => serde_json::json!({ "type": "integer", "minimum": 0 }),
        FieldKind::Number => serde_json::json!({ "type": "number" }),
        FieldKind::Boolean => serde_json::json!({ "type": "boolean" }),
        FieldKind::StringArray => {
            serde_json::json!({ "type": "array", "items": { "type": "string" } })
        }
        FieldKind::IntegerArray => {
            serde_json::json!({ "type": "array", "items": { "type": "integer", "minimum": 0 } })
        }
        FieldKind::NumberArray => {
            serde_json::json!({ "type": "array", "items": { "type": "number" } })
        }
        FieldKind::Object => serde_json::json!({ "type": "object" }),
        FieldKind::ObjectArray => {
            serde_json::json!({ "type": "array", "items": { "type": "object" } })
        }
    }
}

fn validate_field_type(
    stage_id: &str,
    field: &VcfNormalizedMetricsFieldContract,
    value: &serde_json::Value,
) -> Result<()> {
    let valid = match field.kind {
        FieldKind::String => value.is_string(),
        FieldKind::Integer => value.is_u64(),
        FieldKind::Number => value.is_number(),
        FieldKind::Boolean => value.is_boolean(),
        FieldKind::StringArray => {
            value.as_array().is_some_and(|items| items.iter().all(serde_json::Value::is_string))
        }
        FieldKind::IntegerArray => {
            value.as_array().is_some_and(|items| items.iter().all(serde_json::Value::is_u64))
        }
        FieldKind::NumberArray => {
            value.as_array().is_some_and(|items| items.iter().all(serde_json::Value::is_number))
        }
        FieldKind::Object => value.is_object(),
        FieldKind::ObjectArray => {
            value.as_array().is_some_and(|items| items.iter().all(serde_json::Value::is_object))
        }
    };
    if !valid {
        bail!(
            "VCF normalized metrics payload for `{stage_id}` has invalid type for `{}`",
            field.key
        );
    }
    Ok(())
}

fn stage_suffix(stage_id: &str) -> Result<String> {
    let suffix = stage_id
        .strip_prefix("vcf.")
        .ok_or_else(|| anyhow!("VCF stage id `{stage_id}` must start with `vcf.`"))?;
    Ok(suffix.replace('_', "-"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vcf_normalized_metrics_contract_tracks_parser_supported_stage_specs() {
        let descriptors =
            vcf_normalized_metrics_stage_descriptors().expect("descriptors must render");
        let parser_supported_specs =
            vcf_stage_catalog().iter().filter(|spec| spec.parser_supported).collect::<Vec<_>>();

        assert_eq!(descriptors.len(), parser_supported_specs.len());
        for spec in parser_supported_specs {
            let descriptor = descriptors
                .iter()
                .find(|descriptor| descriptor.stage_id == spec.stage_id)
                .unwrap_or_else(|| panic!("missing descriptor for {}", spec.stage_id));
            assert_eq!(descriptor.schema_version, spec.metrics_schema);
            assert!(descriptor.required_key_count > BASE_REQUIRED_KEYS.len());
        }
    }

    #[test]
    fn vcf_normalized_metrics_schema_declares_every_stage_extension() {
        let schema = render_vcf_normalized_metrics_schema().expect("schema must render");
        let stages = schema
            .get("$defs")
            .and_then(|value| value.get("stages"))
            .and_then(serde_json::Value::as_object)
            .expect("stage defs");
        let descriptors =
            vcf_normalized_metrics_stage_descriptors().expect("descriptors must render");

        assert_eq!(stages.len(), descriptors.len());
        for descriptor in descriptors {
            let stage = stages
                .get(&descriptor.stage_id)
                .unwrap_or_else(|| panic!("missing shared schema stage {}", descriptor.stage_id));
            let extension = stage
                .get("x-bijux-extension-id")
                .and_then(serde_json::Value::as_str)
                .expect("extension id");
            assert_eq!(extension, descriptor.extension_id);
        }
    }

    #[test]
    fn vcf_stage_specific_schema_carries_stage_identity_and_required_keys() {
        let schema = render_vcf_normalized_metrics_stage_schema("vcf.qc")
            .expect("stage-specific schema must render");
        assert_eq!(
            schema.get("$id").and_then(serde_json::Value::as_str),
            Some("bijux.schemas.bench.vcf-normalized-metrics.qc.v1")
        );
        let extension = schema
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .expect("stage extension");
        assert_eq!(
            extension
                .get("properties")
                .and_then(|value| value.get("stage_id"))
                .and_then(|value| value.get("const"))
                .and_then(serde_json::Value::as_str),
            Some("vcf.qc")
        );
        let required =
            extension.get("required").and_then(serde_json::Value::as_array).expect("required keys");
        assert!(required.contains(&serde_json::json!("sample_missingness")));
        assert!(required.contains(&serde_json::json!("maf_summary")));
    }
}
