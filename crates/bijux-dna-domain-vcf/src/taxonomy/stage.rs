use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfStageKind {
    CallRegime,
    DamageFilter,
    Phasing,
    Imputation,
    PopulationStructure,
    Ibd,
    Roh,
    Demography,
    Qc,
    Postprocess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageRegime {
    LowCovGl,
    Diploid,
    Pseudohaploid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainSupportStatus {
    Supported,
    Planned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfDomainStage {
    Admixture,
    Call,
    CallDiploid,
    CallGl,
    CallPseudohaploid,
    DamageFilter,
    Demography,
    Filter,
    GlPropagation,
    Ibd,
    ImputationMetrics,
    Impute,
    Pca,
    Phasing,
    PopulationStructure,
    Postprocess,
    PrepareReferencePanel,
    Qc,
    Roh,
    Stats,
}

impl VcfDomainStage {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Admixture => "vcf.admixture",
            Self::Call => "vcf.call",
            Self::CallDiploid => "vcf.call_diploid",
            Self::CallGl => "vcf.call_gl",
            Self::CallPseudohaploid => "vcf.call_pseudohaploid",
            Self::DamageFilter => "vcf.damage_filter",
            Self::Demography => "vcf.demography",
            Self::Filter => "vcf.filter",
            Self::GlPropagation => "vcf.gl_propagation",
            Self::Ibd => "vcf.ibd",
            Self::ImputationMetrics => "vcf.imputation_metrics",
            Self::Impute => "vcf.impute",
            Self::Pca => "vcf.pca",
            Self::Phasing => "vcf.phasing",
            Self::PopulationStructure => "vcf.population_structure",
            Self::Postprocess => "vcf.postprocess",
            Self::PrepareReferencePanel => "vcf.prepare_reference_panel",
            Self::Qc => "vcf.qc",
            Self::Roh => "vcf.roh",
            Self::Stats => "vcf.stats",
        }
    }

    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Admixture,
            Self::Call,
            Self::CallDiploid,
            Self::CallGl,
            Self::CallPseudohaploid,
            Self::DamageFilter,
            Self::Demography,
            Self::Filter,
            Self::GlPropagation,
            Self::Ibd,
            Self::ImputationMetrics,
            Self::Impute,
            Self::Pca,
            Self::Phasing,
            Self::PopulationStructure,
            Self::Postprocess,
            Self::PrepareReferencePanel,
            Self::Qc,
            Self::Roh,
            Self::Stats,
        ]
    }

    #[must_use]
    /// # Panics
    /// Panics if the static taxonomy table does not contain an entry for this stage.
    pub fn taxonomy(self) -> &'static super::VcfStageTaxonomyRecord {
        if let Some(record) = super::VCF_STAGE_TAXONOMY.iter().find(|record| record.stage == self) {
            return record;
        }
        unreachable!("taxonomy entry must exist for every stage")
    }
}

impl TryFrom<&str> for VcfDomainStage {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|stage| stage.as_str() == value)
            .ok_or_else(|| anyhow!("unknown VCF domain stage: {value}"))
    }
}
