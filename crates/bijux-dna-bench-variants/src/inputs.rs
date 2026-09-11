//! Scoped declarations admitted before variant benchmark effects.
use anyhow::{ensure, Result};
use bijux_dna_bench_model::VcfStageMatrixRow;
use bijux_dna_core::id_catalog as stage;
use std::collections::{BTreeMap, BTreeSet};

/// Independent governed processing templates and explicitly supported QC bindings.
pub struct VariantBenchmarkInputs {
    governed: BTreeMap<String, VcfStageMatrixRow>,
    qc: BTreeMap<String, VcfStageMatrixRow>,
}

impl VariantBenchmarkInputs {
    /// Admit declared stage/tool identities and unique artifact roles before execution.
    ///
    /// # Errors
    /// Rejects foreign stages, duplicate templates, malformed identities and output declarations.
    pub fn new(governed: Vec<VcfStageMatrixRow>, qc: Vec<VcfStageMatrixRow>) -> Result<Self> {
        let mut templates = BTreeMap::new();
        for row in governed {
            validate_row(&row)?;
            ensure!(
                [stage::VCF_FILTER, stage::VCF_DAMAGE_FILTER, stage::VCF_POSTPROCESS]
                    .contains(&row.stage_id.as_str()),
                "variant processing received a foreign governed stage"
            );
            ensure!(
                templates.insert(row.stage_id.clone(), row).is_none(),
                "duplicate governed processing template"
            );
        }
        let mut supported = BTreeMap::new();
        for row in qc {
            validate_row(&row)?;
            ensure!(row.stage_id == stage::VCF_QC, "QC declaration has a foreign stage");
            ensure!(
                supported.insert(row.tool_id.clone(), row).is_none(),
                "duplicate QC tool binding"
            );
        }
        Ok(Self { governed: templates, qc: supported })
    }

    pub(crate) fn governed(&self) -> impl Iterator<Item = &VcfStageMatrixRow> {
        self.governed.values()
    }

    pub(crate) fn qc(&self) -> impl Iterator<Item = &VcfStageMatrixRow> {
        self.qc.values()
    }
}

fn validate_row(row: &VcfStageMatrixRow) -> Result<()> {
    use bijux_dna_domain_vcf::VcfDomainStage;
    use bijux_dna_stages_vcf::stage_specs::{
        vcf_domain_stage_adapter_id, vcf_domain_stage_parser_id,
    };
    let domain_stage = match row.stage_id.as_str() {
        stage::VCF_FILTER => VcfDomainStage::Filter,
        stage::VCF_DAMAGE_FILTER => VcfDomainStage::DamageFilter,
        stage::VCF_POSTPROCESS => VcfDomainStage::Postprocess,
        stage::VCF_QC => VcfDomainStage::Qc,
        _ => anyhow::bail!("variant declaration has a foreign stage"),
    };
    ensure!(
        Some(row.adapter_id.as_str()) == vcf_domain_stage_adapter_id(domain_stage)
            && Some(row.parser_id.as_str()) == vcf_domain_stage_parser_id(domain_stage),
        "variant declaration adapter/parser identity disagrees with its stage contract"
    );
    ensure!(
        [
            &row.stage_id,
            &row.tool_id,
            &row.corpus_id,
            &row.asset_profile_id,
            &row.adapter_id,
            &row.parser_id
        ]
        .into_iter()
        .all(|value| !value.is_empty() && value.trim() == *value),
        "variant declaration identities must be nonempty and unpadded"
    );
    ensure!(
        row.tool_id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')),
        "variant tool identity cannot select an output path"
    );
    ensure!(
        !row.expected_outputs.is_empty()
            && row.expected_outputs.iter().all(|value| !value.is_empty() && value.trim() == *value)
            && row.expected_outputs.iter().collect::<BTreeSet<_>>().len()
                == row.expected_outputs.len(),
        "variant output declarations must be nonempty and unique"
    );
    Ok(())
}
