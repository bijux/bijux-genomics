use serde::Serialize;

use crate::taxonomy::{CoverageRegime, VcfDomainStage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfValidationContract {
    pub schema_version: &'static str,
    pub enforced_by: &'static [&'static str],
    pub rejects: &'static [&'static str],
    pub configurable_checks: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfArtifactClass {
    RawVcf,
    RawBcf,
    NormalizedVcf,
    NormalizedBcf,
    FilteredVcf,
    FilteredBcf,
    VariantIndex,
    StatsReport,
    QcReport,
    AnnotationReport,
    ImputationReport,
    PanelEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfArtifactClassContract {
    pub stage: VcfDomainStage,
    pub artifact_classes: &'static [VcfArtifactClass],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfReferenceContextContract {
    pub schema_version: &'static str,
    pub required_context: &'static [&'static str],
    pub resolved_fields: &'static [&'static str],
    pub refusal_reasons: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfFilterEvidenceContract {
    pub schema_version: &'static str,
    pub preserved_fields: &'static [&'static str],
    pub supported_filter_scopes: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfNormalizationContract {
    pub schema_version: &'static str,
    pub declared_behaviors: &'static [&'static str],
    pub retained_views: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfNormalizationPolicyRow {
    pub policy_id: &'static str,
    pub left_normalize: bool,
    pub split_multiallelic: bool,
    pub duplicate_handling: &'static str,
    pub reference_check: &'static str,
    pub raw_fixture_expectation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfNormalizationPolicyMatrixContract {
    pub schema_version: &'static str,
    pub policy_rows: &'static [VcfNormalizationPolicyRow],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfCohortValidationContract {
    pub schema_version: &'static str,
    pub checked_before_analysis: &'static [&'static str],
    pub refusal_reasons: &'static [&'static str],
    pub cohort_analysis_stages: &'static [VcfDomainStage],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfLikelihoodWorkflowContract {
    pub stage: VcfDomainStage,
    pub coverage_regime: CoverageRegime,
    pub accepted_tools: &'static [&'static str],
    pub low_coverage_assumptions: &'static [&'static str],
    pub output_caveats: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfCallingModeContract {
    pub stage: VcfDomainStage,
    pub coverage_regime: CoverageRegime,
    pub evidence_focus: &'static str,
    pub assumptions: &'static [&'static str],
    pub refusal_rules: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfStatsReportContract {
    pub schema_version: &'static str,
    pub stable_metric_ids: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfPanelBoundaryContract {
    pub stage: VcfDomainStage,
    pub required_context: &'static [&'static str],
    pub execution_mode: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfPhasingImputationBoundaryContract {
    pub stage: VcfDomainStage,
    pub accepted_tools: &'static [&'static str],
    pub required_context: &'static [&'static str],
    pub required_outputs: &'static [&'static str],
    pub refusal_reasons: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfDamageFilterContract {
    pub schema_version: &'static str,
    pub declared_actions: &'static [&'static str],
    pub execution_modes: &'static [&'static str],
    pub required_provenance: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfPopulationGuardrailContract {
    pub stage: VcfDomainStage,
    pub required_inputs: &'static [&'static str],
    pub report_caveats: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfCohortAnalysisBoundaryContract {
    pub stage: VcfDomainStage,
    pub minimum_cohort_requirement: &'static str,
    pub marker_density_requirement: &'static str,
    pub missingness_requirement: &'static str,
    pub method_assumptions: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfReportCoverageContract {
    pub schema_version: &'static str,
    pub report_sections: &'static [&'static str],
    pub per_sample_sections: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfProductionCorpusCase {
    pub case_id: &'static str,
    pub expectation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfProductionCorpusContract {
    pub schema_version: &'static str,
    pub covered_cases: &'static [VcfProductionCorpusCase],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct VcfScientificDriftContract {
    pub schema_version: &'static str,
    pub tracked_change_surfaces: &'static [&'static str],
    pub downstream_risks: &'static [&'static str],
    pub required_report_sections: &'static [&'static str],
}

pub const VCF_VALIDATION_CONTRACT: VcfValidationContract = VcfValidationContract {
    schema_version: "bijux.vcf.validation_contract.v1",
    enforced_by: &["vcf.preflight", "vcf.qc", "vcf.stats"],
    rejects: &[
        "malformed_headers",
        "missing_contig_headers",
        "duplicate_sample_ids",
        "bad_info_or_format_definitions",
        "unsorted_records",
        "reference_context_mismatch_when_strict",
    ],
    configurable_checks: &[
        "allow_contig_aliasing",
        "min_overlap_threshold",
        "allowed_missing_fields",
        "require_sex_metadata_for_sex_chr",
    ],
};

#[must_use]
pub fn stage_artifact_class_contract(stage: VcfDomainStage) -> VcfArtifactClassContract {
    match stage {
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => VcfArtifactClassContract {
            stage,
            artifact_classes: &[VcfArtifactClass::RawVcf, VcfArtifactClass::VariantIndex],
        },
        VcfDomainStage::Filter | VcfDomainStage::DamageFilter => VcfArtifactClassContract {
            stage,
            artifact_classes: &[VcfArtifactClass::FilteredVcf, VcfArtifactClass::VariantIndex],
        },
        VcfDomainStage::GlPropagation | VcfDomainStage::Postprocess => VcfArtifactClassContract {
            stage,
            artifact_classes: &[
                VcfArtifactClass::NormalizedVcf,
                VcfArtifactClass::NormalizedBcf,
                VcfArtifactClass::VariantIndex,
                VcfArtifactClass::AnnotationReport,
            ],
        },
        VcfDomainStage::Qc => VcfArtifactClassContract {
            stage,
            artifact_classes: &[VcfArtifactClass::QcReport, VcfArtifactClass::StatsReport],
        },
        VcfDomainStage::Stats => {
            VcfArtifactClassContract { stage, artifact_classes: &[VcfArtifactClass::StatsReport] }
        }
        VcfDomainStage::PrepareReferencePanel => VcfArtifactClassContract {
            stage,
            artifact_classes: &[VcfArtifactClass::PanelEvidence, VcfArtifactClass::VariantIndex],
        },
        VcfDomainStage::Phasing | VcfDomainStage::Imputation | VcfDomainStage::Impute => {
            VcfArtifactClassContract {
                stage,
                artifact_classes: &[
                    VcfArtifactClass::ImputationReport,
                    VcfArtifactClass::PanelEvidence,
                ],
            }
        }
        _ => VcfArtifactClassContract { stage, artifact_classes: &[VcfArtifactClass::StatsReport] },
    }
}

pub const VCF_REFERENCE_CONTEXT_CONTRACT: VcfReferenceContextContract =
    VcfReferenceContextContract {
        schema_version: "bijux.vcf.reference_context.v1",
        required_context: &[
            "reference_build",
            "contig_naming_scheme",
            "alias_map",
            "reference_fasta_checksum",
            "index_state",
            "panel_compatibility",
        ],
        resolved_fields: &[
            "bundle_id",
            "bundle_lock_sha256",
            "contig_set_digest",
            "normalization_policy",
            "panel_id",
            "map_id",
        ],
        refusal_reasons: &[
            "reference_bundle_drift",
            "contig_alias_policy_violation",
            "panel_build_mismatch",
            "map_build_mismatch",
        ],
    };

pub const VCF_FILTER_EVIDENCE_CONTRACT: VcfFilterEvidenceContract = VcfFilterEvidenceContract {
    schema_version: "bijux.vcf.filter_evidence.v1",
    preserved_fields: &[
        "filter_expression",
        "filter_scope",
        "missingness_threshold",
        "damage_filter_policy",
        "output_subset",
    ],
    supported_filter_scopes: &["site", "sample", "genotype", "output_subset"],
};

pub const VCF_NORMALIZATION_CONTRACT: VcfNormalizationContract = VcfNormalizationContract {
    schema_version: "bijux.vcf.normalization_contract.v1",
    declared_behaviors: &[
        "left_normalization",
        "multiallelic_decomposition",
        "duplicate_handling",
        "genotype_retention",
        "variant_id_canonicalization",
    ],
    retained_views: &["raw_input", "normalized_output"],
};

const VCF_NORMALIZATION_POLICY_ROWS: &[VcfNormalizationPolicyRow] = &[
    VcfNormalizationPolicyRow {
        policy_id: "diploid_production",
        left_normalize: true,
        split_multiallelic: true,
        duplicate_handling: "refuse_duplicate_records",
        reference_check: "strict_reference_allele_match",
        raw_fixture_expectation: "raw_vs_normalized_diploid_fixture",
    },
    VcfNormalizationPolicyRow {
        policy_id: "lowcov_gl_production",
        left_normalize: true,
        split_multiallelic: true,
        duplicate_handling: "retain_likelihood_safe_records_only",
        reference_check: "strict_reference_allele_match",
        raw_fixture_expectation: "raw_vs_normalized_gl_fixture",
    },
    VcfNormalizationPolicyRow {
        policy_id: "pseudohaploid_production",
        left_normalize: true,
        split_multiallelic: false,
        duplicate_handling: "refuse_duplicate_records",
        reference_check: "strict_reference_allele_match",
        raw_fixture_expectation: "raw_vs_normalized_pseudohaploid_fixture",
    },
];

pub const VCF_NORMALIZATION_POLICY_MATRIX_CONTRACT: VcfNormalizationPolicyMatrixContract =
    VcfNormalizationPolicyMatrixContract {
        schema_version: "bijux.vcf.normalization_policy_matrix.v1",
        policy_rows: VCF_NORMALIZATION_POLICY_ROWS,
    };

pub const VCF_COHORT_VALIDATION_CONTRACT: VcfCohortValidationContract =
    VcfCohortValidationContract {
        schema_version: "bijux.vcf.cohort_validation.v1",
        checked_before_analysis: &[
            "sample_count",
            "duplicate_sample_ids",
            "missingness_profile",
            "sex_and_ploidy_assumptions",
            "relatedness_flags",
            "panel_compatibility",
        ],
        refusal_reasons: &[
            "duplicate_sample_ids",
            "sample_count_below_stage_minimum",
            "missingness_above_readiness_threshold",
            "ploidy_assumptions_not_declared",
            "panel_compatibility_not_confirmed",
        ],
        cohort_analysis_stages: &[
            VcfDomainStage::Pca,
            VcfDomainStage::Admixture,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Roh,
            VcfDomainStage::Ibd,
            VcfDomainStage::Demography,
        ],
    };

#[must_use]
pub fn vcf_calling_mode_contracts() -> &'static [VcfCallingModeContract] {
    &[
        VcfCallingModeContract {
            stage: VcfDomainStage::CallDiploid,
            coverage_regime: CoverageRegime::Diploid,
            evidence_focus: "called genotypes with diploid heterozygosity summaries",
            assumptions: &[
                "diploid_gt_fields",
                "sample_identity_required",
                "reference_bundle_locked",
            ],
            refusal_rules: &["caller_must_be_bcftools_or_gatk", "non_empty_records_required"],
        },
        VcfCallingModeContract {
            stage: VcfDomainStage::CallPseudohaploid,
            coverage_regime: CoverageRegime::Pseudohaploid,
            evidence_focus: "single-allele calls with pseudo-haploid caveats",
            assumptions: &[
                "single_allele_sampling",
                "sample_identity_required",
                "low_information_expected",
            ],
            refusal_rules: &["caller_must_be_angsd_or_bcftools", "non_empty_records_required"],
        },
        VcfCallingModeContract {
            stage: VcfDomainStage::CallGl,
            coverage_regime: CoverageRegime::LowCovGl,
            evidence_focus: "genotype likelihoods and GL-aware downstream compatibility",
            assumptions: &[
                "gl_gp_pl_fields_required",
                "sample_identity_required",
                "low_coverage_likelihood_model",
            ],
            refusal_rules: &["caller_must_be_angsd_or_bcftools", "gl_fields_required"],
        },
        VcfCallingModeContract {
            stage: VcfDomainStage::GlPropagation,
            coverage_regime: CoverageRegime::LowCovGl,
            evidence_focus: "propagated GL/GP/PL semantics through normalized outputs",
            assumptions: &["likelihood_fields_preserved", "normalization_does_not_strip_gl_fields"],
            refusal_rules: &["gl_fields_required", "bgzip_tabix_required"],
        },
    ]
}

pub const VCF_STATS_REPORT_CONTRACT: VcfStatsReportContract = VcfStatsReportContract {
    schema_version: "bijux.vcf.stats_contract.v1",
    stable_metric_ids: &[
        "variants_total",
        "sample_count",
        "missingness_pre",
        "missingness_post",
        "heterozygosity_ratio",
        "ti_tv",
        "filter_impact",
        "annotation_coverage",
    ],
};

pub const VCF_DAMAGE_FILTER_CONTRACT: VcfDamageFilterContract = VcfDamageFilterContract {
    schema_version: "bijux.vcf.damage_filter_contract.v1",
    declared_actions: &["remove_sites", "mask_genotype_calls", "annotate_damage_risk"],
    execution_modes: &["advisory", "enforced"],
    required_provenance: &[
        "damage_mask_mode",
        "pmd_threshold",
        "udg_regime",
        "site_or_genotype_scope",
    ],
};

pub const VCF_REPORT_COVERAGE_CONTRACT: VcfReportCoverageContract = VcfReportCoverageContract {
    schema_version: "bijux.vcf.report_coverage.v1",
    report_sections: &[
        "variant_counts",
        "pass_rates",
        "missingness",
        "filter_impact",
        "annotation_coverage",
        "per_sample_summary",
    ],
    per_sample_sections: &[
        "missingness_by_sample",
        "called_variants_by_sample",
        "filter_outliers_by_sample",
    ],
};

const VCF_PRODUCTION_CORPUS_CASES: &[VcfProductionCorpusCase] = &[
    VcfProductionCorpusCase {
        case_id: "bad_headers",
        expectation: "preflight_refuses_malformed_or_undeclared_headers",
    },
    VcfProductionCorpusCase {
        case_id: "contig_aliases",
        expectation: "alias_handling_is_explicit_and_build_locked",
    },
    VcfProductionCorpusCase {
        case_id: "multi_sample_cohort",
        expectation: "cohort_validation_and_per_sample_reports_are_exercised",
    },
    VcfProductionCorpusCase {
        case_id: "low_coverage_gl",
        expectation: "gl_workflows_declare_likelihood_assumptions",
    },
    VcfProductionCorpusCase {
        case_id: "phased_inputs",
        expectation: "phase_sensitive_analyses_have_confidence_outputs",
    },
    VcfProductionCorpusCase {
        case_id: "imputed_inputs",
        expectation: "imputation_acceptance_and_panel_identity_are_tracked",
    },
    VcfProductionCorpusCase {
        case_id: "panel_mismatch",
        expectation: "panel_build_or_identity_mismatches_refuse_before_runtime",
    },
];

pub const VCF_PRODUCTION_CORPUS_CONTRACT: VcfProductionCorpusContract =
    VcfProductionCorpusContract {
        schema_version: "bijux.vcf.production_corpus.v1",
        covered_cases: VCF_PRODUCTION_CORPUS_CASES,
    };

pub const VCF_SCIENTIFIC_DRIFT_CONTRACT: VcfScientificDriftContract = VcfScientificDriftContract {
    schema_version: "bijux.vcf.scientific_drift.v1",
    tracked_change_surfaces: &[
        "filter_defaults",
        "normalization_policy",
        "damage_thresholds",
        "phasing_backend",
        "imputation_backend",
    ],
    downstream_risks: &[
        "variant_count_shift",
        "pass_rate_shift",
        "population_analysis_instability",
        "annotation_coverage_shift",
    ],
    required_report_sections: &[
        "before_after_variant_counts",
        "filter_delta_summary",
        "downstream_risk_summary",
    ],
};

#[must_use]
pub fn vcf_panel_boundary_contracts() -> &'static [VcfPanelBoundaryContract] {
    &[
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::PrepareReferencePanel,
            required_context: &[
                "panel_identity",
                "build_compatibility",
                "panel_checksum",
                "license_policy",
            ],
            execution_mode: "enforced",
        },
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::Phasing,
            required_context: &[
                "panel_identity",
                "build_compatibility",
                "sample_eligibility",
                "map_lock",
            ],
            execution_mode: "enforced",
        },
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::Impute,
            required_context: &[
                "panel_identity",
                "build_compatibility",
                "sample_eligibility",
                "map_lock",
            ],
            execution_mode: "enforced",
        },
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::Imputation,
            required_context: &[
                "panel_identity",
                "build_compatibility",
                "sample_eligibility",
                "map_lock",
            ],
            execution_mode: "advisory",
        },
    ]
}

#[must_use]
pub fn vcf_likelihood_workflow_contracts() -> &'static [VcfLikelihoodWorkflowContract] {
    &[
        VcfLikelihoodWorkflowContract {
            stage: VcfDomainStage::CallGl,
            coverage_regime: CoverageRegime::LowCovGl,
            accepted_tools: &["angsd", "bcftools"],
            low_coverage_assumptions: &[
                "gl_or_gp_fields_are_primary_outputs",
                "diploid_genotypes_may_be_unreliable",
                "coverage_model_is_part_of_evidence",
            ],
            output_caveats: &[
                "downstream_callsets_require_gl_aware_consumers",
                "site_level_absence_is_not_homozygous_reference",
            ],
        },
        VcfLikelihoodWorkflowContract {
            stage: VcfDomainStage::GlPropagation,
            coverage_regime: CoverageRegime::LowCovGl,
            accepted_tools: &["bcftools", "custom_gl_normalizer"],
            low_coverage_assumptions: &[
                "gl_pl_gp_fields_must_survive_normalization",
                "allele_reordering_without_gl_reconciliation_is_refused",
            ],
            output_caveats: &[
                "normalized_outputs_remain_likelihood_driven",
                "bcf_is_preferred_for_large_likelihood_payloads",
            ],
        },
        VcfLikelihoodWorkflowContract {
            stage: VcfDomainStage::CallPseudohaploid,
            coverage_regime: CoverageRegime::Pseudohaploid,
            accepted_tools: &["angsd", "bcftools"],
            low_coverage_assumptions: &[
                "single_allele_sampling_is_explicit",
                "heterozygosity_is_not_estimated_from_output_gt",
            ],
            output_caveats: &[
                "pseudo_haploid_calls_are_not_diploid_genotypes",
                "downstream_population_models_must_accept_pseudohaploid_inputs",
            ],
        },
    ]
}

#[must_use]
pub fn vcf_phasing_imputation_boundary_contracts() -> &'static [VcfPhasingImputationBoundaryContract]
{
    &[
        VcfPhasingImputationBoundaryContract {
            stage: VcfDomainStage::Phasing,
            accepted_tools: &["shapeit5", "eagle", "beagle"],
            required_context: &[
                "panel_identity",
                "build_compatibility",
                "map_file",
                "sample_eligibility",
            ],
            required_outputs: &[
                "phased_vcf",
                "phase_block_stats",
                "switch_error_proxy",
                "phasing_manifest",
            ],
            refusal_reasons: &[
                "panel_identity_missing",
                "map_lock_missing",
                "build_compatibility_failed",
                "confidence_outputs_not_declared",
            ],
        },
        VcfPhasingImputationBoundaryContract {
            stage: VcfDomainStage::Impute,
            accepted_tools: &["glimpse", "beagle", "impute5", "minimac4"],
            required_context: &["panel_identity", "build_compatibility", "map_file", "chunk_plan"],
            required_outputs: &[
                "imputed_vcf",
                "imputation_qc",
                "imputation_acceptance",
                "panel_mismatch_diagnostics",
            ],
            refusal_reasons: &[
                "panel_identity_missing",
                "map_lock_missing",
                "panel_build_mismatch",
                "confidence_outputs_not_declared",
            ],
        },
        VcfPhasingImputationBoundaryContract {
            stage: VcfDomainStage::Imputation,
            accepted_tools: &["glimpse", "beagle", "impute5", "minimac4"],
            required_context: &["panel_identity", "build_compatibility", "map_file", "chunk_plan"],
            required_outputs: &[
                "imputed_vcf",
                "imputation_qc",
                "imputation_acceptance",
                "panel_mismatch_diagnostics",
            ],
            refusal_reasons: &[
                "panel_identity_missing",
                "map_lock_missing",
                "panel_build_mismatch",
                "confidence_outputs_not_declared",
            ],
        },
    ]
}

#[must_use]
pub fn vcf_population_guardrail_contracts() -> &'static [VcfPopulationGuardrailContract] {
    &[
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Pca,
            required_inputs: &[
                "sample_metadata_manifest",
                "ld_pruning_policy",
                "maf_threshold",
                "missingness_threshold",
            ],
            report_caveats: &[
                "axes_are_descriptive_not_truth",
                "population_labels_must_be_audited",
            ],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Admixture,
            required_inputs: &[
                "sample_metadata_manifest",
                "ld_pruning_policy",
                "maf_threshold",
                "k_selection_policy",
            ],
            report_caveats: &[
                "cluster_count_is_model_dependent",
                "ancestry_interpretation_requires_context",
            ],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::PopulationStructure,
            required_inputs: &[
                "sample_metadata_manifest",
                "ld_pruning_policy",
                "maf_threshold",
                "missingness_threshold",
            ],
            report_caveats: &[
                "cohort_composition_changes_output",
                "projection_without_context_is_advisory",
            ],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Roh,
            required_inputs: &[
                "marker_density_floor",
                "missingness_threshold",
                "coverage_regime_caveat",
            ],
            report_caveats: &["low_coverage_can_destabilize_roh_calls"],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Ibd,
            required_inputs: &[
                "cohort_sample_count",
                "marker_density_floor",
                "missingness_threshold",
                "phase_quality_or_gl_readiness",
            ],
            report_caveats: &["ibd_segments_depend_on_phase_and_marker_quality"],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Demography,
            required_inputs: &[
                "cohort_sample_count",
                "ibd_segments",
                "method_assumptions_documented",
            ],
            report_caveats: &[
                "demography_estimates_are_model_based",
                "sparse_ibd_reduces_confidence",
            ],
        },
    ]
}

#[must_use]
pub fn vcf_cohort_analysis_boundary_contracts() -> &'static [VcfCohortAnalysisBoundaryContract] {
    &[
        VcfCohortAnalysisBoundaryContract {
            stage: VcfDomainStage::Roh,
            minimum_cohort_requirement: "single_sample_or_larger_with_explicit_coverage_regime",
            marker_density_requirement: "marker_density_floor_declared",
            missingness_requirement: "missingness_below_roh_threshold",
            method_assumptions: &[
                "roh_segments_depend_on_variant_density",
                "pseudohaploid_low_coverage_requires_explicit_support",
            ],
        },
        VcfCohortAnalysisBoundaryContract {
            stage: VcfDomainStage::Ibd,
            minimum_cohort_requirement: "at_least_two_samples",
            marker_density_requirement: "marker_density_floor_declared",
            missingness_requirement: "missingness_below_ibd_threshold",
            method_assumptions: &[
                "phase_or_gl_readiness_is_required",
                "segments_are_model_dependent",
            ],
        },
        VcfCohortAnalysisBoundaryContract {
            stage: VcfDomainStage::Demography,
            minimum_cohort_requirement: "ibd_segments_and_multi_sample_cohort_required",
            marker_density_requirement: "ibd_input_density_must_pass_upstream_gate",
            missingness_requirement: "upstream_ibd_missingness_gate_must_pass",
            method_assumptions: &[
                "demography_requires_ibdne_ready_segments",
                "sparse_ibd_reduces_confidence",
            ],
        },
    ]
}
