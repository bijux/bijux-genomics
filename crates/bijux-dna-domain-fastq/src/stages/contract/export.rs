use anyhow::anyhow;

use crate::metrics::spec::metric_spec_for_stage;
use crate::stage_compatible_tool_ids;
use bijux_dna_core::contract::canonical::canonicalize_json_value;
use bijux_dna_core::prelude::hashing::params_hash;

use super::catalog::contract_for_stage;

fn tool_ids_for_stage(stage_id: &str) -> Vec<String> {
    stage_compatible_tool_ids(stage_id).unwrap_or_default()
}

#[must_use]
pub fn stage_contract_json(stage_id: &str) -> Option<serde_json::Value> {
    let contract = contract_for_stage(stage_id)?;
    let metrics = metric_spec_for_stage(stage_id).map(|spec| {
        serde_json::json!({
            "classes": spec.classes.iter().map(|class| format!("{class:?}")).collect::<Vec<_>>(),
            "invariants": spec.invariants,
            "notes": spec.notes,
        })
    });
    let input_kind = format!("{:?}", contract.input_kind);
    let output_kind = format!("{:?}", contract.output_kind);
    let accepted_input_kinds = contract
        .accepted_input_kinds
        .iter()
        .map(|kind| format!("{kind:?}"))
        .collect::<Vec<_>>();
    let possible_output_kinds = contract
        .possible_output_kinds
        .iter()
        .map(|kind| format!("{kind:?}"))
        .collect::<Vec<_>>();
    let mut payload = serde_json::json!({
        "schema_version": "bijux.stage_contract.v1",
        "stage_id": stage_id,
        "inputs": {
            "kind": input_kind,
            "accepted_kinds": accepted_input_kinds,
        },
        "outputs": {
            "kind": output_kind,
            "possible_kinds": possible_output_kinds,
            "emits_fastq": contract.emits_fastq,
        },
        "retention": {
            "may_drop_reads": contract.may_drop_reads,
            "must_preserve_pairing": contract.must_preserve_pairing,
            "preserves": contract.preserves,
            "may_drop": contract.may_drop,
            "definition": contract.retention_definition,
            "units": contract.retention_units,
        },
        "metrics": metrics.unwrap_or_else(|| serde_json::json!({})),
        "tool_ids": tool_ids_for_stage(stage_id),
    });
    let amplicon_semantics = match stage_id {
        "fastq.normalize_primers" => Some(serde_json::json!({
            "orientation_policy": "normalize_to_primer_forward_orientation",
            "primer_assumptions": ["primer_set_declared", "primer_match_confidence>=0.9"],
        })),
        "fastq.trim_terminal_damage" => Some(serde_json::json!({
            "damage_policy": "terminal_mask_or_trim",
            "udg_classification": "configured_or_inferred",
            "aligner_compatibility": "refuse_if_requested_output_breaks_downstream_expectations",
        })),
        "fastq.remove_chimeras" => Some(serde_json::json!({
            "chimera_removed_definition": "reads flagged as de_novo/reference chimeras are excluded from downstream abundance tables"
        })),
        "fastq.infer_asvs" => Some(serde_json::json!({
            "decision_semantics": "legal in amplicon mode; outputs denoised sequence variants"
        })),
        "fastq.cluster_otus" => Some(serde_json::json!({
            "decision_semantics": "legal in amplicon mode when ASV path disabled; outputs clustered centroids and a canonical governed clustering report"
        })),
        _ => None,
    };
    if let (Some(map), Some(semantics)) = (payload.as_object_mut(), amplicon_semantics) {
        map.insert("amplicon_semantics".to_string(), semantics);
    }
    Some(payload)
}

/// # Errors
/// Returns an error if JSON canonicalization fails.
#[must_use]
pub fn stage_contract_hash(stage_id: &str) -> Option<anyhow::Result<String>> {
    let json = stage_contract_json(stage_id)?;
    let canonical = canonicalize_json_value(&json);
    Some(params_hash(&canonical).map_err(|err| anyhow!("{err}")))
}
