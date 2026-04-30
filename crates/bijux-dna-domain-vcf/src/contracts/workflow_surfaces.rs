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
pub struct VcfPopulationGuardrailContract {
    pub stage: VcfDomainStage,
    pub required_inputs: &'static [&'static str],
    pub report_caveats: &'static [&'static str],
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
        VcfDomainStage::Stats => VcfArtifactClassContract {
            stage,
            artifact_classes: &[VcfArtifactClass::StatsReport],
        },
        VcfDomainStage::PrepareReferencePanel => VcfArtifactClassContract {
            stage,
            artifact_classes: &[VcfArtifactClass::PanelEvidence, VcfArtifactClass::VariantIndex],
        },
        VcfDomainStage::Phasing | VcfDomainStage::Imputation | VcfDomainStage::Impute => {
            VcfArtifactClassContract {
                stage,
                artifact_classes: &[VcfArtifactClass::ImputationReport, VcfArtifactClass::PanelEvidence],
            }
        }
        _ => VcfArtifactClassContract {
            stage,
            artifact_classes: &[VcfArtifactClass::StatsReport],
        },
    }
}

pub const VCF_REFERENCE_CONTEXT_CONTRACT: VcfReferenceContextContract = VcfReferenceContextContract {
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

#[must_use]
pub fn vcf_calling_mode_contracts() -> &'static [VcfCallingModeContract] {
    &[
        VcfCallingModeContract {
            stage: VcfDomainStage::CallDiploid,
            coverage_regime: CoverageRegime::Diploid,
            evidence_focus: "called genotypes with diploid heterozygosity summaries",
            assumptions: &["diploid_gt_fields", "sample_identity_required", "reference_bundle_locked"],
            refusal_rules: &["caller_must_be_bcftools_or_gatk", "non_empty_records_required"],
        },
        VcfCallingModeContract {
            stage: VcfDomainStage::CallPseudohaploid,
            coverage_regime: CoverageRegime::Pseudohaploid,
            evidence_focus: "single-allele calls with pseudo-haploid caveats",
            assumptions: &["single_allele_sampling", "sample_identity_required", "low_information_expected"],
            refusal_rules: &["caller_must_be_angsd_or_bcftools", "non_empty_records_required"],
        },
        VcfCallingModeContract {
            stage: VcfDomainStage::CallGl,
            coverage_regime: CoverageRegime::LowCovGl,
            evidence_focus: "genotype likelihoods and GL-aware downstream compatibility",
            assumptions: &["gl_gp_pl_fields_required", "sample_identity_required", "low_coverage_likelihood_model"],
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

#[must_use]
pub fn vcf_panel_boundary_contracts() -> &'static [VcfPanelBoundaryContract] {
    &[
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::PrepareReferencePanel,
            required_context: &["panel_identity", "build_compatibility", "panel_checksum", "license_policy"],
            execution_mode: "enforced",
        },
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::Phasing,
            required_context: &["panel_identity", "build_compatibility", "sample_eligibility", "map_lock"],
            execution_mode: "enforced",
        },
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::Impute,
            required_context: &["panel_identity", "build_compatibility", "sample_eligibility", "map_lock"],
            execution_mode: "enforced",
        },
        VcfPanelBoundaryContract {
            stage: VcfDomainStage::Imputation,
            required_context: &["panel_identity", "build_compatibility", "sample_eligibility", "map_lock"],
            execution_mode: "advisory",
        },
    ]
}

#[must_use]
pub fn vcf_population_guardrail_contracts() -> &'static [VcfPopulationGuardrailContract] {
    &[
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Pca,
            required_inputs: &["sample_metadata_manifest", "ld_pruning_policy", "maf_threshold", "missingness_threshold"],
            report_caveats: &["axes_are_descriptive_not_truth", "population_labels_must_be_audited"],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Admixture,
            required_inputs: &["sample_metadata_manifest", "ld_pruning_policy", "maf_threshold", "k_selection_policy"],
            report_caveats: &["cluster_count_is_model_dependent", "ancestry_interpretation_requires_context"],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::PopulationStructure,
            required_inputs: &["sample_metadata_manifest", "ld_pruning_policy", "maf_threshold", "missingness_threshold"],
            report_caveats: &["cohort_composition_changes_output", "projection_without_context_is_advisory"],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Roh,
            required_inputs: &["marker_density_floor", "missingness_threshold", "coverage_regime_caveat"],
            report_caveats: &["low_coverage_can_destabilize_roh_calls"],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Ibd,
            required_inputs: &["cohort_sample_count", "marker_density_floor", "missingness_threshold", "phase_quality_or_gl_readiness"],
            report_caveats: &["ibd_segments_depend_on_phase_and_marker_quality"],
        },
        VcfPopulationGuardrailContract {
            stage: VcfDomainStage::Demography,
            required_inputs: &["cohort_sample_count", "ibd_segments", "method_assumptions_documented"],
            report_caveats: &["demography_estimates_are_model_based", "sparse_ibd_reduces_confidence"],
        },
    ]
}
