use serde::Serialize;

use crate::taxonomy::VcfDomainStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfComparableMetricDirection {
    ExactMatchPreferred,
    HigherIsBetter,
    LowerIsBetter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfComparableMetricSpec {
    pub stage: VcfDomainStage,
    pub metric_id: &'static str,
    pub metric_name: &'static str,
    pub unit: &'static str,
    pub direction: VcfComparableMetricDirection,
    pub required: bool,
}

const CALL_GL_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::CallGl,
        metric_id: "sites_with_likelihoods",
        metric_name: "sites with likelihoods",
        unit: "sites",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::CallGl,
        metric_id: "missing_likelihoods",
        metric_name: "missing likelihoods",
        unit: "sites",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
];

const CALL_PSEUDOHAPLOID_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::CallPseudohaploid,
        metric_id: "called_sites",
        metric_name: "called sites",
        unit: "sites",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::CallPseudohaploid,
        metric_id: "missing_sites",
        metric_name: "missing sites",
        unit: "sites",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
];

const DAMAGE_FILTER_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::DamageFilter,
        metric_id: "removed_variants",
        metric_name: "removed variants",
        unit: "variants",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::DamageFilter,
        metric_id: "retained_variants",
        metric_name: "retained variants",
        unit: "variants",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::DamageFilter,
        metric_id: "terminal_damage_filtered_variants",
        metric_name: "terminal damage filtered variants",
        unit: "variants",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
];

const GL_PROPAGATION_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::GlPropagation,
        metric_id: "site_count_before",
        metric_name: "site count before propagation",
        unit: "sites",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::GlPropagation,
        metric_id: "site_count_after",
        metric_name: "site count after propagation",
        unit: "sites",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::GlPropagation,
        metric_id: "sample_count",
        metric_name: "sample count",
        unit: "samples",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
];

const QC_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Qc,
        metric_id: "missingness_post",
        metric_name: "post-qc missingness",
        unit: "fraction",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Qc,
        metric_id: "imputation_info_mean",
        metric_name: "mean imputation info",
        unit: "score",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Qc,
        metric_id: "rsq_mean",
        metric_name: "mean r-squared",
        unit: "score",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Qc,
        metric_id: "concordance",
        metric_name: "concordance",
        unit: "fraction",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
];

const PCA_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Pca,
        metric_id: "sample_count",
        metric_name: "sample count",
        unit: "samples",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Pca,
        metric_id: "variant_count",
        metric_name: "variant count",
        unit: "variants",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
];

const POPULATION_STRUCTURE_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::PopulationStructure,
        metric_id: "sample_count",
        metric_name: "sample count",
        unit: "samples",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::PopulationStructure,
        metric_id: "pair_count",
        metric_name: "pair count",
        unit: "pairs",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
];

const ADMIXTURE_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Admixture,
        metric_id: "selected_k",
        metric_name: "selected cluster count",
        unit: "clusters",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Admixture,
        metric_id: "sample_count",
        metric_name: "sample count",
        unit: "samples",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Admixture,
        metric_id: "population_count",
        metric_name: "population count",
        unit: "populations",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
];

const PHASING_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Phasing,
        metric_id: "switch_error_proxy",
        metric_name: "switch error proxy",
        unit: "fraction",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Phasing,
        metric_id: "phase_block_n50",
        metric_name: "phase block n50",
        unit: "bases",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
];

const IMPUTE_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Impute,
        metric_id: "missing_before",
        metric_name: "missing genotypes before imputation",
        unit: "genotypes",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Impute,
        metric_id: "missing_after",
        metric_name: "missing genotypes after imputation",
        unit: "genotypes",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Impute,
        metric_id: "imputed_genotypes",
        metric_name: "imputed genotypes",
        unit: "genotypes",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Impute,
        metric_id: "low_confidence_count",
        metric_name: "low-confidence sites",
        unit: "sites",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Impute,
        metric_id: "masked_truth_match_count",
        metric_name: "masked-truth matches",
        unit: "sites",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Impute,
        metric_id: "unresolved_count",
        metric_name: "unresolved sites",
        unit: "sites",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
];

const IMPUTATION_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Imputation,
        metric_id: "mean_info_score",
        metric_name: "mean info score",
        unit: "score",
        direction: VcfComparableMetricDirection::HigherIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Imputation,
        metric_id: "low_confidence_sites",
        metric_name: "low-confidence sites",
        unit: "sites",
        direction: VcfComparableMetricDirection::LowerIsBetter,
        required: true,
    },
    VcfComparableMetricSpec {
        stage: VcfDomainStage::Imputation,
        metric_id: "masked_truth_sites",
        metric_name: "masked-truth sites",
        unit: "sites",
        direction: VcfComparableMetricDirection::ExactMatchPreferred,
        required: true,
    },
];

const IBD_COMPARABLE_METRICS: &[VcfComparableMetricSpec] = &[VcfComparableMetricSpec {
    stage: VcfDomainStage::Ibd,
    metric_id: "pair_count",
    metric_name: "pair count",
    unit: "pairs",
    direction: VcfComparableMetricDirection::ExactMatchPreferred,
    required: true,
}];

#[must_use]
pub fn stage_comparable_metric_specs(stage: VcfDomainStage) -> &'static [VcfComparableMetricSpec] {
    match stage {
        VcfDomainStage::CallGl => CALL_GL_COMPARABLE_METRICS,
        VcfDomainStage::CallPseudohaploid => CALL_PSEUDOHAPLOID_COMPARABLE_METRICS,
        VcfDomainStage::DamageFilter => DAMAGE_FILTER_COMPARABLE_METRICS,
        VcfDomainStage::GlPropagation => GL_PROPAGATION_COMPARABLE_METRICS,
        VcfDomainStage::Qc => QC_COMPARABLE_METRICS,
        VcfDomainStage::Pca => PCA_COMPARABLE_METRICS,
        VcfDomainStage::PopulationStructure => POPULATION_STRUCTURE_COMPARABLE_METRICS,
        VcfDomainStage::Admixture => ADMIXTURE_COMPARABLE_METRICS,
        VcfDomainStage::Phasing => PHASING_COMPARABLE_METRICS,
        VcfDomainStage::Impute => IMPUTE_COMPARABLE_METRICS,
        VcfDomainStage::Imputation => IMPUTATION_COMPARABLE_METRICS,
        VcfDomainStage::Ibd => IBD_COMPARABLE_METRICS,
        _ => &[],
    }
}

#[must_use]
pub fn comparable_metric_stage_ids() -> Vec<VcfDomainStage> {
    VcfDomainStage::all()
        .iter()
        .copied()
        .filter(|stage| !stage_comparable_metric_specs(*stage).is_empty())
        .collect()
}
