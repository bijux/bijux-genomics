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

const G221_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "known-sites-filter-boundary",
        focus: "validate known-sites usage for filter and recalibration boundaries",
    },
    ScenarioDefinition {
        scenario_id: "known-sites-annotation-signal-coverage",
        focus: "validate known-sites annotation signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "known-sites-appraisal-proof",
        focus: "capture known-sites bundle appraisal evidence",
    },
];

const G222_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "imputation-panel-map-boundary",
        focus: "validate panel and map boundary compatibility for imputation runs",
    },
    ScenarioDefinition {
        scenario_id: "imputation-panel-trust-signals",
        focus: "validate trust labels and caveat signal coverage for panel bundles",
    },
    ScenarioDefinition {
        scenario_id: "imputation-panel-appraisal-proof",
        focus: "capture imputation panel appraisal evidence",
    },
];

const G223_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "taxonomy-variant-rank-boundary",
        focus: "validate taxonomy variant rank-resolution boundaries",
    },
    ScenarioDefinition {
        scenario_id: "taxonomy-bias-signal-coverage",
        focus: "validate taxonomy bias and drift signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "taxonomy-variant-appraisal-proof",
        focus: "capture taxonomy variant appraisal evidence",
    },
];

const G224_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "contaminant-variant-depletion-boundary",
        focus: "validate contaminant bundle variants for depletion boundaries",
    },
    ScenarioDefinition {
        scenario_id: "contaminant-profile-signal-coverage",
        focus: "validate contaminant profile signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "contaminant-variant-appraisal-proof",
        focus: "capture contaminant variant appraisal evidence",
    },
];

const G225_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "adapter-primer-platform-boundary",
        focus: "validate platform-specific adapter and primer boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "adapter-primer-edna-signal-coverage",
        focus: "validate trim and eDNA signal coverage across adapter/primer variants",
    },
    ScenarioDefinition {
        scenario_id: "adapter-primer-appraisal-proof",
        focus: "capture adapter/primer bundle appraisal evidence",
    },
];

const G226_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "annotation-db-variant-boundary",
        focus: "validate annotation database variant boundaries",
    },
    ScenarioDefinition {
        scenario_id: "annotation-db-functional-signal-coverage",
        focus: "validate functional annotation signal coverage across variants",
    },
    ScenarioDefinition {
        scenario_id: "annotation-db-appraisal-proof",
        focus: "capture annotation database appraisal evidence",
    },
];

const G227_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "corpus-db-compatibility-boundary",
        focus: "validate corpus/database compatibility matrix boundaries",
    },
    ScenarioDefinition {
        scenario_id: "corpus-db-tool-family-signal-coverage",
        focus: "validate compatibility signals across tool families",
    },
    ScenarioDefinition {
        scenario_id: "corpus-db-compatibility-appraisal-proof",
        focus: "capture compatibility matrix appraisal evidence",
    },
];

const G228_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "coverage-planner-stage-tool-boundary",
        focus: "validate campaign coverage planner stage/tool boundary selection",
    },
    ScenarioDefinition {
        scenario_id: "coverage-planner-corpus-db-signal-coverage",
        focus: "validate corpus and database coverage planning signals",
    },
    ScenarioDefinition {
        scenario_id: "coverage-planner-appraisal-proof",
        focus: "capture campaign coverage planner appraisal evidence",
    },
];

const G229_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "budget-planner-resource-boundary",
        focus: "validate budget planner CPU/memory/storage boundary accounting",
    },
    ScenarioDefinition {
        scenario_id: "budget-planner-encryption-appraiser-signals",
        focus: "validate encrypted output and appraiser cost signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "budget-planner-appraisal-proof",
        focus: "capture budget planner appraisal evidence",
    },
];

const G230_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-split-quota-boundary",
        focus: "validate campaign splitting boundaries by quota/account/partition",
    },
    ScenarioDefinition {
        scenario_id: "campaign-split-priority-window-signals",
        focus: "validate split-window and priority signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "campaign-split-appraisal-proof",
        focus: "capture campaign split appraisal evidence",
    },
];

const G231_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-lock-freeze-boundary",
        focus: "validate campaign reproducibility lock freeze boundaries",
    },
    ScenarioDefinition {
        scenario_id: "campaign-lock-asset-signal-coverage",
        focus: "validate lock coverage signals for corpus, DB, image, and code assets",
    },
    ScenarioDefinition {
        scenario_id: "campaign-lock-appraisal-proof",
        focus: "capture campaign lock appraisal evidence",
    },
];

const G232_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-rerun-lock-replay-boundary",
        focus: "validate rerun-from-lock replay boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "campaign-rerun-diff-signal-coverage",
        focus: "validate rerun diff signal coverage for selected updated assets",
    },
    ScenarioDefinition {
        scenario_id: "campaign-rerun-appraisal-proof",
        focus: "capture campaign rerun appraisal evidence",
    },
];

const G233_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "queue-subset-selector-boundary",
        focus: "validate hardening queue subset selection boundaries",
    },
    ScenarioDefinition {
        scenario_id: "queue-subset-validation-signal-coverage",
        focus: "validate queue subset coverage signals for hardening verification rows",
    },
    ScenarioDefinition {
        scenario_id: "queue-subset-appraisal-proof",
        focus: "capture queue subset appraisal evidence",
    },
];

const G234_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "priority-label-scientific-runtime-boundary",
        focus: "validate campaign priority labels across scientific and runtime boundaries",
    },
    ScenarioDefinition {
        scenario_id: "priority-label-hardening-value-signals",
        focus: "validate priority label signals for expected hardening value",
    },
    ScenarioDefinition {
        scenario_id: "priority-label-appraisal-proof",
        focus: "capture priority label appraisal evidence",
    },
];

const G235_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "warm-start-prepared-asset-boundary",
        focus: "validate warm-start campaign boundary with pre-locked assets",
    },
    ScenarioDefinition {
        scenario_id: "warm-start-preflight-signal-coverage",
        focus: "validate warm-start preflight signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "warm-start-appraisal-proof",
        focus: "capture warm-start campaign appraisal evidence",
    },
];

const G236_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "all-in-one-preflight-boundary",
        focus: "validate all-in-one preflight boundaries across required campaign surfaces",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-preflight-failure-signals",
        focus: "validate preflight refusal and warning signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-preflight-appraisal-proof",
        focus: "capture all-in-one preflight appraisal evidence",
    },
];

const G237_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "all-in-one-import-decrypt-boundary",
        focus: "validate all-in-one local import/decrypt boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-import-index-signal-coverage",
        focus: "validate local import indexing and appraisal signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-import-appraisal-proof",
        focus: "capture all-in-one local import appraisal evidence",
    },
];

const G238_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "all-in-one-report-summary-boundary",
        focus: "validate all-in-one report summary boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-report-link-signal-coverage",
        focus: "validate report link coverage signals to detailed artifacts",
    },
    ScenarioDefinition {
        scenario_id: "all-in-one-report-appraisal-proof",
        focus: "capture all-in-one report appraisal evidence",
    },
];

const G239_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "campaign-comparison-paired-boundary",
        focus: "validate campaign comparison boundaries across paired campaigns",
    },
    ScenarioDefinition {
        scenario_id: "campaign-comparison-drift-signal-coverage",
        focus: "validate campaign drift and status comparison signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "campaign-comparison-appraisal-proof",
        focus: "capture campaign comparison appraisal evidence",
    },
];

const G240_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "benchmark-doc-hint-boundary",
        focus: "validate benchmark-to-docs hint boundaries for command-impacting behavior",
    },
    ScenarioDefinition {
        scenario_id: "benchmark-doc-hint-caveat-signal-coverage",
        focus: "validate docs hint caveat signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "benchmark-doc-hint-appraisal-proof",
        focus: "capture benchmark-to-docs hint appraisal evidence",
    },
];

const G241_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "full-fastq-stage-tool-boundary",
        focus: "validate full FASTQ stage/tool campaign coverage boundaries",
    },
    ScenarioDefinition {
        scenario_id: "full-fastq-encryption-appraiser-signals",
        focus: "validate encrypted FASTQ campaign result and appraiser signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "full-fastq-campaign-proof",
        focus: "capture full FASTQ campaign appraisal evidence",
    },
];

const G242_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "full-bam-stage-tool-boundary",
        focus: "validate full BAM stage/tool campaign coverage boundaries",
    },
    ScenarioDefinition {
        scenario_id: "full-bam-encryption-appraiser-signals",
        focus: "validate encrypted BAM campaign result and appraiser signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "full-bam-campaign-proof",
        focus: "capture full BAM campaign appraisal evidence",
    },
];

const G243_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "full-vcf-stage-tool-boundary",
        focus: "validate full VCF stage/tool campaign coverage boundaries",
    },
    ScenarioDefinition {
        scenario_id: "full-vcf-encryption-appraiser-signals",
        focus: "validate encrypted VCF campaign result and appraiser signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "full-vcf-campaign-proof",
        focus: "capture full VCF campaign appraisal evidence",
    },
];

const G244_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "full-cross-domain-boundary",
        focus: "validate full cross-domain campaign coverage boundaries",
    },
    ScenarioDefinition {
        scenario_id: "full-cross-domain-encryption-appraiser-signals",
        focus: "validate encrypted cross-domain campaign and appraiser signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "full-cross-domain-campaign-proof",
        focus: "capture full cross-domain campaign appraisal evidence",
    },
];

const G245_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "all-domain-staged-series-boundary",
        focus: "validate all-domain staged campaign series boundaries",
    },
    ScenarioDefinition {
        scenario_id: "all-domain-series-replay-signals",
        focus: "validate reproducible series replay and evidence signals",
    },
    ScenarioDefinition {
        scenario_id: "all-domain-series-proof",
        focus: "capture end-to-end all-domain series appraisal evidence",
    },
];

const G246_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "local-slurm-parity-boundary",
        focus: "validate local versus Slurm execution parity boundaries",
    },
    ScenarioDefinition {
        scenario_id: "local-slurm-diff-signal-coverage",
        focus: "validate local/Slurm difference signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "local-slurm-parity-proof",
        focus: "capture local versus Slurm parity appraisal evidence",
    },
];

const G247_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "image-version-drift-boundary",
        focus: "validate Apptainer image version drift boundaries",
    },
    ScenarioDefinition {
        scenario_id: "image-version-output-signal-coverage",
        focus: "validate image version output drift signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "image-version-drift-proof",
        focus: "capture image version drift appraisal evidence",
    },
];

const G248_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "database-version-drift-boundary",
        focus: "validate database version drift boundaries",
    },
    ScenarioDefinition {
        scenario_id: "database-version-performance-signal-coverage",
        focus: "validate database version performance and scientific drift signals",
    },
    ScenarioDefinition {
        scenario_id: "database-version-drift-proof",
        focus: "capture database version drift appraisal evidence",
    },
];

const G249_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "corpus-scale-boundary",
        focus: "validate corpus-scale small/medium/large boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "corpus-scale-failure-signal-coverage",
        focus: "validate scale-only failure signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "corpus-scale-proof",
        focus: "capture corpus-scale appraisal evidence",
    },
];

const G250_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "cold-warm-storage-boundary",
        focus: "validate cold versus warm storage execution boundaries",
    },
    ScenarioDefinition {
        scenario_id: "storage-mode-io-signal-coverage",
        focus: "validate storage mode IO and cache behavior signals",
    },
    ScenarioDefinition {
        scenario_id: "storage-mode-proof",
        focus: "capture storage-mode appraisal evidence",
    },
];

const G251_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "flagship-dashboard-readiness-boundary",
        focus: "validate flagship dashboard pass/fail and hardening readiness boundaries",
    },
    ScenarioDefinition {
        scenario_id: "flagship-dashboard-signal-coverage",
        focus: "validate flagship dashboard performance, scientific, and encryption signals",
    },
    ScenarioDefinition {
        scenario_id: "flagship-dashboard-proof",
        focus: "capture imported campaign dashboard evidence",
    },
];

const G252_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "stage-dossier-coverage-boundary",
        focus: "validate per-stage dossier coverage across stage families and outcomes",
    },
    ScenarioDefinition {
        scenario_id: "stage-dossier-failure-caveat-signals",
        focus: "validate per-stage dossier failure and caveat signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "stage-dossier-proof",
        focus: "capture generated per-stage dossier evidence",
    },
];

const G253_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "tool-dossier-support-boundary",
        focus: "validate per-tool dossier support and stage binding boundaries",
    },
    ScenarioDefinition {
        scenario_id: "tool-dossier-drift-signals",
        focus: "validate per-tool dossier drift and failure signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "tool-dossier-proof",
        focus: "capture generated per-tool dossier evidence",
    },
];

const G254_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "corpus-dossier-suitability-boundary",
        focus: "validate per-corpus dossier stage-coverage and suitability boundaries",
    },
    ScenarioDefinition {
        scenario_id: "corpus-dossier-weakness-signals",
        focus: "validate per-corpus dossier weakness and mismatch signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "corpus-dossier-proof",
        focus: "capture generated per-corpus dossier evidence",
    },
];

const G255_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "database-dossier-compatibility-boundary",
        focus: "validate per-database dossier compatibility and stage-usage boundaries",
    },
    ScenarioDefinition {
        scenario_id: "database-dossier-drift-signals",
        focus: "validate per-database dossier drift and mismatch signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "database-dossier-proof",
        focus: "capture generated per-database dossier evidence",
    },
];

const G256_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "image-dossier-inventory-boundary",
        focus: "validate per-image dossier inventory and row-coverage boundaries",
    },
    ScenarioDefinition {
        scenario_id: "image-dossier-drift-signals",
        focus: "validate per-image dossier smoke, failure, and drift signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "image-dossier-proof",
        focus: "capture generated per-image dossier evidence",
    },
];

const G257_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "stage-promotion-pass-gate-boundary",
        focus: "validate benchmark-driven promotion pass-gate boundaries",
    },
    ScenarioDefinition {
        scenario_id: "stage-promotion-rerun-signals",
        focus: "validate promotion rerun-proof and hardening signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "stage-promotion-proof",
        focus: "capture stage promotion records from benchmark outcomes",
    },
];

const G258_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "stage-demotion-failure-gate-boundary",
        focus: "validate benchmark-driven demotion failure-gate boundaries",
    },
    ScenarioDefinition {
        scenario_id: "stage-demotion-severity-signals",
        focus: "validate demotion severity and refusal signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "stage-demotion-proof",
        focus: "capture stage demotion evidence from benchmark outcomes",
    },
];

const G259_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "backend-selection-scenario-boundary",
        focus: "validate benchmark-driven backend selection across scenario boundaries",
    },
    ScenarioDefinition {
        scenario_id: "backend-selection-performance-signals",
        focus: "validate backend selection performance and reliability signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "backend-selection-proof",
        focus: "capture backend recommendation evidence from benchmark outcomes",
    },
];

const G260_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "slurm-resource-default-boundary",
        focus: "validate benchmark-driven Slurm resource default boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "slurm-resource-confidence-signals",
        focus: "validate Slurm resource confidence and runtime signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "slurm-resource-default-proof",
        focus: "capture before/after Slurm resource default evidence",
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
    GoalDefinition {
        goal_id: "G221",
        title: "add known-sites/dbSNP-like database bundles",
        stage_ids: &["bam.validate", "vcf.filter", "vcf.postprocess"],
        scenarios: G221_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G222",
        title: "add imputation panel bundles",
        stage_ids: &["vcf.call", "vcf.impute", "vcf.stats"],
        scenarios: G222_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G223",
        title: "add taxonomy database variants",
        stage_ids: &["fastq.profile_reads", "bam.contamination", "vcf.stats"],
        scenarios: G223_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G224",
        title: "add contaminant database variants",
        stage_ids: &["fastq.validate_reads", "bam.contamination", "vcf.filter"],
        scenarios: G224_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G225",
        title: "add adapter and primer bank variants",
        stage_ids: &["fastq.trim_reads", "fastq.profile_reads", "bam.contamination"],
        scenarios: G225_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G226",
        title: "add annotation database variants",
        stage_ids: &["vcf.call", "vcf.filter", "vcf.postprocess"],
        scenarios: G226_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G227",
        title: "add corpus/database compatibility matrix",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.call"],
        scenarios: G227_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G228",
        title: "add campaign coverage planner",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.impute"],
        scenarios: G228_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G229",
        title: "add expensive-job budget planner",
        stage_ids: &["bam.align", "bam.coverage", "vcf.postprocess"],
        scenarios: G229_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G230",
        title: "add campaign splitting by HPC quota",
        stage_ids: &["fastq.validate_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G230_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G231",
        title: "add campaign reproducibility lock",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.postprocess"],
        scenarios: G231_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G232",
        title: "add campaign rerun from lock",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.postprocess"],
        scenarios: G232_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G233",
        title: "add campaign subset from hardening queue",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.filter"],
        scenarios: G233_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G234",
        title: "add campaign priority labels",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.stats"],
        scenarios: G234_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G235",
        title: "add warm-start campaign mode",
        stage_ids: &["bam.validate", "bam.mapping_summary", "vcf.call"],
        scenarios: G235_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G236",
        title: "add all-in-one preflight",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.filter"],
        scenarios: G236_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G237",
        title: "add all-in-one local import",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G237_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G238",
        title: "add all-in-one report",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.postprocess"],
        scenarios: G238_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G239",
        title: "add campaign comparison report",
        stage_ids: &["bam.align", "vcf.call", "vcf.stats"],
        scenarios: G239_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G240",
        title: "add benchmark-to-docs update hints",
        stage_ids: &["fastq.validate_reads", "vcf.filter", "vcf.postprocess"],
        scenarios: G240_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G241",
        title: "run full FASTQ stage/tool campaign",
        stage_ids: &["fastq.validate_reads", "fastq.trim_reads", "fastq.profile_reads"],
        scenarios: G241_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G242",
        title: "run full BAM stage/tool campaign",
        stage_ids: &["bam.validate", "bam.align", "bam.mapping_summary"],
        scenarios: G242_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G243",
        title: "run full VCF stage/tool campaign",
        stage_ids: &["vcf.call", "vcf.filter", "vcf.stats"],
        scenarios: G243_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G244",
        title: "run full cross-domain campaign",
        stage_ids: &["fastq.profile_reads", "bam.align", "vcf.call"],
        scenarios: G244_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G245",
        title: "run all-domain staged campaign",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.postprocess"],
        scenarios: G245_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G246",
        title: "compare local and Slurm execution",
        stage_ids: &["fastq.trim_reads", "bam.coverage", "vcf.filter"],
        scenarios: G246_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G247",
        title: "compare Apptainer image versions",
        stage_ids: &["fastq.trim_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G247_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G248",
        title: "compare database versions",
        stage_ids: &["bam.align", "vcf.call", "vcf.impute"],
        scenarios: G248_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G249",
        title: "compare corpus scales",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.filter"],
        scenarios: G249_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G250",
        title: "compare cold and warm storage modes",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G250_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G251",
        title: "build flagship HPC benchmark dashboard",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G251_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G252",
        title: "build per-stage evidence dossiers",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.filter"],
        scenarios: G252_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G253",
        title: "build per-tool evidence dossiers",
        stage_ids: &["fastq.trim_reads", "bam.align", "vcf.call"],
        scenarios: G253_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G254",
        title: "build per-corpus evidence dossiers",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.filter"],
        scenarios: G254_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G255",
        title: "build per-database evidence dossiers",
        stage_ids: &["bam.align", "vcf.impute", "vcf.filter"],
        scenarios: G255_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G256",
        title: "build per-image evidence dossiers",
        stage_ids: &["fastq.trim_reads", "bam.mapping_summary", "vcf.stats"],
        scenarios: G256_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G257",
        title: "add benchmark-driven stage promotion",
        stage_ids: &["fastq.validate_reads", "bam.validate", "vcf.stats"],
        scenarios: G257_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G258",
        title: "add benchmark-driven stage demotion",
        stage_ids: &["fastq.profile_reads", "bam.contamination", "vcf.filter"],
        scenarios: G258_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G259",
        title: "add benchmark-driven default backend selection",
        stage_ids: &["fastq.trim_reads", "bam.align", "vcf.call"],
        scenarios: G259_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G260",
        title: "add benchmark-driven Slurm resource defaults",
        stage_ids: &["bam.coverage", "bam.mapping_summary", "vcf.postprocess"],
        scenarios: G260_SCENARIOS,
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
        "G212" => vec![
            format!("sample_swap_rows_present={}", !rows.is_empty()),
            format!(
                "sample_swap_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "sample_swap_profile_validate_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "sample_swap_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!("sample_swap_findings_count={}", findings.len()),
        ],
        "G213" => vec![
            format!("duplicate_cohort_rows_present={}", !rows.is_empty()),
            format!(
                "duplicate_cohort_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "duplicate_cohort_profile_coverage_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "duplicate_cohort_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("duplicate_cohort_findings_count={}", findings.len()),
        ],
        "G214" => vec![
            format!("panel_choice_rows_present={}", !rows.is_empty()),
            format!(
                "panel_choice_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "panel_choice_impute_call_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.impute")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "panel_choice_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!("panel_choice_findings_count={}", findings.len()),
        ],
        "G215" => vec![
            format!("annotation_drift_rows_present={}", !rows.is_empty()),
            format!(
                "annotation_drift_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "annotation_drift_call_filter_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "annotation_drift_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("annotation_drift_findings_count={}", findings.len()),
        ],
        "G216" => vec![
            format!("qc_threshold_rows_present={}", !rows.is_empty()),
            format!(
                "qc_threshold_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "qc_threshold_trim_coverage_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "qc_threshold_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!("qc_threshold_findings_count={}", findings.len()),
        ],
        "G217" => vec![
            format!("artifact_minimization_rows_present={}", !rows.is_empty()),
            format!(
                "artifact_minimization_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "artifact_minimization_profile_summary_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "artifact_minimization_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "artifact_minimization_code_scope_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.result_scope == "encrypted-code")
                    .count()
            ),
        ],
        "G218" => vec![
            format!("shared_storage_io_rows_present={}", !rows.is_empty()),
            format!(
                "shared_storage_io_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "shared_storage_io_profile_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "shared_storage_io_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("shared_storage_io_queue_entries={}", queue_entries.len()),
        ],
        "G219" => vec![
            format!("database_build_rows_present={}", !rows.is_empty()),
            format!(
                "database_build_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "database_build_align_call_impute_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.impute")
            ),
            format!(
                "database_build_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!("database_build_findings_count={}", findings.len()),
        ],
        "G220" => vec![
            format!("truth_set_rows_present={}", !rows.is_empty()),
            format!(
                "truth_set_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "truth_set_validate_call_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "truth_set_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("truth_set_findings_count={}", findings.len()),
        ],
        "G221" => vec![
            format!("known_sites_rows_present={}", !rows.is_empty()),
            format!(
                "known_sites_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "known_sites_validate_filter_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "known_sites_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("known_sites_findings_count={}", findings.len()),
        ],
        "G222" => vec![
            format!("imputation_panel_rows_present={}", !rows.is_empty()),
            format!(
                "imputation_panel_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "imputation_panel_call_impute_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.impute")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "imputation_panel_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!("imputation_panel_findings_count={}", findings.len()),
        ],
        "G223" => vec![
            format!("taxonomy_variant_rows_present={}", !rows.is_empty()),
            format!(
                "taxonomy_variant_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "taxonomy_variant_profile_contam_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.contamination")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "taxonomy_variant_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("taxonomy_variant_findings_count={}", findings.len()),
        ],
        "G224" => vec![
            format!("contaminant_variant_rows_present={}", !rows.is_empty()),
            format!(
                "contaminant_variant_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "contaminant_variant_validate_contam_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.contamination")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "contaminant_variant_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("contaminant_variant_queue_entries={}", queue_entries.len()),
        ],
        "G225" => vec![
            format!("adapter_primer_rows_present={}", !rows.is_empty()),
            format!(
                "adapter_primer_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "adapter_primer_trim_profile_contam_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.contamination")
            ),
            format!(
                "adapter_primer_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("adapter_primer_findings_count={}", findings.len()),
        ],
        "G226" => vec![
            format!("annotation_db_rows_present={}", !rows.is_empty()),
            format!(
                "annotation_db_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "annotation_db_call_filter_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "annotation_db_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("annotation_db_findings_count={}", findings.len()),
        ],
        "G227" => vec![
            format!("corpus_db_matrix_rows_present={}", !rows.is_empty()),
            format!(
                "corpus_db_matrix_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "corpus_db_matrix_validate_align_call_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "corpus_db_matrix_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("corpus_db_matrix_findings_count={}", findings.len()),
        ],
        "G228" => vec![
            format!("campaign_coverage_rows_present={}", !rows.is_empty()),
            format!(
                "campaign_coverage_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "campaign_coverage_profile_coverage_impute_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.impute")
            ),
            format!(
                "campaign_coverage_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("campaign_coverage_findings_count={}", findings.len()),
        ],
        "G229" => vec![
            format!("budget_planner_rows_present={}", !rows.is_empty()),
            format!(
                "budget_planner_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "budget_planner_align_coverage_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "budget_planner_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("budget_planner_findings_count={}", findings.len()),
        ],
        "G230" => vec![
            format!("campaign_split_rows_present={}", !rows.is_empty()),
            format!(
                "campaign_split_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "campaign_split_validate_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "campaign_split_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("campaign_split_queue_entries={}", queue_entries.len()),
        ],
        "G231" => vec![
            format!("campaign_lock_rows_present={}", !rows.is_empty()),
            format!(
                "campaign_lock_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "campaign_lock_validate_validate_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "campaign_lock_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("campaign_lock_findings_count={}", findings.len()),
        ],
        "G232" => vec![
            format!("campaign_rerun_rows_present={}", !rows.is_empty()),
            format!(
                "campaign_rerun_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "campaign_rerun_profile_summary_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "campaign_rerun_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("campaign_rerun_findings_count={}", findings.len()),
        ],
        "G233" => vec![
            format!("queue_subset_rows_present={}", !rows.is_empty()),
            format!(
                "queue_subset_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "queue_subset_validate_align_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "queue_subset_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("queue_subset_queue_entries={}", queue_entries.len()),
        ],
        "G234" => vec![
            format!("priority_label_rows_present={}", !rows.is_empty()),
            format!(
                "priority_label_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "priority_label_profile_coverage_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "priority_label_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("priority_label_findings_count={}", findings.len()),
        ],
        "G235" => vec![
            format!("warm_start_rows_present={}", !rows.is_empty()),
            format!(
                "warm_start_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "warm_start_validate_summary_call_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "warm_start_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("warm_start_findings_count={}", findings.len()),
        ],
        "G236" => vec![
            format!("all_in_one_preflight_rows_present={}", !rows.is_empty()),
            format!(
                "all_in_one_preflight_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "all_in_one_preflight_validate_validate_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "all_in_one_preflight_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("all_in_one_preflight_queue_entries={}", queue_entries.len()),
        ],
        "G237" => vec![
            format!("all_in_one_import_rows_present={}", !rows.is_empty()),
            format!(
                "all_in_one_import_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "all_in_one_import_profile_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "all_in_one_import_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("all_in_one_import_findings_count={}", findings.len()),
        ],
        "G238" => vec![
            format!("all_in_one_report_rows_present={}", !rows.is_empty()),
            format!(
                "all_in_one_report_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "all_in_one_report_profile_coverage_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "all_in_one_report_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("all_in_one_report_findings_count={}", findings.len()),
        ],
        "G239" => vec![
            format!("campaign_comparison_rows_present={}", !rows.is_empty()),
            format!(
                "campaign_comparison_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "campaign_comparison_align_call_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "campaign_comparison_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("campaign_comparison_findings_count={}", findings.len()),
        ],
        "G240" => vec![
            format!("benchmark_docs_rows_present={}", !rows.is_empty()),
            format!(
                "benchmark_docs_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "benchmark_docs_validate_filter_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "benchmark_docs_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("benchmark_docs_findings_count={}", findings.len()),
        ],
        "G241" => vec![
            format!("full_fastq_rows_present={}", !rows.is_empty()),
            format!(
                "full_fastq_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "full_fastq_validate_trim_profile_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
            ),
            format!(
                "full_fastq_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("full_fastq_findings_count={}", findings.len()),
        ],
        "G242" => vec![
            format!("full_bam_rows_present={}", !rows.is_empty()),
            format!(
                "full_bam_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "full_bam_validate_align_summary_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
            ),
            format!(
                "full_bam_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("full_bam_findings_count={}", findings.len()),
        ],
        "G243" => vec![
            format!("full_vcf_rows_present={}", !rows.is_empty()),
            format!(
                "full_vcf_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "full_vcf_call_filter_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "full_vcf_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("full_vcf_findings_count={}", findings.len()),
        ],
        "G244" => vec![
            format!("full_cross_domain_rows_present={}", !rows.is_empty()),
            format!(
                "full_cross_domain_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "full_cross_domain_profile_align_call_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "full_cross_domain_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("full_cross_domain_findings_count={}", findings.len()),
        ],
        "G245" => vec![
            format!("all_domain_series_rows_present={}", !rows.is_empty()),
            format!(
                "all_domain_series_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "all_domain_series_validate_validate_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "all_domain_series_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("all_domain_series_findings_count={}", findings.len()),
        ],
        "G246" => vec![
            format!("local_slurm_parity_rows_present={}", !rows.is_empty()),
            format!(
                "local_slurm_parity_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "local_slurm_parity_trim_coverage_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "local_slurm_parity_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!("local_slurm_parity_findings_count={}", findings.len()),
        ],
        "G247" => vec![
            format!("image_version_rows_present={}", !rows.is_empty()),
            format!(
                "image_version_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "image_version_trim_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "image_version_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("image_version_findings_count={}", findings.len()),
        ],
        "G248" => vec![
            format!("database_version_rows_present={}", !rows.is_empty()),
            format!(
                "database_version_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "database_version_align_call_impute_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
                    && rows.iter().any(|row| row.stage_id == "vcf.impute")
            ),
            format!(
                "database_version_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("database_version_findings_count={}", findings.len()),
        ],
        "G249" => vec![
            format!("corpus_scale_rows_present={}", !rows.is_empty()),
            format!(
                "corpus_scale_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "corpus_scale_profile_coverage_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "corpus_scale_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!("corpus_scale_findings_count={}", findings.len()),
        ],
        "G250" => vec![
            format!("storage_mode_rows_present={}", !rows.is_empty()),
            format!(
                "storage_mode_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "storage_mode_profile_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "storage_mode_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!("storage_mode_findings_count={}", findings.len()),
        ],
        "G251" => vec![
            format!("flagship_dashboard_rows_present={}", !rows.is_empty()),
            format!(
                "flagship_dashboard_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "flagship_dashboard_profile_summary_stats_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
                    && rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "flagship_dashboard_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "flagship_dashboard_encrypted_scope_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.result_scope.starts_with("encrypted-"))
                    .count()
            ),
        ],
        "G252" => vec![
            format!("stage_dossier_rows_present={}", !rows.is_empty()),
            format!(
                "stage_dossier_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "stage_dossier_validate_validate_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.validate_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.validate")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "stage_dossier_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!("stage_dossier_findings_count={}", findings.len()),
        ],
        "G253" => vec![
            format!("tool_dossier_rows_present={}", !rows.is_empty()),
            format!(
                "tool_dossier_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "tool_dossier_trim_align_call_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.trim_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.align")
                    && rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "tool_dossier_tool_count={}",
                rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!("tool_dossier_findings_count={}", findings.len()),
        ],
        "G254" => vec![
            format!("corpus_dossier_rows_present={}", !rows.is_empty()),
            format!(
                "corpus_dossier_stage_count={}",
                rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
            ),
            format!(
                "corpus_dossier_profile_coverage_filter_bound={}",
                rows.iter().any(|row| row.stage_id == "fastq.profile_reads")
                    && rows.iter().any(|row| row.stage_id == "bam.coverage")
                    && rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "corpus_dossier_non_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class != "ready").count()
            ),
            format!(
                "corpus_dossier_corpus_mismatch_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.failure_class == "corpus-mismatch")
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
    fn hardening_catalog_includes_iteration_23_goals() {
        assert_eq!(HARDENING_GOALS_CATALOG.len(), 90);
        assert_eq!(HARDENING_GOALS_CATALOG[0].goal_id, "G171");
        assert_eq!(HARDENING_GOALS_CATALOG[89].goal_id, "G260");
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
            row_id: "h6".to_string(),
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
    fn goal_212_emits_sample_swap_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G212".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h11".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "scientific-caveat".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "sample-swap caveat surfaced in stats stage".to_string(),
            recommendation: "gate downstream interpretation on swap review".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "sample_swap_profile_validate_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "sample_swap_findings_count=1"));
    }

    #[test]
    fn goal_213_emits_duplicate_cohort_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G213".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "duplicate cohort coverage stage requires extra repeats".to_string(),
            recommendation: "set repetitions >= 3 for reproducibility confidence".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "duplicate_cohort_profile_coverage_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "duplicate_cohort_findings_count=1"));
    }

    #[test]
    fn goal_214_emits_panel_choice_sensitivity_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G214".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "database-compatibility".to_string(),
            row_id: "h12".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "database-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "panel-choice imputation boundary needs map compatibility review".to_string(),
            recommendation: "align panel and map bundle with selected reference".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "panel_choice_impute_call_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "panel_choice_findings_count=1"));
    }

    #[test]
    fn goal_215_emits_annotation_drift_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G215".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h13".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "scientific-caveat".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "annotation source drift changed postprocess expectations".to_string(),
            recommendation: "review annotation source provenance and caveat scope".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "annotation_drift_call_filter_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "annotation_drift_findings_count=1"));
    }

    #[test]
    fn goal_216_emits_qc_threshold_sensitivity_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G216".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "QC-threshold boundary triggers refuse path".to_string(),
            recommendation: "resolve threshold gate reasons and re-run appraisal".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "qc_threshold_trim_coverage_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "qc_threshold_findings_count=1"));
    }

    #[test]
    fn goal_217_emits_artifact_minimization_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G217".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "artifact minimization bundle misses required provenance metadata".to_string(),
            recommendation: "include provenance lock and script digest entries".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "artifact_minimization_profile_summary_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "artifact_minimization_code_scope_findings=1"));
    }

    #[test]
    fn goal_218_emits_shared_storage_io_stress_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G218".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1701".to_string(),
            severity: "warning".to_string(),
            failure_class: "runtime-outlier".to_string(),
            recommendation: "review shared-storage IO staging and scratch placement".to_string(),
            affected_rows: vec!["h3".to_string()],
            source_appraisers: vec!["runtime-profile".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &[], &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "shared_storage_io_profile_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "shared_storage_io_queue_entries=1"));
    }

    #[test]
    fn goal_219_emits_database_build_benchmark_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G219".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h12".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "database-build benchmark impute stage has runtime outlier".to_string(),
            recommendation: "profile database build assets and rebuild expensive indexes".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "database_build_align_call_impute_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "database_build_findings_count=1"));
    }

    #[test]
    fn goal_220_emits_truth_set_anchored_corpus_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G220".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "truth-set anchored boundary hits filter refuse row".to_string(),
            recommendation: "resolve refusal causes before truth-set scoring".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "truth_set_validate_call_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "truth_set_findings_count=1"));
    }

    #[test]
    fn goal_221_emits_known_sites_bundle_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G221".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "database-compatibility".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "database-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "known-sites bundle mismatch at filter boundary".to_string(),
            recommendation: "align known-sites bundle with reference build".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "known_sites_validate_filter_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "known_sites_findings_count=1"));
    }

    #[test]
    fn goal_222_emits_imputation_panel_bundle_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G222".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "database-compatibility".to_string(),
            row_id: "h12".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "database-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "imputation panel map trust label mismatch".to_string(),
            recommendation: "align panel-map trust profile before campaign promotion".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "imputation_panel_call_impute_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "imputation_panel_findings_count=1"));
    }

    #[test]
    fn goal_223_emits_taxonomy_variant_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G223".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "corpus-suitability".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "corpus-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "taxonomy variant profile requires rank-resolution caveat".to_string(),
            recommendation: "bind taxonomy variant profile to intended rank boundary".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "taxonomy_variant_profile_contam_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "taxonomy_variant_findings_count=1"));
    }

    #[test]
    fn goal_224_emits_contaminant_variant_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G224".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1801".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve contaminant boundary refusal reasons".to_string(),
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &[], &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "contaminant_variant_validate_contam_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "contaminant_variant_queue_entries=1"));
    }

    #[test]
    fn goal_225_emits_adapter_primer_variant_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G225".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "scientific-caveat".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "adapter-primer variant raises profile caveat".to_string(),
            recommendation: "review platform/assay primer bank selection".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "adapter_primer_trim_profile_contam_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "adapter_primer_findings_count=1"));
    }

    #[test]
    fn goal_226_emits_annotation_db_variant_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G226".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h13".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "scientific-caveat".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "annotation DB variant changes postprocess caveat".to_string(),
            recommendation: "review annotation DB provenance and caveat policy".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "annotation_db_call_filter_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "annotation_db_findings_count=1"));
    }

    #[test]
    fn goal_227_emits_corpus_db_compatibility_matrix_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G227".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "database-compatibility".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "database-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "corpus/database matrix mismatch for align boundary".to_string(),
            recommendation: "update compatibility matrix for selected corpus and DB bundle".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_db_matrix_validate_align_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_db_matrix_findings_count=1"));
    }

    #[test]
    fn goal_228_emits_campaign_coverage_planner_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G228".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "coverage planner includes runtime outlier row".to_string(),
            recommendation: "rebalance coverage planner row selection by runtime budget".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_coverage_profile_coverage_impute_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_coverage_findings_count=1"));
    }

    #[test]
    fn goal_229_emits_budget_planner_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G229".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "budget planner detects expensive coverage-stage profile".to_string(),
            recommendation: "rebalance budget planner row-set for high-cost boundaries".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "budget_planner_align_coverage_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "budget_planner_findings_count=1"));
    }

    #[test]
    fn goal_230_emits_campaign_split_quota_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G230".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-1901".to_string(),
            severity: "warning".to_string(),
            failure_class: "runtime-outlier".to_string(),
            recommendation: "split campaign rows by quota windows".to_string(),
            affected_rows: vec!["h6".to_string()],
            source_appraisers: vec!["runtime-profile".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &[], &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_split_validate_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_split_queue_entries=1"));
    }

    #[test]
    fn goal_231_emits_campaign_lock_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G231".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "campaign lock misses postprocess provenance anchor".to_string(),
            recommendation: "freeze postprocess provenance and lock digest state".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_lock_validate_validate_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_lock_findings_count=1"));
    }

    #[test]
    fn goal_232_emits_campaign_rerun_lock_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G232".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "rerun-from-lock profile stage needs repeat reinforcement".to_string(),
            recommendation: "set rerun repetitions >= 3 for lock replay confidence".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_rerun_profile_summary_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_rerun_findings_count=1"));
    }

    #[test]
    fn goal_233_emits_hardening_queue_subset_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G233".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-2001".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "rerun only hardening queue rows for validation".to_string(),
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &[], &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "queue_subset_validate_align_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "queue_subset_queue_entries=1"));
    }

    #[test]
    fn goal_234_emits_campaign_priority_label_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G234".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h7".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "priority labels need cost-value rebalance for coverage row".to_string(),
            recommendation: "raise priority for high scientific value low-cost rows".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "priority_label_profile_coverage_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "priority_label_findings_count=1"));
    }

    #[test]
    fn goal_235_emits_warm_start_campaign_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G235".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h6".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "readiness-degraded".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "warm-start campaign has degraded pre-locked align surface".to_string(),
            recommendation: "restore ready warm-start assets before submit".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "warm_start_validate_summary_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "warm_start_findings_count=1"));
    }

    #[test]
    fn goal_236_emits_all_in_one_preflight_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G236".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-2101".to_string(),
            severity: "critical".to_string(),
            failure_class: "readiness-refuse".to_string(),
            recommendation: "resolve preflight refusal row before campaign submit".to_string(),
            affected_rows: vec!["h10".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &[], &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_in_one_preflight_validate_validate_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_in_one_preflight_queue_entries=1"));
    }

    #[test]
    fn goal_237_emits_all_in_one_import_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G237".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "all-in-one import profile stage has runtime caveat".to_string(),
            recommendation: "optimize import pipeline and re-index replay artifacts".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_in_one_import_profile_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_in_one_import_findings_count=1"));
    }

    #[test]
    fn goal_238_emits_all_in_one_report_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G238".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h13".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "scientific-caveat".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "all-in-one report requires caveat link to postprocess drift".to_string(),
            recommendation: "include postprocess caveat links in generated summary reports".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_in_one_report_profile_coverage_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_in_one_report_findings_count=1"));
    }

    #[test]
    fn goal_239_emits_campaign_comparison_report_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G239".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "campaign comparison detected unstable align boundary".to_string(),
            recommendation: "increase repetitions before campaign drift comparison".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_comparison_align_call_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "campaign_comparison_findings_count=1"));
    }

    #[test]
    fn goal_240_emits_benchmark_docs_hint_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G240".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "benchmark output indicates docs caveat for filter refusal behavior".to_string(),
            recommendation: "add docs hint when refusal behavior changes user-visible guidance".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "benchmark_docs_validate_filter_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "benchmark_docs_findings_count=1"));
    }

    #[test]
    fn goal_241_emits_full_fastq_campaign_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G241".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "full FASTQ campaign has profile-stage runtime outlier".to_string(),
            recommendation: "tune FASTQ-stage runtime profile before full promotion".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_fastq_validate_trim_profile_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_fastq_findings_count=1"));
    }

    #[test]
    fn goal_242_emits_full_bam_campaign_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G242".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "full BAM campaign has align-stage runtime outlier".to_string(),
            recommendation: "tune BAM-stage runtime before full campaign promotion".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_bam_validate_align_summary_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_bam_findings_count=1"));
    }

    #[test]
    fn goal_243_emits_full_vcf_campaign_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G243".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "full VCF campaign includes filter refusal boundary".to_string(),
            recommendation: "resolve filter refusal before full VCF promotion".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_vcf_call_filter_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_vcf_findings_count=1"));
    }

    #[test]
    fn goal_244_emits_full_cross_domain_campaign_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G244".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "reproducibility-low-repeats".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "cross-domain campaign align stage lacks repeat confidence".to_string(),
            recommendation: "increase repetitions for cross-domain campaign parity".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_cross_domain_profile_align_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "full_cross_domain_findings_count=1"));
    }

    #[test]
    fn goal_245_emits_all_domain_series_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G245".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "code-freeze".to_string(),
            row_id: "h13".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "code-freeze-incomplete".to_string(),
            result_scope: "encrypted-code".to_string(),
            summary: "all-domain series lacks complete postprocess code-freeze fields".to_string(),
            recommendation: "freeze series provenance for all-domain campaign replay".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_domain_series_validate_validate_postprocess_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "all_domain_series_findings_count=1"));
    }

    #[test]
    fn goal_246_emits_local_slurm_parity_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G246".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "reproducibility".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "local/slurm parity mismatch at filter refusal boundary".to_string(),
            recommendation: "align local and Slurm runtime flags for parity checks".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "local_slurm_parity_trim_coverage_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "local_slurm_parity_findings_count=1"));
    }

    #[test]
    fn goal_247_emits_image_version_drift_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G247".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h6".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "image-version comparison shows mapping-summary runtime drift".to_string(),
            recommendation: "pin image digest or tune image runtime profile".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "image_version_trim_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "image_version_findings_count=1"));
    }

    #[test]
    fn goal_248_emits_database_version_drift_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G248".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "database-compatibility".to_string(),
            row_id: "h12".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "database-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "database-version comparison shows imputation boundary mismatch".to_string(),
            recommendation: "align database version rollout across campaign rows".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "database_version_align_call_impute_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "database_version_findings_count=1"));
    }

    #[test]
    fn goal_249_emits_corpus_scale_comparison_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G249".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "readiness-refuse".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "corpus-scale comparison exposes scale-only refusal path".to_string(),
            recommendation: "partition scale tiers and isolate refusal-only large-corpus rows".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_scale_profile_coverage_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_scale_findings_count=1"));
    }

    #[test]
    fn goal_250_emits_storage_mode_comparison_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G250".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-profile".to_string(),
            row_id: "h3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "runtime-outlier".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "cold/warm storage comparison shows profile-stage IO drift".to_string(),
            recommendation: "benchmark cache-state and staging policy across storage modes".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "storage_mode_profile_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "storage_mode_findings_count=1"));
    }

    #[test]
    fn goal_251_emits_flagship_dashboard_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G251".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h11".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "scientific-invalidity".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "flagship dashboard import shows scientific fail row in stats stage".to_string(),
            recommendation: "highlight pass/fail and appraiser scientific status in dashboard".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "flagship_dashboard_profile_summary_stats_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "flagship_dashboard_encrypted_scope_findings=1"));
    }

    #[test]
    fn goal_252_emits_stage_dossier_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G252".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "h10".to_string(),
            severity: "critical".to_string(),
            confidence: "high".to_string(),
            failure_class: "scientific-invalidity".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "stage dossier import includes refused filter row for scientific caveat".to_string(),
            recommendation: "expose failed stage caveat details in per-stage dossier output".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "stage_dossier_validate_validate_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "stage_dossier_refuse_rows=1"));
    }

    #[test]
    fn goal_253_emits_tool_dossier_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G253".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "artifact-validity".to_string(),
            row_id: "h5".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "missing-tool-binding".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "tool dossier import reports missing tool-binding around align stage".to_string(),
            recommendation: "record tool support and binding caveats in per-tool dossier output".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "tool_dossier_trim_align_call_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "tool_dossier_findings_count=1"));
    }

    #[test]
    fn goal_254_emits_corpus_dossier_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G254".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "corpus-suitability".to_string(),
            row_id: "h10".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "corpus-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "corpus dossier import includes corpus mismatch at filter boundary".to_string(),
            recommendation: "record corpus suitability weaknesses and mismatch caveats per stage".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_dossier_profile_coverage_filter_bound=true"));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check == "corpus_dossier_corpus_mismatch_findings=1"));
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
