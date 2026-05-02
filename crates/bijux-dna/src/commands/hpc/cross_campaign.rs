use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{BenchmarkMatrixArgs, CrossBenchmarkCampaignArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal, AppraisalFinding,
    BenchmarkMatrixReport, HardeningQueueEntry,
};

const CROSS_CAMPAIGN_SCHEMA_VERSION: &str = "bijux.hpc.cross_benchmark_campaign.v1";

const BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN: &str = "fastq.trim_reads=>bam.align";
const BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN: &str = "fastq.validate_reads=>bam.align";
const BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE: &str = "fastq.trim_terminal_damage=>bam.damage";
const BRIDGE_BAM_GENOTYPING_TO_VCF_CALL: &str = "bam.genotyping=>vcf.call";
const BRIDGE_FASTQ_TRIM_TO_VCF_GL: &str = "fastq.trim_reads=>vcf.call_gl";
const BRIDGE_BAM_SUMMARY_TO_VCF_STATS: &str = "bam.mapping_summary=>vcf.stats";
const BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY: &str = "fastq.profile_reads=>bam.mapping_summary";
const BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION: &str =
    "fastq.validate_reads=>bam.contamination";
const BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER: &str = "bam.contamination=>vcf.filter";
const BRIDGE_VCF_FILTER_TO_VCF_STATS: &str = "vcf.filter=>vcf.stats";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossBenchmarkCampaignReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: CrossBenchmarkCampaignSummary,
    pub entries: Vec<CrossBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossBenchmarkCampaignSummary {
    pub total_goals: usize,
    pub total_rows: usize,
    pub total_findings: usize,
    pub total_queue_entries: usize,
    pub status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossBenchmarkGoalEntry {
    pub goal_id: String,
    pub title: String,
    pub stage_ids: Vec<String>,
    pub scenarios: Vec<CrossScenario>,
    pub matched_rows: Vec<CrossMatrixRowRef>,
    pub appraisal_findings: Vec<AppraisalFinding>,
    pub hardening_entries: Vec<HardeningQueueEntry>,
    pub goal_checks: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossScenario {
    pub scenario_id: String,
    pub focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossMatrixRowRef {
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

const G161_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "modern-fastq-bam-validate-trim-align",
        focus: "validate, trim, align, summary, and coverage handoff",
    },
    ScenarioDefinition {
        scenario_id: "modern-reference-preflight",
        focus: "modern reference preparation and handoff integrity",
    },
    ScenarioDefinition {
        scenario_id: "modern-handoff-evidence",
        focus: "handoff evidence continuity from FASTQ to BAM",
    },
];

const G162_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "adna-preprocess-align-damage",
        focus: "aDNA preprocessing, alignment, and damage evidence",
    },
    ScenarioDefinition {
        scenario_id: "adna-contamination-authenticity",
        focus: "aDNA contamination and authenticity caveat flow",
    },
    ScenarioDefinition {
        scenario_id: "adna-cross-domain-appraisal",
        focus: "aDNA cross-domain appraisal consistency",
    },
];

const G163_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "bam-to-vcf-reference-preflight",
        focus: "reference preflight and callable assumptions",
    },
    ScenarioDefinition {
        scenario_id: "bam-to-vcf-call-filter-stats",
        focus: "call/filter/stats handoff for BAM-to-VCF",
    },
    ScenarioDefinition {
        scenario_id: "bam-to-vcf-handoff-appraisal",
        focus: "BAM-to-VCF handoff appraisal stability",
    },
];

const G164_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "fastq-to-vcf-raw-to-call",
        focus: "raw FASTQ to VCF calling bridge",
    },
    ScenarioDefinition {
        scenario_id: "fastq-bam-vcf-artifact-preserve",
        focus: "artifact preservation across FASTQ/BAM/VCF surfaces",
    },
    ScenarioDefinition {
        scenario_id: "fastq-to-vcf-full-chain-appraisal",
        focus: "full-chain appraisal consistency",
    },
];

const G165_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "batch-fanout-per-sample",
        focus: "per-sample fan-out boundaries",
    },
    ScenarioDefinition {
        scenario_id: "batch-fanin-cohort-aggregate",
        focus: "cohort fan-in and aggregate report boundaries",
    },
    ScenarioDefinition {
        scenario_id: "batch-lineage-overwrite-guard",
        focus: "lineage continuity and overwrite guardrails",
    },
];

const G166_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "partial-failure-fastq-boundary",
        focus: "partial failure behavior when FASTQ stage fails",
    },
    ScenarioDefinition {
        scenario_id: "partial-failure-bam-boundary",
        focus: "partial failure behavior when BAM stage fails",
    },
    ScenarioDefinition {
        scenario_id: "partial-failure-vcf-boundary",
        focus: "partial failure behavior when VCF stage fails",
    },
];

const G167_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sample-sheet-good-vs-bad",
        focus: "good and bad sample sheet validation boundaries",
    },
    ScenarioDefinition {
        scenario_id: "sample-sheet-duplicate-missing",
        focus: "duplicate samples and missing file refusal",
    },
    ScenarioDefinition {
        scenario_id: "sample-sheet-reference-mode-conflicts",
        focus: "reference and mode conflict refusal behavior",
    },
];

const G168_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "reference-index-change-before-after",
        focus: "before/after reference and index change propagation",
    },
    ScenarioDefinition {
        scenario_id: "panel-map-alias-invalidation",
        focus: "panel/map/alias invalidation behavior",
    },
    ScenarioDefinition {
        scenario_id: "cross-domain-invalidation-report",
        focus: "cross-domain invalidation report continuity",
    },
];

const G169_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "contamination-fastq-to-bam",
        focus: "contamination propagation from FASTQ to BAM",
    },
    ScenarioDefinition {
        scenario_id: "contamination-bam-to-vcf",
        focus: "contamination propagation from BAM to VCF",
    },
    ScenarioDefinition {
        scenario_id: "contamination-population-caveat-flow",
        focus: "population caveat propagation from contamination signals",
    },
];

const G170_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "reviewer-bundle-export",
        focus: "encrypted result/code bundle export integrity",
    },
    ScenarioDefinition {
        scenario_id: "reviewer-bundle-decrypt-import",
        focus: "local decrypt/import validation boundaries",
    },
    ScenarioDefinition {
        scenario_id: "reviewer-bundle-cross-domain-audit",
        focus: "cross-domain reviewer audit continuity",
    },
];

const CROSS_GOALS_CATALOG: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G161",
        title: "benchmark FASTQ-to-BAM modern workflow",
        stage_ids: &[
            BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN,
            BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
            BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY,
            BRIDGE_BAM_SUMMARY_TO_VCF_STATS,
        ],
        scenarios: G161_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G162",
        title: "benchmark FASTQ-to-BAM aDNA workflow",
        stage_ids: &[
            BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE,
            BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION,
        ],
        scenarios: G162_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G163",
        title: "benchmark BAM-to-VCF workflow",
        stage_ids: &[
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G163_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G164",
        title: "benchmark FASTQ-to-VCF workflow",
        stage_ids: &[
            BRIDGE_FASTQ_TRIM_TO_VCF_GL,
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G164_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G165",
        title: "benchmark batch fan-out/fan-in",
        stage_ids: &[
            BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
            BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY,
            BRIDGE_BAM_SUMMARY_TO_VCF_STATS,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G165_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G166",
        title: "benchmark partial failure policy",
        stage_ids: &[
            BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_FASTQ_TRIM_TO_VCF_GL,
        ],
        scenarios: G166_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G167",
        title: "benchmark sample-sheet validation",
        stage_ids: &[
            BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN,
            BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION,
        ],
        scenarios: G167_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G168",
        title: "benchmark reference-change propagation",
        stage_ids: &[
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_BAM_SUMMARY_TO_VCF_STATS,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G168_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G169",
        title: "benchmark contamination propagation",
        stage_ids: &[
            BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION,
            BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G169_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G170",
        title: "benchmark cross-domain reviewer bundle generation",
        stage_ids: &[
            BRIDGE_FASTQ_TRIM_TO_VCF_GL,
            BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
            BRIDGE_VCF_FILTER_TO_VCF_STATS,
        ],
        scenarios: G170_SCENARIOS,
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
    let available = CROSS_GOALS_CATALOG
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
        return Err(anyhow!("cross-benchmark-campaign requires at least one goal"));
    }
    let unknown = selected
        .iter()
        .filter(|goal| !available.contains(*goal))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!(
            "unknown cross goals requested: {}",
            unknown.join(",")
        ));
    }
    let mut ordered = CROSS_GOALS_CATALOG
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

fn status_for_goal(rows: &[CrossMatrixRowRef], queue_entries: &[HardeningQueueEntry]) -> String {
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
    rows: &[CrossMatrixRowRef],
    _findings: &[AppraisalFinding],
    _queue_entries: &[HardeningQueueEntry],
) -> Vec<String> {
    match goal_id {
        "G161" => vec![
            format!("modern_workflow_rows_present={}", !rows.is_empty()),
            format!(
                "modern_workflow_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "modern_workflow_fastq_to_bam_bound={}",
                rows.iter().any(|row| row.stage_id == BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN)
            ),
            format!(
                "modern_workflow_summary_to_stats_bound={}",
                rows.iter().any(|row| row.stage_id == BRIDGE_BAM_SUMMARY_TO_VCF_STATS)
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
) -> Vec<CrossBenchmarkGoalEntry> {
    let selected = selected_goal_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut entries = Vec::new();

    for goal in CROSS_GOALS_CATALOG.iter().filter(|goal| selected.contains(goal.goal_id)) {
        let stage_set = goal.stage_ids.iter().copied().collect::<BTreeSet<_>>();
        let matched_rows = matrix
            .rows
            .iter()
            .filter(|row| stage_set.contains(row.stage_id.as_str()))
            .map(|row| CrossMatrixRowRef {
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

        entries.push(CrossBenchmarkGoalEntry {
            goal_id: goal.goal_id.to_string(),
            title: goal.title.to_string(),
            stage_ids: goal.stage_ids.iter().map(|stage| (*stage).to_string()).collect(),
            scenarios: goal
                .scenarios
                .iter()
                .map(|scenario| CrossScenario {
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

fn summarize(entries: &[CrossBenchmarkGoalEntry]) -> CrossBenchmarkCampaignSummary {
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
    CrossBenchmarkCampaignSummary {
        total_goals: entries.len(),
        total_rows,
        total_findings,
        total_queue_entries,
        status_counts,
    }
}

pub fn cross_benchmark_campaign(args: &CrossBenchmarkCampaignArgs) -> Result<CrossBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_overrides: args.user_overrides.clone(),
        domain: "cross".to_string(),
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
    let report = CrossBenchmarkCampaignReport {
        schema_version: CROSS_CAMPAIGN_SCHEMA_VERSION.to_string(),
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
        build_goal_entries, selected_goal_ids, summarize, AppraisalFinding, CrossMatrixRowRef,
        HardeningQueueEntry, BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER, BRIDGE_BAM_GENOTYPING_TO_VCF_CALL,
        BRIDGE_BAM_SUMMARY_TO_VCF_STATS, BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE,
        BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY, BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN,
        BRIDGE_FASTQ_TRIM_TO_VCF_GL, BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN,
        BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION, BRIDGE_VCF_FILTER_TO_VCF_STATS,
        CROSS_GOALS_CATALOG,
    };
    use crate::commands::hpc::{
        BenchmarkMatrixReport, BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness,
        BenchmarkSurfaceMatch,
    };

    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: "cross".to_string(),
            stage_id: stage_id.to_string(),
            tool_id: "seqkit_v2=>bwa_v0".to_string(),
            corpus_match: BenchmarkSurfaceMatch {
                required_profile: "wgs".to_string(),
                matched_profile: "wgs".to_string(),
                ready: true,
            },
            database_match: BenchmarkSurfaceMatch {
                required_profile: "vcf".to_string(),
                matched_profile: "vcf".to_string(),
                ready: true,
            },
            image_match: BenchmarkSurfaceMatch {
                required_profile: "tool-images".to_string(),
                matched_profile: "apptainer".to_string(),
                ready: true,
            },
            readiness: BenchmarkReadiness {
                class: readiness.to_string(),
                reasons: Vec::new(),
            },
            repetitions: if readiness == "refuse" { 0 } else { 5 },
        }
    }

    fn matrix_fixture() -> BenchmarkMatrixReport {
        BenchmarkMatrixReport {
            schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
            campaign_id: "campaign-cross".to_string(),
            domain: "cross".to_string(),
            domains: vec!["cross".to_string()],
            generated_at: "0".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 10,
                readiness_counts: std::collections::BTreeMap::new(),
                domain_counts: std::collections::BTreeMap::new(),
            },
            rows: vec![
                row("c1", BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN, "ready"),
                row("c2", BRIDGE_FASTQ_VALIDATE_TO_BAM_ALIGN, "ready"),
                row("c3", BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE, "degraded"),
                row("c4", BRIDGE_BAM_GENOTYPING_TO_VCF_CALL, "ready"),
                row("c5", BRIDGE_FASTQ_TRIM_TO_VCF_GL, "degraded"),
                row("c6", BRIDGE_BAM_SUMMARY_TO_VCF_STATS, "ready"),
                row("c7", BRIDGE_FASTQ_PROFILE_TO_BAM_SUMMARY, "ready"),
                row("c8", BRIDGE_FASTQ_VALIDATE_TO_BAM_CONTAMINATION, "degraded"),
                row("c9", BRIDGE_BAM_CONTAMINATION_TO_VCF_FILTER, "ready"),
                row("c10", BRIDGE_VCF_FILTER_TO_VCF_STATS, "refuse"),
            ],
        }
    }

    #[test]
    fn cross_catalog_includes_iteration_14_goals() {
        assert_eq!(CROSS_GOALS_CATALOG.len(), 10);
        assert_eq!(CROSS_GOALS_CATALOG[0].goal_id, "G161");
        assert_eq!(CROSS_GOALS_CATALOG[9].goal_id, "G170");
    }

    #[test]
    fn selected_goal_ids_reject_unknown_entries() {
        let err = selected_goal_ids(Some("G161,G999")).expect_err("expected invalid goal");
        assert!(err.to_string().contains("unknown cross goals requested"));
    }

    #[test]
    fn goal_entries_map_rows_findings_and_queue() {
        let matrix = matrix_fixture();
        let selected = vec!["G161".to_string(), "G166".to_string(), "G169".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "c10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0001".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            affected_rows: vec!["c10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];

        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].goal_id, "G161");
        assert_eq!(entries[1].goal_id, "G166");
        assert_eq!(entries[2].goal_id, "G169");

        let summary = summarize(&entries);
        assert_eq!(summary.total_goals, 3);
        assert_eq!(summary.status_counts.get("requires-hardening"), Some(&1));
    }

    #[test]
    fn goal_161_emits_modern_cross_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G161".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("modern_workflow_stage_count=4")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("modern_workflow_summary_to_stats_bound=true")));
    }

    #[test]
    fn summary_counts_mixed_status_entries() {
        let entries = vec![
            super::CrossBenchmarkGoalEntry {
                goal_id: "G161".to_string(),
                title: "benchmark FASTQ-to-BAM modern workflow".to_string(),
                stage_ids: vec![BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN.to_string()],
                scenarios: Vec::new(),
                matched_rows: vec![CrossMatrixRowRef {
                    row_id: "c1".to_string(),
                    stage_id: BRIDGE_FASTQ_TRIM_TO_BAM_ALIGN.to_string(),
                    tool_id: "seqkit_v2=>bwa_v0".to_string(),
                    readiness_class: "ready".to_string(),
                    repetitions: 5,
                }],
                appraisal_findings: Vec::new(),
                hardening_entries: Vec::new(),
                goal_checks: Vec::new(),
                status: "ready-for-benchmark-run".to_string(),
            },
            super::CrossBenchmarkGoalEntry {
                goal_id: "G162".to_string(),
                title: "benchmark FASTQ-to-BAM aDNA workflow".to_string(),
                stage_ids: vec![BRIDGE_FASTQ_ADNA_TO_BAM_DAMAGE.to_string()],
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
}
