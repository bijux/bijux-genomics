use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BamNormalizedMetricsStageContract {
    pub(crate) stage_id: &'static str,
    pub(crate) extension_id: &'static str,
}

pub(crate) const BAM_NORMALIZED_METRICS_SCHEMA_VERSION: &str = "bijux.bam.metrics.normalized.v1";
pub(crate) const BAM_NORMALIZED_METRICS_SCHEMA_ID: &str =
    "bijux.schemas.bench.bam-normalized-metrics.v1";

const BAM_NORMALIZED_METRICS_REQUIRED_KEYS: &[&str] = &[
    "schema_version",
    "stage_id",
    "tool_id",
    "tool_version",
    "execution",
    "outputs_count",
    "artifacts",
    "contracts",
    "normalized_keys",
];

const BAM_NORMALIZED_KEYS: &[&str] = &[
    "stage_id",
    "tool_id",
    "tool_version",
    "execution.runtime_s",
    "execution.memory_mb",
    "execution.exit_code",
    "artifacts.required_count",
    "artifacts.missing_required",
    "contracts.bam_invariants",
    "contracts.output_contract",
    "contracts.tool_wrapper",
];

const BAM_NORMALIZED_METRICS_STAGE_CONTRACTS: &[BamNormalizedMetricsStageContract] = &[
    BamNormalizedMetricsStageContract {
        stage_id: "bam.align",
        extension_id: "bam_align_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.validate",
        extension_id: "bam_validate_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.qc_pre",
        extension_id: "bam_qc_pre_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.mapping_summary",
        extension_id: "bam_mapping_summary_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.filter",
        extension_id: "bam_filter_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.mapq_filter",
        extension_id: "bam_mapq_filter_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.length_filter",
        extension_id: "bam_length_filter_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.markdup",
        extension_id: "bam_markdup_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.duplication_metrics",
        extension_id: "bam_duplication_metrics_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.complexity",
        extension_id: "bam_complexity_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.coverage",
        extension_id: "bam_coverage_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.insert_size",
        extension_id: "bam_insert_size_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.gc_bias",
        extension_id: "bam_gc_bias_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.endogenous_content",
        extension_id: "bam_endogenous_content_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.overlap_correction",
        extension_id: "bam_overlap_correction_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.damage",
        extension_id: "bam_damage_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.contamination",
        extension_id: "bam_contamination_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.sex",
        extension_id: "bam_sex_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.bias_mitigation",
        extension_id: "bam_bias_mitigation_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.recalibration",
        extension_id: "bam_recalibration_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.haplogroups",
        extension_id: "bam_haplogroups_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.genotyping",
        extension_id: "bam_genotyping_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.kinship",
        extension_id: "bam_kinship_normalized_v1",
    },
    BamNormalizedMetricsStageContract {
        stage_id: "bam.authenticity",
        extension_id: "bam_authenticity_normalized_v1",
    },
];

#[cfg(test)]
pub(crate) fn bam_normalized_metrics_stage_contracts(
) -> &'static [BamNormalizedMetricsStageContract] {
    BAM_NORMALIZED_METRICS_STAGE_CONTRACTS
}

pub(crate) fn bam_normalized_metrics_contract_for_stage(
    stage_id: &str,
) -> Option<&'static BamNormalizedMetricsStageContract> {
    BAM_NORMALIZED_METRICS_STAGE_CONTRACTS.iter().find(|contract| contract.stage_id == stage_id)
}

pub(crate) fn validate_bam_normalized_metrics(
    metrics: &serde_json::Value,
) -> Result<&'static BamNormalizedMetricsStageContract> {
    let object = metrics
        .as_object()
        .ok_or_else(|| anyhow!("BAM normalized metrics payload must be a JSON object"))?;
    let schema_version =
        object.get("schema_version").and_then(serde_json::Value::as_str).ok_or_else(|| {
            anyhow!("BAM normalized metrics payload is missing string `schema_version`")
        })?;
    if schema_version != BAM_NORMALIZED_METRICS_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM normalized metrics payload uses unsupported schema_version `{schema_version}`"
        ));
    }
    let stage_id = object
        .get("stage_id")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("BAM normalized metrics payload is missing string `stage_id`"))?;
    let contract = bam_normalized_metrics_contract_for_stage(stage_id).ok_or_else(|| {
        anyhow!("BAM normalized metrics payload declares unknown stage `{stage_id}`")
    })?;

    for key in BAM_NORMALIZED_METRICS_REQUIRED_KEYS {
        if !object.contains_key(*key) {
            return Err(anyhow!(
                "BAM normalized metrics payload for `{stage_id}` is missing key `{key}`"
            ));
        }
    }

    if !object.get("tool_id").is_some_and(serde_json::Value::is_string) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing string `tool_id`"
        ));
    }
    if !object.get("tool_version").is_some_and(serde_json::Value::is_string) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing string `tool_version`"
        ));
    }
    if !object.get("outputs_count").is_some_and(serde_json::Value::is_u64) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing integer `outputs_count`"
        ));
    }

    validate_execution_object(stage_id, object.get("execution"))?;
    validate_artifacts_object(stage_id, object.get("artifacts"))?;
    validate_contracts_object(stage_id, object.get("contracts"))?;
    validate_normalized_keys(stage_id, object.get("normalized_keys"))?;

    Ok(contract)
}

fn validate_execution_object(stage_id: &str, value: Option<&serde_json::Value>) -> Result<()> {
    let object = value.and_then(serde_json::Value::as_object).ok_or_else(|| {
        anyhow!("BAM normalized metrics payload for `{stage_id}` is missing object `execution`")
    })?;
    if !object.get("runtime_s").is_some_and(serde_json::Value::is_number) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing numeric `execution.runtime_s`"
        ));
    }
    if !object.get("memory_mb").is_some_and(serde_json::Value::is_number) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing numeric `execution.memory_mb`"
        ));
    }
    if !object.get("exit_code").is_some_and(serde_json::Value::is_i64) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing integer `execution.exit_code`"
        ));
    }
    let Some(status) = object.get("status").and_then(serde_json::Value::as_str) else {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing string `execution.status`"
        ));
    };
    if !matches!(status, "ok" | "failed") {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` uses unsupported `execution.status` `{status}`"
        ));
    }
    Ok(())
}

fn validate_artifacts_object(stage_id: &str, value: Option<&serde_json::Value>) -> Result<()> {
    let object = value.and_then(serde_json::Value::as_object).ok_or_else(|| {
        anyhow!("BAM normalized metrics payload for `{stage_id}` is missing object `artifacts`")
    })?;
    if !object.get("required_count").is_some_and(serde_json::Value::is_u64) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing integer `artifacts.required_count`"
        ));
    }
    let Some(missing_required) =
        object.get("missing_required").and_then(serde_json::Value::as_array)
    else {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing array `artifacts.missing_required`"
        ));
    };
    if !missing_required.iter().all(serde_json::Value::is_string) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` must keep `artifacts.missing_required` as a string array"
        ));
    }
    let Some(items) = object.get("items").and_then(serde_json::Value::as_object) else {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` is missing object `artifacts.items`"
        ));
    };
    for (artifact_name, artifact_value) in items {
        let artifact = artifact_value.as_object().ok_or_else(|| {
            anyhow!(
                "BAM normalized metrics payload for `{stage_id}` has non-object artifact entry `{artifact_name}`"
            )
        })?;
        if !artifact.get("path").is_some_and(serde_json::Value::is_string) {
            return Err(anyhow!(
                "BAM normalized metrics payload for `{stage_id}` artifact `{artifact_name}` is missing string `path`"
            ));
        }
        if !artifact.get("exists").is_some_and(serde_json::Value::is_boolean) {
            return Err(anyhow!(
                "BAM normalized metrics payload for `{stage_id}` artifact `{artifact_name}` is missing boolean `exists`"
            ));
        }
    }
    Ok(())
}

fn validate_contracts_object(stage_id: &str, value: Option<&serde_json::Value>) -> Result<()> {
    let object = value.and_then(serde_json::Value::as_object).ok_or_else(|| {
        anyhow!("BAM normalized metrics payload for `{stage_id}` is missing object `contracts`")
    })?;
    for key in ["bam_invariants", "output_contract", "tool_wrapper"] {
        if !object.get(key).is_some_and(serde_json::Value::is_boolean) {
            return Err(anyhow!(
                "BAM normalized metrics payload for `{stage_id}` is missing boolean `contracts.{key}`"
            ));
        }
    }
    Ok(())
}

fn validate_normalized_keys(stage_id: &str, value: Option<&serde_json::Value>) -> Result<()> {
    let expected = serde_json::json!(BAM_NORMALIZED_KEYS);
    if value != Some(&expected) {
        return Err(anyhow!(
            "BAM normalized metrics payload for `{stage_id}` must keep the governed `normalized_keys` ordering"
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn render_bam_normalized_metrics_schema() -> serde_json::Value {
    let stage_ids = BAM_NORMALIZED_METRICS_STAGE_CONTRACTS
        .iter()
        .map(|contract| contract.stage_id)
        .collect::<Vec<_>>();
    let mut stage_defs = serde_json::Map::new();
    let mut one_of = Vec::new();

    for contract in BAM_NORMALIZED_METRICS_STAGE_CONTRACTS {
        stage_defs.insert(
            contract.stage_id.to_string(),
            serde_json::json!({
                "allOf": [
                    { "$ref": "#/$defs/base" },
                    {
                        "type": "object",
                        "properties": {
                            "stage_id": { "const": contract.stage_id }
                        },
                        "required": BAM_NORMALIZED_METRICS_REQUIRED_KEYS,
                        "additionalProperties": true,
                        "x-bijux-extension-id": contract.extension_id
                    }
                ]
            }),
        );
        one_of.push(serde_json::json!({
            "$ref": format!("#/$defs/stages/{}", contract.stage_id)
        }));
    }

    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": BAM_NORMALIZED_METRICS_SCHEMA_ID,
        "title": "BAM normalized benchmark metrics",
        "description": "Governed normalized parser outputs for BAM benchmark stages. Each payload must satisfy the shared envelope and exactly one stage-specific extension.",
        "type": "object",
        "oneOf": one_of,
        "$defs": {
            "artifact_item": {
                "type": "object",
                "required": ["path", "exists"],
                "properties": {
                    "path": { "type": "string" },
                    "exists": { "type": "boolean" }
                },
                "additionalProperties": true
            },
            "base": {
                "type": "object",
                "required": BAM_NORMALIZED_METRICS_REQUIRED_KEYS,
                "properties": {
                    "schema_version": { "const": BAM_NORMALIZED_METRICS_SCHEMA_VERSION },
                    "stage_id": {
                        "type": "string",
                        "enum": stage_ids
                    },
                    "tool_id": { "type": "string" },
                    "tool_version": { "type": "string" },
                    "execution": {
                        "type": "object",
                        "required": ["runtime_s", "memory_mb", "exit_code", "status"],
                        "properties": {
                            "runtime_s": { "type": "number" },
                            "memory_mb": { "type": "number" },
                            "exit_code": { "type": "integer" },
                            "status": { "enum": ["ok", "failed"] }
                        },
                        "additionalProperties": true
                    },
                    "outputs_count": { "type": "integer", "minimum": 0 },
                    "artifacts": {
                        "type": "object",
                        "required": ["required_count", "missing_required", "items"],
                        "properties": {
                            "required_count": { "type": "integer", "minimum": 0 },
                            "missing_required": {
                                "type": "array",
                                "items": { "type": "string" }
                            },
                            "items": {
                                "type": "object",
                                "additionalProperties": { "$ref": "#/$defs/artifact_item" }
                            }
                        },
                        "additionalProperties": true
                    },
                    "contracts": {
                        "type": "object",
                        "required": ["bam_invariants", "output_contract", "tool_wrapper"],
                        "properties": {
                            "bam_invariants": { "type": "boolean" },
                            "output_contract": { "type": "boolean" },
                            "tool_wrapper": { "type": "boolean" }
                        },
                        "additionalProperties": true
                    },
                    "normalized_keys": { "const": BAM_NORMALIZED_KEYS }
                },
                "additionalProperties": true
            },
            "stages": stage_defs
        }
    })
}
