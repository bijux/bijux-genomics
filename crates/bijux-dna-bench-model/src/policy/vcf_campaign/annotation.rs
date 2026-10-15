//! Owner: pure VCF campaign annotation declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G156_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "annotation-source-version",
        focus: "annotation source and version provenance",
    },
    ScenarioDefinition {
        scenario_id: "field-coverage-drift",
        focus: "annotation field coverage and drift",
    },
    ScenarioDefinition {
        scenario_id: "gene-transcript-mapping",
        focus: "gene/transcript mapping stability",
    },
];

fn annotation_provenance_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("annotation_rows_present={}", !rows.is_empty()),
        format!(
            "annotation_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "annotation_postprocess_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.postprocess")
        ),
        format!(
            "annotation_filter_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.filter")
        ),
        format!(
            "annotation_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.appraiser_id == "artifact-validity"
                        || finding.appraiser_id == "scientific-output"
                })
                .count()
        ),
    ]
}

const G157_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sv-like-record-detection",
        focus: "SV-like record detection and classification",
    },
    ScenarioDefinition {
        scenario_id: "small-variant-misuse-refusal",
        focus: "refusal when small-variant workflow is unsuitable",
    },
    ScenarioDefinition {
        scenario_id: "support-vs-refuse-boundary",
        focus: "support versus refusal boundary for SV-like inputs",
    },
];

fn structural_variant_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("sv_boundary_rows_present={}", !rows.is_empty()),
        format!(
            "sv_boundary_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "sv_boundary_call_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "vcf.call")
        ),
        format!("sv_boundary_qc_stage_bound={}", rows.iter().any(|row| row.stage_id == "vcf.qc")),
        format!(
            "sv_boundary_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G156",
        title: "benchmark annotation provenance",
        stage_ids: &["vcf.postprocess", "vcf.filter", "vcf.stats"],
        scenarios: G156_SCENARIOS,
        checks: annotation_provenance_checks,
    },
    GoalDefinition {
        goal_id: "G157",
        title: "benchmark structural-variant boundary",
        stage_ids: &["vcf.call", "vcf.filter", "vcf.qc"],
        scenarios: G157_SCENARIOS,
        checks: structural_variant_checks,
    },
];
