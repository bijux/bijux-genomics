//! Owner: hardening campaign evidence declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

const FLAGSHIP_HPC_BENCHMARK_DASHBOARD_SCENARIOS: &[ScenarioDefinition] = &[
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

const PER_STAGE_EVIDENCE_DOSSIERS_SCENARIOS: &[ScenarioDefinition] = &[
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

const PER_TOOL_EVIDENCE_DOSSIERS_SCENARIOS: &[ScenarioDefinition] = &[
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

const PER_CORPUS_EVIDENCE_DOSSIERS_SCENARIOS: &[ScenarioDefinition] = &[
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

const PER_DATABASE_EVIDENCE_DOSSIERS_SCENARIOS: &[ScenarioDefinition] = &[
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

const PER_IMAGE_EVIDENCE_DOSSIERS_SCENARIOS: &[ScenarioDefinition] = &[
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

const BENCHMARK_DRIVEN_STAGE_PROMOTION_SCENARIOS: &[ScenarioDefinition] = &[
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

const BENCHMARK_DRIVEN_STAGE_DEMOTION_SCENARIOS: &[ScenarioDefinition] = &[
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

const BENCHMARK_DRIVEN_DEFAULT_BACKEND_SELECTION_SCENARIOS: &[ScenarioDefinition] = &[
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

const BENCHMARK_DRIVEN_SLURM_RESOURCE_DEFAULTS_SCENARIOS: &[ScenarioDefinition] = &[
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

const METHODS_APPENDIX_FROM_HPC_BENCHMARK_EVIDENCE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "methods-appendix-evidence-boundary",
        focus: "validate methods appendix evidence extraction boundaries",
    },
    ScenarioDefinition {
        scenario_id: "methods-appendix-runtime-signals",
        focus: "validate methods appendix runtime and environment signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "methods-appendix-proof",
        focus: "capture methods appendix generation evidence",
    },
];

const PUBLIC_CAVEAT_SUMMARY_FROM_HPC_BENCHMARK_EVIDENCE_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "public-caveat-summary-boundary",
        focus: "validate public caveat summary evidence boundaries",
    },
    ScenarioDefinition {
        scenario_id: "public-caveat-summary-signal-coverage",
        focus: "validate tested/weak/simulated/advisory caveat signal coverage",
    },
    ScenarioDefinition {
        scenario_id: "public-caveat-summary-proof",
        focus: "capture public caveat summary evidence",
    },
];

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G251",
        title: "build flagship HPC benchmark dashboard",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: flagship_hpc_benchmark_dashboard,
        scenarios: FLAGSHIP_HPC_BENCHMARK_DASHBOARD_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G252",
        title: "build per-stage evidence dossiers",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_FILTER],
        checks: per_stage_evidence_dossiers,
        scenarios: PER_STAGE_EVIDENCE_DOSSIERS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G253",
        title: "build per-tool evidence dossiers",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_ALIGN, stage::VCF_CALL],
        checks: per_tool_evidence_dossiers,
        scenarios: PER_TOOL_EVIDENCE_DOSSIERS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G254",
        title: "build per-corpus evidence dossiers",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_FILTER],
        checks: per_corpus_evidence_dossiers,
        scenarios: PER_CORPUS_EVIDENCE_DOSSIERS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G255",
        title: "build per-database evidence dossiers",
        stage_ids: &[stage::BAM_ALIGN, stage::VCF_IMPUTE, stage::VCF_FILTER],
        checks: per_database_evidence_dossiers,
        scenarios: PER_DATABASE_EVIDENCE_DOSSIERS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G256",
        title: "build per-image evidence dossiers",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: per_image_evidence_dossiers,
        scenarios: PER_IMAGE_EVIDENCE_DOSSIERS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G257",
        title: "add benchmark-driven stage promotion",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_STATS],
        checks: benchmark_driven_stage_promotion,
        scenarios: BENCHMARK_DRIVEN_STAGE_PROMOTION_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G258",
        title: "add benchmark-driven stage demotion",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_CONTAMINATION, stage::VCF_FILTER],
        checks: benchmark_driven_stage_demotion,
        scenarios: BENCHMARK_DRIVEN_STAGE_DEMOTION_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G259",
        title: "add benchmark-driven default backend selection",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_ALIGN, stage::VCF_CALL],
        checks: benchmark_driven_default_backend_selection,
        scenarios: BENCHMARK_DRIVEN_DEFAULT_BACKEND_SELECTION_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G260",
        title: "add benchmark-driven Slurm resource defaults",
        stage_ids: &[stage::BAM_COVERAGE, stage::BAM_MAPPING_SUMMARY, stage::VCF_POSTPROCESS],
        checks: benchmark_driven_slurm_resource_defaults,
        scenarios: BENCHMARK_DRIVEN_SLURM_RESOURCE_DEFAULTS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G279",
        title: "add methods appendix from HPC benchmark evidence",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: methods_appendix_from_hpc_benchmark_evidence,
        scenarios: METHODS_APPENDIX_FROM_HPC_BENCHMARK_EVIDENCE_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G280",
        title: "add public caveat summary from HPC benchmark evidence",
        stage_ids: &[stage::BAM_CONTAMINATION, stage::VCF_FILTER, stage::VCF_STATS],
        checks: public_caveat_summary_from_hpc_benchmark_evidence,
        scenarios: PUBLIC_CAVEAT_SUMMARY_FROM_HPC_BENCHMARK_EVIDENCE_SCENARIOS,
    },
];

fn flagship_hpc_benchmark_dashboard(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("flagship_dashboard_rows_present={}", !rows.is_empty()),
        format!(
            "flagship_dashboard_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "flagship_dashboard_profile_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
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
    ]
}

fn per_stage_evidence_dossiers(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("stage_dossier_rows_present={}", !rows.is_empty()),
        format!(
            "stage_dossier_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "stage_dossier_validate_validate_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "stage_dossier_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!("stage_dossier_findings_count={}", findings.len()),
    ]
}

fn per_tool_evidence_dossiers(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("tool_dossier_rows_present={}", !rows.is_empty()),
        format!(
            "tool_dossier_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "tool_dossier_trim_align_call_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
        ),
        format!(
            "tool_dossier_tool_count={}",
            rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!("tool_dossier_findings_count={}", findings.len()),
    ]
}

fn per_corpus_evidence_dossiers(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("corpus_dossier_rows_present={}", !rows.is_empty()),
        format!(
            "corpus_dossier_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "corpus_dossier_profile_coverage_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "corpus_dossier_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "corpus_dossier_corpus_mismatch_findings={}",
            findings.iter().filter(|finding| finding.failure_class == "corpus-mismatch").count()
        ),
    ]
}

fn per_database_evidence_dossiers(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("database_dossier_rows_present={}", !rows.is_empty()),
        format!(
            "database_dossier_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "database_dossier_align_impute_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_IMPUTE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "database_dossier_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "database_dossier_mismatch_findings={}",
            findings.iter().filter(|finding| finding.failure_class == "database-mismatch").count()
        ),
    ]
}

fn per_image_evidence_dossiers(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("image_dossier_rows_present={}", !rows.is_empty()),
        format!(
            "image_dossier_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "image_dossier_trim_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "image_dossier_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "image_dossier_drift_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "image-mismatch"
                        || finding.failure_class == "runtime-outlier"
                })
                .count()
        ),
    ]
}

fn benchmark_driven_stage_promotion(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("stage_promotion_rows_present={}", !rows.is_empty()),
        format!(
            "stage_promotion_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "stage_promotion_validate_validate_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "stage_promotion_pass_gate={}",
            rows.iter().all(|row| row.readiness_class == "ready")
                && findings.iter().all(|finding| finding.severity != "critical")
                && queue_entries.iter().all(|entry| entry.severity != "critical")
        ),
        format!(
            "stage_promotion_critical_queue_entries={}",
            queue_entries.iter().filter(|entry| entry.severity == "critical").count()
        ),
    ]
}

fn benchmark_driven_stage_demotion(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
        format!("stage_demotion_rows_present={}", !rows.is_empty()),
        format!(
            "stage_demotion_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "stage_demotion_profile_contamination_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "stage_demotion_requires_demotion={}",
            rows.iter().any(|row| row.readiness_class == "refuse")
                || findings.iter().any(|finding| finding.severity == "critical")
                || queue_entries.iter().any(|entry| entry.severity == "critical")
        ),
        format!(
            "stage_demotion_refuse_or_critical_signals={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
                + findings.iter().filter(|finding| finding.severity == "critical").count()
                + queue_entries.iter().filter(|entry| entry.severity == "critical").count()
        ),
    ]
}

fn benchmark_driven_default_backend_selection(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("backend_selection_rows_present={}", !rows.is_empty()),
        format!(
            "backend_selection_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "backend_selection_trim_align_call_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
        ),
        format!(
            "backend_selection_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "backend_selection_runtime_repro_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "runtime-outlier"
                        || finding.failure_class == "reproducibility-low-repeats"
                })
                .count()
        ),
    ]
}

fn benchmark_driven_slurm_resource_defaults(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("slurm_resource_rows_present={}", !rows.is_empty()),
        format!(
            "slurm_resource_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "slurm_resource_coverage_summary_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "slurm_resource_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!(
            "slurm_resource_runtime_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "runtime-under-sampled"
                        || finding.failure_class == "runtime-outlier"
                })
                .count()
        ),
    ]
}

fn methods_appendix_from_hpc_benchmark_evidence(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("methods_appendix_rows_present={}", !rows.is_empty()),
        format!(
            "methods_appendix_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "methods_appendix_profile_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "methods_appendix_runtime_repro_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "runtime-outlier"
                        || finding.failure_class == "reproducibility-low-repeats"
                })
                .count()
        ),
        format!(
            "methods_appendix_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn public_caveat_summary_from_hpc_benchmark_evidence(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("public_caveat_rows_present={}", !rows.is_empty()),
        format!(
            "public_caveat_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "public_caveat_contamination_filter_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "public_caveat_scientific_or_corpus_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.failure_class == "scientific-invalidity"
                        || finding.failure_class == "corpus-mismatch"
                })
                .count()
        ),
        format!(
            "public_caveat_summary_required={}",
            rows.iter().any(|row| row.readiness_class == "refuse")
                || findings.iter().any(|finding| {
                    finding.failure_class == "scientific-invalidity"
                        || finding.failure_class == "corpus-mismatch"
                })
        ),
    ]
}
