//! Owner: pure FASTQ campaign evidence declarations and checks.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog::{
    FASTQ_DEPLETE_HOST, FASTQ_FILTER, FASTQ_NORMALIZE_PRIMERS,
    FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES, FASTQ_PROFILE_READ_LENGTHS, FASTQ_STATS_NEUTRAL,
    FASTQ_TRIM, FASTQ_TRIM_TERMINAL_DAMAGE, FASTQ_UMI, FASTQ_VALIDATE_READS,
};

const G117_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "chunk-equivalence",
        focus: "chunked vs unchunked output equivalence",
    },
    ScenarioDefinition {
        scenario_id: "chunk-memory-io-profile",
        focus: "chunked processing memory and IO profile",
    },
    ScenarioDefinition {
        scenario_id: "chunked-qc-aggregate",
        focus: "chunked aggregate QC stability",
    },
];

fn chunked_processing_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("chunked_rows_present={}", !rows.is_empty()),
        format!("chunk_filter_stage_bound={}", rows.iter().any(|row| row.stage_id == FASTQ_FILTER)),
        format!(
            "chunk_profile_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_STATS_NEUTRAL)
        ),
        format!(
            "chunk_equivalence_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "backend-equivalence").count()
        ),
    ]
}

const G118_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "qc-count-and-quality",
        focus: "read count and quality profile consistency",
    },
    ScenarioDefinition {
        scenario_id: "qc-length-and-overrep",
        focus: "length and overrepresented sequence profiling",
    },
    ScenarioDefinition {
        scenario_id: "qc-report-stability",
        focus: "QC manifest output stability over repeats",
    },
];

fn qc_manifest_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("qc_manifest_rows_present={}", !rows.is_empty()),
        format!(
            "qc_manifest_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.materialize_qc_manifest")
        ),
        format!(
            "qc_profile_stages_covered={}",
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.stage_id.as_str(),
                        FASTQ_PROFILE_READ_LENGTHS
                            | FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES
                            | "fastq.materialize_qc_manifest"
                    )
                })
                .count()
        ),
        format!(
            "qc_stability_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "reproducibility").count()
        ),
    ]
}

const G119_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "provenance-capture-overhead",
        focus: "provenance snapshot runtime overhead",
    },
    ScenarioDefinition {
        scenario_id: "provenance-tool-db-image-delta",
        focus: "provenance deltas across tool/db/image changes",
    },
    ScenarioDefinition {
        scenario_id: "provenance-replay-utility",
        focus: "replay and audit utility of provenance outputs",
    },
];

fn provenance_capture_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("provenance_rows_present={}", !rows.is_empty()),
        format!(
            "provenance_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.capture_provenance_snapshot")
        ),
        format!(
            "provenance_code_findings={}",
            findings.iter().filter(|finding| finding.result_scope == "encrypted-code").count()
        ),
        format!(
            "provenance_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.failure_class.contains("code-freeze"))
                .count()
        ),
    ]
}

const G120_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "qc-only-template",
        focus: "QC-only template benchmark behavior",
    },
    ScenarioDefinition {
        scenario_id: "trim-umi-depletion-template",
        focus: "trim, UMI, and depletion workflow templates",
    },
    ScenarioDefinition {
        scenario_id: "adna-edna-policy-diff-template",
        focus: "aDNA, eDNA, and policy-diff template comparison",
    },
];

fn template_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("template_rows_present={}", !rows.is_empty()),
        format!(
            "template_core_stages_covered={}",
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.stage_id.as_str(),
                        FASTQ_VALIDATE_READS
                            | FASTQ_TRIM
                            | FASTQ_UMI
                            | FASTQ_DEPLETE_HOST
                            | FASTQ_TRIM_TERMINAL_DAMAGE
                            | FASTQ_NORMALIZE_PRIMERS
                    )
                })
                .count()
        ),
        format!(
            "template_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "template_hardening_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G117",
        title: "benchmark chunked FASTQ processing",
        stage_ids: &[FASTQ_FILTER, FASTQ_STATS_NEUTRAL],
        scenarios: G117_SCENARIOS,
        checks: chunked_processing_checks,
    },
    GoalDefinition {
        goal_id: "G118",
        title: "benchmark FASTQ QC manifest generation",
        stage_ids: &[
            "fastq.materialize_qc_manifest",
            FASTQ_PROFILE_READ_LENGTHS,
            FASTQ_PROFILE_OVERREPRESENTED_SEQUENCES,
        ],
        scenarios: G118_SCENARIOS,
        checks: qc_manifest_checks,
    },
    GoalDefinition {
        goal_id: "G119",
        title: "benchmark FASTQ provenance capture",
        stage_ids: &["fastq.capture_provenance_snapshot"],
        scenarios: G119_SCENARIOS,
        checks: provenance_capture_checks,
    },
    GoalDefinition {
        goal_id: "G120",
        title: "benchmark full FASTQ templates",
        stage_ids: &[
            FASTQ_VALIDATE_READS,
            FASTQ_TRIM,
            FASTQ_UMI,
            FASTQ_DEPLETE_HOST,
            FASTQ_TRIM_TERMINAL_DAMAGE,
            FASTQ_NORMALIZE_PRIMERS,
        ],
        scenarios: G120_SCENARIOS,
        checks: template_checks,
    },
];
