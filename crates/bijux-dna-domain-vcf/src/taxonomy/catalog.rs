use anyhow::{anyhow, Result};

use super::{CoverageRegime, DomainSupportStatus, VcfDomainStage, VcfStageKind};

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
    VcfDomainStage::ImputationMetrics,
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
    (VcfDomainStage::ImputationMetrics, VcfDomainStage::Call),
    (VcfDomainStage::Impute, VcfDomainStage::Call),
    (VcfDomainStage::Postprocess, VcfDomainStage::Call),
    (VcfDomainStage::Demography, VcfDomainStage::Ibd),
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
        stage: VcfDomainStage::ImputationMetrics,
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

/// # Errors
/// Returns an error when a transition violates downstream order or is explicitly forbidden.
pub fn validate_downstream_transition(from: VcfDomainStage, to: VcfDomainStage) -> Result<()> {
    if from == to {
        return Err(anyhow!("self transition is not downstream: {}", from.as_str()));
    }
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
