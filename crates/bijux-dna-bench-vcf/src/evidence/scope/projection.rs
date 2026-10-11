use super::{
    model::{BindingKey, RetainedVcfBinding},
    status::{
        ACTIVE_SCOPE_DETAIL, ACTIVE_SCOPE_PROOF_PATH, ACTIVE_SCOPE_STATE,
        BENCHMARK_NOT_READY_SCOPE_DETAIL, BENCHMARK_NOT_READY_SCOPE_PROOF_PATH,
        COMPLETE_SCOPE_DETAIL, COMPLETE_SCOPE_PROOF_PATH, COMPLETE_SCOPE_STATE,
        LIFECYCLE_NOT_ACTIVE_SCOPE_DETAIL, LIFECYCLE_NOT_ACTIVE_SCOPE_PROOF_PATH,
        REMOVED_FROM_SCOPE_STATE,
    },
};
use anyhow::{anyhow, Result};
use bijux_dna_bench_model::{VcfStageCatalogRow, VcfStageMatrixRow};
use bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
/// Project retained VCF scope from explicit active and expected binding identities.
///
/// # Errors
/// Rejects ambiguous or foreign identities, malformed retained registries and governed scope drift.
pub fn collect_vcf_active_stage_tool_matrix_rows_from_expected(
    inputs: &crate::inputs::VcfAdapterInputs,
    repo_root: &Path,
    active: &[bijux_dna_bench_vcf_evidence::snapshot::VcfStageReadinessBindingKey],
    expected: &[bijux_dna_bench_vcf_evidence::model::expectations::VcfExpectedBenchmarkResultRow],
) -> Result<Vec<VcfActiveStageToolMatrixRow>> {
    let matrix_by_pair = super::admission::admitted_matrix(inputs, active, expected)?;
    let retained_bindings = super::registry::load_retained_vcf_bindings(repo_root)?;
    let catalog_by_stage =
        inputs.catalog().iter().map(|row| (row.stage_id.as_str(), row)).collect::<BTreeMap<_, _>>();
    let schema_id_by_stage=bijux_dna_bench_model::normalized_result::vcf_envelope::vcf_normalized_metrics_stage_descriptors()?.into_iter().map(|row|(row.stage_id,row.schema_id)).collect::<BTreeMap<_,_>>();
    let active_bindings = active
        .iter()
        .map(|row| BindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() })
        .collect::<BTreeSet<_>>();
    let complete_bindings = expected
        .iter()
        .map(|row| BindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() })
        .collect::<BTreeSet<_>>();
    let mut rows = Vec::with_capacity(retained_bindings.len());
    for binding in retained_bindings {
        let catalog_row = catalog_by_stage.get(binding.stage_id.as_str()).ok_or_else(|| {
            anyhow!(
                "VCF active-stage-tool matrix is missing stage catalog coverage for `{}`",
                binding.stage_id
            )
        })?;
        let stage_matrix_row = matrix_by_pair
            .get(binding.stage_id.as_str())
            .and_then(|tools| tools.get(binding.tool_id.as_str()))
            .copied()
            .ok_or_else(|| {
                anyhow!(
                    "VCF active-stage-tool matrix is missing stage matrix coverage for `{}`",
                    binding.stage_id
                )
            })?;
        let schema_id = schema_id_by_stage.get(binding.stage_id.as_str()).cloned().ok_or_else(|| {
            anyhow!(
                "VCF active-stage-tool matrix is missing normalized metrics schema coverage for `{}`",
                binding.stage_id
            )
        })?;

        rows.push(build_vcf_active_stage_tool_matrix_row(
            binding,
            catalog_row,
            stage_matrix_row,
            &schema_id,
            &active_bindings,
            &complete_bindings,
        ));
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    super::contract::ensure_vcf_active_stage_tool_matrix_contract(&rows)?;
    Ok(rows)
}
fn build_vcf_active_stage_tool_matrix_row(
    binding: RetainedVcfBinding,
    catalog_row: &VcfStageCatalogRow,
    stage_matrix_row: &VcfStageMatrixRow,
    schema_id: &str,
    active_bindings: &BTreeSet<BindingKey>,
    complete_bindings: &BTreeSet<BindingKey>,
) -> VcfActiveStageToolMatrixRow {
    let binding_key =
        BindingKey { stage_id: binding.stage_id.clone(), tool_id: binding.tool_id.clone() };
    let (scope_state, scope_detail, scope_proof_path, reason) = if active_bindings
        .contains(&binding_key)
    {
        (
            ACTIVE_SCOPE_STATE.to_string(),
            ACTIVE_SCOPE_DETAIL.to_string(),
            ACTIVE_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is part of the governed all-domain active benchmark matrix",
                binding.stage_id, binding.tool_id
            ),
        )
    } else if complete_bindings.contains(&binding_key) {
        (
            COMPLETE_SCOPE_STATE.to_string(),
            COMPLETE_SCOPE_DETAIL.to_string(),
            COMPLETE_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is already represented in the governed VCF expected-result surface even though it is not part of the final job-bearing active matrix",
                binding.stage_id, binding.tool_id
            ),
        )
    } else if catalog_row.support_status == "supported" {
        (
            REMOVED_FROM_SCOPE_STATE.to_string(),
            BENCHMARK_NOT_READY_SCOPE_DETAIL.to_string(),
            BENCHMARK_NOT_READY_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is retained for a supported stage but remains outside active scope because it is not benchmark ready",
                binding.stage_id, binding.tool_id
            ),
        )
    } else {
        (
            REMOVED_FROM_SCOPE_STATE.to_string(),
            LIFECYCLE_NOT_ACTIVE_SCOPE_DETAIL.to_string(),
            LIFECYCLE_NOT_ACTIVE_SCOPE_PROOF_PATH.to_string(),
            format!(
                "binding `{}` / `{}` is retained but remains outside active scope because the stage lifecycle is not active",
                binding.stage_id, binding.tool_id
            ),
        )
    };

    VcfActiveStageToolMatrixRow {
        stage_id: binding.stage_id,
        tool_id: binding.tool_id,
        tool_status: sorted_status_label(&binding.tool_statuses),
        stage_support_status: catalog_row.support_status.clone(),
        corpus_id: stage_matrix_row.corpus_id.clone(),
        asset_profile_id: stage_matrix_row.asset_profile_id.clone(),
        adapter_id: stage_matrix_row.adapter_id.clone(),
        parser_id: stage_matrix_row.parser_id.clone(),
        schema_id: schema_id.to_string(),
        scope_state,
        scope_detail,
        scope_proof_path,
        reason,
    }
}
fn sorted_status_label(statuses: &BTreeSet<String>) -> String {
    statuses.iter().cloned().collect::<Vec<_>>().join(",")
}
