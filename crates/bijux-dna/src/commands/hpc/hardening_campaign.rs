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

const G191_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sprint-top-risk-selection",
        focus: "select top-risk hardening items for sprint generation",
    },
    ScenarioDefinition {
        scenario_id: "sprint-bookkeeping-exclusion",
        focus: "exclude bookkeeping-only items from sprint backlog",
    },
    ScenarioDefinition {
        scenario_id: "sprint-output-proof",
        focus: "capture sprint output evidence from hardening queue",
    },
];

const G192_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "ambiguous-finding-detection",
        focus: "detect weak or conflicting findings for manual review",
    },
    ScenarioDefinition {
        scenario_id: "manual-review-routing",
        focus: "route ambiguous findings into manual review queue",
    },
    ScenarioDefinition {
        scenario_id: "review-queue-proof",
        focus: "capture manual review queue evidence",
    },
];

const G193_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "upstream-tool-suspect-detection",
        focus: "detect suspected upstream tool behavior from appraisals",
    },
    ScenarioDefinition {
        scenario_id: "upstream-tool-bundle-selection",
        focus: "select minimal rows for encrypted upstream tool issue bundles",
    },
    ScenarioDefinition {
        scenario_id: "upstream-tool-bundle-proof",
        focus: "capture issue bundle readiness proof for upstream reports",
    },
];

const G194_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "data-issue-suspect-detection",
        focus: "detect suspected corpus or database issue findings",
    },
    ScenarioDefinition {
        scenario_id: "data-issue-bundle-selection",
        focus: "select minimal rows for encrypted data issue bundles",
    },
    ScenarioDefinition {
        scenario_id: "data-issue-bundle-proof",
        focus: "capture data issue bundle readiness proof",
    },
];

const G195_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "runtime-issue-suspect-detection",
        focus: "detect runtime and wrapper issue findings",
    },
    ScenarioDefinition {
        scenario_id: "runtime-issue-bundle-selection",
        focus: "select minimal rows for encrypted runtime issue bundles",
    },
    ScenarioDefinition {
        scenario_id: "runtime-issue-bundle-proof",
        focus: "capture runtime issue bundle readiness proof",
    },
];

const G196_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "partial-campaign-evidence-selection",
        focus: "extract valid evidence from partial campaigns",
    },
    ScenarioDefinition {
        scenario_id: "partial-campaign-noncompletion-guard",
        focus: "guard against treating partial campaign as complete",
    },
    ScenarioDefinition {
        scenario_id: "salvage-workflow-proof",
        focus: "capture campaign salvage workflow evidence",
    },
];

const G197_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "confidence-band-detection",
        focus: "label findings into strong, weak, and manual-review-required bands",
    },
    ScenarioDefinition {
        scenario_id: "confidence-score-routing",
        focus: "route weak findings to review while preserving strong findings",
    },
    ScenarioDefinition {
        scenario_id: "confidence-proof",
        focus: "capture confidence scoring evidence in hardening outputs",
    },
];

const G198_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "history-trend-anchor-selection",
        focus: "anchor stage/tool/corpus/db/image/runtime trend evidence",
    },
    ScenarioDefinition {
        scenario_id: "history-analytics-signal-coverage",
        focus: "cover degraded and critical trend signals for analytics",
    },
    ScenarioDefinition {
        scenario_id: "history-report-proof",
        focus: "capture benchmark history analytics report evidence",
    },
];

const G199_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "stage-risk-severity-selection",
        focus: "rank stage risk by severity and failure frequency",
    },
    ScenarioDefinition {
        scenario_id: "stage-risk-priority-ordering",
        focus: "order stage hardening priorities from risk signals",
    },
    ScenarioDefinition {
        scenario_id: "stage-risk-proof",
        focus: "capture stage-risk ranking report evidence",
    },
];

const G200_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "acceptance-rerun-anchor-selection",
        focus: "select exact failing and neighboring rows for acceptance rerun",
    },
    ScenarioDefinition {
        scenario_id: "acceptance-rerun-before-after-guard",
        focus: "require before/after evidence for acceptance reruns",
    },
    ScenarioDefinition {
        scenario_id: "acceptance-rerun-proof",
        focus: "capture hardening acceptance rerun evidence",
    },
];

const G201_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "modern-wgs-input-baseline",
        focus: "validate modern WGS paired-read, BAM, and VCF baseline readiness",
    },
    ScenarioDefinition {
        scenario_id: "modern-wgs-reference-known-sites",
        focus: "validate modern WGS reference and known-sites coverage requirements",
    },
    ScenarioDefinition {
        scenario_id: "modern-wgs-campaign-proof",
        focus: "capture modern WGS campaign coverage evidence",
    },
];

const G202_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "adna-damage-and-contamination-signals",
        focus: "validate aDNA damage/degraded read and contamination signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "adna-low-endogenous-readiness",
        focus: "validate low endogenous-content handling and caveat propagation",
    },
    ScenarioDefinition {
        scenario_id: "adna-appraisal-proof",
        focus: "capture aDNA appraisal evidence from benchmark rows",
    },
];

const G203_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "edna-primer-taxonomy-boundaries",
        focus: "validate primer-rich and taxonomy boundary readiness for eDNA workflows",
    },
    ScenarioDefinition {
        scenario_id: "edna-abundance-caveat-signals",
        focus: "validate abundance caveat and contamination signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "edna-campaign-proof",
        focus: "capture eDNA campaign appraisal evidence",
    },
];

const G204_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "low-pass-gl-impute-boundaries",
        focus: "validate low-pass GL/imputation boundary readiness",
    },
    ScenarioDefinition {
        scenario_id: "low-pass-population-signal-coverage",
        focus: "validate low-pass population signal coverage and caveat flow",
    },
    ScenarioDefinition {
        scenario_id: "low-pass-campaign-proof",
        focus: "capture low-pass campaign evidence",
    },
];

const G205_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "contamination-rich-readiness",
        focus: "validate contaminant-rich row readiness and refusal behavior",
    },
    ScenarioDefinition {
        scenario_id: "contamination-caveat-propagation",
        focus: "validate contamination caveat propagation across BAM and VCF surfaces",
    },
    ScenarioDefinition {
        scenario_id: "contamination-campaign-proof",
        focus: "capture contamination campaign appraisal evidence",
    },
];

const G206_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "nonhuman-reference-context-boundary",
        focus: "validate non-human reference context and species/build constraints",
    },
    ScenarioDefinition {
        scenario_id: "nonhuman-panel-compatibility-signals",
        focus: "validate panel/reference compatibility signals for non-human flows",
    },
    ScenarioDefinition {
        scenario_id: "nonhuman-appraisal-proof",
        focus: "capture non-human reference appraisal evidence",
    },
];

const G207_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "microbial-reference-annotation-boundary",
        focus: "validate microbial reference and annotation boundary readiness",
    },
    ScenarioDefinition {
        scenario_id: "microbial-variant-taxonomy-signals",
        focus: "validate microbial variant and taxonomy signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "microbial-campaign-proof",
        focus: "capture microbial campaign appraisal evidence",
    },
];

const G208_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sv-boundary-support-refusal",
        focus: "validate structural-variant support/refusal boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "sv-evidence-caveat-signals",
        focus: "validate SV evidence and caveat propagation signals",
    },
    ScenarioDefinition {
        scenario_id: "sv-appraisal-proof",
        focus: "capture SV boundary appraisal evidence",
    },
];

const G209_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "cnv-coverage-boundary",
        focus: "validate CNV coverage boundary support/refusal behavior",
    },
    ScenarioDefinition {
        scenario_id: "cnv-variant-caveat-signals",
        focus: "validate CNV variant and caveat signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "cnv-appraisal-proof",
        focus: "capture CNV boundary appraisal evidence",
    },
];

const G210_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "cram-reference-requirement-boundary",
        focus: "validate CRAM reference requirement and missing-reference refusal behavior",
    },
    ScenarioDefinition {
        scenario_id: "cram-io-runtime-signals",
        focus: "validate CRAM IO/runtime signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "cram-appraisal-proof",
        focus: "capture CRAM boundary appraisal evidence",
    },
];

const G211_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "long-read-fastq-bam-boundary",
        focus: "validate long-read FASTQ and BAM boundary readiness semantics",
    },
    ScenarioDefinition {
        scenario_id: "long-read-support-refusal-paths",
        focus: "validate explicit support/refusal semantics for long-read boundary runs",
    },
    ScenarioDefinition {
        scenario_id: "long-read-appraisal-proof",
        focus: "capture long-read appraisal evidence",
    },
];

const G212_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sample-swap-metadata-genotype-boundary",
        focus: "validate sample-swap metadata and genotype conflict boundaries",
    },
    ScenarioDefinition {
        scenario_id: "sample-swap-sex-coverage-conflict-signals",
        focus: "validate sex and coverage conflict signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "sample-swap-appraisal-proof",
        focus: "capture sample-swap appraisal evidence",
    },
];

const G213_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "duplicate-cohort-cross-batch-boundary",
        focus: "validate duplicate-cohort boundaries across repeated and related samples",
    },
    ScenarioDefinition {
        scenario_id: "duplicate-cohort-consistency-signals",
        focus: "validate cohort consistency signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "duplicate-cohort-appraisal-proof",
        focus: "capture duplicate-cohort appraisal evidence",
    },
];

const G214_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "panel-choice-imputation-map-boundary",
        focus: "validate imputation panel and genetic-map boundary readiness",
    },
    ScenarioDefinition {
        scenario_id: "panel-choice-sensitivity-signals",
        focus: "validate panel-choice sensitivity signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "panel-choice-appraisal-proof",
        focus: "capture panel-choice appraisal evidence",
    },
];

const G215_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "annotation-drift-source-update-boundary",
        focus: "validate annotation source update boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "annotation-drift-impact-signals",
        focus: "validate annotation-drift impact signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "annotation-drift-appraisal-proof",
        focus: "capture annotation-drift appraisal evidence",
    },
];

const G216_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "qc-threshold-near-boundary-cases",
        focus: "validate near-threshold FASTQ/BAM/VCF boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "qc-threshold-caveat-sensitivity-signals",
        focus: "validate threshold-sensitivity caveat signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "qc-threshold-appraisal-proof",
        focus: "capture QC-threshold appraisal evidence",
    },
];

const G217_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "artifact-minimization-large-output-boundary",
        focus: "validate large-output reviewer-bundle minimization boundaries",
    },
    ScenarioDefinition {
        scenario_id: "artifact-minimization-bundle-size-signals",
        focus: "validate bundle-size minimization signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "artifact-minimization-appraisal-proof",
        focus: "capture artifact-minimization appraisal evidence",
    },
];

const G218_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "shared-storage-io-bottleneck-boundary",
        focus: "validate shared-storage bottleneck and scratch-staging boundaries",
    },
    ScenarioDefinition {
        scenario_id: "shared-storage-io-stress-signals",
        focus: "validate shared-storage IO stress signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "shared-storage-io-appraisal-proof",
        focus: "capture shared-storage IO appraisal evidence",
    },
];

const G219_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "database-build-input-benchmark-boundary",
        focus: "validate reference/index/taxonomy/panel input benchmark boundaries",
    },
    ScenarioDefinition {
        scenario_id: "database-build-runtime-size-signals",
        focus: "validate database-build runtime and size signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "database-build-appraisal-proof",
        focus: "capture database-build appraisal evidence",
    },
];

const G220_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "truth-set-anchor-legality-boundary",
        focus: "validate truth-set anchored boundaries for meaningful and legal usage",
    },
    ScenarioDefinition {
        scenario_id: "truth-set-caveat-coverage-signals",
        focus: "validate truth-set caveat and coverage signal flow",
    },
    ScenarioDefinition {
        scenario_id: "truth-set-appraisal-proof",
        focus: "capture truth-set anchored appraisal evidence",
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
    GoalDefinition {
        goal_id: "G191",
        title: "add hardening sprint generator",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.filter"],
        scenarios: G191_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G192",
        title: "add ambiguous-result review queue",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.filter"],
        scenarios: G192_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G193",
        title: "add suspected upstream tool issue bundle",
        stage_ids: &["fastq.trim_reads", "bam.align", "vcf.call"],
        scenarios: G193_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G194",
        title: "add suspected data issue bundle",
        stage_ids: &["fastq.validate_reads", "bam.contamination", "vcf.filter"],
        scenarios: G194_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G195",
        title: "add suspected runtime issue bundle",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G195_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G196",
        title: "add campaign salvage workflow",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.filter"],
        scenarios: G196_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G197",
        title: "add appraiser confidence scores",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.stats"],
        scenarios: G197_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G198",
        title: "add benchmark history analytics",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.impute"],
        scenarios: G198_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G199",
        title: "add stage-risk ranking",
        stage_ids: &["fastq.trim_reads", "bam.coverage", "vcf.filter"],
        scenarios: G199_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G200",
        title: "add hardening acceptance rerun",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.filter"],
        scenarios: G200_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G201",
        title: "build modern WGS benchmark corpus",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.call"],
        scenarios: G201_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G202",
        title: "build ancient-DNA benchmark corpus",
        stage_ids: &["fastq.trim_reads", "bam.contamination", "vcf.filter"],
        scenarios: G202_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G203",
        title: "build eDNA/metabarcoding benchmark corpus",
        stage_ids: &["fastq.profile_reads", "bam.contamination", "vcf.stats"],
        scenarios: G203_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G204",
        title: "build low-pass population benchmark corpus",
        stage_ids: &["fastq.profile_reads", "vcf.impute", "vcf.stats"],
        scenarios: G204_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G205",
        title: "build contamination-heavy benchmark corpus",
        stage_ids: &["fastq.validate_reads", "bam.contamination", "vcf.filter"],
        scenarios: G205_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G206",
        title: "build non-human reference benchmark corpus",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.postprocess"],
        scenarios: G206_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G207",
        title: "build microbial benchmark corpus",
        stage_ids: &["fastq.trim_reads", "bam.align", "vcf.call"],
        scenarios: G207_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G208",
        title: "build structural-variant boundary corpus",
        stage_ids: &["bam.coverage", "vcf.call", "vcf.filter"],
        scenarios: G208_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G209",
        title: "build CNV boundary corpus",
        stage_ids: &["bam.coverage", "vcf.filter", "vcf.stats"],
        scenarios: G209_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G210",
        title: "build CRAM boundary corpus",
        stage_ids: &["bam.validate", "bam.mapping_summary", "vcf.filter"],
        scenarios: G210_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G211",
        title: "build long-read boundary corpus",
        stage_ids: &["fastq.validate_reads", "bam.align", "bam.mapping_summary"],
        scenarios: G211_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G212",
        title: "build sample-swap corpus",
        stage_ids: &["fastq.profile_reads", "bam.validate", "vcf.stats"],
        scenarios: G212_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G213",
        title: "build duplicate-cohort corpus",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.stats"],
        scenarios: G213_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G214",
        title: "build panel-choice sensitivity corpus",
        stage_ids: &["vcf.impute", "vcf.call", "vcf.stats"],
        scenarios: G214_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G215",
        title: "build annotation-drift corpus",
        stage_ids: &["vcf.call", "vcf.filter", "vcf.postprocess"],
        scenarios: G215_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G216",
        title: "build QC-threshold sensitivity corpus",
        stage_ids: &["fastq.trim_reads", "bam.coverage", "vcf.filter"],
        scenarios: G216_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G217",
        title: "build artifact-minimization corpus",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.postprocess"],
        scenarios: G217_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G218",
        title: "build shared-storage IO stress corpus",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G218_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G219",
        title: "build database-build benchmark corpus",
        stage_ids: &["bam.align", "vcf.call", "vcf.impute"],
        scenarios: G219_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G220",
        title: "add truth-set anchored corpus mode",
        stage_ids: &["fastq.validate_reads", "vcf.call", "vcf.filter"],
        scenarios: G220_SCENARIOS,
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
        "G181" => vec![
            format!("nondeterminism_rows_present={}", !rows.is_empty()),
            format!(
                "nondeterminism_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "nondeterminism_profile_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "nondeterminism_repeat_risk_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "nondeterminism_repro_runtime_findings={}",
                findings
                    .iter()
                    .filter(|finding| {
                        finding.failure_class.starts_with("runtime-")
                            || finding.failure_class == "reproducibility-low-repeats"
                    })
                    .count()
            ),
        ],
        "G182" => vec![
            format!("output_schema_rows_present={}", !rows.is_empty()),
            format!(
                "output_schema_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "output_schema_validate_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "output_schema_code_freeze_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "code-freeze-incomplete")
                    .count()
            ),
            format!(
                "output_schema_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
        ],
        "G183" => vec![
            format!("caveat_hardening_rows_present={}", !rows.is_empty()),
            format!(
                "caveat_hardening_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "caveat_hardening_contam_filter_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.contamination")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "caveat_hardening_scientific_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "scientific-invalidity")
                    .count()
            ),
            format!(
                "caveat_hardening_corpus_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "corpus-mismatch")
                    .count()
            ),
        ],
        "G184" => vec![
            format!("failure_taxonomy_rows_present={}", !rows.is_empty()),
            format!(
                "failure_taxonomy_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "failure_taxonomy_class_count={}",
                findings
                    .iter()
                    .map(|finding| finding.failure_class.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "failure_taxonomy_runtime_tool_science_classes={}",
                findings.iter().any(|finding| finding.failure_class.starts_with("runtime-"))
                    && findings.iter().any(|finding| finding.failure_class == "missing-tool-binding")
                    && findings.iter().any(|finding| finding.failure_class == "scientific-invalidity")
            ),
            format!(
                "failure_taxonomy_queue_class_count={}",
                queue_entries
                    .iter()
                    .map(|entry| entry.failure_class.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
        ],
        "G185" => vec![
            format!("benchmark_minimizer_rows_present={}", !rows.is_empty()),
            format!(
                "benchmark_minimizer_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "benchmark_minimizer_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "benchmark_minimizer_critical_findings={}",
                findings.iter().filter(|finding| finding.severity == "critical").count()
            ),
            format!(
                "benchmark_minimizer_critical_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity == "critical")
                    .count()
            ),
        ],
        "G186" => vec![
            format!("investigation_workspace_rows_present={}", !rows.is_empty()),
            format!(
                "investigation_workspace_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "investigation_workspace_profile_summary_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "investigation_workspace_code_scope_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.result_scope == "encrypted-code")
                    .count()
            ),
            format!(
                "investigation_workspace_queue_entries={}",
                queue_entries.len()
            ),
        ],
        "G187" => vec![
            format!("result_diff_rows_present={}", !rows.is_empty()),
            format!(
                "result_diff_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "result_diff_profile_coverage_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "result_diff_encrypted_result_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.result_scope == "encrypted-results")
                    .count()
            ),
            format!(
                "result_diff_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
        ],
        "G188" => vec![
            format!("code_diff_rows_present={}", !rows.is_empty()),
            format!(
                "code_diff_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "code_diff_trim_summary_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "code_diff_code_scope_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.result_scope == "encrypted-code")
                    .count()
            ),
            format!(
                "code_diff_code_freeze_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.source_appraisers.iter().any(|id| id == "code-freeze"))
                    .count()
            ),
        ],
        "G189" => vec![
            format!("baseline_workflow_rows_present={}", !rows.is_empty()),
            format!(
                "baseline_workflow_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "baseline_workflow_validate_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "baseline_workflow_critical_findings={}",
                findings.iter().filter(|finding| finding.severity == "critical").count()
            ),
            format!(
                "baseline_workflow_noncritical_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "critical")
                    .count()
            ),
        ],
        "G190" => vec![
            format!("regression_workflow_rows_present={}", !rows.is_empty()),
            format!(
                "regression_workflow_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "regression_workflow_profile_coverage_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "regression_workflow_critical_findings={}",
                findings.iter().filter(|finding| finding.severity == "critical").count()
            ),
            format!(
                "regression_workflow_non_info_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "info")
                    .count()
            ),
        ],
        "G191" => vec![
            format!("sprint_generator_rows_present={}", !rows.is_empty()),
            format!(
                "sprint_generator_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "sprint_generator_high_risk_findings={}",
                findings.iter().filter(|finding| finding.severity == "critical").count()
            ),
            format!(
                "sprint_generator_non_bookkeeping_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class != "code-freeze-incomplete")
                    .count()
            ),
            format!(
                "sprint_generator_profile_coverage_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
        ],
        "G192" => vec![
            format!("ambiguous_review_rows_present={}", !rows.is_empty()),
            format!(
                "ambiguous_review_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "ambiguous_review_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "ambiguous_review_warning_findings={}",
                findings.iter().filter(|finding| finding.severity == "warning").count()
            ),
            format!(
                "ambiguous_review_queue_warning_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity == "warning")
                    .count()
            ),
        ],
        "G193" => vec![
            format!("upstream_tool_bundle_rows_present={}", !rows.is_empty()),
            format!(
                "upstream_tool_bundle_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "upstream_tool_bundle_trim_align_call_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "upstream_tool_bundle_tool_binding_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "missing-tool-binding")
                    .count()
            ),
            format!(
                "upstream_tool_bundle_queue_non_info={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "info")
                    .count()
            ),
        ],
        "G194" => vec![
            format!("data_issue_bundle_rows_present={}", !rows.is_empty()),
            format!(
                "data_issue_bundle_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "data_issue_bundle_validate_contam_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.contamination")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "data_issue_bundle_corpus_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "corpus-mismatch")
                    .count()
            ),
            format!(
                "data_issue_bundle_security_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness") || entry.failure_class == "corpus-mismatch")
                    .count()
            ),
        ],
        "G195" => vec![
            format!("runtime_issue_bundle_rows_present={}", !rows.is_empty()),
            format!(
                "runtime_issue_bundle_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "runtime_issue_bundle_profile_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "runtime_issue_bundle_runtime_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class.starts_with("runtime-"))
                    .count()
            ),
            format!(
                "runtime_issue_bundle_runtime_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.starts_with("runtime-"))
                    .count()
            ),
        ],
        "G196" => vec![
            format!("campaign_salvage_rows_present={}", !rows.is_empty()),
            format!(
                "campaign_salvage_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "campaign_salvage_partial_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "campaign_salvage_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "campaign_salvage_noncritical_findings={}",
                findings.iter().filter(|finding| finding.severity != "critical").count()
            ),
        ],
        "G197" => vec![
            format!("confidence_scores_rows_present={}", !rows.is_empty()),
            format!(
                "confidence_scores_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "confidence_scores_high_confidence_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.confidence == "high")
                    .count()
            ),
            format!(
                "confidence_scores_medium_confidence_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.confidence == "medium")
                    .count()
            ),
            format!(
                "confidence_scores_manual_review_proxy={}",
                findings
                    .iter()
                    .any(|finding| finding.confidence == "medium" && finding.severity == "warning")
            ),
        ],
        "G198" => vec![
            format!("history_analytics_rows_present={}", !rows.is_empty()),
            format!(
                "history_analytics_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "history_analytics_profile_coverage_impute_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.impute")
            ),
            format!(
                "history_analytics_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!(
                "history_analytics_unique_failure_classes={}",
                findings
                    .iter()
                    .map(|finding| finding.failure_class.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
        ],
        "G199" => vec![
            format!("stage_risk_ranking_rows_present={}", !rows.is_empty()),
            format!(
                "stage_risk_ranking_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "stage_risk_ranking_trim_coverage_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "stage_risk_ranking_critical_findings={}",
                findings.iter().filter(|finding| finding.severity == "critical").count()
            ),
            format!(
                "stage_risk_ranking_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
        ],
        "G200" => vec![
            format!("acceptance_rerun_rows_present={}", !rows.is_empty()),
            format!(
                "acceptance_rerun_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "acceptance_rerun_validate_align_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "acceptance_rerun_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "acceptance_rerun_before_after_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "info")
                    .count()
            ),
        ],
        "G201" => vec![
            format!("modern_wgs_rows_present={}", !rows.is_empty()),
            format!(
                "modern_wgs_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "modern_wgs_validate_align_call_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "modern_wgs_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "modern_wgs_findings_count={}",
                findings.len()
            ),
        ],
        "G202" => vec![
            format!("adna_corpus_rows_present={}", !rows.is_empty()),
            format!(
                "adna_corpus_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "adna_corpus_trim_contam_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.contamination")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "adna_corpus_contamination_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class.contains("contamination") || finding.failure_class == "readiness-refuse")
                    .count()
            ),
            format!("adna_corpus_queue_entries={}", queue_entries.len()),
        ],
        "G203" => vec![
            format!("edna_corpus_rows_present={}", !rows.is_empty()),
            format!(
                "edna_corpus_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "edna_corpus_profile_contam_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.contamination")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "edna_corpus_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "edna_corpus_warning_findings={}",
                findings.iter().filter(|finding| finding.severity == "warning").count()
            ),
        ],
        "G204" => vec![
            format!("lowpass_corpus_rows_present={}", !rows.is_empty()),
            format!(
                "lowpass_corpus_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "lowpass_corpus_profile_impute_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "vcf.impute")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "lowpass_corpus_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "lowpass_corpus_findings_count={}",
                findings.len()
            ),
        ],
        "G205" => vec![
            format!("contamination_heavy_rows_present={}", !rows.is_empty()),
            format!(
                "contamination_heavy_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "contamination_heavy_validate_contam_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.contamination")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "contamination_heavy_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "contamination_heavy_queue_entries={}",
                queue_entries.len()
            ),
        ],
        "G206" => vec![
            format!("nonhuman_corpus_rows_present={}", !rows.is_empty()),
            format!(
                "nonhuman_corpus_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "nonhuman_corpus_validate_align_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "nonhuman_corpus_code_scope_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.result_scope == "encrypted-code")
                    .count()
            ),
            format!(
                "nonhuman_corpus_findings_count={}",
                findings.len()
            ),
        ],
        "G207" => vec![
            format!("microbial_corpus_rows_present={}", !rows.is_empty()),
            format!(
                "microbial_corpus_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "microbial_corpus_trim_align_call_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "microbial_corpus_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "microbial_corpus_findings_count={}",
                findings.len()
            ),
        ],
        "G208" => vec![
            format!("sv_boundary_rows_present={}", !rows.is_empty()),
            format!(
                "sv_boundary_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "sv_boundary_coverage_call_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "sv_boundary_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "sv_boundary_non_info_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "info")
                    .count()
            ),
        ],
        "G209" => vec![
            format!("cnv_boundary_rows_present={}", !rows.is_empty()),
            format!(
                "cnv_boundary_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "cnv_boundary_coverage_filter_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "cnv_boundary_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!("cnv_boundary_findings_count={}", findings.len()),
        ],
        "G210" => vec![
            format!("cram_boundary_rows_present={}", !rows.is_empty()),
            format!(
                "cram_boundary_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "cram_boundary_validate_summary_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "cram_boundary_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "cram_boundary_queue_entries={}",
                queue_entries.len()
            ),
        ],
        "G211" => vec![
            format!("long_read_boundary_rows_present={}", !rows.is_empty()),
            format!(
                "long_read_boundary_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "long_read_boundary_validate_align_summary_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
            ),
            format!(
                "long_read_boundary_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!("long_read_boundary_findings_count={}", findings.len()),
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
    fn hardening_catalog_includes_iteration_19_goals() {
        assert_eq!(HARDENING_GOALS_CATALOG.len(), 50);
        assert_eq!(HARDENING_GOALS_CATALOG[0].goal_id, "G171");
        assert_eq!(HARDENING_GOALS_CATALOG[49].goal_id, "G220");
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
    fn goal_181_emits_nondeterminism_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G181".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h6".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "low repeats for reproducibility".to_string(),
            recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "nondeterminism_profile_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "nondeterminism_repro_runtime_findings=1"));
    }

    #[test]
    fn goal_182_emits_output_schema_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G182".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "schema bundle missing lock metadata".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "output_schema_validate_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "output_schema_code_freeze_findings=1"));
    }

    #[test]
    fn goal_183_emits_caveat_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G183".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "scientific-invalidity".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "invalid scientific readiness for caveat outputs".to_string(),
            recommendation: "resolve readiness mismatches before scientific evaluation".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "caveat_hardening_contam_filter_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "caveat_hardening_scientific_findings=1"));
    }

    #[test]
    fn goal_184_emits_failure_taxonomy_hardening_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G184".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "runtime-performance".to_string(),
                row_id: "h5".to_string(),
                severity: "warning".to_string(),
                confidence: "medium".to_string(),
                failure_class: "runtime-under-sampled".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "runtime under sampled".to_string(),
                recommendation: "increase repetitions to at least 2".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "artifact-validity".to_string(),
                row_id: "h5".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "missing-tool-binding".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "missing tool binding".to_string(),
                recommendation: "bind stage to at least one governed tool".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "scientific-output".to_string(),
                row_id: "h10".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "scientific-invalidity".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "scientific invalidity".to_string(),
                recommendation: "resolve readiness mismatches before scientific evaluation".to_string(),
            },
        ];
        let queue = vec![
            HardeningQueueEntry {
                queue_id: "hardening-0201".to_string(),
                severity: "critical".to_string(),
                failure_class: "missing-tool-binding".to_string(),
                recommendation: "bind stage to at least one governed tool".to_string(),
                affected_rows: vec!["h5".to_string()],
                source_appraisers: vec!["artifact-validity".to_string()],
            },
            HardeningQueueEntry {
                queue_id: "hardening-0202".to_string(),
                severity: "warning".to_string(),
                failure_class: "runtime-under-sampled".to_string(),
                recommendation: "increase repetitions to at least 2".to_string(),
                affected_rows: vec!["h5".to_string()],
                source_appraisers: vec!["runtime-performance".to_string()],
            },
        ];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "failure_taxonomy_runtime_tool_science_classes=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "failure_taxonomy_queue_class_count=2"));
    }

    #[test]
    fn goal_185_emits_benchmark_minimizer_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G185".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "refuse row available for minimizer anchoring".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0301".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "benchmark_minimizer_refuse_rows=1"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "benchmark_minimizer_critical_queue_entries=1"));
    }

    #[test]
    fn goal_186_emits_local_investigation_workspace_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G186".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "local workspace indexing missing code lock metadata".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0401".to_string(),
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
            .any(|check| check == "investigation_workspace_profile_summary_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "investigation_workspace_code_scope_findings=1"));
    }

    #[test]
    fn goal_187_emits_result_diff_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G187".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "result diff candidate row is degraded".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "result_diff_profile_coverage_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "result_diff_encrypted_result_findings=1"));
    }

    #[test]
    fn goal_188_emits_code_diff_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G188".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "code bundle diff reveals missing lock metadata".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0501".to_string(),
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
            .any(|check| check == "code_diff_trim_summary_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "code_diff_code_freeze_queue_entries=1"));
    }

    #[test]
    fn goal_189_emits_accepted_baseline_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G189".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h1".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "baseline candidate requires repeat strengthening".to_string(),
            recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0601".to_string(),
            severity: "warning".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
            affected_rows: vec!["h1".to_string()],
            source_appraisers: vec!["reproducibility".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "baseline_workflow_validate_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "baseline_workflow_critical_findings=0"));
    }

    #[test]
    fn goal_190_emits_regression_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G190".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "h10".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "readiness-refuse".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "regression rerun surfaced refuse row".to_string(),
                recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "code-freeze".to_string(),
                row_id: "h13".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "code-freeze-incomplete".to_string(),
                result_scope: "encrypted-code".to_string(),
                summary: "regression rerun surfaced replay/code drift".to_string(),
                recommendation: "bind tool and image lock before code freeze publication".to_string(),
            },
        ];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0701".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            affected_rows: vec!["h10".to_string(), "h13".to_string()],
            source_appraisers: vec!["failure-class".to_string(), "code-freeze".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "regression_workflow_profile_coverage_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "regression_workflow_critical_findings=1"));
    }

    #[test]
    fn goal_191_emits_hardening_sprint_generator_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G191".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "high-risk row selected for sprint".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0801".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "sprint_generator_high_risk_findings=1"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "sprint_generator_non_bookkeeping_entries=1"));
    }

    #[test]
    fn goal_192_emits_ambiguous_review_queue_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G192".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "ambiguous warning routed for manual review".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0901".to_string(),
            severity: "warning".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
            affected_rows: vec!["h5".to_string()],
            source_appraisers: vec!["runtime-performance".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "ambiguous_review_warning_findings=1"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "ambiguous_review_queue_warning_entries=1"));
    }

    #[test]
    fn goal_193_emits_upstream_tool_issue_bundle_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G193".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "artifact-validity".to_string(),
            row_id: "h5".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "missing-tool-binding".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "suspected upstream tool issue due to tool binding break".to_string(),
            recommendation: "bind stage to at least one governed tool".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1001".to_string(),
            severity: "critical".to_string(),
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
            .any(|check| check == "upstream_tool_bundle_trim_align_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "upstream_tool_bundle_tool_binding_findings=1"));
    }

    #[test]
    fn goal_194_emits_data_issue_bundle_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G194".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "corpus-suitability".to_string(),
            row_id: "h10".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "corpus-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "suspected data issue from corpus mismatch".to_string(),
            recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1101".to_string(),
            severity: "warning".to_string(),
            failure_class: "corpus-mismatch".to_string(),
            recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["corpus-suitability".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "data_issue_bundle_validate_contam_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "data_issue_bundle_corpus_findings=1"));
    }

    #[test]
    fn goal_195_emits_runtime_issue_bundle_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G195".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "h6".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "suspected runtime issue from under-sampled row".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
        }];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1201".to_string(),
            severity: "warning".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
            affected_rows: vec!["h6".to_string()],
            source_appraisers: vec!["runtime-performance".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "runtime_issue_bundle_profile_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "runtime_issue_bundle_runtime_findings=1"));
    }

    #[test]
    fn goal_196_emits_campaign_salvage_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G196".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-under-sampled".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "partial campaign row preserved for salvage".to_string(),
            recommendation: "increase repetitions to at least 2".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_salvage_partial_rows=3"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_salvage_noncritical_findings=1"));
    }

    #[test]
    fn goal_197_emits_appraiser_confidence_score_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G197".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "h1".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "readiness-refuse".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "strong confidence finding".to_string(),
                recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "runtime-performance".to_string(),
                row_id: "h4".to_string(),
                severity: "warning".to_string(),
                confidence: "medium".to_string(),
                failure_class: "runtime-under-sampled".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "weak confidence finding".to_string(),
                recommendation: "increase repetitions to at least 2".to_string(),
            },
        ];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "confidence_scores_high_confidence_findings=1"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "confidence_scores_manual_review_proxy=true"));
    }

    #[test]
    fn goal_198_emits_benchmark_history_analytics_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G198".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "h12".to_string(),
                severity: "warning".to_string(),
                confidence: "medium".to_string(),
                failure_class: "readiness-degraded".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "history trend: degraded imputation row".to_string(),
                recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "code-freeze".to_string(),
                row_id: "h12".to_string(),
                severity: "critical".to_string(),
                confidence: "high".to_string(),
                failure_class: "code-freeze-incomplete".to_string(),
                result_scope: "encrypted-code".to_string(),
                summary: "history trend: code drift in imputation branch".to_string(),
                recommendation: "bind tool and image lock before code freeze publication".to_string(),
            },
        ];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "history_analytics_profile_coverage_impute_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "history_analytics_unique_failure_classes=2"));
    }

    #[test]
    fn goal_199_emits_stage_risk_ranking_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G199".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "stage risk ranking anchor from refuse row".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "stage_risk_ranking_trim_coverage_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "stage_risk_ranking_refuse_rows=1"));
    }

    #[test]
    fn goal_200_emits_hardening_acceptance_rerun_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G200".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1301".to_string(),
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
            .any(|check| check == "acceptance_rerun_validate_align_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "acceptance_rerun_before_after_queue_entries=1"));
    }

    #[test]
    fn goal_201_emits_modern_wgs_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G201".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "modern WGS readiness needs tuning".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "modern_wgs_validate_align_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "modern_wgs_findings_count=1"));
    }

    #[test]
    fn goal_202_emits_adna_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G202".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "aDNA boundary triggers refuse contamination-ready path".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "adna_corpus_trim_contam_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "adna_corpus_contamination_findings=1"));
    }

    #[test]
    fn goal_203_emits_edna_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G203".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "corpus-suitability".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "corpus-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "eDNA corpus boundary warning".to_string(),
            recommendation: "materialize corpus profile matching stage scientific claim".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "edna_corpus_profile_contam_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "edna_corpus_warning_findings=1"));
    }

    #[test]
    fn goal_204_emits_lowpass_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G204".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h12".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "low-pass imputation needs repeat reinforcement".to_string(),
            recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "lowpass_corpus_profile_impute_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "lowpass_corpus_findings_count=1"));
    }

    #[test]
    fn goal_205_emits_contamination_heavy_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G205".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1401".to_string(),
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
            .any(|check| check == "contamination_heavy_validate_contam_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "contamination_heavy_queue_entries=1"));
    }

    #[test]
    fn goal_206_emits_nonhuman_reference_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G206".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "non-human reference bundle needs lock metadata".to_string(),
            recommendation: "bind tool and image lock before code freeze publication".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "nonhuman_corpus_validate_align_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "nonhuman_corpus_code_scope_findings=1"));
    }

    #[test]
    fn goal_207_emits_microbial_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G207".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h9".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "scientific-invalidity".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "microbial boundary advisory warning".to_string(),
            recommendation: "resolve readiness mismatches before scientific evaluation".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "microbial_corpus_trim_align_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "microbial_corpus_findings_count=1"));
    }

    #[test]
    fn goal_208_emits_sv_boundary_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G208".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1501".to_string(),
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
            .any(|check| check == "sv_boundary_coverage_call_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "sv_boundary_non_info_queue_entries=1"));
    }

    #[test]
    fn goal_209_emits_cnv_boundary_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G209".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "CNV boundary refuse anchor row".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "cnv_boundary_coverage_filter_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "cnv_boundary_findings_count=1"));
    }

    #[test]
    fn goal_210_emits_cram_boundary_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G210".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1601".to_string(),
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
            .any(|check| check == "cram_boundary_validate_summary_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "cram_boundary_queue_entries=1"));
    }

    #[test]
    fn goal_211_emits_long_read_boundary_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G211".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "long-read align stage is degraded".to_string(),
            recommendation: "resolve readiness reasons and re-run appraisal".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "long_read_boundary_validate_align_summary_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "long_read_boundary_findings_count=1"));
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
