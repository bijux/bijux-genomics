//! Owner: bijux-dna-bench-population
//! Admitted declaration observations supplied by the composition owner.
use anyhow::{bail, Result};
use bijux_dna_bench_model::{AllDomainActiveStageToolMatrixRow, VcfStageMatrixRow};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

/// Population workflows consume declaration observations without acquiring other domains.
///
/// Callers supply only the observation families needed by the selected service. A
/// service still validates its required binding and expected artifacts before effects.
pub struct PopulationBenchmarkInputs {
    supported: Vec<VcfStageMatrixRow>,
    governed: Vec<VcfStageMatrixRow>,
    active_ibd: Vec<AllDomainActiveStageToolMatrixRow>,
}

impl PopulationBenchmarkInputs {
    /// Admit unambiguous supported bindings, governed templates and active IBD observations.
    ///
    /// # Errors
    /// Rejects foreign stages, empty or padded identities and duplicate declarations.
    pub fn new(
        supported: Vec<VcfStageMatrixRow>,
        governed: Vec<VcfStageMatrixRow>,
        active_ibd: Vec<AllDomainActiveStageToolMatrixRow>,
    ) -> Result<Self> {
        admit_matrix_rows(
            &supported,
            &[stage::VCF_PCA, stage::VCF_ADMIXTURE, stage::VCF_POPULATION_STRUCTURE],
            BindingMultiplicity::StageAndTool,
        )?;
        admit_matrix_rows(
            &governed,
            &[stage::VCF_ROH, stage::VCF_IBD, stage::VCF_DEMOGRAPHY],
            BindingMultiplicity::Stage,
        )?;
        let mut tools = BTreeSet::new();
        for row in &active_ibd {
            if [
                &row.corpus_id,
                &row.asset_profile_id,
                &row.adapter_id,
                &row.parser_id,
                &row.schema_id,
                &row.status,
            ]
            .into_iter()
            .any(|value| value.trim().is_empty() || value.trim() != *value)
                || row.domain != "vcf"
                || row.stage_id != stage::VCF_IBD
                || !valid_tool_identity(&row.tool_id)
                || row.tool_id.trim() != row.tool_id
                || !tools.insert(row.tool_id.as_str())
            {
                bail!("population workflow requires unique, explicitly scoped active IBD bindings");
            }
        }
        Ok(Self { supported, governed, active_ibd })
    }

    pub(crate) fn supported_rows(&self) -> &[VcfStageMatrixRow] {
        &self.supported
    }
    pub(crate) fn governed_rows(&self) -> &[VcfStageMatrixRow] {
        &self.governed
    }
    pub(crate) fn active_ibd_rows(&self) -> &[AllDomainActiveStageToolMatrixRow] {
        &self.active_ibd
    }
}

fn admit_matrix_rows(
    rows: &[VcfStageMatrixRow],
    stages: &[&str],
    multiplicity: BindingMultiplicity,
) -> Result<()> {
    let mut identities = BTreeSet::new();
    for row in rows {
        if !stages.contains(&row.stage_id.as_str())
            || !valid_tool_identity(&row.tool_id)
            || row.tool_id.trim() != row.tool_id
        {
            bail!("population workflow received an invalid stage/tool identity");
        }
        if [&row.corpus_id, &row.asset_profile_id, &row.adapter_id, &row.parser_id]
            .into_iter()
            .any(|value| value.trim().is_empty() || value.trim() != *value)
            || row.expected_outputs.is_empty()
            || row
                .expected_outputs
                .iter()
                .any(|value| value.trim().is_empty() || value.trim() != *value)
            || row.expected_outputs.iter().collect::<BTreeSet<_>>().len()
                != row.expected_outputs.len()
        {
            bail!("population workflow received malformed binding metadata or artifact identities");
        }
        let key = (
            row.stage_id.as_str(),
            match multiplicity {
                BindingMultiplicity::Stage => "",
                BindingMultiplicity::StageAndTool => row.tool_id.as_str(),
            },
        );
        if !identities.insert(key) {
            bail!("population workflow received duplicate declaration observations");
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum BindingMultiplicity {
    Stage,
    StageAndTool,
}

fn valid_tool_identity(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
