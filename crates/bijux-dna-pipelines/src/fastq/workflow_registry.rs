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

fn fastq_failure_policy() -> Vec<CrossDomainFailurePolicyV1> {
    vec![
        CrossDomainFailurePolicyV1 {
            stage_family: "preflight".to_string(),
            action: TemplateFailureActionV1::SkipFailedSample,
            downstream_effect:
                "sample fails quality admission and is skipped with explicit caveats".to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "preprocessing".to_string(),
            action: TemplateFailureActionV1::SkipFailedSample,
            downstream_effect:
                "sample preprocessing halts for failed samples while other batch members continue"
                    .to_string(),
            allows_partial_batch: true,
        },
        CrossDomainFailurePolicyV1 {
            stage_family: "reporting".to_string(),
            action: TemplateFailureActionV1::ContinueCohort,
            downstream_effect:
                "reporting continues and emits incomplete-sample caveats for any skipped sample"
                    .to_string(),
            allows_partial_batch: true,
        },
    ]
}

fn fan_rules_from_reference(source_stage: &str, target_stage: &str) -> Vec<FanArtifactRuleV1> {
    vec![FanArtifactRuleV1 {
        source_stage: source_stage.to_string(),
        target_stage: target_stage.to_string(),
        fan_pattern: FanPatternV1::FanOut,
        artifact_scope: "shared_reference_bundle".to_string(),
        lineage_fields: vec!["reference_id".to_string()],
        overwrite_strategy: "write_once_reuse_many".to_string(),
    }]
}

#[allow(clippy::too_many_arguments)]
fn template(
    template_id: &str,
    pipeline_id: &str,
    summary: &str,
    requested_stages: Vec<String>,
    shared_reference_stages: Vec<String>,
    fan_artifact_rules: Vec<FanArtifactRuleV1>,
    evidence_story: &[&str],
    caveats: &[&str],
    parameter_policy: TemplateParameterPolicyV1,
    example_ids: &[&str],
) -> CrossWorkflowTemplateV1 {
    CrossWorkflowTemplateV1 {
        schema_version: "bijux.cross.workflow_template.v1".to_string(),
        template_id: template_id.to_string(),
        pipeline_id: pipeline_id.to_string(),
        summary: summary.to_string(),
        requested_stages: requested_stages.clone(),
        supported_layouts: vec![ReadLayoutMode::SingleEnd, ReadLayoutMode::PairedEnd],
        requires_reference_assets: !shared_reference_stages.is_empty(),
        requires_bam_index: false,
        requires_sample_metadata: vec![
            "run_id".to_string(),
            "sample_id".to_string(),
            "library_id".to_string(),
            "lane_id".to_string(),
        ],
        sample_sheet_supported: true,
        batch_semantics: BatchWorkflowSemanticsV1 {
            per_sample_stages: requested_stages,
            cohort_stages: vec![],
            shared_reference_stages,
        },
        fan_artifact_rules,
        failure_policy: fastq_failure_policy(),
        evidence_summary: CrossDomainEvidenceSummaryV1 {
            story_order: evidence_story.iter().map(|item| (*item).to_string()).collect(),
            final_caveat_topics: caveats.iter().map(|item| (*item).to_string()).collect(),
        },
        parameter_policy,
        example_ids: example_ids.iter().map(|item| (*item).to_string()).collect(),
    }
}

#[must_use]
pub fn fastq_workflow_templates() -> Vec<CrossWorkflowTemplateV1> {
    vec![
        template(
            "fastq.qc_only_review",
            id_catalog::PIPELINE_FASTQ_QC_ONLY,
            "Raw-read QC-only template with validation, read-length profiling, overrepresented sequence profiling, and governed report caveats.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_PROFILE_READ_LENGTHS.to_string(),
                id_catalog::FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![],
            vec![],
            &["read_admission", "sequence_composition", "qc_summary"],
            &["raw_input_only", "no_trimming_or_filtering"],
            parameter_policy(
                &[(id_catalog::FASTQ_STATS_NEUTRAL, &["gc_content_bins"])],
                &[(id_catalog::FASTQ_VALIDATE_READS, &["strictness_level"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.trim_qc",
            id_catalog::PIPELINE_FASTQ_TRIM_QC,
            "Trim-and-QC preprocessing template with adapter detection, trim/filter policy, and post-trim report outputs.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![],
            vec![],
            &["read_admission", "trimming_effect", "qc_summary"],
            &["adapter_detection_confidence", "low_complexity_drop_risk"],
            parameter_policy(
                &[
                    (id_catalog::FASTQ_DETECT_ADAPTERS, &["kmer_size", "seed_mismatches"]),
                    (id_catalog::FASTQ_TRIM, &["min_length", "adapter_mode"]),
                    (id_catalog::FASTQ_FILTER, &["minimum_quality", "max_n_rate"]),
                ],
                &[(id_catalog::FASTQ_TRIM, &["preserve_pairing"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.umi_aware_preprocessing",
            id_catalog::PIPELINE_FASTQ_UMI,
            "UMI-aware FASTQ template for UMI extraction, grouping-oriented dedup, and provenance-rich QC handoff.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_UMI.to_string(),
                id_catalog::FASTQ_DEDUPLICATE.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![],
            vec![],
            &["read_admission", "umi_provenance", "dedup_effect", "qc_summary"],
            &["umi_pattern_assumptions", "deduplication_bias"],
            parameter_policy(
                &[
                    (id_catalog::FASTQ_UMI, &["pattern", "allow_partial"]),
                    (id_catalog::FASTQ_DEDUPLICATE, &["method", "umi_edit_distance"]),
                ],
                &[(id_catalog::FASTQ_UMI, &["barcode_slot_order"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.host_depletion",
            id_catalog::PIPELINE_FASTQ_HOST_DEPLETION,
            "Host-depletion FASTQ template with governed reference preparation, host depletion, and retained-versus-rejected evidence reporting.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::CORE_PREPARE_REFERENCE.to_string(),
                id_catalog::FASTQ_DEPLETE_HOST.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![id_catalog::CORE_PREPARE_REFERENCE.to_string()],
            fan_rules_from_reference(id_catalog::CORE_PREPARE_REFERENCE, id_catalog::FASTQ_DEPLETE_HOST),
            &["read_admission", "depletion_balance", "qc_summary"],
            &["reference_identity", "off_target_depletion_risk"],
            parameter_policy(
                &[
                    (id_catalog::FASTQ_DEPLETE_HOST, &["min_match_identity", "max_multimap"]),
                    (id_catalog::FASTQ_FILTER, &["minimum_quality"]),
                ],
                &[(id_catalog::CORE_PREPARE_REFERENCE, &["reference_build"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.rrna_depletion",
            id_catalog::PIPELINE_FASTQ_RRNA_DEPLETION,
            "rRNA-depletion FASTQ template with reference DB preparation, depletion, and DB identity caveats in reports.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::CORE_PREPARE_REFERENCE.to_string(),
                id_catalog::FASTQ_DEPLETE_RRNA.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![id_catalog::CORE_PREPARE_REFERENCE.to_string()],
            fan_rules_from_reference(id_catalog::CORE_PREPARE_REFERENCE, id_catalog::FASTQ_DEPLETE_RRNA),
            &["read_admission", "depletion_balance", "qc_summary"],
            &["rrna_database_identity", "cross_species_hit_risk"],
            parameter_policy(
                &[(id_catalog::FASTQ_DEPLETE_RRNA, &["seed_length", "max_edit_distance"])],
                &[(id_catalog::CORE_PREPARE_REFERENCE, &["rrna_db_release"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.contaminant_depletion",
            id_catalog::PIPELINE_FASTQ_CONTAMINANT_DEPLETION,
            "Contaminant-depletion FASTQ template with governed contaminant DB preparation and caveated depletion reports.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::CORE_PREPARE_REFERENCE.to_string(),
                id_catalog::FASTQ_DEPLETE_REFERENCE_CONTAMINANTS.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![id_catalog::CORE_PREPARE_REFERENCE.to_string()],
            fan_rules_from_reference(
                id_catalog::CORE_PREPARE_REFERENCE,
                id_catalog::FASTQ_DEPLETE_REFERENCE_CONTAMINANTS,
            ),
            &["read_admission", "depletion_balance", "qc_summary"],
            &["contaminant_db_scope", "false_positive_depletion"],
            parameter_policy(
                &[(
                    id_catalog::FASTQ_DEPLETE_REFERENCE_CONTAMINANTS,
                    &["minimum_match_length", "max_mismatch_rate"],
                )],
                &[(id_catalog::CORE_PREPARE_REFERENCE, &["contaminant_catalog_hash"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.edna_metabarcoding",
            id_catalog::PIPELINE_FASTQ_EDNA_METABARCODING,
            "eDNA/metabarcoding FASTQ template with primer handling, chimera control, OTU/ASV branching, and taxonomy screening caveats.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_NORMALIZE_PRIMERS.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::FASTQ_REMOVE_CHIMERAS.to_string(),
                id_catalog::FASTQ_CLUSTER_OTUS.to_string(),
                id_catalog::FASTQ_INFER_ASVS.to_string(),
                id_catalog::FASTQ_SCREEN.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![],
            vec![],
            &["read_admission", "amplicon_cleaning", "taxonomy_screening", "qc_summary"],
            &["primer_bank_selection", "taxonomy_db_limits"],
            parameter_policy(
                &[
                    (id_catalog::FASTQ_NORMALIZE_PRIMERS, &["primer_bank_id", "mismatch_budget"]),
                    (id_catalog::FASTQ_CLUSTER_OTUS, &["identity_threshold"]),
                    (id_catalog::FASTQ_INFER_ASVS, &["denoise_method"]),
                    (id_catalog::FASTQ_SCREEN, &["taxonomy_rank", "confidence_floor"]),
                ],
                &[(id_catalog::FASTQ_NORMALIZE_PRIMERS, &["primer_version"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.ancient_dna_preprocessing",
            id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA,
            "Ancient-DNA FASTQ preprocessing template with merge, damage-aware trimming, low-complexity control, and low-input caveats.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_MERGE.to_string(),
                id_catalog::FASTQ_TRIM_TERMINAL_DAMAGE.to_string(),
                id_catalog::FASTQ_LOW_COMPLEXITY.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![],
            vec![],
            &["read_admission", "damage_aware_preprocessing", "qc_summary"],
            &["low_input_regime", "post_mortem_damage_bias"],
            parameter_policy(
                &[
                    (
                        id_catalog::FASTQ_TRIM_TERMINAL_DAMAGE,
                        &["five_prime_trim", "three_prime_trim"],
                    ),
                    (id_catalog::FASTQ_MERGE, &["minimum_overlap", "max_mismatch_rate"]),
                ],
                &[(id_catalog::FASTQ_TRIM_TERMINAL_DAMAGE, &["udg_assumption"])],
            ),
            &["fastq_reference_adna_review"],
        ),
        template(
            "fastq.adapter_primer_bank_review",
            id_catalog::PIPELINE_FASTQ_AMPLICON_STANDARD,
            "Adapter/primer bank review workflow that compares bank selection, assay applicability, and version-to-version changes.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
                id_catalog::FASTQ_NORMALIZE_PRIMERS.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![],
            vec![],
            &["read_admission", "bank_comparison", "assay_applicability", "qc_summary"],
            &["bank_version_drift", "assay_coverage_gap"],
            parameter_policy(
                &[
                    (id_catalog::FASTQ_DETECT_ADAPTERS, &["seed_mismatches"]),
                    (id_catalog::FASTQ_NORMALIZE_PRIMERS, &["primer_bank_id", "assay_id"]),
                ],
                &[(id_catalog::FASTQ_NORMALIZE_PRIMERS, &["primer_version"])],
            ),
            &["fastq_essential_qc"],
        ),
        template(
            "fastq.preprocessing_policy_diff",
            id_catalog::PIPELINE_FASTQ_TRIM_QC,
            "FASTQ preprocessing policy-diff workflow comparing trim/filter/depletion policy deltas against read retention and downstream artifact shifts.",
            vec![
                id_catalog::FASTQ_VALIDATE_READS.to_string(),
                id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
                id_catalog::FASTQ_TRIM.to_string(),
                id_catalog::FASTQ_FILTER.to_string(),
                id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
                id_catalog::FASTQ_QC_POST.to_string(),
            ],
            vec![],
            vec![],
            &["read_admission", "policy_delta", "retention_shift", "qc_summary"],
            &["policy_comparison_context", "downstream_shift_uncertainty"],
            parameter_policy(
                &[
                    (id_catalog::FASTQ_TRIM, &["min_length", "adapter_mode"]),
                    (id_catalog::FASTQ_FILTER, &["minimum_quality", "max_n_rate"]),
                    (id_catalog::FASTQ_STATS_NEUTRAL, &["policy_run_label"]),
                ],
                &[(id_catalog::FASTQ_VALIDATE_READS, &["strictness_level"])],
            ),
            &["fastq_essential_qc"],
        ),
    ]
}

#[must_use]
pub fn fastq_workflow_template_by_id(template_id: &str) -> Option<CrossWorkflowTemplateV1> {
    fastq_workflow_templates().into_iter().find(|template| template.template_id == template_id)
}

#[must_use]
pub fn fastq_workflow_templates_for_pipeline(pipeline_id: &str) -> Vec<CrossWorkflowTemplateV1> {
    fastq_workflow_templates()
        .into_iter()
        .filter(|template| template.pipeline_id == pipeline_id)
        .collect()
}
