//! Admitted stage templates and explicit tool bindings supplied by composition.
use anyhow::{anyhow, ensure, Result};
use bijux_dna_bench_model::VcfStageMatrixRow;
use bijux_dna_core::id_catalog as stage;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::ToolBindingRegistry;
use std::collections::{BTreeMap, BTreeSet};

/// Selected haplotype declarations, independent of CLI acquisition and readiness aggregation.
pub struct HaplotypeBenchmarkInputs {
    matrix: BTreeMap<String, VcfStageMatrixRow>,
    bindings: ToolBindingRegistry,
}

impl HaplotypeBenchmarkInputs {
    /// Admit scoped, unique stage templates and validate their retained tool binding.
    ///
    /// # Errors
    /// Rejects foreign stages, malformed identities, duplicate outputs or absent tool bindings.
    pub fn new(matrix: Vec<VcfStageMatrixRow>, bindings: ToolBindingRegistry) -> Result<Self> {
        let mut admitted = BTreeMap::new();
        for row in matrix {
            ensure!(
                [
                    stage::VCF_PREPARE_REFERENCE_PANEL,
                    stage::VCF_PHASING,
                    stage::VCF_IMPUTE,
                    stage::VCF_IMPUTATION_METRICS
                ]
                .contains(&row.stage_id.as_str()),
                "haplotype inputs contain a foreign stage"
            );
            ensure!(
                [
                    &row.tool_id,
                    &row.corpus_id,
                    &row.asset_profile_id,
                    &row.adapter_id,
                    &row.parser_id
                ]
                .into_iter()
                .all(|value| !value.is_empty() && value.trim() == *value),
                "haplotype declaration identities must be nonempty and unpadded"
            );
            ensure!(valid_tool(&row.tool_id), "haplotype tool identity cannot select a path");
            ensure!(
                !row.expected_outputs.is_empty()
                    && row
                        .expected_outputs
                        .iter()
                        .all(|value| !value.is_empty() && value.trim() == *value)
                    && row.expected_outputs.iter().collect::<BTreeSet<_>>().len()
                        == row.expected_outputs.len(),
                "haplotype output declarations must be nonempty and unique"
            );
            ensure!(
                bindings
                    .binding(&StageId::new(&row.stage_id), &ToolId::new(&row.tool_id))
                    .is_some(),
                "haplotype stage template has no admitted tool binding"
            );
            ensure!(
                admitted.insert(row.stage_id.clone(), row).is_none(),
                "duplicate haplotype stage template"
            );
        }
        Ok(Self { matrix: admitted, bindings })
    }

    pub(crate) fn matrix_rows(&self) -> impl Iterator<Item = &VcfStageMatrixRow> {
        self.matrix.values()
    }

    pub(crate) fn bindings(&self) -> &ToolBindingRegistry {
        &self.bindings
    }

    pub(crate) fn admit_requested_tool(&self, stage_id: &str, tool_id: &str) -> Result<()> {
        ensure!(valid_tool(tool_id), "haplotype tool identity cannot select a path");
        self.matrix.get(stage_id).ok_or_else(|| anyhow!("haplotype stage template is absent"))?;
        ensure!(
            self.bindings.binding(&StageId::new(stage_id), &ToolId::new(tool_id)).is_some(),
            "VCF panel workflow smoke has no canonical binding for `{stage_id}` / `{tool_id}`"
        );
        Ok(())
    }
}

fn valid_tool(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
