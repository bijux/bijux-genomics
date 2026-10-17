//! Owner: pure FASTQ campaign read layout declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

const G102_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "read-id-variants",
        focus: "heterogeneous read identifier formats",
    },
    ScenarioDefinition {
        scenario_id: "lane-run-sample-renaming",
        focus: "lane, run, and sample naming normalization",
    },
    ScenarioDefinition {
        scenario_id: "reversible-normalization",
        focus: "normalization reversibility and round-trip",
    },
];

fn read_name_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("read_name_rows_present={}", !rows.is_empty()),
        format!(
            "normalization_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.normalize_read_names")
        ),
        format!(
            "normalization_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "normalization_backend_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class.contains("readiness")
                        || finding.failure_class.contains("runtime")
                })
                .count()
        ),
    ]
}

const G103_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "broken-pairs",
        focus: "broken mate pairs and orphan recovery",
    },
    ScenarioDefinition {
        scenario_id: "unsorted-mates",
        focus: "unsorted mate alignment and repair order",
    },
    ScenarioDefinition {
        scenario_id: "reject-reporting",
        focus: "rejected output semantics and runtime scale",
    },
];

fn pair_repair_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("pair_repair_rows_present={}", !rows.is_empty()),
        format!(
            "pair_repair_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.repair_pairs")
        ),
        format!(
            "pair_repair_orphan_signals={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class.contains("runtime")
                        || finding.failure_class.contains("readiness")
                })
                .count()
        ),
        format!(
            "pair_repair_queue_pressure={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

const G104_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "pair-sync-stability",
        focus: "pair synchronization and order stability",
    },
    ScenarioDefinition {
        scenario_id: "streaming-memory",
        focus: "streaming memory profile under interleave flow",
    },
    ScenarioDefinition {
        scenario_id: "roundtrip-equivalence",
        focus: "interleave/deinterleave round-trip equivalence",
    },
];

fn interleave_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("interleave_rows_present={}", !rows.is_empty()),
        format!(
            "interleave_stage_pair_covered={}",
            rows.iter().any(|row| row.stage_id == "fastq.interleave_reads")
                && rows.iter().any(|row| row.stage_id == "fastq.deinterleave_reads")
        ),
        format!(
            "interleave_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!(
            "interleave_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.failure_class.contains("readiness")).count()
        ),
    ]
}

const G105_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "multi-lane-concat",
        focus: "multi-lane concatenation with stable ordering",
    },
    ScenarioDefinition {
        scenario_id: "duplicate-run-detection",
        focus: "duplicate run and lane conflict detection",
    },
    ScenarioDefinition {
        scenario_id: "sample-conflict-guard",
        focus: "sample identity conflict refusal",
    },
];

fn lane_concat_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, queue_entries, .. } = inputs;
    vec![
        format!("lane_concat_rows_present={}", !rows.is_empty()),
        format!(
            "lane_concat_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.concatenate_lanes")
        ),
        format!(
            "lane_concat_tool_diversity={}",
            rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "lane_concat_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

const G106_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "barcode-mismatch-policy",
        focus: "barcode mismatch threshold behavior",
    },
    ScenarioDefinition { scenario_id: "index-collision", focus: "index collision resolution" },
    ScenarioDefinition {
        scenario_id: "undetermined-and-sheet-errors",
        focus: "undetermined reads and sample sheet validation",
    },
];

fn demultiplex_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, queue_entries, .. } = inputs;
    vec![
        format!("demux_rows_present={}", !rows.is_empty()),
        format!(
            "demux_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.demultiplex_reads")
        ),
        format!(
            "demux_refusal_pressure={}",
            findings.iter().filter(|finding| finding.severity == "critical").count()
        ),
        format!(
            "demux_queue_entries={}",
            queue_entries
                .iter()
                .filter(|entry| entry.failure_class.contains("readiness")
                    || entry.failure_class.contains("runtime"))
                .count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G102",
        title: "benchmark FASTQ read-name normalization",
        stage_ids: &["fastq.normalize_read_names"],
        scenarios: G102_SCENARIOS,
        checks: read_name_checks,
    },
    GoalDefinition {
        goal_id: "G103",
        title: "benchmark FASTQ pair repair",
        stage_ids: &["fastq.repair_pairs"],
        scenarios: G103_SCENARIOS,
        checks: pair_repair_checks,
    },
    GoalDefinition {
        goal_id: "G104",
        title: "benchmark FASTQ interleave/deinterleave",
        stage_ids: &["fastq.interleave_reads", "fastq.deinterleave_reads"],
        scenarios: G104_SCENARIOS,
        checks: interleave_checks,
    },
    GoalDefinition {
        goal_id: "G105",
        title: "benchmark FASTQ lane concatenation",
        stage_ids: &["fastq.concatenate_lanes"],
        scenarios: G105_SCENARIOS,
        checks: lane_concat_checks,
    },
    GoalDefinition {
        goal_id: "G106",
        title: "benchmark FASTQ demultiplexing",
        stage_ids: &["fastq.demultiplex_reads"],
        scenarios: G106_SCENARIOS,
        checks: demultiplex_checks,
    },
];
