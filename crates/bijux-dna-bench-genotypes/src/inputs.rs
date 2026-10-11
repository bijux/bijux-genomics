//! Explicit, scoped stage declarations admitted before genotype benchmark effects.
use anyhow::{ensure, Result};
use bijux_dna_bench_model::VcfStageMatrixRow;
use bijux_dna_domain_vcf::VcfDomainStage;
use bijux_dna_stages_vcf::stage_specs::{vcf_domain_stage_adapter_id, vcf_domain_stage_parser_id};
use std::collections::{BTreeMap, BTreeSet};

/// Governed calling and likelihood propagation declarations, independent of catalog acquisition.
pub struct GenotypeBenchmarkInputs {
    rows: BTreeMap<String, VcfStageMatrixRow>,
}
impl GenotypeBenchmarkInputs {
    /// Admit unique declarations with canonical stage, adapter, parser and artifact identities.
    ///
    /// # Errors
    /// Rejects foreign stages, duplicate declarations and malformed or incompatible identities.
    pub fn new(rows: Vec<VcfStageMatrixRow>) -> Result<Self> {
        let mut admitted = BTreeMap::new();
        for row in rows {
            let stage = match row.stage_id.as_str() {
                "vcf.call" => VcfDomainStage::Call,
                "vcf.call_gl" => VcfDomainStage::CallGl,
                "vcf.call_diploid" => VcfDomainStage::CallDiploid,
                "vcf.call_pseudohaploid" => VcfDomainStage::CallPseudohaploid,
                "vcf.gl_propagation" => VcfDomainStage::GlPropagation,
                _ => anyhow::bail!("genotype benchmark received a foreign stage"),
            };
            ensure!(
                Some(row.adapter_id.as_str()) == vcf_domain_stage_adapter_id(stage)
                    && Some(row.parser_id.as_str()) == vcf_domain_stage_parser_id(stage),
                "genotype declaration adapter/parser identity disagrees with stage contract"
            );
            ensure!(
                [&row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id]
                    .into_iter()
                    .all(|value| !value.is_empty() && value.trim() == *value),
                "genotype identities must be nonempty and unpadded"
            );
            ensure!(
                row.tool_id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
                "genotype tool identity cannot select an output path"
            );
            ensure!(
                !row.expected_outputs.is_empty()
                    && row
                        .expected_outputs
                        .iter()
                        .all(|value| !value.is_empty() && value.trim() == *value)
                    && row.expected_outputs.iter().collect::<BTreeSet<_>>().len()
                        == row.expected_outputs.len(),
                "genotype output declarations must be nonempty and unique"
            );
            ensure!(
                admitted.insert(row.stage_id.clone(), row).is_none(),
                "duplicate genotype stage declaration"
            );
        }
        Ok(Self { rows: admitted })
    }
    pub(crate) fn rows(&self) -> impl Iterator<Item = &VcfStageMatrixRow> {
        self.rows.values()
    }
}
