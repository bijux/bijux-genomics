use super::declarations::FILTER_STAGE_SPECS;
use super::models::FilterStageSchemaContract;
use crate::proofs::inputs::SchemaContext;
use anyhow::anyhow;
use anyhow::Result;
use std::collections::BTreeMap;
pub(super) fn collect_filter_stage_schema_contracts(
    context: &SchemaContext<'_>,
) -> Result<BTreeMap<String, FilterStageSchemaContract>> {
    let schema = context.schema.document();
    let stage_defs = schema
        .get("$defs")
        .and_then(|value| value.get("stages"))
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            anyhow!("FASTQ normalized metrics schema is missing object `$defs.stages`")
        })?;

    let mut contracts = BTreeMap::new();
    for spec in FILTER_STAGE_SPECS {
        let stage_contract = stage_defs
            .get(spec.stage_id)
            .ok_or_else(|| {
                anyhow!("FASTQ normalized metrics schema is missing `{}`", spec.stage_id)
            })?
            .get("allOf")
            .and_then(serde_json::Value::as_array)
            .and_then(|items| items.get(1))
            .ok_or_else(|| {
                anyhow!(
                    "FASTQ normalized metrics stage `{}` is missing stage extension",
                    spec.stage_id
                )
            })?;
        let extension_id = stage_contract
            .get("x-bijux-extension-id")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                anyhow!(
                    "FASTQ normalized metrics stage `{}` is missing string `x-bijux-extension-id`",
                    spec.stage_id
                )
            })?;
        let required_fields = stage_contract
            .get("required")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                anyhow!(
                    "FASTQ normalized metrics stage `{}` is missing `required` keys",
                    spec.stage_id
                )
            })?
            .iter()
            .map(|value| {
                value.as_str().map(str::to_string).ok_or_else(|| {
                    anyhow!(
                        "FASTQ normalized metrics stage `{}` has non-string required key",
                        spec.stage_id
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        contracts.insert(
            spec.stage_id.to_string(),
            FilterStageSchemaContract { extension_id: extension_id.to_string(), required_fields },
        );
    }

    Ok(contracts)
}
