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
    const CALL_METRICS: &[&str] = &["variant_count", "snp_count", "indel_count", "sample_count"];
    const CALL_DIPLOID_METRICS: &[&str] = &[
        "ploidy",
        "called_genotypes",
        "heterozygous_count",
        "homozygous_ref_count",
        "homozygous_alt_count",
        "missing_count",
        "sample_count",
    ];
    const CALL_GL_METRICS: &[&str] = &[
        "likelihood_field",
        "sites_with_likelihoods",
        "samples_with_likelihoods",
        "missing_likelihoods",
        "sample_count",
    ];
    const CALL_PSEUDOHAPLOID_METRICS: &[&str] = &[
        "target_sites",
        "covered_sites",
        "called_sites",
        "missing_sites",
        "sampling_policy",
        "random_seed",
        "sample_count",
    ];
    const DAMAGE_FILTER_METRICS: &[&str] = &[
        "input_variants",
        "removed_variants",
        "retained_variants",
        "low_quality_filtered_variants",
        "damage_ratio_filtered_variants",
        "terminal_damage_filtered_variants",
        "damage_context_rule",
        "terminal_context_count",
        "sample_count",
    ];
    const FILTER_METRICS: &[&str] = &[
        "input_variants",
        "pass_variants",
        "failed_variants",
        "filter_ids",
        "depth_threshold",
        "quality_threshold",
        "missingness_threshold",
        "sample_count",
    ];
    const GL_PROPAGATION_METRICS: &[&str] = &[
        "input_likelihood_fields",
        "output_likelihood_fields",
        "lost_fields",
        "site_count_before",
        "site_count_after",
        "sample_count",
    ];
    const PHASING_METRICS: &[&str] = &["switch_error_proxy", "phase_block_n50"];
    const IMPUTATION_METRICS: &[&str] = &[
        "status",
        "mean_info_score",
        "r2_available",
        "low_confidence_sites",
        "masked_truth_sites",
        "missing_quality_fields",
    ];
    const IMPUTE_METRICS: &[&str] = &[
        "variant_count",
        "missing_before",
        "missing_after",
        "imputed_genotypes",
        "low_confidence_count",
        "masked_truth_site_count",
        "masked_truth_match_count",
        "unresolved_count",
        "not_imputable_reasons",
        "sample_count",
        "sample_ids",
    ];
    const POPULATION_STRUCTURE_METRICS: &[&str] = &[
        "sample_count",
        "pair_count",
        "within_population_pair_count",
        "cross_population_pair_count",
    ];
    const PCA_METRICS: &[&str] =
        &["sample_count", "variant_count", "excluded_samples", "unexpected_samples", "eigenvalues"];
    const ADMIXTURE_METRICS: &[&str] =
        &["selected_k", "sample_count", "population_count", "status"];
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
    const POSTPROCESS_METRICS: &[&str] = &[
        "readable_vcf",
        "tabix_present",
        "contigs_consistent_with_species_context",
        "left_align_applied",
        "multiallelic_records_split",
        "indels_normalized",
        "variant_ids_normalized",
        "invalid_records_removed",
        "filter_standardized_to_pass",
    ];
    const PREPARE_REFERENCE_PANEL_METRICS: &[&str] = &[
        "input_variants",
        "output_variants",
        "sample_count",
        "sample_ids",
        "sample_consistent",
        "duplicate_sites_removed",
        "normalization_status",
        "parseable",
    ];
    const STATS_METRICS: &[&str] = &[
        "variant_count",
        "snp_count",
        "indel_count",
        "transition_count",
        "transversion_count",
        "ti_tv",
        "sample_count",
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
        VcfDomainStage::Imputation => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.imputation.v1",
            required_metrics: IMPUTATION_METRICS,
        },
        VcfDomainStage::Impute => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.impute.v1",
            required_metrics: IMPUTE_METRICS,
        },
        VcfDomainStage::PopulationStructure => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.population_structure.v1",
            required_metrics: POPULATION_STRUCTURE_METRICS,
        },
        VcfDomainStage::Pca => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.pca.v1",
            required_metrics: PCA_METRICS,
        },
        VcfDomainStage::Admixture => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.admixture.v1",
            required_metrics: ADMIXTURE_METRICS,
        },
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
        VcfDomainStage::Postprocess => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.postprocess.v1",
            required_metrics: POSTPROCESS_METRICS,
        },
        VcfDomainStage::PrepareReferencePanel => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.prepare_reference_panel.v1",
            required_metrics: PREPARE_REFERENCE_PANEL_METRICS,
        },
        VcfDomainStage::Filter => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.filter.v1",
            required_metrics: FILTER_METRICS,
        },
        VcfDomainStage::GlPropagation => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.gl_propagation.v1",
            required_metrics: GL_PROPAGATION_METRICS,
        },
        VcfDomainStage::Stats => StageMetricsContract {
            stage,
            metrics_schema_id: "bijux.vcf.stats.v1",
            required_metrics: STATS_METRICS,
        },
    }
}
