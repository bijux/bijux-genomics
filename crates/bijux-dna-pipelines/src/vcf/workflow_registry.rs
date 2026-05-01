use std::collections::BTreeMap;

use bijux_dna_core::contract::ReadLayoutMode;
use bijux_dna_core::prelude::id_catalog;

use crate::{
    BatchWorkflowSemanticsV1, CrossDomainEvidenceSummaryV1, CrossDomainFailurePolicyV1,
    CrossWorkflowTemplateV1, FanArtifactRuleV1, FanPatternV1, TemplateFailureActionV1,
    TemplateParameterPolicyV1,
};

fn parameter_policy(
    entries: &[(&str, &[&str])],
    locked: &[(&str, &[&str])],
) -> TemplateParameterPolicyV1 {
    TemplateParameterPolicyV1 {
        expert_mode_required_for_locked_overrides: true,
        configurable_by_stage: entries
            .iter()
            .map(|(stage, params)| {
                ((*stage).to_string(), params.iter().map(|param| (*param).to_string()).collect())
            })
            .collect::<BTreeMap<_, _>>(),
        locked_by_stage: locked
            .iter()
            .map(|(stage, params)| {
                ((*stage).to_string(), params.iter().map(|param| (*param).to_string()).collect())
            })
            .collect::<BTreeMap<_, _>>(),
    }
}

fn vcf_failure_policy() -> Vec<CrossDomainFailurePolicyV1> {
    vec![
        CrossDomainFailurePolicyV1 {
            stage_family: "ingestion".to_string(),
            action: TemplateFailureActionV1::SkipFailedSample,
            downstream_effect:
                "samples with malformed or incompatible VCF content are skipped with refusal evidence"
                    .to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "variant_processing".to_string(),
            action: TemplateFailureActionV1::SkipFailedSample,
            downstream_effect:
                "failed processing blocks downstream sample-level interpretation while cohort peers continue"
                    .to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "reporting".to_string(),
            action: TemplateFailureActionV1::ContinueCohort,
            downstream_effect:
                "cohort reports proceed with explicit missing-sample and advisory caveats".to_string(),
            allows_partial_batch: true,
        },
    ]
}

fn cohort_qc_fan_rules() -> Vec<FanArtifactRuleV1> {
    vec![FanArtifactRuleV1 {
        source_stage: id_catalog::VCF_FILTER.to_string(),
        target_stage: id_catalog::VCF_STATS.to_string(),
        fan_pattern: FanPatternV1::FanIn,
        artifact_scope: "cohort_variant_statistics".to_string(),
        lineage_fields: vec![
            "batch_id".to_string(),
            "sample_id".to_string(),
            "reference_id".to_string(),
        ],
        overwrite_strategy: "sample_scoped_stats_then_stable_cohort_aggregate".to_string(),
    }]
}

struct VcfTemplateSpec {
    template_id: &'static str,
    pipeline_id: &'static str,
    summary: &'static str,
    requested_stages: Vec<String>,
    shared_reference_stages: Vec<String>,
    fan_artifact_rules: Vec<FanArtifactRuleV1>,
    evidence_story: &'static [&'static str],
    caveats: &'static [&'static str],
    parameter_policy: TemplateParameterPolicyV1,
    example_ids: &'static [&'static str],
}

fn template(spec: VcfTemplateSpec) -> CrossWorkflowTemplateV1 {
    CrossWorkflowTemplateV1 {
        schema_version: "bijux.cross.workflow_template.v1".to_string(),
        template_id: spec.template_id.to_string(),
        pipeline_id: spec.pipeline_id.to_string(),
        summary: spec.summary.to_string(),
        requested_stages: spec.requested_stages.clone(),
        supported_layouts: vec![ReadLayoutMode::SingleEnd, ReadLayoutMode::PairedEnd],
        requires_reference_assets: !spec.shared_reference_stages.is_empty(),
        requires_bam_index: false,
        requires_sample_metadata: vec![
            "run_id".to_string(),
            "sample_id".to_string(),
            "batch_id".to_string(),
            "reference_id".to_string(),
        ],
        sample_sheet_supported: true,
        batch_semantics: BatchWorkflowSemanticsV1 {
            per_sample_stages: spec.requested_stages,
            cohort_stages: vec![],
            shared_reference_stages: spec.shared_reference_stages,
        },
        fan_artifact_rules: spec.fan_artifact_rules,
        failure_policy: vcf_failure_policy(),
        evidence_summary: CrossDomainEvidenceSummaryV1 {
            story_order: spec.evidence_story.iter().map(|item| (*item).to_string()).collect(),
            final_caveat_topics: spec.caveats.iter().map(|item| (*item).to_string()).collect(),
        },
        parameter_policy: spec.parameter_policy,
        example_ids: spec.example_ids.iter().map(|item| (*item).to_string()).collect(),
    }
}

#[must_use]
pub fn vcf_workflow_templates() -> Vec<CrossWorkflowTemplateV1> {
    vec![
        template(VcfTemplateSpec {
            template_id: "vcf.validation_normalization_qc",
            pipeline_id: id_catalog::PIPELINE_VCF_REFERENCE_BASIC,
            summary: "Validation-normalization VCF template with explicit filter semantics and production QC caveats.",
            requested_stages: vec![
                id_catalog::VCF_CALL.to_string(),
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![id_catalog::VCF_CALL.to_string()],
            fan_artifact_rules: vec![],
            evidence_story: &[
                "vcf_admission",
                "normalization_boundaries",
                "filter_impact",
                "qc_summary",
            ],
            caveats: &["reference_aliasing", "normalization_scope_limits"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_CALL, &["caller", "min_base_quality", "min_mapping_quality"]),
                    (
                        id_catalog::VCF_FILTER,
                        &["min_qual", "require_pass", "normalize", "require_bgzip_tabix"],
                    ),
                ],
                &[(id_catalog::VCF_CALL, &["reference_fasta"])],
            ),
            example_ids: &["vcf_reference_qc"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.cohort_qc_review",
            pipeline_id: id_catalog::PIPELINE_VCF_REFERENCE_BASIC,
            summary: "Cohort VCF QC template with missingness/heterozygosity/relatedness proxy reporting boundaries.",
            requested_stages: vec![
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: cohort_qc_fan_rules(),
            evidence_story: &["vcf_admission", "cohort_qc", "filter_impact", "qc_summary"],
            caveats: &["cohort_size_sensitivity", "relatedness_proxy_not_kinship_truth"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_FILTER, &["min_qual", "require_pass"]),
                    (id_catalog::VCF_STATS, &["compute_titv", "collect_depth_distribution"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_cohort_qc_review"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.low_pass_gl_readiness",
            pipeline_id: id_catalog::PIPELINE_VCF_MINIMAL,
            summary: "Low-pass GL readiness template for genotype likelihood handling and caveated downstream interpretation.",
            requested_stages: vec![
                id_catalog::VCF_CALL.to_string(),
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![id_catalog::VCF_CALL.to_string()],
            fan_artifact_rules: vec![],
            evidence_story: &["vcf_admission", "gl_boundary", "low_pass_readiness", "qc_summary"],
            caveats: &["pseudohaploid_boundary", "imputation_readiness_uncertainty"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_CALL, &["min_mapping_quality"]),
                    (id_catalog::VCF_FILTER, &["min_qual", "require_pass"]),
                    (id_catalog::VCF_STATS, &["collect_depth_distribution"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_low_pass_gl"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.imputation_simulation",
            pipeline_id: id_catalog::PIPELINE_VCF_MINIMAL,
            summary: "Imputation simulation template with advisory/simulation boundaries and panel-map readiness caveats.",
            requested_stages: vec![
                id_catalog::VCF_CALL.to_string(),
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![id_catalog::VCF_CALL.to_string()],
            fan_artifact_rules: vec![],
            evidence_story: &[
                "vcf_admission",
                "panel_map_readiness",
                "simulation_boundary",
                "qc_summary",
            ],
            caveats: &["simulation_only_label", "panel_map_mismatch_risk"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_CALL, &["caller", "reference_fasta"]),
                    (id_catalog::VCF_FILTER, &["normalize"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_imputation_simulation"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.population_structure_guardrail",
            pipeline_id: id_catalog::PIPELINE_VCF_REFERENCE_BASIC,
            summary: "Population-structure guardrail template with pruning/filtering readiness and interpretation caveats.",
            requested_stages: vec![
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: cohort_qc_fan_rules(),
            evidence_story: &[
                "vcf_admission",
                "population_structure_readiness",
                "cohort_qc",
                "qc_summary",
            ],
            caveats: &["population_structure_non_diagnostic", "sampling_bias_risk"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_FILTER, &["min_qual", "require_pass", "normalize"]),
                    (id_catalog::VCF_STATS, &["compute_titv"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_population_structure"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.roh_ibd_boundary",
            pipeline_id: id_catalog::PIPELINE_VCF_REFERENCE_BASIC,
            summary: "ROH/IBD boundary template with marker-density and missingness refusal-aware checks.",
            requested_stages: vec![
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: cohort_qc_fan_rules(),
            evidence_story: &["vcf_admission", "marker_density_check", "roh_ibd_boundary", "qc_summary"],
            caveats: &["marker_density_floor", "cohort_size_requirement"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_FILTER, &["min_qual", "require_pass"]),
                    (id_catalog::VCF_STATS, &["collect_depth_distribution"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_roh_ibd_boundary"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.demography_boundary_refusal",
            pipeline_id: id_catalog::PIPELINE_VCF_MINIMAL,
            summary: "Demography boundary template focused on explicit refusal for unsupported or underpowered interpretations.",
            requested_stages: vec![
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &[
                "vcf_admission",
                "demography_boundary",
                "refusal_reasoning",
                "qc_summary",
            ],
            caveats: &["unsupported_demography_scope", "insufficient_power"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_FILTER, &["require_pass", "normalize"]),
                    (id_catalog::VCF_STATS, &["compute_titv"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_demography_boundary"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.annotation_provenance_workflow",
            pipeline_id: id_catalog::PIPELINE_VCF_REFERENCE_BASIC,
            summary: "VCF annotation workflow template for provenance-first field coverage and source identity reporting.",
            requested_stages: vec![
                id_catalog::VCF_CALL.to_string(),
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![id_catalog::VCF_CALL.to_string()],
            fan_artifact_rules: vec![],
            evidence_story: &[
                "vcf_admission",
                "annotation_coverage",
                "provenance_chain",
                "qc_summary",
            ],
            caveats: &["annotation_source_versioning", "transcript_model_variability"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_CALL, &["caller", "reference_fasta"]),
                    (id_catalog::VCF_FILTER, &["normalize", "require_bgzip_tabix"]),
                ],
                &[(id_catalog::VCF_CALL, &["sample_name"])],
            ),
            example_ids: &["vcf_annotation_provenance"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.semantic_diff_report",
            pipeline_id: id_catalog::PIPELINE_VCF_MINIMAL,
            summary: "VCF semantic-diff template explaining filter/reference/panel normalization deltas as scientific caveats.",
            requested_stages: vec![
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["vcf_admission", "semantic_delta", "policy_impact", "qc_summary"],
            caveats: &["filter_policy_comparability", "reference_panel_drift"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_FILTER, &["min_qual", "require_pass", "normalize"]),
                    (id_catalog::VCF_STATS, &["compute_titv", "collect_depth_distribution"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_semantic_diff"],
        }),
        template(VcfTemplateSpec {
            template_id: "vcf.large_file_performance_profile",
            pipeline_id: id_catalog::PIPELINE_VCF_MINIMAL,
            summary: "Large-file VCF performance template tracking stats/filter/normalize runtime and memory caveats.",
            requested_stages: vec![
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &[
                "vcf_admission",
                "performance_profile",
                "resource_limits",
                "qc_summary",
            ],
            caveats: &["medium_fixture_only", "hardware_dependency"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::VCF_FILTER, &["min_qual", "normalize", "require_bgzip_tabix"]),
                    (id_catalog::VCF_STATS, &["collect_depth_distribution"]),
                ],
                &[(id_catalog::VCF_FILTER, &["production_profile"])],
            ),
            example_ids: &["vcf_large_file_performance"],
        }),
    ]
}

#[must_use]
pub fn vcf_workflow_template_by_id(template_id: &str) -> Option<CrossWorkflowTemplateV1> {
    vcf_workflow_templates().into_iter().find(|template| template.template_id == template_id)
}

#[must_use]
pub fn vcf_workflow_templates_for_pipeline(pipeline_id: &str) -> Vec<CrossWorkflowTemplateV1> {
    vcf_workflow_templates()
        .into_iter()
        .filter(|template| template.pipeline_id == pipeline_id)
        .collect()
}
