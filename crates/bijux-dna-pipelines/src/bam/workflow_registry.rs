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

fn bam_failure_policy() -> Vec<CrossDomainFailurePolicyV1> {
    vec![
        CrossDomainFailurePolicyV1 {
            stage_family: "ingestion".to_string(),
            action: TemplateFailureActionV1::SkipFailedSample,
            downstream_effect:
                "samples that fail BAM integrity checks are skipped with explicit refusal evidence"
                    .to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "preprocessing".to_string(),
            action: TemplateFailureActionV1::SkipFailedSample,
            downstream_effect:
                "failed preprocessing blocks downstream sample metrics while batch peers continue"
                    .to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "reporting".to_string(),
            action: TemplateFailureActionV1::ContinueCohort,
            downstream_effect:
                "cohort summaries proceed with missing-sample caveats and retained refusal reasons"
                    .to_string(),
            allows_partial_batch: true,
        },
    ]
}

fn merge_fan_rules() -> Vec<FanArtifactRuleV1> {
    vec![FanArtifactRuleV1 {
        source_stage: id_catalog::BAM_VALIDATE.to_string(),
        target_stage: id_catalog::BAM_MAPPING_SUMMARY.to_string(),
        fan_pattern: FanPatternV1::FanIn,
        artifact_scope: "lane_library_merge_bundle".to_string(),
        lineage_fields: vec![
            "run_id".to_string(),
            "sample_id".to_string(),
            "library_id".to_string(),
        ],
        overwrite_strategy: "sample_scoped_inputs_then_stable_cohort_merge".to_string(),
    }]
}

struct BamTemplateSpec {
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

fn template(spec: BamTemplateSpec) -> CrossWorkflowTemplateV1 {
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
            "library_id".to_string(),
            "reference_id".to_string(),
        ],
        sample_sheet_supported: true,
        batch_semantics: BatchWorkflowSemanticsV1 {
            per_sample_stages: spec.requested_stages,
            cohort_stages: vec![],
            shared_reference_stages: spec.shared_reference_stages,
        },
        fan_artifact_rules: spec.fan_artifact_rules,
        failure_policy: bam_failure_policy(),
        evidence_summary: CrossDomainEvidenceSummaryV1 {
            story_order: spec.evidence_story.iter().map(|value| (*value).to_string()).collect(),
            final_caveat_topics: spec.caveats.iter().map(|value| (*value).to_string()).collect(),
        },
        parameter_policy: spec.parameter_policy,
        example_ids: spec.example_ids.iter().map(|value| (*value).to_string()).collect(),
    }
}

#[must_use]
pub fn bam_workflow_templates() -> Vec<CrossWorkflowTemplateV1> {
    vec![
        template(BamTemplateSpec {
            template_id: "bam.modern_wgs_qc",
            pipeline_id: id_catalog::PIPELINE_BAM_DEFAULT,
            summary: "Modern WGS BAM QC template with alignment-readiness checks, duplicate review, and coverage-centric reporting.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_QC_PRE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_FILTER.to_string(),
                id_catalog::BAM_DUPLICATION_METRICS.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "alignment_quality", "coverage_readiness", "qc_summary"],
            caveats: &["reference_identity_required", "duplicate_handling_limits"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::BAM_FILTER, &["mapq_threshold", "min_length"]),
                    (id_catalog::BAM_COVERAGE, &["depth_thresholds"]),
                ],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_essential_alignment_qc"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.ancient_dna_qc",
            pipeline_id: id_catalog::PIPELINE_BAM_ADNA_SHOTGUN,
            summary: "Ancient-DNA BAM QC template with damage/authenticity/contamination evidence and low-input caveats.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_FILTER.to_string(),
                id_catalog::BAM_LENGTH_FILTER.to_string(),
                id_catalog::BAM_DAMAGE.to_string(),
                id_catalog::BAM_AUTHENTICITY.to_string(),
                id_catalog::BAM_CONTAMINATION.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "damage_evidence", "contamination_evidence", "qc_summary"],
            caveats: &["post_mortem_damage", "low_coverage_uncertainty"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::BAM_DAMAGE, &["pmd_threshold_5p", "pmd_threshold_3p"]),
                    (id_catalog::BAM_CONTAMINATION, &["minimum_mean_coverage", "scope"]),
                ],
                &[(id_catalog::BAM_AUTHENTICITY, &["disallow_certification"])],
            ),
            example_ids: &["bam_adna_damage_review"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.low_pass_readiness",
            pipeline_id: id_catalog::PIPELINE_BAM_DEFAULT,
            summary: "Low-pass BAM readiness template with coverage-classification thresholds and downstream calling caveats.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_QC_PRE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
                id_catalog::BAM_FILTER.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "coverage_regime", "calling_readiness", "qc_summary"],
            caveats: &["low_pass_classification", "downstream_gl_uncertainty"],
            parameter_policy: parameter_policy(
                &[(id_catalog::BAM_COVERAGE, &["depth_thresholds", "regime_mode"])],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_low_pass_qc"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.targeted_amplicon_qc",
            pipeline_id: id_catalog::PIPELINE_BAM_ADNA_CAPTURE,
            summary: "Targeted/amplicon BAM template with target-coverage, off-target, duplication, and assay caveat reporting.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_DUPLICATION_METRICS.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
                id_catalog::BAM_ENDOGENOUS_CONTENT.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "target_coverage", "off_target_signal", "qc_summary"],
            caveats: &["assay_specific_bias", "capture_panel_dependency"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::BAM_COVERAGE, &["regions", "depth_thresholds"]),
                    (id_catalog::BAM_ENDOGENOUS_CONTENT, &["depth_thresholds"]),
                ],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_targeted_qc"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.aligner_comparison_report",
            pipeline_id: id_catalog::PIPELINE_BAM_DEFAULT,
            summary: "BAM aligner-comparison report template contrasting mapping quality and downstream metric behavior across aligner modes.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_QC_PRE.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "aligner_delta", "mapping_metric_delta", "qc_summary"],
            caveats: &["aligner_parameter_equivalence", "reference_index_equivalence"],
            parameter_policy: parameter_policy(
                &[(id_catalog::BAM_MAPPING_SUMMARY, &["regions"])],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_aligner_compare"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.duplicate_method_comparison_report",
            pipeline_id: id_catalog::PIPELINE_BAM_DEFAULT,
            summary: "BAM duplicate-method comparison template for Picard/samtools/UMI-aware downstream impact review.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_DUPLICATION_METRICS.to_string(),
                id_catalog::BAM_FILTER.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "duplicate_policy_delta", "coverage_effect", "qc_summary"],
            caveats: &["umi_presence_required", "duplicate_marking_method_bias"],
            parameter_policy: parameter_policy(
                &[(id_catalog::BAM_DUPLICATION_METRICS, &["duplicate_action", "umi_policy"])],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_duplicate_method_compare"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.contamination_method_comparison_report",
            pipeline_id: id_catalog::PIPELINE_BAM_REFERENCE_ADNA,
            summary: "BAM contamination estimator comparison template with prerequisite disclosure and disagreement caveats.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_CONTAMINATION.to_string(),
                id_catalog::BAM_AUTHENTICITY.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &[
                "bam_admission",
                "contamination_prerequisites",
                "method_disagreement",
                "qc_summary",
            ],
            caveats: &["coverage_floor_dependency", "panel_compatibility_dependency"],
            parameter_policy: parameter_policy(
                &[(id_catalog::BAM_CONTAMINATION, &["scope", "minimum_mean_coverage"])],
                &[(id_catalog::BAM_AUTHENTICITY, &["disallow_certification"])],
            ),
            example_ids: &["bam_contamination_compare"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.batch_merge_workflow",
            pipeline_id: id_catalog::PIPELINE_BAM_DEFAULT,
            summary: "BAM batch merge template for lane/library merge conflict detection and aggregate review evidence.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_FILTER.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: merge_fan_rules(),
            evidence_story: &["bam_admission", "merge_conflict_review", "aggregate_qc_summary"],
            caveats: &["lane_conflict_policy", "library_identity_consistency"],
            parameter_policy: parameter_policy(
                &[(id_catalog::BAM_FILTER, &["include_flags", "exclude_flags"])],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_lane_merge_review"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.coverage_review_report",
            pipeline_id: id_catalog::PIPELINE_BAM_DEFAULT,
            summary: "BAM coverage-to-review report template for concise operator/scientific summary outputs.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
                id_catalog::BAM_QC_PRE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "coverage_snapshot", "review_summary"],
            caveats: &["regional_coverage_interpretation", "reference_version_compatibility"],
            parameter_policy: parameter_policy(
                &[(id_catalog::BAM_COVERAGE, &["regions", "depth_thresholds"])],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_coverage_review"],
        }),
        template(BamTemplateSpec {
            template_id: "bam.large_file_performance_profile",
            pipeline_id: id_catalog::PIPELINE_BAM_DEFAULT,
            summary: "BAM large-file performance template tracking runtime and memory behavior for sort/index/coverage-like heavy paths.",
            requested_stages: vec![
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_FILTER.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            shared_reference_stages: vec![],
            fan_artifact_rules: vec![],
            evidence_story: &["bam_admission", "resource_profile", "throughput_summary"],
            caveats: &["fixture_scale_limitations", "hardware_dependency"],
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::BAM_FILTER, &["mapq_threshold", "min_length"]),
                    (id_catalog::BAM_COVERAGE, &["depth_thresholds"]),
                ],
                &[(id_catalog::BAM_VALIDATE, &["strict"])],
            ),
            example_ids: &["bam_performance_profile"],
        }),
    ]
}

#[must_use]
pub fn bam_workflow_template_by_id(template_id: &str) -> Option<CrossWorkflowTemplateV1> {
    bam_workflow_templates().into_iter().find(|template| template.template_id == template_id)
}

#[must_use]
pub fn bam_workflow_templates_for_pipeline(pipeline_id: &str) -> Vec<CrossWorkflowTemplateV1> {
    bam_workflow_templates()
        .into_iter()
        .filter(|template| template.pipeline_id == pipeline_id)
        .collect()
}
