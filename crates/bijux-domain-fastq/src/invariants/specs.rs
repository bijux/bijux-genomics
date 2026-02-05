use bijux_core::primitives::invariants::{InvariantSpecV1, InvariantStatusV1};

#[must_use]
pub fn fastq_invariant_specs() -> Vec<InvariantSpecV1> {
    vec![
        InvariantSpecV1 {
            id: "effective_params_present".to_string(),
            definition: "Effective params are present and include required fields.".to_string(),
            threshold_provenance:
                "No numeric thresholds; required field contract from stage params schema."
                    .to_string(),
            severity: InvariantStatusV1::Fail,
            next_steps: "Ensure stage planners emit canonical effective params JSON.".to_string(),
        },
        InvariantSpecV1 {
            id: "metrics_parse".to_string(),
            definition: "Stage metrics JSON can be parsed into the expected schema.".to_string(),
            threshold_provenance: "Schema versioned in bijux-core metrics.".to_string(),
            severity: InvariantStatusV1::Fail,
            next_steps: "Verify metrics schema versions and stage output files.".to_string(),
        },
        InvariantSpecV1 {
            id: "validate_malformed_reads".to_string(),
            definition: "Validation reports malformed reads proportion below threshold."
                .to_string(),
            threshold_provenance: "Default thresholds in bijux_domain_fastq::InvariantThresholds."
                .to_string(),
            severity: InvariantStatusV1::Fail,
            next_steps: "Inspect input FASTQ integrity and rerun validation.".to_string(),
        },
        InvariantSpecV1 {
            id: "retention_sanity".to_string(),
            definition: "Read retention remains within expected bounds.".to_string(),
            threshold_provenance:
                "Warn/fail thresholds from InvariantThresholds, override via env.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Review trimming/filter settings and input quality.".to_string(),
        },
        InvariantSpecV1 {
            id: "quality_direction".to_string(),
            definition: "Mean quality delta does not regress beyond thresholds.".to_string(),
            threshold_provenance:
                "Warn/fail thresholds from InvariantThresholds, override via env.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Inspect trimming parameters and quality regression reports.".to_string(),
        },
        InvariantSpecV1 {
            id: "merge_rate_range".to_string(),
            definition: "Merge rate stays within expected [0,1] range.".to_string(),
            threshold_provenance: "Range checks in invariant evaluation.".to_string(),
            severity: InvariantStatusV1::Fail,
            next_steps: "Check merge input pairing and overlap settings.".to_string(),
        },
        InvariantSpecV1 {
            id: "n_rate_sanity".to_string(),
            definition: "N-base rate remains within expected bounds.".to_string(),
            threshold_provenance:
                "Warn/fail thresholds from InvariantThresholds, override via env.".to_string(),
            severity: InvariantStatusV1::Warn,
            next_steps: "Inspect filtering thresholds and input quality.".to_string(),
        },
    ]
}
