use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{BamBenchmarkCampaignArgs, BenchmarkMatrixArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal, AppraisalFinding,
    BenchmarkMatrixReport, HardeningQueueEntry,
};

const BAM_CAMPAIGN_SCHEMA_VERSION: &str = "bijux.hpc.bam_benchmark_campaign.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamBenchmarkCampaignReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: BamBenchmarkCampaignSummary,
    pub entries: Vec<BamBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamBenchmarkCampaignSummary {
    pub total_goals: usize,
    pub total_rows: usize,
    pub total_findings: usize,
    pub total_queue_entries: usize,
    pub status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamBenchmarkGoalEntry {
    pub goal_id: String,
    pub title: String,
    pub stage_ids: Vec<String>,
    pub scenarios: Vec<BamScenario>,
    pub matched_rows: Vec<BamMatrixRowRef>,
    pub appraisal_findings: Vec<AppraisalFinding>,
    pub hardening_entries: Vec<HardeningQueueEntry>,
    pub goal_checks: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamScenario {
    pub scenario_id: String,
    pub focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BamMatrixRowRef {
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

const G121_SCENARIOS: &[ScenarioDefinition] = &[
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

const G122_SCENARIOS: &[ScenarioDefinition] = &[
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

const G123_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "local-vs-end-to-end",
        focus: "local and end-to-end parameter differences",
    },
    ScenarioDefinition {
        scenario_id: "short-read-stress",
        focus: "short-read alignment behavior",
    },
    ScenarioDefinition {
        scenario_id: "backend-diff-surface",
        focus: "backend equivalence and divergence surface",
    },
];

const G124_SCENARIOS: &[ScenarioDefinition] = &[
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

const G125_SCENARIOS: &[ScenarioDefinition] = &[
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

const G126_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "compatible-merge-path",
        focus: "compatible BAM merge path",
    },
    ScenarioDefinition {
        scenario_id: "reference-and-rg-conflicts",
        focus: "reference and read-group conflict handling",
    },
    ScenarioDefinition {
        scenario_id: "platform-and-metadata-conflicts",
        focus: "platform and duplicate metadata conflicts",
    },
];

const G127_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "mark-vs-remove-modes",
        focus: "mark-only vs remove duplicate modes",
    },
    ScenarioDefinition {
        scenario_id: "picard-vs-samtools-like",
        focus: "method differences across duplicate handlers",
    },
    ScenarioDefinition {
        scenario_id: "umi-aware-duplicate-policy",
        focus: "UMI-aware duplicate handling boundaries",
    },
];

const G128_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "threshold-impact-surface",
        focus: "MAPQ threshold impact surface",
    },
    ScenarioDefinition {
        scenario_id: "retained-removed-coverage-effect",
        focus: "retained/removed read coverage effect",
    },
    ScenarioDefinition {
        scenario_id: "vcf-readiness-impact",
        focus: "downstream VCF readiness impact",
    },
];

const G129_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "mapped-unmapped-balance",
        focus: "mapped/unmapped count stability",
    },
    ScenarioDefinition {
        scenario_id: "pair-secondary-supplementary",
        focus: "proper-pair/secondary/supplementary counting",
    },
    ScenarioDefinition {
        scenario_id: "mapq-distribution-stability",
        focus: "MAPQ distribution stability",
    },
];

const G130_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sparse-lowpass-regimes",
        focus: "sparse and low-pass coverage regimes",
    },
    ScenarioDefinition {
        scenario_id: "target-wgs-regimes",
        focus: "target-like and WGS-like coverage regimes",
    },
    ScenarioDefinition {
        scenario_id: "coverage-refusal-boundary",
        focus: "unusable coverage refusal boundary",
    },
];

const BAM_GOALS_CATALOG: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G121",
        title: "benchmark BAM/SAM/CRAM validation boundaries",
        stage_ids: &["bam.validate"],
        scenarios: G121_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G122",
        title: "benchmark BWA-style alignment",
        stage_ids: &["bam.align"],
        scenarios: G122_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G123",
        title: "benchmark Bowtie2-style alignment",
        stage_ids: &["bam.align"],
        scenarios: G123_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G124",
        title: "benchmark BAM sort and index",
        stage_ids: &["bam.validate", "bam.mapping_summary"],
        scenarios: G124_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G125",
        title: "benchmark read-group propagation",
        stage_ids: &["bam.validate", "bam.mapping_summary"],
        scenarios: G125_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G126",
        title: "benchmark BAM merge",
        stage_ids: &["bam.validate", "bam.qc_pre"],
        scenarios: G126_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G127",
        title: "benchmark duplicate handling",
        stage_ids: &["bam.markdup", "bam.duplication_metrics"],
        scenarios: G127_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G128",
        title: "benchmark MAPQ filtering",
        stage_ids: &["bam.mapq_filter"],
        scenarios: G128_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G129",
        title: "benchmark mapping summaries",
        stage_ids: &["bam.mapping_summary"],
        scenarios: G129_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G130",
        title: "benchmark coverage regimes",
        stage_ids: &["bam.coverage"],
        scenarios: G130_SCENARIOS,
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
    let available = BAM_GOALS_CATALOG
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
        return Err(anyhow!("bam-benchmark-campaign requires at least one goal"));
    }
    let unknown = selected
        .iter()
        .filter(|goal| !available.contains(*goal))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!(
            "unknown bam goals requested: {}",
            unknown.join(",")
        ));
    }
    let mut ordered = BAM_GOALS_CATALOG
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

fn status_for_goal(rows: &[BamMatrixRowRef], queue_entries: &[HardeningQueueEntry]) -> String {
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
    rows: &[BamMatrixRowRef],
    findings: &[AppraisalFinding],
    queue_entries: &[HardeningQueueEntry],
) -> Vec<String> {
    match goal_id {
        "G121" => vec![
            format!("validation_rows_present={}", !rows.is_empty()),
            format!(
                "validation_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.validate")
            ),
            format!(
                "validation_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "validation_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness"))
                    .count()
            ),
            format!("validation_findings={}", findings.len()),
        ],
        "G122" => vec![
            format!("bwa_alignment_rows_present={}", !rows.is_empty()),
            format!(
                "bwa_alignment_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.align")
            ),
            format!(
                "bwa_alignment_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "bwa_alignment_runtime_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "runtime-performance")
                    .count()
            ),
        ],
        "G123" => vec![
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
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "backend-equivalence")
                    .count()
            ),
        ],
        "G124" => vec![
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
        ],
        _ => Vec::new(),
    }
}

fn build_goal_entries(
    selected_goal_ids: &[String],
    matrix: &BenchmarkMatrixReport,
    findings: &[AppraisalFinding],
    queue: &[HardeningQueueEntry],
) -> Vec<BamBenchmarkGoalEntry> {
    let selected = selected_goal_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut entries = Vec::new();

    for goal in BAM_GOALS_CATALOG.iter().filter(|goal| selected.contains(goal.goal_id)) {
        let stage_set = goal.stage_ids.iter().copied().collect::<BTreeSet<_>>();
        let matched_rows = matrix
            .rows
            .iter()
            .filter(|row| stage_set.contains(row.stage_id.as_str()))
            .map(|row| BamMatrixRowRef {
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

        entries.push(BamBenchmarkGoalEntry {
            goal_id: goal.goal_id.to_string(),
            title: goal.title.to_string(),
            stage_ids: goal.stage_ids.iter().map(|stage| (*stage).to_string()).collect(),
            scenarios: goal
                .scenarios
                .iter()
                .map(|scenario| BamScenario {
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

fn summarize(entries: &[BamBenchmarkGoalEntry]) -> BamBenchmarkCampaignSummary {
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
    BamBenchmarkCampaignSummary {
        total_goals: entries.len(),
        total_rows,
        total_findings,
        total_queue_entries,
        status_counts,
    }
}

pub fn bam_benchmark_campaign(args: &BamBenchmarkCampaignArgs) -> Result<BamBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_overrides: args.user_overrides.clone(),
        domain: "bam".to_string(),
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
    let report = BamBenchmarkCampaignReport {
        schema_version: BAM_CAMPAIGN_SCHEMA_VERSION.to_string(),
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
        build_goal_entries, selected_goal_ids, summarize, AppraisalFinding, BamMatrixRowRef,
        HardeningQueueEntry, BAM_GOALS_CATALOG,
    };
    use crate::commands::hpc::{
        BenchmarkMatrixReport, BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness,
        BenchmarkSurfaceMatch,
    };

    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: "bam".to_string(),
            stage_id: stage_id.to_string(),
            tool_id: "samtools_v1".to_string(),
            corpus_match: BenchmarkSurfaceMatch {
                required_profile: "wgs".to_string(),
                matched_profile: "wgs".to_string(),
                ready: true,
            },
            database_match: BenchmarkSurfaceMatch {
                required_profile: "align".to_string(),
                matched_profile: "align".to_string(),
                ready: true,
            },
            image_match: BenchmarkSurfaceMatch {
                required_profile: "tool-images".to_string(),
                matched_profile: "samtools".to_string(),
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
            campaign_id: "campaign-bam".to_string(),
            domain: "bam".to_string(),
            domains: vec!["bam".to_string()],
            generated_at: "0".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 9,
                readiness_counts: std::collections::BTreeMap::new(),
                domain_counts: std::collections::BTreeMap::new(),
            },
            rows: vec![
                row("b1", "bam.validate", "ready"),
                row("b2", "bam.align", "ready"),
                row("b3", "bam.qc_pre", "degraded"),
                row("b4", "bam.markdup", "ready"),
                row("b5", "bam.duplication_metrics", "ready"),
                row("b6", "bam.mapq_filter", "ready"),
                row("b7", "bam.mapping_summary", "ready"),
                row("b8", "bam.coverage", "degraded"),
                row("b9", "bam.validate", "refuse"),
            ],
        }
    }

    #[test]
    fn bam_catalog_has_ten_iteration_10_goals() {
        assert_eq!(BAM_GOALS_CATALOG.len(), 10);
        assert_eq!(BAM_GOALS_CATALOG[0].goal_id, "G121");
        assert_eq!(BAM_GOALS_CATALOG[9].goal_id, "G130");
    }

    #[test]
    fn selected_goal_ids_reject_unknown_entries() {
        let err = selected_goal_ids(Some("G121,G999")).expect_err("expected invalid goal");
        assert!(err.to_string().contains("unknown bam goals requested"));
    }

    #[test]
    fn goal_entries_map_rows_findings_and_queue() {
        let matrix = matrix_fixture();
        let selected = vec!["G121".to_string(), "G127".to_string(), "G130".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "b1".to_string(),
                severity: "warning".to_string(),
                confidence: "high".to_string(),
                failure_class: "readiness-degraded".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "x".to_string(),
                recommendation: "y".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "scientific-output".to_string(),
                row_id: "b8".to_string(),
                severity: "warning".to_string(),
                confidence: "medium".to_string(),
                failure_class: "coverage-uncertain".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "x".to_string(),
                recommendation: "y".to_string(),
            },
        ];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0001".to_string(),
            severity: "warning".to_string(),
            failure_class: "coverage-uncertain".to_string(),
            recommendation: "tighten coverage policy".to_string(),
            affected_rows: vec!["b8".to_string()],
            source_appraisers: vec!["scientific-output".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].goal_id, "G121");
        assert_eq!(entries[1].goal_id, "G127");
        assert_eq!(entries[2].goal_id, "G130");

        let summary = summarize(&entries);
        assert_eq!(summary.total_goals, 3);
    }

    #[test]
    fn summarize_counts_statuses() {
        let entries = vec![
            super::BamBenchmarkGoalEntry {
                goal_id: "G121".to_string(),
                title: "benchmark BAM/SAM/CRAM validation boundaries".to_string(),
                stage_ids: vec!["bam.validate".to_string()],
                scenarios: Vec::new(),
                matched_rows: vec![BamMatrixRowRef {
                    row_id: "b1".to_string(),
                    stage_id: "bam.validate".to_string(),
                    tool_id: "samtools_v1".to_string(),
                    readiness_class: "ready".to_string(),
                    repetitions: 3,
                }],
                appraisal_findings: Vec::new(),
                hardening_entries: Vec::new(),
                goal_checks: Vec::new(),
                status: "ready-for-benchmark-run".to_string(),
            },
            super::BamBenchmarkGoalEntry {
                goal_id: "G122".to_string(),
                title: "benchmark BWA-style alignment".to_string(),
                stage_ids: vec!["bam.align".to_string()],
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
    fn goal_121_emits_validation_boundary_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G121".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b9".to_string(),
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
            .any(|check| check.starts_with("validation_stage_bound=true")));
    }

    #[test]
    fn goal_122_emits_bwa_alignment_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G122".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "b2".to_string(),
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
            .any(|check| check.starts_with("bwa_alignment_stage_bound=true")));
    }

    #[test]
    fn goal_123_emits_bowtie2_alignment_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G123".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "backend-equivalence".to_string(),
            row_id: "b2".to_string(),
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
            .any(|check| check.starts_with("bowtie2_alignment_stage_bound=true")));
    }

    #[test]
    fn goal_124_emits_sort_index_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G124".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("sort_index_validate_stage_bound=true")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("sort_index_mapping_summary_bound=true")));
    }
}
