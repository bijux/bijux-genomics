use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{BenchmarkMatrixArgs, HardeningBenchmarkCampaignArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal, AppraisalFinding,
    BenchmarkMatrixReport, HardeningQueueEntry,
};

const HARDENING_CAMPAIGN_SCHEMA_VERSION: &str = "bijux.hpc.hardening_benchmark_campaign.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningBenchmarkCampaignReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: HardeningBenchmarkCampaignSummary,
    pub entries: Vec<HardeningBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningBenchmarkCampaignSummary {
    pub total_goals: usize,
    pub total_rows: usize,
    pub total_findings: usize,
    pub total_queue_entries: usize,
    pub status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningBenchmarkGoalEntry {
    pub goal_id: String,
    pub title: String,
    pub stage_ids: Vec<String>,
    pub scenarios: Vec<HardeningScenario>,
    pub matched_rows: Vec<HardeningMatrixRowRef>,
    pub appraisal_findings: Vec<AppraisalFinding>,
    pub hardening_entries: Vec<HardeningQueueEntry>,
    pub goal_checks: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningScenario {
    pub scenario_id: String,
    pub focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardeningMatrixRowRef {
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

const G171_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "per-stage-failure-to-playbook",
        focus: "convert stage failures into concrete hardening playbooks",
    },
    ScenarioDefinition {
        scenario_id: "stage-rerun-proof-path",
        focus: "capture rerun proof path per failed stage",
    },
    ScenarioDefinition {
        scenario_id: "stage-hardening-queue-export",
        focus: "export stage-specific hardening queue views",
    },
];

const G172_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "tool-version-grouping",
        focus: "group failures by tool and version",
    },
    ScenarioDefinition {
        scenario_id: "tool-specific-hardening-queue",
        focus: "emit tool-specific hardening queue items",
    },
    ScenarioDefinition {
        scenario_id: "tool-rerun-proof",
        focus: "attach rerun proof for tool-level hardening",
    },
];

const G173_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "corpus-weakness-detection",
        focus: "detect weak corpus signals from appraisers",
    },
    ScenarioDefinition {
        scenario_id: "corpus-improvement-task-generation",
        focus: "generate corpus-improvement hardening tasks",
    },
    ScenarioDefinition {
        scenario_id: "corpus-rerun-proof",
        focus: "capture rerun proof after corpus changes",
    },
];

const G174_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "database-weakness-detection",
        focus: "detect DB incompatibility and weakness findings",
    },
    ScenarioDefinition {
        scenario_id: "database-fix-task-generation",
        focus: "generate DB hardening task outputs",
    },
    ScenarioDefinition {
        scenario_id: "database-rerun-proof",
        focus: "capture rerun proof after DB fixes",
    },
];

const G175_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "image-smoke-bind-failure-detection",
        focus: "detect image smoke/bind/version failures",
    },
    ScenarioDefinition {
        scenario_id: "image-fix-task-generation",
        focus: "generate image-hardening tasks",
    },
    ScenarioDefinition {
        scenario_id: "image-rerun-proof",
        focus: "capture rerun proof after image fixes",
    },
];

const G176_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "slurm-wrapper-failure-detection",
        focus: "detect wrapper path/env/signal/output failures",
    },
    ScenarioDefinition {
        scenario_id: "wrapper-fix-task-generation",
        focus: "generate wrapper-hardening tasks",
    },
    ScenarioDefinition {
        scenario_id: "wrapper-rerun-proof",
        focus: "capture rerun proof for wrapper fixes",
    },
];

const G177_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "resource-drift-detection",
        focus: "detect CPU/mem/walltime/scratch/IO drift",
    },
    ScenarioDefinition {
        scenario_id: "resource-tuning-task-generation",
        focus: "generate resource tuning patch tasks",
    },
    ScenarioDefinition {
        scenario_id: "resource-before-after-benchmark",
        focus: "record before/after benchmark evidence",
    },
];

const G178_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "io-staging-policy-detection",
        focus: "detect scratch/shared-storage IO policy issues",
    },
    ScenarioDefinition {
        scenario_id: "io-staging-task-generation",
        focus: "generate IO staging hardening tasks",
    },
    ScenarioDefinition {
        scenario_id: "io-improvement-report",
        focus: "capture IO improvement evidence report",
    },
];

const G179_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "encryption-redaction-failure-detection",
        focus: "detect encryption, redaction, key, sidecar failures",
    },
    ScenarioDefinition {
        scenario_id: "security-hardening-task-generation",
        focus: "generate security hardening tasks",
    },
    ScenarioDefinition {
        scenario_id: "security-appraisal-proof",
        focus: "capture security appraisal proof after fixes",
    },
];

const G180_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "replay-field-gap-detection",
        focus: "detect missing replay fields in code/results bundles",
    },
    ScenarioDefinition {
        scenario_id: "replay-hardening-task-generation",
        focus: "generate replay hardening tasks",
    },
    ScenarioDefinition {
        scenario_id: "replay-success-proof",
        focus: "capture replay success evidence",
    },
];

const G181_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "repeat-run-drift-detection",
        focus: "detect unstable outputs and ordering drift between repeated runs",
    },
    ScenarioDefinition {
        scenario_id: "seed-and-temp-leakage-detection",
        focus: "detect missing seed control and temp-path leakage in artifacts",
    },
    ScenarioDefinition {
        scenario_id: "determinism-rerun-proof",
        focus: "capture rerun proof after determinism hardening",
    },
];

const G182_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "schema-parse-failure-detection",
        focus: "detect parser and schema contract failures in result artifacts",
    },
    ScenarioDefinition {
        scenario_id: "schema-contract-fix-queue",
        focus: "queue schema contract hardening tasks from appraisal signals",
    },
    ScenarioDefinition {
        scenario_id: "schema-rerun-proof",
        focus: "capture schema-pass proof after hardening",
    },
];

const G183_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "advisory-caveat-coverage",
        focus: "detect missing or over-promoted advisory caveats",
    },
    ScenarioDefinition {
        scenario_id: "scientific-caveat-correction",
        focus: "generate caveat correction tasks from scientific findings",
    },
    ScenarioDefinition {
        scenario_id: "caveat-appraiser-pass",
        focus: "capture caveat appraiser pass evidence after fixes",
    },
];

const G184_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "taxonomy-classification-gaps",
        focus: "detect failure classification gaps across runtime/tool/io/scientific classes",
    },
    ScenarioDefinition {
        scenario_id: "taxonomy-grouping-hardening",
        focus: "harden grouping quality for actionable hardening queues",
    },
    ScenarioDefinition {
        scenario_id: "taxonomy-report-quality-proof",
        focus: "capture evidence that grouped reports improved after hardening",
    },
];

const G185_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "minimizer-failure-anchor-detection",
        focus: "anchor reproducible failure rows for minimization candidates",
    },
    ScenarioDefinition {
        scenario_id: "minimized-input-plan",
        focus: "generate minimal corpus/config/database reproduction plans",
    },
    ScenarioDefinition {
        scenario_id: "minimized-bundle-proof",
        focus: "capture minimized encrypted failure bundle evidence",
    },
];

const G186_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "local-investigation-layout-validation",
        focus: "validate local investigation workspace structure for copied bundles",
    },
    ScenarioDefinition {
        scenario_id: "local-index-and-catalog-checks",
        focus: "validate indexing and organization checks for local investigation",
    },
    ScenarioDefinition {
        scenario_id: "investigation-workflow-proof",
        focus: "capture local investigation workflow proof from hardening checks",
    },
];

const G187_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "result-metric-diff-detection",
        focus: "detect changed metrics and caveats across result bundles",
    },
    ScenarioDefinition {
        scenario_id: "result-artifact-diff-detection",
        focus: "detect changed artifacts/logs/appraisals across result bundles",
    },
    ScenarioDefinition {
        scenario_id: "result-diff-proof",
        focus: "capture evidence that result diff workflow is actionable",
    },
];

const G188_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "code-snapshot-diff-detection",
        focus: "detect source/config/dag drift between code bundles",
    },
    ScenarioDefinition {
        scenario_id: "image-db-corpus-diff-detection",
        focus: "detect image/database/corpus lock drift between code bundles",
    },
    ScenarioDefinition {
        scenario_id: "code-diff-proof",
        focus: "capture evidence that code diff workflow is actionable",
    },
];

const G189_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "baseline-promotion-gate-checks",
        focus: "enforce appraisal-pass gates before baseline promotion",
    },
    ScenarioDefinition {
        scenario_id: "baseline-lock-completeness-checks",
        focus: "validate baseline lock completeness for accepted rows",
    },
    ScenarioDefinition {
        scenario_id: "baseline-promotion-proof",
        focus: "capture baseline promotion proof with lock records",
    },
];

const G190_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "baseline-rerun-regression-detection",
        focus: "detect performance/scientific/artifact/replay/security regressions",
    },
    ScenarioDefinition {
        scenario_id: "regression-severity-queue-generation",
        focus: "generate severity-sorted regression hardening queue entries",
    },
    ScenarioDefinition {
        scenario_id: "regression-report-proof",
        focus: "capture regression report evidence from rerun appraisals",
    },
];

const HARDENING_GOALS_CATALOG: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G171",
        title: "add per-stage hardening playbooks",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.filter"],
        scenarios: G171_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G172",
        title: "add per-tool hardening playbooks",
        stage_ids: &["fastq.trim_reads", "bam.align", "vcf.call"],
        scenarios: G172_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G173",
        title: "add corpus-improvement hardening",
        stage_ids: &["fastq.validate_reads", "fastq.profile_reads", "bam.align"],
        scenarios: G173_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G174",
        title: "add database-improvement hardening",
        stage_ids: &["bam.align", "vcf.call", "vcf.prepare_reference_panel"],
        scenarios: G174_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G175",
        title: "add Apptainer-image hardening",
        stage_ids: &["fastq.trim_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G175_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G176",
        title: "add Slurm-wrapper hardening",
        stage_ids: &["bam.mapping_summary", "bam.coverage", "vcf.stats"],
        scenarios: G176_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G177",
        title: "add resource-tuning hardening",
        stage_ids: &["bam.align", "bam.coverage", "vcf.call"],
        scenarios: G177_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G178",
        title: "add IO-staging hardening",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G178_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G179",
        title: "add encryption hardening",
        stage_ids: &["bam.contamination", "vcf.impute", "vcf.postprocess"],
        scenarios: G179_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G180",
        title: "add replay hardening",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.postprocess"],
        scenarios: G180_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G181",
        title: "add nondeterminism hardening",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G181_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G182",
        title: "add output-schema hardening",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.postprocess"],
        scenarios: G182_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G183",
        title: "add caveat hardening",
        stage_ids: &["bam.contamination", "vcf.filter", "vcf.stats"],
        scenarios: G183_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G184",
        title: "add failure taxonomy hardening",
        stage_ids: &["fastq.trim_reads", "bam.align", "vcf.filter"],
        scenarios: G184_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G185",
        title: "add benchmark minimizer",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.filter"],
        scenarios: G185_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G186",
        title: "add local investigation workspace",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.postprocess"],
        scenarios: G186_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G187",
        title: "add result diff workflow",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.stats"],
        scenarios: G187_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G188",
        title: "add code diff workflow",
        stage_ids: &["fastq.trim_reads", "bam.mapping_summary", "vcf.postprocess"],
        scenarios: G188_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G189",
        title: "add accepted-baseline workflow",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.stats"],
        scenarios: G189_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G190",
        title: "add regression workflow",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.postprocess"],
        scenarios: G190_SCENARIOS,
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
    let available = HARDENING_GOALS_CATALOG
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
        return Err(anyhow!("hardening-benchmark-campaign requires at least one goal"));
    }
    let unknown = selected
        .iter()
        .filter(|goal| !available.contains(*goal))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!(
            "unknown hardening goals requested: {}",
            unknown.join(",")
        ));
    }
    let mut ordered = HARDENING_GOALS_CATALOG
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

fn status_for_goal(rows: &[HardeningMatrixRowRef], queue_entries: &[HardeningQueueEntry]) -> String {
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
    rows: &[HardeningMatrixRowRef],
    findings: &[AppraisalFinding],
    queue_entries: &[HardeningQueueEntry],
) -> Vec<String> {
    match goal_id {
        "G171" => vec![
            format!("stage_playbook_rows_present={}", !rows.is_empty()),
            format!(
                "stage_playbook_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "stage_playbook_validate_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
            ),
            format!(
                "stage_playbook_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "stage_playbook_non_info_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "info")
                    .count()
            ),
        ],
        "G172" => vec![
            format!("tool_playbook_rows_present={}", !rows.is_empty()),
            format!(
                "tool_playbook_tool_count={}",
                rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "tool_playbook_degraded_or_refuse_tools={}",
                rows.iter()
                    .filter(|row| row.readiness_class != "ready")
                    .map(|row| row.tool_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "tool_playbook_tool_binding_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "missing-tool-binding")
                    .count()
            ),
            format!(
                "tool_playbook_queue_non_info={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "info")
                    .count()
            ),
        ],
        "G173" => vec![
            format!("corpus_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "corpus_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "corpus_hardening_profile_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
            ),
            format!(
                "corpus_hardening_corpus_mismatch_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "corpus-mismatch")
                    .count()
            ),
            format!(
                "corpus_hardening_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
        ],
        "G174" => vec![
            format!("database_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "database_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "database_hardening_reference_panel_bound={}",
                rows.iter()
                    .any(|row| row.stage_id == "vcf.prepare_reference_panel")
            ),
            format!(
                "database_hardening_alignment_and_call_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "database_hardening_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
        ],
        "G175" => vec![
            format!("image_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "image_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "image_hardening_mapping_summary_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
            ),
            format!(
                "image_hardening_image_mismatch_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "image-mismatch")
                    .count()
            ),
            format!(
                "image_hardening_image_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class == "image-mismatch")
                    .count()
            ),
        ],
        "G176" => vec![
            format!("wrapper_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "wrapper_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "wrapper_hardening_summary_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
            ),
            format!(
                "wrapper_hardening_runtime_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class.starts_with("runtime-"))
                    .count()
            ),
            format!(
                "wrapper_hardening_repro_low_repeat_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "reproducibility-low-repeats")
                    .count()
            ),
        ],
        "G177" => vec![
            format!("resource_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "resource_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "resource_hardening_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "resource_hardening_min_repetitions={}",
                rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
            ),
            format!(
                "resource_hardening_runtime_under_sampled_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "runtime-under-sampled")
                    .count()
            ),
        ],
        "G178" => vec![
            format!("io_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "io_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "io_hardening_profile_to_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "io_hardening_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "io_hardening_code_freeze_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                    .count()
            ),
        ],
        "G179" => vec![
            format!("security_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "security_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "security_hardening_encrypted_scope_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                    .count()
            ),
            format!(
                "security_hardening_critical_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity == "critical")
                    .count()
            ),
            format!(
                "security_hardening_code_freeze_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                    .count()
            ),
        ],
        "G180" => vec![
            format!("replay_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "replay_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "replay_hardening_chain_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "replay_hardening_code_freeze_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                    .count()
            ),
            format!(
                "replay_hardening_code_freeze_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.source_appraisers.iter().any(|id| id == "code-freeze"))
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
) -> Vec<HardeningBenchmarkGoalEntry> {
    let selected = selected_goal_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut entries = Vec::new();

    for goal in HARDENING_GOALS_CATALOG.iter().filter(|goal| selected.contains(goal.goal_id)) {
        let stage_set = goal.stage_ids.iter().copied().collect::<BTreeSet<_>>();
        let matched_rows = matrix
            .rows
            .iter()
            .filter(|row| stage_set.contains(row.stage_id.as_str()))
            .map(|row| HardeningMatrixRowRef {
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

        entries.push(HardeningBenchmarkGoalEntry {
            goal_id: goal.goal_id.to_string(),
            title: goal.title.to_string(),
            stage_ids: goal.stage_ids.iter().map(|stage| (*stage).to_string()).collect(),
            scenarios: goal
                .scenarios
                .iter()
                .map(|scenario| HardeningScenario {
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

fn summarize(entries: &[HardeningBenchmarkGoalEntry]) -> HardeningBenchmarkCampaignSummary {
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
    HardeningBenchmarkCampaignSummary {
        total_goals: entries.len(),
        total_rows,
        total_findings,
        total_queue_entries,
        status_counts,
    }
}

pub fn hardening_benchmark_campaign(
    args: &HardeningBenchmarkCampaignArgs,
) -> Result<HardeningBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_overrides: args.user_overrides.clone(),
        domain: "all".to_string(),
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
    let report = HardeningBenchmarkCampaignReport {
        schema_version: HARDENING_CAMPAIGN_SCHEMA_VERSION.to_string(),
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
        build_goal_entries, selected_goal_ids, summarize, AppraisalFinding, HardeningMatrixRowRef,
        HardeningQueueEntry, HARDENING_GOALS_CATALOG,
    };
    use crate::commands::hpc::{
        BenchmarkMatrixReport, BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness,
        BenchmarkSurfaceMatch,
    };

    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: if stage_id.contains("=>") {
                "cross".to_string()
            } else {
                stage_id.split('.').next().unwrap_or("fastq").to_string()
            },
            stage_id: stage_id.to_string(),
            tool_id: "tool_v1".to_string(),
            corpus_match: BenchmarkSurfaceMatch {
                required_profile: "general".to_string(),
                matched_profile: "general".to_string(),
                ready: true,
            },
            database_match: BenchmarkSurfaceMatch {
                required_profile: "general".to_string(),
                matched_profile: "general".to_string(),
                ready: true,
            },
            image_match: BenchmarkSurfaceMatch {
                required_profile: "tool-images".to_string(),
                matched_profile: "tool".to_string(),
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
            campaign_id: "campaign-hardening".to_string(),
            domain: "all".to_string(),
            domains: vec![
                "fastq".to_string(),
                "bam".to_string(),
                "vcf".to_string(),
                "cross".to_string(),
            ],
            generated_at: "0".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 13,
                readiness_counts: std::collections::BTreeMap::new(),
                domain_counts: std::collections::BTreeMap::new(),
            },
            rows: vec![
                row("h1", "fastq.validate_reads", "ready"),
                row("h2", "fastq.trim_reads", "ready"),
                row("h3", "fastq.profile_reads", "degraded"),
                row("h4", "bam.validate", "ready"),
                row("h5", "bam.align", "degraded"),
                row("h6", "bam.mapping_summary", "ready"),
                row("h7", "bam.coverage", "degraded"),
                row("h8", "bam.contamination", "ready"),
                row("h9", "vcf.call", "ready"),
                row("h10", "vcf.filter", "refuse"),
                row("h11", "vcf.stats", "ready"),
                row("h12", "vcf.impute", "degraded"),
                row("h13", "vcf.postprocess", "ready"),
            ],
        }
    }

    #[test]
    fn hardening_catalog_includes_iteration_15_goals() {
        assert_eq!(HARDENING_GOALS_CATALOG.len(), 20);
        assert_eq!(HARDENING_GOALS_CATALOG[0].goal_id, "G171");
        assert_eq!(HARDENING_GOALS_CATALOG[19].goal_id, "G190");
    }

    #[test]
    fn selected_goal_ids_reject_unknown_entries() {
        let err = selected_goal_ids(Some("G171,G999")).expect_err("expected invalid goal");
        assert!(err.to_string().contains("unknown hardening goals requested"));
    }

    #[test]
    fn goal_entries_map_rows_findings_and_queue() {
        let matrix = matrix_fixture();
        let selected = vec!["G171".to_string(), "G175".to_string(), "G179".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
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
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];

        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].goal_id, "G171");
        assert_eq!(entries[1].goal_id, "G175");
        assert_eq!(entries[2].goal_id, "G179");

        let summary = summarize(&entries);
        assert_eq!(summary.total_goals, 3);
        assert_eq!(summary.status_counts.get("requires-hardening"), Some(&1));
    }

    #[test]
    fn goal_171_emits_stage_playbook_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G171".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0101".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &[], &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("stage_playbook_stage_count=3")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "stage_playbook_refuse_rows=1"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "stage_playbook_non_info_queue_entries=1"));
    }

    #[test]
    fn goal_172_emits_tool_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G172".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "artifact-validity".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "missing-tool-binding".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "tool binding missing".to_string(),
            recommendation: "bind stage to at least one governed tool".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0002".to_string(),
            severity: "warning".to_string(),
            failure_class: "missing-tool-binding".to_string(),
            recommendation: "bind stage to at least one governed tool".to_string(),
            affected_rows: vec!["h5".to_string()],
            source_appraisers: vec!["artifact-validity".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "tool_playbook_tool_count=1"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "tool_playbook_tool_binding_findings=1"));
    }

    #[test]
    fn goal_173_emits_corpus_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G173".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "corpus-suitability".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "corpus-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "corpus profile mismatch".to_string(),
            recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_hardening_profile_stage_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_hardening_corpus_mismatch_findings=1"));
    }

    #[test]
    fn goal_174_emits_database_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G174".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "database_hardening_alignment_and_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "database_hardening_reference_panel_bound=false"));
    }

    #[test]
    fn goal_175_emits_image_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G175".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "artifact-validity".to_string(),
            row_id: "h6".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "image-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "image mismatch".to_string(),
            recommendation: "prepare or stage matching image before benchmark".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0003".to_string(),
            severity: "critical".to_string(),
            failure_class: "image-mismatch".to_string(),
            recommendation: "prepare or stage matching image before benchmark".to_string(),
            affected_rows: vec!["h6".to_string()],
            source_appraisers: vec!["artifact-validity".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "image_hardening_mapping_summary_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "image_hardening_image_mismatch_findings=1"));
    }

    #[test]
    fn goal_176_emits_slurm_wrapper_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G176".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "runtime-performance".to_string(),
                row_id: "h7".to_string(),
                severity: "warning".to_string(),
                confidence: "medium".to_string(),
                failure_class: "runtime-under-sampled".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "benchmark row has too few repetitions".to_string(),
                recommendation: "increase repetitions to at least 2".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "reproducibility".to_string(),
                row_id: "h6".to_string(),
                severity: "warning".to_string(),
                confidence: "medium".to_string(),
                failure_class: "reproducibility-low-repeats".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "low repeats for reproducibility".to_string(),
                recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
            },
        ];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "wrapper_hardening_runtime_findings=1"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "wrapper_hardening_repro_low_repeat_findings=1"));
    }

    #[test]
    fn goal_177_emits_resource_tuning_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G177".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "benchmark row has too few repetitions".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "resource_hardening_degraded_rows=2"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "resource_hardening_runtime_under_sampled_findings=1"));
    }

    #[test]
    fn goal_178_emits_io_staging_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G178".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h11".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "code freeze missing lock metadata".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "io_hardening_profile_to_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "io_hardening_code_freeze_findings=1"));
    }

    #[test]
    fn goal_179_emits_encryption_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G179".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "code-freeze".to_string(),
                row_id: "h13".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "code-freeze-incomplete".to_string(),
                result_scope: "encrypted-code".to_string(),
                summary: "code freeze metadata incomplete".to_string(),
                recommendation: "bind tool and image lock before code freeze publication".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "h12".to_string(),
                severity: "warning".to_string(),
                confidence: "medium".to_string(),
                failure_class: "readiness-degraded".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "readiness degraded".to_string(),
                recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            },
        ];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0004".to_string(),
            severity: "critical".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
            affected_rows: vec!["h13".to_string()],
            source_appraisers: vec!["code-freeze".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "security_hardening_encrypted_scope_findings=2"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "security_hardening_critical_queue_entries=1"));
    }

    #[test]
    fn goal_180_emits_replay_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G180".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "replay snapshot missing lock metadata".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0005".to_string(),
            severity: "critical".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
            affected_rows: vec!["h13".to_string()],
            source_appraisers: vec!["code-freeze".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "replay_hardening_chain_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "replay_hardening_code_freeze_queue_entries=1"));
    }

    #[test]
    fn summary_counts_mixed_status_entries() {
        let entries = vec![
            super::HardeningBenchmarkGoalEntry {
                goal_id: "G171".to_string(),
                title: "add per-stage hardening playbooks".to_string(),
                stage_ids: vec!["fastq.validate_reads".to_string()],
                scenarios: Vec::new(),
                matched_rows: vec![HardeningMatrixRowRef {
                    row_id: "h1".to_string(),
                    stage_id: "fastq.validate_reads".to_string(),
                    tool_id: "tool_v1".to_string(),
                    readiness_class: "ready".to_string(),
                    repetitions: 3,
                }],
                appraisal_findings: Vec::new(),
                hardening_entries: Vec::new(),
                goal_checks: Vec::new(),
                status: "ready-for-benchmark-run".to_string(),
            },
            super::HardeningBenchmarkGoalEntry {
                goal_id: "G172".to_string(),
                title: "add per-tool hardening playbooks".to_string(),
                stage_ids: vec!["fastq.trim_reads".to_string()],
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
