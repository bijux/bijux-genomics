use super::values::{
    insert_bool_field_from, insert_f64_field_from, insert_string_field_from, insert_u64_field_from,
    parse_cli_arg_pairs,
};
use crate::artifacts::{
    artifact_bundle_manifest_fields, artifact_bundle_size_bytes, sha256_artifact_bundle,
};
use crate::cache::resolve_deplete_rrna_stage_options;
use crate::qc::{report_qc_contributor_tool_ids, report_qc_upstream_stage_ids};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
pub(crate) fn collect_stage_manifest_fields(
    stage_id: &str,
    stage_args: &[String],
    manifest_args: &[String],
) -> Result<BTreeMap<String, serde_json::Value>> {
    let stage_values = parse_cli_arg_pairs("stage-arg", stage_args)?;
    let manifest_values = parse_cli_arg_pairs("manifest-arg", manifest_args)?;
    let mut fields = BTreeMap::new();

    match stage_id {
        "fastq.normalize_primers" => {
            insert_string_field_from(&mut fields, &stage_values, "primer_set_id");
            insert_string_field_from(&mut fields, &stage_values, "orientation_policy");
            insert_f64_field_from(&mut fields, &stage_values, "max_mismatch_rate")?;
            insert_u64_field_from(&mut fields, &stage_values, "min_overlap_bp")?;
            insert_bool_field_from(&mut fields, &stage_values, "strict_5p_anchor")?;
            insert_bool_field_from(&mut fields, &stage_values, "allow_iupac_codes")?;
        }
        "fastq.remove_duplicates" => {
            insert_string_field_from(&mut fields, &stage_values, "dedup_mode");
            insert_bool_field_from(&mut fields, &stage_values, "keep_order")?;
        }
        "fastq.deplete_host" => {
            if let Some(reference_index) = stage_values.get("reference_index") {
                fields.extend(artifact_bundle_manifest_fields(
                    "reference_index",
                    "reference_index",
                    Path::new(reference_index),
                )?);
            }
            insert_string_field_from(&mut fields, &manifest_values, "reference_catalog_id");
            insert_string_field_from(&mut fields, &manifest_values, "reference_index_backend");
            insert_f64_field_from(&mut fields, &stage_values, "host_identity_threshold")?;
            insert_bool_field_from(&mut fields, &stage_values, "retain_unmapped_only")?;
        }
        "fastq.deplete_reference_contaminants" => {
            if let Some(reference_index) = stage_values.get("reference_index") {
                fields.extend(artifact_bundle_manifest_fields(
                    "reference_index",
                    "reference_index",
                    Path::new(reference_index),
                )?);
            }
            insert_string_field_from(&mut fields, &manifest_values, "reference_catalog_id");
            insert_string_field_from(&mut fields, &manifest_values, "reference_index_backend");
            insert_string_field_from(&mut fields, &stage_values, "decoy_mode");
        }
        "fastq.screen_taxonomy" => {
            if let Some(database_root) = stage_values.get("database_root") {
                fields.extend(artifact_bundle_manifest_fields(
                    "database_root",
                    "database",
                    Path::new(database_root),
                )?);
            }
            insert_string_field_from(&mut fields, &manifest_values, "database_catalog_id");
            insert_string_field_from(&mut fields, &manifest_values, "database_artifact_id");
            insert_string_field_from(&mut fields, &manifest_values, "database_namespace");
            insert_string_field_from(&mut fields, &manifest_values, "database_scope");
        }
        "fastq.deplete_rrna" => {
            let options = resolve_deplete_rrna_stage_options(stage_args)?;
            fields.insert(
                "rrna_db".to_string(),
                serde_json::Value::String(options.rrna_db.display().to_string()),
            );
            fields.insert(
                "rrna_bundle_id".to_string(),
                serde_json::Value::String(options.rrna_bundle_id),
            );
            fields.insert(
                "rrna_bundle_digest".to_string(),
                serde_json::Value::String(sha256_artifact_bundle(&options.rrna_db)?),
            );
            fields.insert(
                "rrna_bundle_size_bytes".to_string(),
                serde_json::Value::Number(serde_json::Number::from(artifact_bundle_size_bytes(
                    &options.rrna_db,
                )?)),
            );
            fields.insert("min_identity".to_string(), serde_json::json!(options.min_identity));
        }
        "fastq.report_qc" => add_qc_manifest_fields(&mut fields),
        _ => {}
    }

    Ok(fields)
}

fn add_qc_manifest_fields(fields: &mut BTreeMap<String, serde_json::Value>) {
    fields
        .insert("aggregation_engine".to_string(), serde_json::Value::String("multiqc".to_string()));
    fields.insert(
        "aggregation_scope".to_string(),
        serde_json::Value::String("governed_qc_artifacts".to_string()),
    );
    fields.insert("report_only".to_string(), serde_json::Value::Bool(true));
    fields.insert("mutates_fastq".to_string(), serde_json::Value::Bool(false));
    fields.insert("may_change_read_count".to_string(), serde_json::Value::Bool(false));
    fields.insert(
        "governed_contributor_stage_ids".to_string(),
        serde_json::Value::Array(
            report_qc_upstream_stage_ids()
                .iter()
                .map(|row| serde_json::Value::String(row.clone()))
                .collect(),
        ),
    );
    fields.insert(
        "governed_contributor_tool_ids".to_string(),
        serde_json::Value::Array(
            report_qc_contributor_tool_ids().into_iter().map(serde_json::Value::String).collect(),
        ),
    );
}
