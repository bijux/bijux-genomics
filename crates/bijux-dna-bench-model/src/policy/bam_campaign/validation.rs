//! Owner: pure BAM campaign validation goal declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};

const VALIDATION_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "headers-and-indexes",
        focus: "header/index boundary validation",
    },
    ScenarioDefinition {
        scenario_id: "read-group-and-sort-state",
        focus: "read group and sort-state validation",
    },
    ScenarioDefinition {
        scenario_id: "cram-reference-preconditions",
        focus: "CRAM reference compatibility requirements",
    },
];

fn validation_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("validation_rows_present={}", !rows.is_empty()),
        format!("validation_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.validate")),
        format!(
            "validation_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "validation_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
        format!("validation_findings={}", findings.len()),
    ]
}

const SORTING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "coordinate-name-sort-boundary",
        focus: "coordinate/name sort compatibility",
    },
    ScenarioDefinition {
        scenario_id: "bai-csi-compatibility",
        focus: "BAI/CSI index compatibility",
    },
    ScenarioDefinition {
        scenario_id: "dictionary-corruption-handling",
        focus: "dictionary mismatch and corrupted index handling",
    },
];

fn sorting_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, .. } = inputs;
    vec![
        format!("sort_index_rows_present={}", !rows.is_empty()),
        format!(
            "sort_index_validate_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.validate")
        ),
        format!(
            "sort_index_mapping_summary_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
        ),
        format!(
            "sort_index_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
    ]
}

const READ_GROUPS_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "lane-library-sample-propagation",
        focus: "lane/library/sample propagation",
    },
    ScenarioDefinition {
        scenario_id: "run-subject-cohort-identity",
        focus: "run/subject/cohort identity propagation",
    },
    ScenarioDefinition {
        scenario_id: "identity-caveat-surface",
        focus: "identity caveat propagation and refusal",
    },
];

fn read_groups_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("read_group_rows_present={}", !rows.is_empty()),
        format!(
            "read_group_validate_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.validate")
        ),
        format!(
            "read_group_summary_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
        ),
        format!(
            "read_group_identity_findings={}",
            findings.iter().filter(|finding| finding.failure_class.contains("readiness")).count()
        ),
    ]
}

const MERGING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition { scenario_id: "compatible-merge-path", focus: "compatible BAM merge path" },
    ScenarioDefinition {
        scenario_id: "reference-and-rg-conflicts",
        focus: "reference and read-group conflict handling",
    },
    ScenarioDefinition {
        scenario_id: "platform-and-metadata-conflicts",
        focus: "platform and duplicate metadata conflicts",
    },
];

fn merging_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("bam_merge_rows_present={}", !rows.is_empty()),
        format!(
            "bam_merge_validate_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.validate")
        ),
        format!(
            "bam_merge_qc_pre_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "bam.qc_pre")
        ),
        format!(
            "bam_merge_conflict_findings={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G121",
        title: "benchmark BAM/SAM/CRAM validation boundaries",
        stage_ids: &["bam.validate"],
        scenarios: VALIDATION_SCENARIOS,
        checks: validation_checks,
    },
    GoalDefinition {
        goal_id: "G124",
        title: "benchmark BAM sort and index",
        stage_ids: &["bam.validate", "bam.mapping_summary"],
        scenarios: SORTING_SCENARIOS,
        checks: sorting_checks,
    },
    GoalDefinition {
        goal_id: "G125",
        title: "benchmark read-group propagation",
        stage_ids: &["bam.validate", "bam.mapping_summary"],
        scenarios: READ_GROUPS_SCENARIOS,
        checks: read_groups_checks,
    },
    GoalDefinition {
        goal_id: "G126",
        title: "benchmark BAM merge",
        stage_ids: &["bam.validate", "bam.qc_pre"],
        scenarios: MERGING_SCENARIOS,
        checks: merging_checks,
    },
];
