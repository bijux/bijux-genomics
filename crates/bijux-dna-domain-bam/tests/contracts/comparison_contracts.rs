use std::collections::BTreeSet;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::{
    comparable_benchmark_stage_contracts, comparable_benchmark_stage_ids,
    comparable_tool_ids_for_stage, stage_comparable_metric_contracts_for_stage,
    stage_comparable_metric_fields_for_stage, BamScientificInsufficiencyPolicy,
    BamScientificPassDirection, BamScientificToleranceKind,
};

#[test]
fn multi_tool_bam_comparable_stage_slice_stays_explicit() {
    let stage_ids = comparable_benchmark_stage_ids()
        .into_iter()
        .map(|stage_id| stage_id.to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        stage_ids,
        BTreeSet::from([
            "bam.align".to_string(),
            "bam.authenticity".to_string(),
            "bam.contamination".to_string(),
            "bam.coverage".to_string(),
            "bam.damage".to_string(),
            "bam.duplication_metrics".to_string(),
            "bam.filter".to_string(),
            "bam.kinship".to_string(),
            "bam.length_filter".to_string(),
            "bam.mapping_summary".to_string(),
            "bam.mapq_filter".to_string(),
            "bam.markdup".to_string(),
            "bam.qc_pre".to_string(),
            "bam.sex".to_string(),
            "bam.validate".to_string(),
        ]),
        "the governed BAM multi-tool comparable slice must stay explicit"
    );
}

#[test]
fn multi_tool_bam_comparable_stages_publish_shared_metrics() {
    let contracts = comparable_benchmark_stage_contracts();
    assert_eq!(contracts.len(), 15);
    for contract in &contracts {
        assert!(
            contract.compatible_tool_ids.len() >= 2,
            "multi-tool comparable BAM stage `{}` must admit at least two tools",
            contract.stage_id
        );
        assert!(
            !contract.shared_metrics.is_empty(),
            "multi-tool comparable BAM stage `{}` must publish governed shared metrics",
            contract.stage_id
        );
        assert!(
            contract.shared_metrics.iter().all(|metric| metric.scientific_threshold.is_some()),
            "multi-tool comparable BAM stage `{}` must publish scientific threshold semantics for every shared metric",
            contract.stage_id
        );
    }
}

#[test]
fn bam_comparable_contracts_retain_shared_metric_fields_for_real_comparison_surfaces() {
    assert_eq!(
        comparable_tool_ids_for_stage(&StageId::from_static("bam.coverage")),
        vec![
            ToolId::from_static("bedtools"),
            ToolId::from_static("mosdepth"),
            ToolId::from_static("samtools"),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.validate")),
        vec![
            "validation_status".to_string(),
            "validation_errors".to_string(),
            "validation_warnings".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.mapping_summary")),
        vec![
            "mapping_fraction".to_string(),
            "mapped_reads".to_string(),
            "unmapped_reads".to_string(),
            "secondary_reads".to_string(),
            "supplementary_reads".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.coverage")),
        vec!["mean_depth".to_string(), "breadth_1x".to_string(), "covered_bases".to_string(),]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.authenticity")),
        vec![
            "score".to_string(),
            "confidence".to_string(),
            "status".to_string(),
            "pmd_like_signal_present".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.contamination")),
        vec!["estimate".to_string(), "ci_low".to_string(), "ci_high".to_string(),]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.sex")),
        vec![
            "x_coverage".to_string(),
            "y_coverage".to_string(),
            "autosomal_coverage".to_string(),
            "call".to_string(),
            "confidence".to_string(),
            "status".to_string(),
        ]
    );
    assert_eq!(
        stage_comparable_metric_fields_for_stage(&StageId::from_static("bam.kinship")),
        vec![
            "observed_max_overlap_snps".to_string(),
            "pair_count".to_string(),
            "status".to_string(),
            "pairwise_results".to_string(),
        ]
    );
}

#[test]
fn bam_comparable_metrics_carry_governed_scientific_threshold_contracts() {
    let damage_metrics =
        stage_comparable_metric_contracts_for_stage(&StageId::from_static("bam.damage"));
    assert_eq!(damage_metrics.len(), 3);

    let damage_signal = damage_metrics
        .iter()
        .find(|metric| metric.name == "damage_signal")
        .expect("damage signal metric");
    let threshold = damage_signal.scientific_threshold.as_ref().expect("damage threshold");
    assert_eq!(threshold.pass_direction, BamScientificPassDirection::ExactMatch);
    assert_eq!(threshold.tolerance_kind, BamScientificToleranceKind::ExactMatch);
    assert_eq!(threshold.tolerance_value, 0.0);
    assert_eq!(
        threshold.insufficiency_policy,
        BamScientificInsufficiencyPolicy::WarnAndExcludeStage
    );

    let validation_errors =
        stage_comparable_metric_contracts_for_stage(&StageId::from_static("bam.validate"))
            .into_iter()
            .find(|metric| metric.name == "validation_errors")
            .expect("validation error metric");
    let threshold = validation_errors.scientific_threshold.as_ref().expect("validation threshold");
    assert_eq!(threshold.pass_direction, BamScientificPassDirection::StructuredMatch);
    assert_eq!(threshold.tolerance_kind, BamScientificToleranceKind::NormalizedSetOverlap);
    assert_eq!(threshold.tolerance_value, 1.0);
    assert_eq!(
        threshold.insufficiency_policy,
        BamScientificInsufficiencyPolicy::RefuseStageComparison
    );
}
