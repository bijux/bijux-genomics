use bijux_core::primitives::invariants::{InvariantSpecV1, InvariantStatusV1};

#[must_use]
pub fn bam_invariant_specs() -> Vec<InvariantSpecV1> {
    vec![
        InvariantSpecV1 {
            id: "insufficient_data".to_string(),
            definition: "Input data meets minimum coverage/reads required for analysis."
                .to_string(),
            threshold_provenance:
                "Thresholds in BamInvariantThresholds (defaults, override via env).".to_string(),
            severity: InvariantStatusV1::Fail,
            next_steps: "Collect more data or relax minimum coverage thresholds.".to_string(),
        },
        InvariantSpecV1 {
            id: "coverage_mean".to_string(),
            definition: "Mean coverage exceeds minimum thresholds.".to_string(),
            threshold_provenance:
                "Thresholds in BamInvariantThresholds (defaults, override via env).".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Inspect coverage plots and consider deeper sequencing.".to_string(),
        },
        InvariantSpecV1 {
            id: "duplicate_fraction".to_string(),
            definition: "Duplicate fraction remains within expected bounds.".to_string(),
            threshold_provenance:
                "Thresholds in BamInvariantThresholds (defaults, override via env).".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Review library complexity and deduplication settings.".to_string(),
        },
        InvariantSpecV1 {
            id: "complexity_vs_duplicates".to_string(),
            definition: "Complexity metrics align with duplicate fraction.".to_string(),
            threshold_provenance: "Heuristic check in bam invariants evaluation.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Validate complexity estimates and check duplication artifacts."
                .to_string(),
        },
        InvariantSpecV1 {
            id: "sequencing_saturation".to_string(),
            definition: "Sequencing saturation metrics indicate sufficient complexity.".to_string(),
            threshold_provenance: "Heuristic check in bam invariants evaluation.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Inspect preseq curves and saturation estimates.".to_string(),
        },
        InvariantSpecV1 {
            id: "contamination_rate".to_string(),
            definition: "Contamination estimate within expected range.".to_string(),
            threshold_provenance:
                "Thresholds in BamInvariantThresholds (defaults, override via env).".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Review contamination reports and references.".to_string(),
        },
        InvariantSpecV1 {
            id: "contamination_damage_check".to_string(),
            definition: "Contamination signal consistent with damage profile.".to_string(),
            threshold_provenance: "Heuristic in bam invariants evaluation.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Inspect damage plots and contamination estimates.".to_string(),
        },
        InvariantSpecV1 {
            id: "damage_mapq_correlation".to_string(),
            definition: "Damage signal is not driven solely by low MAPQ reads.".to_string(),
            threshold_provenance: "Heuristic in bam invariants evaluation.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Check MAPQ filters and damage stratification.".to_string(),
        },
        InvariantSpecV1 {
            id: "damage_tool_disagreement".to_string(),
            definition: "Damage estimates across tools are consistent.".to_string(),
            threshold_provenance: "Heuristic in bam invariants evaluation.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Compare damage tools and review inputs.".to_string(),
        },
        InvariantSpecV1 {
            id: "declared_vs_inferred_library".to_string(),
            definition: "Declared library type matches inferred signals.".to_string(),
            threshold_provenance: "Heuristic in bam invariants evaluation.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Confirm library metadata and update run parameters.".to_string(),
        },
        InvariantSpecV1 {
            id: "reference_mismatch".to_string(),
            definition: "Alignment reference matches expected genome.".to_string(),
            threshold_provenance: "Heuristic in bam invariants evaluation.".to_string(),
            severity: InvariantStatusV1::Fail,
            next_steps: "Verify reference genome selection and re-align if needed.".to_string(),
        },
    ]
}
