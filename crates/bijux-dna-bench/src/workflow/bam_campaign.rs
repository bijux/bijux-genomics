//! Owner: bijux-dna-bench
//! BAM benchmark campaign derivation and persistence workflow.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use bijux_dna_bench_model::{
    validate_appraisal_report, validate_bam_benchmark_campaign_report,
    validate_benchmark_matrix_report, validate_hardening_queue_report, AppraisalFinding,
    AppraisalReport, BamBenchmarkCampaignReport, BamBenchmarkCampaignSummary,
    BamBenchmarkGoalEntry, BamMatrixRowRef, BamScenario, BenchmarkMatrixReport,
    HardeningQueueEntry, HardeningQueueReport, BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1,
};

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
    ScenarioDefinition { scenario_id: "short-read-stress", focus: "short-read alignment behavior" },
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

const G131_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "damage-fragmentation-signals",
        focus: "terminal damage and fragmentation evidence",
    },
    ScenarioDefinition {
        scenario_id: "low-endogenous-caveats",
        focus: "low endogenous content caveat behavior",
    },
    ScenarioDefinition {
        scenario_id: "adna-damage-refusal-boundary",
        focus: "aDNA damage refusal and caveat boundary",
    },
];

const G132_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "pmd-evidence-surface",
        focus: "PMD-like evidence reporting surface",
    },
    ScenarioDefinition {
        scenario_id: "authenticity-overclaim-guard",
        focus: "authenticity over-claim guardrails",
    },
    ScenarioDefinition {
        scenario_id: "authenticity-caveat-stability",
        focus: "authenticity caveat stability across repeats",
    },
];

const G133_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "mito-contam-prerequisites",
        focus: "mitochondrial contamination prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "mito-contam-confidence",
        focus: "confidence and caveat behavior for mito contamination",
    },
    ScenarioDefinition {
        scenario_id: "mito-contam-failure-modes",
        focus: "mitochondrial contamination failure modes",
    },
];

const G134_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "nuclear-panel-build-compat",
        focus: "nuclear contamination panel/build compatibility",
    },
    ScenarioDefinition {
        scenario_id: "nuclear-coverage-constraints",
        focus: "coverage constraints for nuclear contamination",
    },
    ScenarioDefinition {
        scenario_id: "nuclear-estimator-output-boundary",
        focus: "estimator output and caveat boundary",
    },
];

const G135_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "prealign-endogenous-interpretation",
        focus: "prealignment endogenous-content interpretation",
    },
    ScenarioDefinition {
        scenario_id: "postalign-endogenous-interpretation",
        focus: "postalignment endogenous-content interpretation",
    },
    ScenarioDefinition {
        scenario_id: "contaminant-interaction-effects",
        focus: "contaminant interaction effects on endogenous estimates",
    },
];

const G136_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sex-inference-boundary",
        focus: "sex inference boundary conditions",
    },
    ScenarioDefinition {
        scenario_id: "par-caveats",
        focus: "PAR caveat behavior and missing coverage handling",
    },
    ScenarioDefinition {
        scenario_id: "unsuitable-reference-refusal",
        focus: "unsuitable reference refusal behavior",
    },
];

const G137_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "organelle-y-readiness",
        focus: "organellar and Y evidence readiness",
    },
    ScenarioDefinition {
        scenario_id: "haplogroup-overclaim-guard",
        focus: "haplogroup over-claim prevention",
    },
    ScenarioDefinition {
        scenario_id: "haplogroup-readiness-reporting",
        focus: "haplogroup readiness reporting stability",
    },
];

const G138_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "kinship-marker-overlap",
        focus: "kinship marker overlap prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "kinship-coverage-identity",
        focus: "coverage and sample identity prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "kinship-contam-prechecks",
        focus: "contamination precheck behavior for kinship",
    },
];

const G139_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "predicted-vs-actual-cpu-mem",
        focus: "predicted vs actual CPU and memory",
    },
    ScenarioDefinition {
        scenario_id: "predicted-vs-actual-io-scratch",
        focus: "predicted vs actual IO and scratch usage",
    },
    ScenarioDefinition {
        scenario_id: "resource-tuning-patch-surface",
        focus: "resource tuning patch surface and impact",
    },
];

const G140_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "modern-wgs-template",
        focus: "modern WGS QC template behavior",
    },
    ScenarioDefinition {
        scenario_id: "adna-lowpass-template",
        focus: "aDNA and low-pass readiness templates",
    },
    ScenarioDefinition {
        scenario_id: "targeted-batch-coverage-template",
        focus: "targeted, batch merge, and coverage review templates",
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
    GoalDefinition {
        goal_id: "G131",
        title: "benchmark ancient-DNA damage evidence",
        stage_ids: &["bam.damage"],
        scenarios: G131_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G132",
        title: "benchmark PMD/authenticity advisory outputs",
        stage_ids: &["bam.authenticity"],
        scenarios: G132_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G133",
        title: "benchmark mitochondrial contamination",
        stage_ids: &["bam.contamination"],
        scenarios: G133_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G134",
        title: "benchmark nuclear contamination",
        stage_ids: &["bam.contamination"],
        scenarios: G134_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G135",
        title: "benchmark endogenous-content estimation",
        stage_ids: &["bam.endogenous_content"],
        scenarios: G135_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G136",
        title: "benchmark sex/PAR evidence",
        stage_ids: &["bam.sex"],
        scenarios: G136_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G137",
        title: "benchmark haplogroup-readiness",
        stage_ids: &["bam.haplogroups"],
        scenarios: G137_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G138",
        title: "benchmark kinship prerequisites",
        stage_ids: &["bam.kinship"],
        scenarios: G138_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G139",
        title: "benchmark BAM resource estimation",
        stage_ids: &["bam.qc_pre", "bam.mapping_summary", "bam.coverage"],
        scenarios: G139_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G140",
        title: "benchmark full BAM templates",
        stage_ids: &[
            "bam.qc_pre",
            "bam.align",
            "bam.markdup",
            "bam.mapping_summary",
            "bam.coverage",
            "bam.damage",
            "bam.authenticity",
        ],
        scenarios: G140_SCENARIOS,
    },
];

fn publish_json(path: &Path, value: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let payload = serde_json::to_vec_pretty(value)?;
    bijux_dna_infra::atomic_write_bytes(path, &payload)
        .with_context(|| format!("publish {}", path.display()))
}

fn selected_goal_ids(raw: Option<&str>) -> Result<Vec<String>> {
    let available =
        BAM_GOALS_CATALOG.iter().map(|goal| goal.goal_id.to_string()).collect::<BTreeSet<_>>();
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
    let unknown =
        selected.iter().filter(|goal| !available.contains(*goal)).cloned().collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!("unknown bam goals requested: {}", unknown.join(",")));
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
        "G125" => vec![
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
                findings
                    .iter()
                    .filter(|finding| finding.failure_class.contains("readiness"))
                    .count()
            ),
        ],
        "G126" => vec![
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
        ],
        "G127" => vec![
            format!("duplicate_rows_present={}", !rows.is_empty()),
            format!(
                "duplicate_markdup_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.markdup")
            ),
            format!(
                "duplicate_metrics_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.duplication_metrics")
            ),
            format!(
                "duplicate_method_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G128" => vec![
            format!("mapq_rows_present={}", !rows.is_empty()),
            format!(
                "mapq_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.mapq_filter")
            ),
            format!(
                "mapq_repetition_floor={}",
                rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
            ),
            format!(
                "mapq_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G129" => vec![
            format!("mapping_summary_rows_present={}", !rows.is_empty()),
            format!(
                "mapping_summary_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
            ),
            format!(
                "mapping_summary_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "mapping_summary_findings={}",
                findings
                    .iter()
                    .filter(|finding| {
                        finding.appraiser_id == "artifact-validity"
                            || finding.appraiser_id == "scientific-output"
                    })
                    .count()
            ),
        ],
        "G130" => vec![
            format!("coverage_rows_present={}", !rows.is_empty()),
            format!(
                "coverage_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.coverage")
            ),
            format!(
                "coverage_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(
                        |row| row.readiness_class == "degraded" || row.readiness_class == "refuse"
                    )
                    .count()
            ),
            format!(
                "coverage_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness"))
                    .count()
            ),
        ],
        "G131" => vec![
            format!("adna_damage_rows_present={}", !rows.is_empty()),
            format!(
                "adna_damage_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.damage")
            ),
            format!(
                "adna_damage_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "adna_damage_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G132" => vec![
            format!("authenticity_rows_present={}", !rows.is_empty()),
            format!(
                "authenticity_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.authenticity")
            ),
            format!(
                "authenticity_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(
                        |row| row.readiness_class == "degraded" || row.readiness_class == "refuse"
                    )
                    .count()
            ),
            format!(
                "authenticity_caveat_findings={}",
                findings.iter().filter(|finding| finding.appraiser_id == "failure-class").count()
            ),
        ],
        "G133" => vec![
            format!("mito_contam_rows_present={}", !rows.is_empty()),
            format!(
                "mito_contam_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.contamination")
            ),
            format!(
                "mito_contam_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "mito_contam_findings={}",
                findings
                    .iter()
                    .filter(|finding| {
                        finding.appraiser_id == "scientific-output"
                            || finding.appraiser_id == "failure-class"
                    })
                    .count()
            ),
        ],
        "G134" => vec![
            format!("nuclear_contam_rows_present={}", !rows.is_empty()),
            format!(
                "nuclear_contam_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.contamination")
            ),
            format!(
                "nuclear_contam_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "nuclear_contam_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness"))
                    .count()
            ),
        ],
        "G135" => vec![
            format!("endogenous_rows_present={}", !rows.is_empty()),
            format!(
                "endogenous_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.endogenous_content")
            ),
            format!(
                "endogenous_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "endogenous_scientific_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G136" => vec![
            format!("sex_par_rows_present={}", !rows.is_empty()),
            format!("sex_par_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.sex")),
            format!(
                "sex_par_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "sex_par_caveat_findings={}",
                findings.iter().filter(|finding| finding.appraiser_id == "failure-class").count()
            ),
        ],
        "G137" => vec![
            format!("haplogroup_rows_present={}", !rows.is_empty()),
            format!(
                "haplogroup_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.haplogroups")
            ),
            format!(
                "haplogroup_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "haplogroup_readiness_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G138" => vec![
            format!("kinship_rows_present={}", !rows.is_empty()),
            format!("kinship_stage_bound={}", rows.iter().any(|row| row.stage_id == "bam.kinship")),
            format!(
                "kinship_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "kinship_refusal_findings={}",
                findings.iter().filter(|finding| finding.severity == "critical").count()
            ),
        ],
        "G139" => vec![
            format!("resource_rows_present={}", !rows.is_empty()),
            format!(
                "resource_qc_pre_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.qc_pre")
            ),
            format!(
                "resource_summary_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.mapping_summary")
            ),
            format!(
                "resource_coverage_bound={}",
                rows.iter().any(|row| row.stage_id == "bam.coverage")
            ),
            format!(
                "resource_runtime_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "runtime-performance")
                    .count()
            ),
        ],
        "G140" => vec![
            format!("full_template_rows_present={}", !rows.is_empty()),
            format!(
                "full_template_core_stages_covered={}",
                rows.iter()
                    .filter(|row| {
                        matches!(
                            row.stage_id.as_str(),
                            "bam.qc_pre"
                                | "bam.align"
                                | "bam.markdup"
                                | "bam.mapping_summary"
                                | "bam.coverage"
                                | "bam.damage"
                                | "bam.authenticity"
                        )
                    })
                    .count()
            ),
            format!(
                "full_template_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(
                        |row| row.readiness_class == "degraded" || row.readiness_class == "refuse"
                    )
                    .count()
            ),
            format!(
                "full_template_hardening_entries={}",
                queue_entries.iter().filter(|entry| entry.severity != "info").count()
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

        let row_ids = matched_rows.iter().map(|row| row.row_id.clone()).collect::<BTreeSet<_>>();

        let appraisal_findings = findings
            .iter()
            .filter(|finding| row_ids.contains(&finding.row_id))
            .cloned()
            .collect::<Vec<_>>();

        let hardening_entries = queue
            .iter()
            .filter_map(|entry| {
                let mut projected = entry.clone();
                projected.affected_rows.retain(|row| row_ids.contains(row));
                (!projected.affected_rows.is_empty()).then_some(projected)
            })
            .collect::<Vec<_>>();

        let status = status_for_goal(&matched_rows, &hardening_entries);
        let goal_checks = goal_specific_checks(
            goal.goal_id,
            &matched_rows,
            &appraisal_findings,
            &hardening_entries,
        );

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

/// Derive a validated BAM campaign from admitted matrix and appraisal products.
///
/// # Errors
/// Returns an error when a source product is invalid, campaign identities disagree, requested
/// goals are unknown, or the derived report violates its persisted contract.
pub fn derive_bam_benchmark_campaign(
    matrix: BenchmarkMatrixReport,
    appraisal: &AppraisalReport,
    queue: &HardeningQueueReport,
    requested_goals: Option<&str>,
) -> Result<BamBenchmarkCampaignReport> {
    validate_benchmark_matrix_report(&matrix).context("validate BAM campaign source matrix")?;
    validate_appraisal_report(appraisal).context("validate BAM campaign source appraisal")?;
    validate_hardening_queue_report(queue).context("validate BAM campaign source queue")?;
    if matrix.domain != "bam"
        || appraisal.domain != matrix.domain
        || queue.domain != matrix.domain
        || appraisal.campaign_id != matrix.campaign_id
        || queue.campaign_id != matrix.campaign_id
    {
        return Err(anyhow!("BAM campaign source identities do not align"));
    }
    let selected_goals = selected_goal_ids(requested_goals)?;
    let entries = build_goal_entries(&selected_goals, &matrix, &appraisal.findings, &queue.entries);
    let report = BamBenchmarkCampaignReport {
        schema_version: BAM_BENCHMARK_CAMPAIGN_SCHEMA_V1.to_string(),
        campaign_id: matrix.campaign_id,
        domain: matrix.domain,
        selected_goals,
        summary: summarize(&entries),
        entries,
    };
    validate_bam_benchmark_campaign_report(&report)?;
    Ok(report)
}

/// Load and admit a persisted BAM campaign report.
///
/// # Errors
/// Returns an error when the report cannot be read, parsed, or admitted.
pub fn load_bam_benchmark_campaign_report(path: &Path) -> Result<BamBenchmarkCampaignReport> {
    let raw = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let report: BamBenchmarkCampaignReport =
        serde_json::from_slice(&raw).with_context(|| format!("parse {}", path.display()))?;
    validate_bam_benchmark_campaign_report(&report)
        .with_context(|| format!("validate {}", path.display()))?;
    Ok(report)
}

/// Validate and atomically publish a BAM campaign report.
///
/// # Errors
/// Returns an error when admission or atomic publication fails.
pub fn publish_bam_benchmark_campaign_report(
    path: &Path,
    report: &BamBenchmarkCampaignReport,
) -> Result<()> {
    validate_bam_benchmark_campaign_report(report)
        .context("validate BAM campaign report publication")?;
    publish_json(path, report)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use std::collections::BTreeMap;

    use super::{
        build_goal_entries, derive_bam_benchmark_campaign, load_bam_benchmark_campaign_report,
        publish_bam_benchmark_campaign_report, selected_goal_ids, summarize, AppraisalFinding,
        BamMatrixRowRef, HardeningQueueEntry, BAM_GOALS_CATALOG,
    };
    use bijux_dna_bench_model::{
        AppraisalReport, AppraisalSummary, BenchmarkMatrixReport, BenchmarkMatrixRow,
        BenchmarkMatrixSummary, BenchmarkReadiness, BenchmarkSurfaceMatch, HardeningQueueReport,
        APPRAISAL_REPORT_SCHEMA_V1, HARDENING_QUEUE_REPORT_SCHEMA_V1,
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
            readiness: BenchmarkReadiness { class: readiness.to_string(), reasons: Vec::new() },
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
                total_rows: 16,
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
                row("b10", "bam.damage", "ready"),
                row("b11", "bam.authenticity", "degraded"),
                row("b12", "bam.contamination", "ready"),
                row("b13", "bam.endogenous_content", "degraded"),
                row("b14", "bam.sex", "ready"),
                row("b15", "bam.haplogroups", "degraded"),
                row("b16", "bam.kinship", "ready"),
            ],
        }
    }

    fn valid_source_matrix() -> BenchmarkMatrixReport {
        BenchmarkMatrixReport {
            schema_version: "bijux.hpc.benchmark_matrix.v1".to_string(),
            campaign_id: "campaign-bam".to_string(),
            domain: "bam".to_string(),
            domains: vec!["bam".to_string()],
            generated_at: "2026-08-11T00:00:00Z".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 1,
                readiness_counts: BTreeMap::from([("ready".to_string(), 1)]),
                domain_counts: BTreeMap::from([("bam".to_string(), 1)]),
            },
            rows: vec![row("bam-row-1", "bam.validate", "ready")],
        }
    }

    fn valid_appraisal() -> AppraisalReport {
        AppraisalReport {
            schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
            campaign_id: "campaign-bam".to_string(),
            domain: "bam".to_string(),
            findings: Vec::new(),
            summary: AppraisalSummary {
                total_findings: 0,
                by_appraiser: BTreeMap::new(),
                by_severity: BTreeMap::new(),
            },
        }
    }

    fn valid_queue() -> HardeningQueueReport {
        HardeningQueueReport {
            schema_version: HARDENING_QUEUE_REPORT_SCHEMA_V1.to_string(),
            campaign_id: "campaign-bam".to_string(),
            domain: "bam".to_string(),
            entries: Vec::new(),
        }
    }

    #[test]
    fn derives_publishes_and_loads_admitted_bam_campaign() -> anyhow::Result<()> {
        let report = derive_bam_benchmark_campaign(
            valid_source_matrix(),
            &valid_appraisal(),
            &valid_queue(),
            Some("G121"),
        )?;
        assert_eq!(report.summary.total_goals, 1);
        assert_eq!(report.summary.total_rows, 1);
        assert_eq!(report.entries[0].status, "ready-for-benchmark-run");

        let root = tempfile::tempdir()?;
        let path = root.path().join("bam-campaign.json");
        publish_bam_benchmark_campaign_report(&path, &report)?;
        assert_eq!(load_bam_benchmark_campaign_report(&path)?, report);
        Ok(())
    }

    #[test]
    fn derivation_rejects_misaligned_source_identity() {
        let mut queue = valid_queue();
        queue.campaign_id = "another-campaign".to_string();
        assert!(derive_bam_benchmark_campaign(
            valid_source_matrix(),
            &valid_appraisal(),
            &queue,
            Some("G121"),
        )
        .is_err_and(|error| error.to_string().contains("source identities")));
    }

    #[test]
    fn bam_catalog_covers_declared_id_span() {
        assert_eq!(BAM_GOALS_CATALOG.len(), 20);
        assert_eq!(BAM_GOALS_CATALOG[0].goal_id, "G121");
        assert_eq!(BAM_GOALS_CATALOG[9].goal_id, "G130");
        assert_eq!(BAM_GOALS_CATALOG[10].goal_id, "G131");
        assert_eq!(BAM_GOALS_CATALOG[19].goal_id, "G140");
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
            affected_rows: vec!["b8".to_string(), "b1".to_string()],
            source_appraisers: vec!["scientific-output".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].goal_id, "G121");
        assert_eq!(entries[1].goal_id, "G127");
        assert_eq!(entries[2].goal_id, "G130");
        assert_eq!(entries[0].hardening_entries[0].affected_rows, ["b1"]);
        assert_eq!(entries[2].hardening_entries[0].affected_rows, ["b8"]);

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
    fn emits_validation_boundary_checks() {
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
    fn emits_bwa_alignment_checks() {
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
    fn emits_bowtie2_alignment_checks() {
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
    fn emits_sort_index_checks() {
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

    #[test]
    fn emits_read_group_propagation_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G125".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b1".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
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
            .any(|check| check.starts_with("read_group_validate_stage_bound=true")));
    }

    #[test]
    fn emits_bam_merge_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G126".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b3".to_string(),
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
            .any(|check| check.starts_with("bam_merge_qc_pre_stage_bound=true")));
    }

    #[test]
    fn emits_duplicate_handling_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G127".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "b5".to_string(),
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
            .any(|check| check.starts_with("duplicate_metrics_stage_bound=true")));
    }

    #[test]
    fn emits_mapq_filtering_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G128".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "b6".to_string(),
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
            .any(|check| check.starts_with("mapq_stage_bound=true")));
    }

    #[test]
    fn emits_mapping_summary_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G129".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "artifact-validity".to_string(),
            row_id: "b7".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "image-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("mapping_summary_stage_bound=true")));
    }

    #[test]
    fn emits_coverage_regime_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G130".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b8".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
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
            .any(|check| check.starts_with("coverage_stage_bound=true")));
    }

    #[test]
    fn emits_adna_damage_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G131".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "b10".to_string(),
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
            .any(|check| check.starts_with("adna_damage_stage_bound=true")));
    }

    #[test]
    fn emits_authenticity_advisory_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G132".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b11".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
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
            .any(|check| check.starts_with("authenticity_stage_bound=true")));
    }

    #[test]
    fn emits_mito_contamination_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G133".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "b12".to_string(),
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
            .any(|check| check.starts_with("mito_contam_stage_bound=true")));
    }

    #[test]
    fn emits_nuclear_contamination_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G134".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b12".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
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
            .any(|check| check.starts_with("nuclear_contam_stage_bound=true")));
    }

    #[test]
    fn emits_endogenous_content_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G135".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "b13".to_string(),
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
            .any(|check| check.starts_with("endogenous_stage_bound=true")));
    }

    #[test]
    fn emits_sex_par_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G136".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b14".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
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
            .any(|check| check.starts_with("sex_par_stage_bound=true")));
    }

    #[test]
    fn emits_haplogroup_readiness_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G137".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "b15".to_string(),
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
            .any(|check| check.starts_with("haplogroup_stage_bound=true")));
    }

    #[test]
    fn emits_kinship_prerequisite_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G138".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "b16".to_string(),
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
            .any(|check| check.starts_with("kinship_stage_bound=true")));
    }

    #[test]
    fn emits_resource_estimation_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G139".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "runtime-performance".to_string(),
            row_id: "b8".to_string(),
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
            .any(|check| check.starts_with("resource_qc_pre_bound=true")));
    }

    #[test]
    fn emits_full_bam_template_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G140".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("full_template_core_stages_covered=")));
    }
}
