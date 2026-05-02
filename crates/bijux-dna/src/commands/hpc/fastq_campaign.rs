use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{BenchmarkMatrixArgs, FastqBenchmarkCampaignArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal, AppraisalFinding,
    BenchmarkMatrixReport, HardeningQueueEntry,
};

const FASTQ_CAMPAIGN_SCHEMA_VERSION: &str = "bijux.hpc.fastq_benchmark_campaign.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastqBenchmarkCampaignReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: FastqBenchmarkCampaignSummary,
    pub entries: Vec<FastqBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastqBenchmarkCampaignSummary {
    pub total_goals: usize,
    pub total_rows: usize,
    pub total_findings: usize,
    pub total_queue_entries: usize,
    pub status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastqBenchmarkGoalEntry {
    pub goal_id: String,
    pub title: String,
    pub stage_ids: Vec<String>,
    pub scenarios: Vec<FastqScenario>,
    pub matched_rows: Vec<FastqMatrixRowRef>,
    pub appraisal_findings: Vec<AppraisalFinding>,
    pub hardening_entries: Vec<HardeningQueueEntry>,
    pub goal_checks: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastqScenario {
    pub scenario_id: String,
    pub focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastqMatrixRowRef {
    pub row_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub readiness_class: String,
    pub repetitions: u32,
}

struct GoalDefinition {
    goal_id: &'static str,
    title: &'static str,
    stage_ids: &'static [&'static str],
    scenarios: &'static [ScenarioDefinition],
}

struct ScenarioDefinition {
    scenario_id: &'static str,
    focus: &'static str,
}

const G101_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "malformed-fastq",
        focus: "malformed records and parse refusal",
    },
    ScenarioDefinition {
        scenario_id: "compressed-layouts",
        focus: "gzip and mixed compression handling",
    },
    ScenarioDefinition {
        scenario_id: "truncated-stream",
        focus: "truncated file detection",
    },
    ScenarioDefinition {
        scenario_id: "paired-interleaved-mixed",
        focus: "paired and interleaved layout validation",
    },
    ScenarioDefinition {
        scenario_id: "large-fastq",
        focus: "large input scale for validation throughput",
    },
];

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

const G106_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "barcode-mismatch-policy",
        focus: "barcode mismatch threshold behavior",
    },
    ScenarioDefinition {
        scenario_id: "index-collision",
        focus: "index collision resolution",
    },
    ScenarioDefinition {
        scenario_id: "undetermined-and-sheet-errors",
        focus: "undetermined reads and sample sheet validation",
    },
];

const G107_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "count-mode",
        focus: "count-based subsampling correctness",
    },
    ScenarioDefinition {
        scenario_id: "fraction-mode",
        focus: "fraction-based subsampling consistency",
    },
    ScenarioDefinition {
        scenario_id: "seeded-pair-preservation",
        focus: "seed reproducibility with pair preservation",
    },
];

const G108_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "adapter-rich",
        focus: "adapter-rich corpus detection sensitivity",
    },
    ScenarioDefinition {
        scenario_id: "adapter-free",
        focus: "adapter-free corpus specificity",
    },
    ScenarioDefinition {
        scenario_id: "backend-equivalence-window",
        focus: "cross-backend adapter detection equivalence",
    },
];

const G109_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "quality-minlen-policy",
        focus: "quality and min-length trimming policy",
    },
    ScenarioDefinition {
        scenario_id: "pair-retention",
        focus: "paired-end retention semantics",
    },
    ScenarioDefinition {
        scenario_id: "adna-damage-policy",
        focus: "aDNA-aware trimming policy",
    },
];

const G110_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "umi-patterns",
        focus: "UMI pattern extraction coverage",
    },
    ScenarioDefinition {
        scenario_id: "failed-extraction",
        focus: "failed UMI extraction reporting",
    },
    ScenarioDefinition {
        scenario_id: "grouping-dedup-handoff",
        focus: "grouping and deduplication handoff",
    },
];

const FASTQ_GOALS_ITERATION_08: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G101",
        title: "benchmark FASTQ validation",
        stage_ids: &["fastq.validate_reads"],
        scenarios: G101_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G102",
        title: "benchmark FASTQ read-name normalization",
        stage_ids: &["fastq.normalize_read_names"],
        scenarios: G102_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G103",
        title: "benchmark FASTQ pair repair",
        stage_ids: &["fastq.repair_pairs"],
        scenarios: G103_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G104",
        title: "benchmark FASTQ interleave/deinterleave",
        stage_ids: &["fastq.interleave_reads", "fastq.deinterleave_reads"],
        scenarios: G104_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G105",
        title: "benchmark FASTQ lane concatenation",
        stage_ids: &["fastq.concatenate_lanes"],
        scenarios: G105_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G106",
        title: "benchmark FASTQ demultiplexing",
        stage_ids: &["fastq.demultiplex_reads"],
        scenarios: G106_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G107",
        title: "benchmark FASTQ subsampling",
        stage_ids: &["fastq.subsample_reads"],
        scenarios: G107_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G108",
        title: "benchmark adapter detection",
        stage_ids: &["fastq.detect_adapters"],
        scenarios: G108_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G109",
        title: "benchmark FASTQ trimming",
        stage_ids: &["fastq.trim_reads"],
        scenarios: G109_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G110",
        title: "benchmark UMI extraction and grouping",
        stage_ids: &["fastq.extract_umis", "fastq.remove_duplicates"],
        scenarios: G110_SCENARIOS,
    },
];

fn write_json_pretty(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)?;
    bijux_dna_api::v1::api::run::atomic_write_bytes(path, &payload)?;
    Ok(())
}

fn selected_goal_ids(raw: Option<&str>) -> Result<Vec<String>> {
    let available = FASTQ_GOALS_ITERATION_08
        .iter()
        .map(|goal| goal.goal_id.to_string())
        .collect::<BTreeSet<_>>();
    let mut selected = if let Some(value) = raw {
        value
            .split(',')
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(|token| token.to_ascii_uppercase())
            .collect::<BTreeSet<_>>()
    } else {
        available.clone()
    };
    if selected.is_empty() {
        return Err(anyhow!("fastq-benchmark-campaign requires at least one goal"));
    }
    let unknown = selected
        .iter()
        .filter(|goal| !available.contains(*goal))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!(
            "unknown fastq goals requested: {}",
            unknown.join(",")
        ));
    }
    let mut ordered = FASTQ_GOALS_ITERATION_08
        .iter()
        .filter_map(|goal| {
            if selected.remove(goal.goal_id) {
                Some(goal.goal_id.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    ordered.sort();
    Ok(ordered)
}

fn status_for_goal(rows: &[FastqMatrixRowRef], queue_entries: &[HardeningQueueEntry]) -> String {
    if rows.is_empty() {
        return "missing-stage-binding".to_string();
    }
    if rows.iter().any(|row| row.readiness_class == "refuse") {
        return "requires-hardening".to_string();
    }
    if !queue_entries.is_empty() {
        return "requires-hardening".to_string();
    }
    "ready-for-benchmark-run".to_string()
}

fn goal_specific_checks(
    goal_id: &str,
    rows: &[FastqMatrixRowRef],
    findings: &[AppraisalFinding],
    queue_entries: &[HardeningQueueEntry],
) -> Vec<String> {
    match goal_id {
        "G101" => vec![
            format!("validation_rows_present={}", !rows.is_empty()),
            format!(
                "layout_diversity_detected={}",
                rows.iter()
                    .any(|row| row.stage_id == "fastq.validate_reads" && row.tool_id != "<unbound>")
            ),
            format!(
                "validation_failure_classes={}",
                findings
                    .iter()
                    .map(|finding| finding.failure_class.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "validation_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("runtime") || entry.failure_class.contains("readiness"))
                    .count()
            ),
        ],
        "G102" => vec![
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
        ],
        "G103" => vec![
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
        ],
        "G104" => vec![
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
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness"))
                    .count()
            ),
        ],
        "G105" => vec![
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
        ],
        "G106" => vec![
            format!("demux_rows_present={}", !rows.is_empty()),
            format!(
                "demux_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.demultiplex_reads")
            ),
            format!(
                "demux_refusal_pressure={}",
                findings
                    .iter()
                    .filter(|finding| finding.severity == "critical")
                    .count()
            ),
            format!(
                "demux_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness") || entry.failure_class.contains("runtime"))
                    .count()
            ),
        ],
        "G107" => vec![
            format!("subsample_rows_present={}", !rows.is_empty()),
            format!(
                "subsample_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.subsample_reads")
            ),
            format!(
                "subsample_repetition_floor={}",
                rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
            ),
            format!(
                "subsample_repro_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "reproducibility")
                    .count()
            ),
        ],
        "G108" => vec![
            format!("adapter_rows_present={}", !rows.is_empty()),
            format!(
                "adapter_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.detect_adapters")
            ),
            format!(
                "adapter_backend_candidates={}",
                rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "adapter_equivalence_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "backend-equivalence")
                    .count()
            ),
        ],
        "G109" => vec![
            format!("trim_rows_present={}", !rows.is_empty()),
            format!(
                "trim_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
            ),
            format!(
                "trim_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "trim_backend_findings={}",
                findings
                    .iter()
                    .filter(|finding| {
                        finding.appraiser_id == "backend-equivalence"
                            || finding.appraiser_id == "scientific-output"
                    })
                    .count()
            ),
        ],
        "G110" => vec![
            format!("umi_rows_present={}", !rows.is_empty()),
            format!(
                "umi_extraction_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.extract_umis")
            ),
            format!(
                "umi_grouping_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.remove_duplicates")
            ),
            format!(
                "umi_code_freeze_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "code-freeze")
                    .count()
            ),
        ],
        _ => Vec::new(),
    }
}

fn build_goal_entries(
    selected_goal_ids: &[String],
    matrix: &BenchmarkMatrixReport,
    findings: &[AppraisalFinding],
    queue: &[HardeningQueueEntry],
) -> Vec<FastqBenchmarkGoalEntry> {
    let selected = selected_goal_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut entries = Vec::new();

    for goal in FASTQ_GOALS_ITERATION_08.iter().filter(|goal| selected.contains(goal.goal_id)) {
        let stage_set = goal.stage_ids.iter().copied().collect::<BTreeSet<_>>();
        let matched_rows = matrix
            .rows
            .iter()
            .filter(|row| stage_set.contains(row.stage_id.as_str()))
            .map(|row| FastqMatrixRowRef {
                row_id: row.row_id.clone(),
                stage_id: row.stage_id.clone(),
                tool_id: row.tool_id.clone(),
                readiness_class: row.readiness.class.clone(),
                repetitions: row.repetitions,
            })
            .collect::<Vec<_>>();

        let row_ids = matched_rows
            .iter()
            .map(|row| row.row_id.clone())
            .collect::<BTreeSet<_>>();

        let appraisal_findings = findings
            .iter()
            .filter(|finding| row_ids.contains(&finding.row_id))
            .cloned()
            .collect::<Vec<_>>();

        let hardening_entries = queue
            .iter()
            .filter(|entry| entry.affected_rows.iter().any(|row| row_ids.contains(row)))
            .cloned()
            .collect::<Vec<_>>();

        let status = status_for_goal(&matched_rows, &hardening_entries);
        let goal_checks =
            goal_specific_checks(goal.goal_id, &matched_rows, &appraisal_findings, &hardening_entries);

        entries.push(FastqBenchmarkGoalEntry {
            goal_id: goal.goal_id.to_string(),
            title: goal.title.to_string(),
            stage_ids: goal.stage_ids.iter().map(|stage| (*stage).to_string()).collect(),
            scenarios: goal
                .scenarios
                .iter()
                .map(|scenario| FastqScenario {
                    scenario_id: scenario.scenario_id.to_string(),
                    focus: scenario.focus.to_string(),
                })
                .collect(),
            matched_rows,
            appraisal_findings,
            hardening_entries,
            goal_checks,
            status,
        });
    }

    entries
}

fn summarize(entries: &[FastqBenchmarkGoalEntry]) -> FastqBenchmarkCampaignSummary {
    let mut status_counts = BTreeMap::new();
    let mut total_rows = 0;
    let mut total_findings = 0;
    let mut total_queue_entries = 0;
    for entry in entries {
        *status_counts.entry(entry.status.clone()).or_insert(0) += 1;
        total_rows += entry.matched_rows.len();
        total_findings += entry.appraisal_findings.len();
        total_queue_entries += entry.hardening_entries.len();
    }
    FastqBenchmarkCampaignSummary {
        total_goals: entries.len(),
        total_rows,
        total_findings,
        total_queue_entries,
        status_counts,
    }
}

pub fn fastq_benchmark_campaign(args: &FastqBenchmarkCampaignArgs) -> Result<FastqBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_overrides: args.user_overrides.clone(),
        domain: "fastq".to_string(),
        out: None,
        fail_on_refuse: false,
        json: false,
    })?;
    let appraisal = appraise_matrix_report(matrix.clone());
    let queue = hardening_queue_from_appraisal(appraisal.clone());
    let selected_goals = selected_goal_ids(args.goals.as_deref())?;
    let entries = build_goal_entries(
        &selected_goals,
        &matrix,
        &appraisal.findings,
        &queue.entries,
    );
    let report = FastqBenchmarkCampaignReport {
        schema_version: FASTQ_CAMPAIGN_SCHEMA_VERSION.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        selected_goals,
        summary: summarize(&entries),
        entries,
    };
    if let Some(path) = &args.out {
        write_json_pretty(path, &report)?;
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::{
        build_goal_entries, selected_goal_ids, summarize, AppraisalFinding, FastqMatrixRowRef,
        HardeningQueueEntry, FASTQ_GOALS_ITERATION_08,
    };
    use crate::commands::hpc::{
        BenchmarkMatrixReport, BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness,
        BenchmarkSurfaceMatch,
    };

    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: "fastq".to_string(),
            stage_id: stage_id.to_string(),
            tool_id: "seqkit_v2".to_string(),
            corpus_match: BenchmarkSurfaceMatch {
                required_profile: "general".to_string(),
                matched_profile: "general".to_string(),
                ready: true,
            },
            database_match: BenchmarkSurfaceMatch {
                required_profile: "not-required".to_string(),
                matched_profile: "not-required".to_string(),
                ready: true,
            },
            image_match: BenchmarkSurfaceMatch {
                required_profile: "tool-images".to_string(),
                matched_profile: "seqkit".to_string(),
                ready: true,
            },
            readiness: BenchmarkReadiness {
                class: readiness.to_string(),
                reasons: Vec::new(),
            },
            repetitions: if readiness == "refuse" { 0 } else { 3 },
        }
    }

    fn matrix_fixture() -> BenchmarkMatrixReport {
        BenchmarkMatrixReport {
            schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
            campaign_id: "campaign-fastq".to_string(),
            domain: "fastq".to_string(),
            domains: vec!["fastq".to_string()],
            generated_at: "0".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 4,
                readiness_counts: std::collections::BTreeMap::new(),
                domain_counts: std::collections::BTreeMap::new(),
            },
            rows: vec![
                row("r1", "fastq.validate_reads", "ready"),
                row("r2", "fastq.normalize_read_names", "refuse"),
                row("r3", "fastq.trim_reads", "ready"),
                row("r4", "fastq.extract_umis", "ready"),
            ],
        }
    }

    #[test]
    fn iteration_08_catalog_has_ten_fastq_goals() {
        assert_eq!(FASTQ_GOALS_ITERATION_08.len(), 10);
        assert_eq!(FASTQ_GOALS_ITERATION_08[0].goal_id, "G101");
        assert_eq!(FASTQ_GOALS_ITERATION_08[9].goal_id, "G110");
    }

    #[test]
    fn selected_goal_ids_reject_unknown_entries() {
        let err = selected_goal_ids(Some("G101,G999")).expect_err("expected invalid goal");
        assert!(err.to_string().contains("unknown fastq goals requested"));
    }

    #[test]
    fn goal_entries_map_rows_findings_and_queue() {
        let matrix = matrix_fixture();
        let selected = vec!["G101".to_string(), "G102".to_string(), "G109".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "runtime-performance".to_string(),
                row_id: "r1".to_string(),
                severity: "warning".to_string(),
                confidence: "high".to_string(),
                failure_class: "runtime-under-sampled".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "x".to_string(),
                recommendation: "y".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "r2".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "readiness-refuse".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "x".to_string(),
                recommendation: "y".to_string(),
            },
        ];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0001".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            affected_rows: vec!["r2".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];

        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].goal_id, "G101");
        assert_eq!(entries[0].matched_rows.len(), 1);
        assert_eq!(entries[1].goal_id, "G102");
        assert_eq!(entries[1].status, "requires-hardening");
        assert_eq!(entries[2].goal_id, "G109");
        assert_eq!(entries[2].matched_rows.len(), 1);

        let summary = summarize(&entries);
        assert_eq!(summary.total_goals, 3);
        assert!(summary.status_counts.contains_key("requires-hardening"));
    }

    #[test]
    fn summarize_counts_statuses() {
        let entries = vec![
            super::FastqBenchmarkGoalEntry {
                goal_id: "G101".to_string(),
                title: "benchmark FASTQ validation".to_string(),
                stage_ids: vec!["fastq.validate_reads".to_string()],
                scenarios: Vec::new(),
                matched_rows: vec![FastqMatrixRowRef {
                    row_id: "r1".to_string(),
                    stage_id: "fastq.validate_reads".to_string(),
                    tool_id: "seqkit_v2".to_string(),
                    readiness_class: "ready".to_string(),
                    repetitions: 3,
                }],
                appraisal_findings: Vec::new(),
                hardening_entries: Vec::new(),
                goal_checks: Vec::new(),
                status: "ready-for-benchmark-run".to_string(),
            },
            super::FastqBenchmarkGoalEntry {
                goal_id: "G102".to_string(),
                title: "benchmark FASTQ read-name normalization".to_string(),
                stage_ids: vec!["fastq.normalize_read_names".to_string()],
                scenarios: Vec::new(),
                matched_rows: Vec::new(),
                appraisal_findings: Vec::new(),
                hardening_entries: Vec::new(),
                goal_checks: Vec::new(),
                status: "missing-stage-binding".to_string(),
            },
        ];
        let summary = summarize(&entries);
        assert_eq!(summary.total_rows, 1);
        assert_eq!(summary.total_goals, 2);
        assert_eq!(summary.status_counts.get("ready-for-benchmark-run"), Some(&1));
        assert_eq!(summary.status_counts.get("missing-stage-binding"), Some(&1));
    }

    #[test]
    fn goal_101_emits_validation_specific_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G101".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "r1".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0001".to_string(),
            severity: "warning".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            recommendation: "increase repetitions".to_string(),
            affected_rows: vec!["r1".to_string()],
            source_appraisers: vec!["runtime-performance".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("validation_rows_present=")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("validation_queue_entries=")));
    }

    #[test]
    fn goal_102_emits_read_name_specific_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G102".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "r2".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("normalization_stage_bound=true")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("normalization_refuse_rows=")));
    }

    #[test]
    fn goal_103_emits_pair_repair_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G103".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "r3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("pair_repair_rows_present=")));
    }

    #[test]
    fn goal_104_emits_interleave_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G104".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("interleave_stage_pair_covered=")));
    }

    #[test]
    fn goal_105_emits_lane_concat_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G105".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("lane_concat_stage_bound=")));
    }

    #[test]
    fn goal_106_emits_demultiplex_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G106".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("demux_stage_bound=")));
    }

    #[test]
    fn goal_107_emits_subsample_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G107".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "r3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("subsample_repro_findings=")));
    }

    #[test]
    fn goal_108_emits_adapter_detection_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G108".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "backend-equivalence".to_string(),
            row_id: "r3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "single-backend".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("adapter_equivalence_findings=")));
    }

    #[test]
    fn goal_109_emits_trimming_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G109".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "r3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("trim_backend_findings=")));
    }

    #[test]
    fn goal_110_emits_umi_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G110".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "r4".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("umi_code_freeze_findings=")));
    }
}
