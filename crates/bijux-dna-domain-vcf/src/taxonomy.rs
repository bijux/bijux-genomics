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
    Imputation,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VcfStageTaxonomyRecord {
    pub stage: VcfDomainStage,
    pub kind: VcfStageKind,
    pub status: DomainSupportStatus,
    pub coverage_regimes: &'static [CoverageRegime],
}

pub const VCF_STAGE_ORDER_DOWNSTREAM: &[VcfDomainStage] = &[
    VcfDomainStage::PrepareReferencePanel,
    VcfDomainStage::Call,
    VcfDomainStage::CallGl,
    VcfDomainStage::CallDiploid,
    VcfDomainStage::CallPseudohaploid,
    VcfDomainStage::DamageFilter,
    VcfDomainStage::Filter,
    VcfDomainStage::GlPropagation,
    VcfDomainStage::Qc,
    VcfDomainStage::Phasing,
    VcfDomainStage::Imputation,
    VcfDomainStage::Impute,
    VcfDomainStage::Postprocess,
    VcfDomainStage::PopulationStructure,
    VcfDomainStage::Pca,
    VcfDomainStage::Admixture,
    VcfDomainStage::Roh,
    VcfDomainStage::Ibd,
    VcfDomainStage::Demography,
    VcfDomainStage::Stats,
];

pub const VCF_FORBIDDEN_TRANSITIONS: &[(VcfDomainStage, VcfDomainStage)] = &[
    // Downstream outputs cannot feed back into calling.
    (VcfDomainStage::Imputation, VcfDomainStage::Call),
    (VcfDomainStage::Impute, VcfDomainStage::Call),
    (VcfDomainStage::Postprocess, VcfDomainStage::Call),
    // Demography must not occur before IBD.
    (VcfDomainStage::Demography, VcfDomainStage::Ibd),
    // Structure/admixture must not come before variant QC/filtering.
    (VcfDomainStage::PopulationStructure, VcfDomainStage::Filter),
    (VcfDomainStage::Admixture, VcfDomainStage::Filter),
];

pub const VCF_STAGE_TAXONOMY: &[VcfStageTaxonomyRecord] = &[
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Admixture,
        kind: VcfStageKind::PopulationStructure,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Call,
        kind: VcfStageKind::CallRegime,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[CoverageRegime::Diploid],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::CallDiploid,
        kind: VcfStageKind::CallRegime,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[CoverageRegime::Diploid],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::CallGl,
        kind: VcfStageKind::CallRegime,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::CallPseudohaploid,
        kind: VcfStageKind::CallRegime,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[CoverageRegime::Pseudohaploid],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::DamageFilter,
        kind: VcfStageKind::DamageFilter,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[
            CoverageRegime::LowCovGl,
            CoverageRegime::Diploid,
            CoverageRegime::Pseudohaploid,
        ],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Demography,
        kind: VcfStageKind::Demography,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Filter,
        kind: VcfStageKind::Qc,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[
            CoverageRegime::LowCovGl,
            CoverageRegime::Diploid,
            CoverageRegime::Pseudohaploid,
        ],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::GlPropagation,
        kind: VcfStageKind::CallRegime,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Ibd,
        kind: VcfStageKind::Ibd,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Imputation,
        kind: VcfStageKind::Imputation,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Impute,
        kind: VcfStageKind::Imputation,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Pca,
        kind: VcfStageKind::PopulationStructure,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Phasing,
        kind: VcfStageKind::Phasing,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::PopulationStructure,
        kind: VcfStageKind::PopulationStructure,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Postprocess,
        kind: VcfStageKind::Postprocess,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::PrepareReferencePanel,
        kind: VcfStageKind::Postprocess,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::LowCovGl],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Qc,
        kind: VcfStageKind::Qc,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[
            CoverageRegime::LowCovGl,
            CoverageRegime::Diploid,
            CoverageRegime::Pseudohaploid,
        ],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Roh,
        kind: VcfStageKind::Roh,
        status: DomainSupportStatus::Planned,
        coverage_regimes: &[CoverageRegime::Diploid, CoverageRegime::Pseudohaploid],
    },
    VcfStageTaxonomyRecord {
        stage: VcfDomainStage::Stats,
        kind: VcfStageKind::Qc,
        status: DomainSupportStatus::Supported,
        coverage_regimes: &[
            CoverageRegime::LowCovGl,
            CoverageRegime::Diploid,
            CoverageRegime::Pseudohaploid,
        ],
    },
];

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
            Self::Imputation => "vcf.imputation",
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
            Self::Imputation,
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
    pub fn taxonomy(self) -> &'static VcfStageTaxonomyRecord {
        if let Some(record) = VCF_STAGE_TAXONOMY
            .iter()
            .find(|record| record.stage == self)
        {
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

/// # Errors
/// Returns an error when a transition violates downstream order or is explicitly forbidden.
pub fn validate_downstream_transition(from: VcfDomainStage, to: VcfDomainStage) -> Result<()> {
    if VCF_FORBIDDEN_TRANSITIONS.contains(&(from, to)) {
        return Err(anyhow!(
            "forbidden downstream transition: {} -> {}",
            from.as_str(),
            to.as_str()
        ));
    }
    let from_pos = VCF_STAGE_ORDER_DOWNSTREAM
        .iter()
        .position(|stage| *stage == from)
        .ok_or_else(|| anyhow!("stage not in downstream order: {}", from.as_str()))?;
    let to_pos = VCF_STAGE_ORDER_DOWNSTREAM
        .iter()
        .position(|stage| *stage == to)
        .ok_or_else(|| anyhow!("stage not in downstream order: {}", to.as_str()))?;
    if to_pos < from_pos {
        return Err(anyhow!(
            "downstream ordering violation: {} cannot precede {}",
            to.as_str(),
            from.as_str()
        ));
    }
    Ok(())
}
