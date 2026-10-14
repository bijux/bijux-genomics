//! Owner: pure BAM campaign alignment goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use std::collections::BTreeSet;

const BWA_ALIGNMENT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "modern-wgs-alignment",
        focus: "modern WGS alignment baseline",
    },
    ScenarioDefinition {
        scenario_id: "adna-nonhuman-microbial",
        focus: "aDNA and non-human alignment robustness",
    },
    ScenarioDefinition {
        scenario_id: "reference-size-bands",
        focus: "small/medium reference performance bands",
    },
];

fn bwa_alignment_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("bwa_alignment_rows_present={}", !rows.is_empty()),
        format!("bwa_alignment_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.align")),
        format!(
            "bwa_alignment_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class == "ready").count()
        ),
        format!(
            "bwa_alignment_runtime_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "runtime-performance").count()
        ),
    ]
}

const BOWTIE_ALIGNMENT_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "local-vs-end-to-end",
        focus: "local and end-to-end parameter differences",
    },
    ScenarioDefinition { scenario_id: "short-read-stress", focus: "short-read alignment behavior" },
    ScenarioDefinition {
        scenario_id: "backend-diff-surface",
        focus: "backend equivalence and divergence surface",
    },
];

fn bowtie_alignment_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("bowtie2_rows_present={}", !rows.is_empty()),
        format!(
            "bowtie2_alignment_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.align")
        ),
        format!(
            "bowtie2_backend_candidates={}",
            rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "bowtie2_equivalence_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "backend-equivalence").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G122",
        title: "benchmark BWA-style alignment",
        stage_ids: &["bam.align"],
        scenarios: BWA_ALIGNMENT_SCENARIOS,
        checks: bwa_alignment_checks,
    },
    GoalDefinition {
        goal_id: "G123",
        title: "benchmark Bowtie2-style alignment",
        stage_ids: &["bam.align"],
        scenarios: BOWTIE_ALIGNMENT_SCENARIOS,
        checks: bowtie_alignment_checks,
    },
];
