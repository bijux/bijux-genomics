#![allow(clippy::map_unwrap_or, clippy::unnecessary_wraps)]

use anyhow::Result;
use bijux_core::InvariantStatusV1;
use bijux_domain_bam::invariants::{bam_invariant_specs, BamInvariantThresholds};
use bijux_domain_bam::metrics::{
    evaluate_bam_invariants, AuthenticityScoreV1, BamMetricsV1, DamageComparisonV1,
    LibraryTypeInferenceV1,
};
use bijux_domain_bam::types::LibraryType;

fn base_metrics() -> BamMetricsV1 {
    let mut metrics = BamMetricsV1::empty();
    metrics.alignment.total = 100;
    metrics.alignment.duplicates = 0;
    metrics.coverage.mean = 1.0;
    metrics.contamination.estimate = 0.0;
    metrics.complexity.observed_reads = 2_000_000;
    metrics.complexity.saturation_estimate = 0.0;
    metrics.damage.c_to_t_5p = 0.1;
    metrics.damage.g_to_a_3p = 0.1;
    metrics.mapq.mean = 30.0;
    metrics
}

fn fixture_for_invariant(id: &str) -> (String, BamMetricsV1) {
    let mut metrics = base_metrics();
    let stage_id = match id {
        "reference_mismatch" => {
            metrics.idxstats.reference_mismatch = true;
            "bam.align"
        }
        "insufficient_data" => {
            metrics.sex_sufficiency.sufficient = false;
            "bam.sex"
        }
        "contamination_rate" => {
            metrics.contamination.estimate = 0.2;
            "bam.contamination"
        }
        "coverage_mean" => {
            metrics.coverage.mean = 0.1;
            "bam.coverage"
        }
        "duplicate_fraction" => {
            metrics.alignment.total = 100;
            metrics.alignment.duplicates = 80;
            "bam.markdup"
        }
        "complexity_vs_duplicates" => {
            metrics.alignment.total = 100;
            metrics.alignment.duplicates = 80;
            metrics.complexity.observed_reads = 10;
            "bam.complexity"
        }
        "sequencing_saturation" => {
            metrics.complexity.saturation_estimate = 0.9;
            "bam.complexity"
        }
        "contamination_damage_check" => {
            metrics.contamination.estimate = 0.2;
            metrics.damage.c_to_t_5p = 0.01;
            metrics.damage.g_to_a_3p = 0.01;
            "bam.contamination"
        }
        "damage_mapq_correlation" => {
            metrics.damage.c_to_t_5p = 0.01;
            metrics.damage.g_to_a_3p = 0.01;
            metrics.mapq.mean = 50.0;
            "bam.damage"
        }
        "damage_tool_disagreement" => {
            metrics.damage_comparison = Some(DamageComparisonV1 {
                tool_a: "tool-a".to_string(),
                tool_b: "tool-b".to_string(),
                c_to_t_diff: 0.2,
                g_to_a_diff: 0.2,
                exceeds_threshold: true,
            });
            "bam.damage"
        }
        "declared_vs_inferred_library" => {
            metrics.authenticity = AuthenticityScoreV1 {
                library_type_inference: Some(LibraryTypeInferenceV1 {
                    inferred: LibraryType::NonUdg,
                    confidence: 0.9,
                    rationale: "fixture".to_string(),
                    declared: Some(LibraryType::Udg),
                }),
                ..AuthenticityScoreV1::empty()
            };
            "bam.authenticity"
        }
        _ => panic!("missing fixture for invariant {id}"),
    };
    (stage_id.to_string(), metrics)
}

#[test]
fn bam_invariants_have_specs_and_fixtures() -> Result<()> {
    let specs = bam_invariant_specs();
    let thresholds = BamInvariantThresholds::default();
    let mut ids = std::collections::BTreeSet::new();
    for spec in &specs {
        assert!(
            ids.insert(spec.id.clone()),
            "duplicate invariant {}",
            spec.id
        );
        assert!(!spec.definition.trim().is_empty());
        assert!(!spec.threshold_provenance.trim().is_empty());
        assert!(!spec.next_steps.trim().is_empty());
        let (stage_id, metrics) = fixture_for_invariant(&spec.id);
        let eval = evaluate_bam_invariants(&stage_id, &metrics, &thresholds);
        let status = eval
            .results
            .iter()
            .find(|entry| entry.id == spec.id)
            .map(|entry| entry.status.clone())
            .unwrap_or_else(|| panic!("missing invariant {} for {stage_id}", spec.id));
        assert!(
            status != InvariantStatusV1::Pass,
            "fixture did not trigger {}",
            spec.id
        );
    }
    Ok(())
}
