use serde::Serialize;

use crate::taxonomy::VcfDomainStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct StageMetricsContract {
    pub stage: VcfDomainStage,
    pub metrics_schema_id: &'static str,
    pub required_metrics: &'static [&'static str],
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn stage_metrics_contract(stage: VcfDomainStage) -> StageMetricsContract {
    const CALL_METRICS: &[&str] = &["variants_called", "snps", "indels"];
    const CALL_DIPLOID_METRICS: &[&str] = &["diploid_called_variants", "het_rate"];
    const CALL_GL_METRICS: &[&str] = &["gl_sites_count", "mean_gl_depth", "gl_parameter_profile"];
    const CALL_PSEUDOHAPLOID_METRICS: &[&str] =
        &["pseudo_called_sites", "pseudo_missing_rate", "pseudo_sampling_seed"];
    const DAMAGE_FILTER_METRICS: &[&str] =
        &["ct_ga_masked_sites", "pmd_filtered_reads", "damage_strategy_applied"];
    const PHASING_METRICS: &[&str] = &["switch_error_proxy", "phase_block_n50"];
    const IMPUTATION_METRICS: &[&str] = &[
        "imputed_variant_count",
        "imputation_info_mean",
        "rsq_mean",
        "missingness_pre",
        "missingness_post",
        "shared_variants_count",
        "per_chr_overlap",
        "readiness_for_ibd_roh",
    ];
    const POP_STRUCTURE_METRICS: &[&str] = &["pc1_variance", "pc2_variance", "cluster_count"];
    const IBD_METRICS: &[&str] =
        &["ibd_segment_count", "ibd_total_length_cM", "pairwise_ibd_sharing_matrix"];
    const ROH_METRICS: &[&str] = &["roh_count", "roh_total_mb", "roh_mean_length_mb"];
    const DEMOGRAPHY_METRICS: &[&str] = &["ne_recent", "ne_time_series", "ne_confidence_interval"];
    const QC_METRICS: &[&str] = &[
        "sample_count",
        "missingness_post",
        "imputation_info_mean",
        "rsq_mean",
        "strand_flip_like_sites",
        "allele_frequency_shift_abs_mean",
        "residual_ct_ga_asymmetry",
        "lowcov_uncertainty_mean",
        "concordance",
        "readiness_for_ibd_roh",
    ];
    const STATS_METRICS: &[&str] = &[
        "variants_total",
        "sample_count",
        "missingness_post",
        "heterozygosity_ratio",
        "ti_tv",
        "annotation_coverage",
    ];

    match stage {
        VcfDomainStage::Call => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call.v1",
            required_metrics: CALL_METRICS,
        },
        VcfDomainStage::CallDiploid => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call_diploid.v1",
            required_metrics: CALL_DIPLOID_METRICS,
        },
        VcfDomainStage::CallGl => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call_gl.v1",
            required_metrics: CALL_GL_METRICS,
        },
        VcfDomainStage::CallPseudohaploid => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.call_pseudohaploid.v1",
            required_metrics: CALL_PSEUDOHAPLOID_METRICS,
        },
        VcfDomainStage::DamageFilter => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.damage_filter.v1",
            required_metrics: DAMAGE_FILTER_METRICS,
        },
        VcfDomainStage::Phasing => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.phasing.v1",
            required_metrics: PHASING_METRICS,
        },
        VcfDomainStage::Imputation | VcfDomainStage::Impute => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.imputation.v1",
            required_metrics: IMPUTATION_METRICS,
        },
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Admixture => {
            StageMetricsContract {
                stage,
                metrics_schema_id: "bijux.vcf.population_structure.v1",
                required_metrics: POP_STRUCTURE_METRICS,
            }
        }
        VcfDomainStage::Ibd => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.ibd.v1",
            required_metrics: IBD_METRICS,
        },
        VcfDomainStage::Roh => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.roh.v1",
            required_metrics: ROH_METRICS,
        },
        VcfDomainStage::Demography => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.demography.v1",
            required_metrics: DEMOGRAPHY_METRICS,
        },
        VcfDomainStage::Qc => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.qc.v1",
            required_metrics: QC_METRICS,
        },
        VcfDomainStage::Postprocess
        | VcfDomainStage::PrepareReferencePanel
        | VcfDomainStage::Filter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Stats => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.stats.v1",
            required_metrics: STATS_METRICS,
        },
    }
}
