use std::collections::BTreeMap;

use bijux_dna_core::contract::ReadLayoutMode;
use bijux_dna_core::prelude::id_catalog;

use crate::{
    BatchWorkflowSemanticsV1, CrossDomainEvidenceSummaryV1, CrossDomainFailurePolicyV1,
    CrossWorkflowTemplateV1, FanArtifactRuleV1, FanPatternV1, TemplateFailureActionV1,
    TemplateParameterPolicyV1,
};

fn parameter_policy(entries: &[(&str, &[&str])], locked: &[(&str, &[&str])]) -> TemplateParameterPolicyV1 {
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

fn shared_failure_policy() -> Vec<CrossDomainFailurePolicyV1> {
    vec![
        CrossDomainFailurePolicyV1 {
            stage_family: "preprocessing".to_string(),
            action: TemplateFailureActionV1::SkipFailedSample,
            downstream_effect: "sample stops before alignment; remaining samples may continue".to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "alignment".to_string(),
            action: TemplateFailureActionV1::BlockDownstream,
            downstream_effect: "downstream BAM or VCF calling is blocked for the failed sample".to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "variant".to_string(),
            action: TemplateFailureActionV1::ContinueCohort,
            downstream_effect: "cohort summaries may proceed with explicit missing-sample caveats".to_string(),
            allows_partial_batch: true,
        },
    ]
}

fn shared_fan_rules() -> Vec<FanArtifactRuleV1> {
    vec![
        FanArtifactRuleV1 {
            source_stage: id_catalog::CORE_PREPARE_REFERENCE.to_string(),
            target_stage: id_catalog::BAM_ALIGN.to_string(),
            fan_pattern: FanPatternV1::FanOut,
            artifact_scope: "shared_reference_bundle".to_string(),
            lineage_fields: vec!["reference_id".to_string()],
            overwrite_strategy: "write_once_reuse_many".to_string(),
        },
        FanArtifactRuleV1 {
            source_stage: "bam.genotyping".to_string(),
            target_stage: id_catalog::VCF_STATS.to_string(),
            fan_pattern: FanPatternV1::FanIn,
            artifact_scope: "cohort_variant_inventory".to_string(),
            lineage_fields: vec!["sample_id".to_string(), "reference_id".to_string()],
            overwrite_strategy: "sample_scoped_outputs_then_cohort_merge".to_string(),
        },
    ]
}

#[must_use]
pub fn cross_workflow_templates() -> Vec<CrossWorkflowTemplateV1> {
    vec![
        CrossWorkflowTemplateV1 {
            schema_version: "bijux.cross.workflow_template.v1".to_string(),
            template_id: "cross.fastq_to_bam_modern".to_string(),
            pipeline_id: id_catalog::PIPELINE_FASTQ_TO_BAM_DEFAULT.to_string(),
            summary: "FASTQ preprocessing to BAM alignment with governed sample, layout, and reference admission rules.".to_string(),
            requested_stages: vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::CORE_PREPARE_REFERENCE.to_string(),
                id_catalog::BAM_ALIGN.to_string(),
                id_catalog::BAM_QC_PRE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
            ],
            supported_layouts: vec![ReadLayoutMode::SingleEnd, ReadLayoutMode::PairedEnd],
            requires_reference_assets: true,
            requires_bam_index: false,
            requires_sample_metadata: vec![
                "sample_id".to_string(),
                "library_id".to_string(),
                "lane_id".to_string(),
                "reference_id".to_string(),
            ],
            sample_sheet_supported: true,
            batch_semantics: BatchWorkflowSemanticsV1 {
                per_sample_stages: vec![
                    id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    id_catalog::FASTQ_TRIM.to_string(),
                    id_catalog::BAM_ALIGN.to_string(),
                    id_catalog::BAM_QC_PRE.to_string(),
                    id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                    id_catalog::BAM_COVERAGE.to_string(),
                ],
                cohort_stages: vec![id_catalog::BAM_MAPPING_SUMMARY.to_string()],
                shared_reference_stages: vec![id_catalog::CORE_PREPARE_REFERENCE.to_string()],
            },
            fan_artifact_rules: shared_fan_rules(),
            failure_policy: shared_failure_policy(),
            evidence_summary: CrossDomainEvidenceSummaryV1 {
                story_order: vec![
                    "read_preprocessing".to_string(),
                    "alignment_quality".to_string(),
                    "handoff_readiness".to_string(),
                ],
                final_caveat_topics: vec![
                    "layout_assumptions".to_string(),
                    "reference_identity".to_string(),
                ],
            },
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::FASTQ_TRIM, &["min_length", "adapter_mode"]),
                    (id_catalog::BAM_ALIGN, &["seed_length", "min_mapq"]),
                ],
                &[
                    (id_catalog::BAM_ALIGN, &["reference_fasta"]),
                    (id_catalog::CORE_PREPARE_REFERENCE, &["reference_build"]),
                ],
            ),
            example_ids: vec!["fastq_essential_qc".to_string(), "bam_essential_alignment_qc".to_string()],
        },
        CrossWorkflowTemplateV1 {
            schema_version: "bijux.cross.workflow_template.v1".to_string(),
            template_id: "cross.bam_to_vcf_default".to_string(),
            pipeline_id: id_catalog::PIPELINE_BAM_TO_VCF_DEFAULT.to_string(),
            summary: "Aligned BAM to VCF calling with explicit BAM index, coverage, sample, and reference guardrails.".to_string(),
            requested_stages: vec![
                id_catalog::CORE_PREPARE_REFERENCE.to_string(),
                id_catalog::BAM_VALIDATE.to_string(),
                id_catalog::BAM_QC_PRE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                id_catalog::BAM_COVERAGE.to_string(),
                "bam.genotyping".to_string(),
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            supported_layouts: vec![ReadLayoutMode::SingleEnd, ReadLayoutMode::PairedEnd],
            requires_reference_assets: true,
            requires_bam_index: true,
            requires_sample_metadata: vec!["sample_id".to_string(), "reference_id".to_string()],
            sample_sheet_supported: true,
            batch_semantics: BatchWorkflowSemanticsV1 {
                per_sample_stages: vec![
                    id_catalog::BAM_VALIDATE.to_string(),
                    id_catalog::BAM_QC_PRE.to_string(),
                    id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                    id_catalog::BAM_COVERAGE.to_string(),
                    "bam.genotyping".to_string(),
                ],
                cohort_stages: vec![id_catalog::VCF_FILTER.to_string(), id_catalog::VCF_STATS.to_string()],
                shared_reference_stages: vec![id_catalog::CORE_PREPARE_REFERENCE.to_string()],
            },
            fan_artifact_rules: shared_fan_rules(),
            failure_policy: shared_failure_policy(),
            evidence_summary: CrossDomainEvidenceSummaryV1 {
                story_order: vec![
                    "alignment_quality".to_string(),
                    "coverage_readiness".to_string(),
                    "variant_processing".to_string(),
                ],
                final_caveat_topics: vec![
                    "coverage_regime".to_string(),
                    "sample_prerequisites".to_string(),
                ],
            },
            parameter_policy: parameter_policy(
                &[
                    ("bam.genotyping", &["minimum_mean_coverage", "genotype_mode"]),
                    (id_catalog::VCF_FILTER, &["require_pass"]),
                ],
                &[
                    ("bam.genotyping", &["reference_fasta"]),
                    (id_catalog::VCF_FILTER, &["production_profile"]),
                ],
            ),
            example_ids: vec!["bam_essential_alignment_qc".to_string(), "vcf_essential_qc".to_string()],
        },
        CrossWorkflowTemplateV1 {
            schema_version: "bijux.cross.workflow_template.v1".to_string(),
            template_id: "cross.fastq_to_vcf_minimal".to_string(),
            pipeline_id: id_catalog::PIPELINE_FASTQ_TO_VCF_MINIMAL.to_string(),
            summary: "Tiny FASTQ-to-VCF template that plans validate, trim, align, sort/index, calling, and stats with stable evidence semantics.".to_string(),
            requested_stages: vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::CORE_PREPARE_REFERENCE.to_string(),
                id_catalog::BAM_ALIGN.to_string(),
                id_catalog::BAM_QC_PRE.to_string(),
                id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                "bam.genotyping".to_string(),
                id_catalog::VCF_FILTER.to_string(),
                id_catalog::VCF_STATS.to_string(),
            ],
            supported_layouts: vec![ReadLayoutMode::SingleEnd, ReadLayoutMode::PairedEnd],
            requires_reference_assets: true,
            requires_bam_index: false,
            requires_sample_metadata: vec![
                "sample_id".to_string(),
                "library_id".to_string(),
                "lane_id".to_string(),
                "reference_id".to_string(),
            ],
            sample_sheet_supported: true,
            batch_semantics: BatchWorkflowSemanticsV1 {
                per_sample_stages: vec![
                    id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    id_catalog::FASTQ_TRIM.to_string(),
                    id_catalog::BAM_ALIGN.to_string(),
                    id_catalog::BAM_QC_PRE.to_string(),
                    id_catalog::BAM_MAPPING_SUMMARY.to_string(),
                    "bam.genotyping".to_string(),
                ],
                cohort_stages: vec![id_catalog::VCF_FILTER.to_string(), id_catalog::VCF_STATS.to_string()],
                shared_reference_stages: vec![id_catalog::CORE_PREPARE_REFERENCE.to_string()],
            },
            fan_artifact_rules: shared_fan_rules(),
            failure_policy: shared_failure_policy(),
            evidence_summary: CrossDomainEvidenceSummaryV1 {
                story_order: vec![
                    "read_preprocessing".to_string(),
                    "alignment_quality".to_string(),
                    "variant_processing".to_string(),
                ],
                final_caveat_topics: vec![
                    "tiny_fixture_limits".to_string(),
                    "final_call_confidence".to_string(),
                ],
            },
            parameter_policy: parameter_policy(
                &[
                    (id_catalog::FASTQ_TRIM, &["min_length", "adapter_mode"]),
                    ("bam.genotyping", &["genotype_mode"]),
                    (id_catalog::VCF_FILTER, &["require_pass"]),
                ],
                &[
                    (id_catalog::BAM_ALIGN, &["reference_fasta"]),
                    ("bam.genotyping", &["minimum_mean_coverage"]),
                ],
            ),
            example_ids: vec!["fastq_essential_qc".to_string(), "vcf_essential_qc".to_string()],
        },
    ]
}

#[must_use]
pub fn cross_workflow_template_by_id(template_id: &str) -> Option<CrossWorkflowTemplateV1> {
    cross_workflow_templates()
        .into_iter()
        .find(|template| template.template_id == template_id)
}

#[must_use]
pub fn cross_workflow_templates_for_pipeline(pipeline_id: &str) -> Vec<CrossWorkflowTemplateV1> {
    cross_workflow_templates()
        .into_iter()
        .filter(|template| template.pipeline_id == pipeline_id)
        .collect()
}
