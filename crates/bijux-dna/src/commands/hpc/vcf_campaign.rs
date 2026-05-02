use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::commands::cli::{BenchmarkMatrixArgs, VcfBenchmarkCampaignArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal, AppraisalFinding,
    BenchmarkMatrixReport, HardeningQueueEntry,
};

const VCF_CAMPAIGN_SCHEMA_VERSION: &str = "bijux.hpc.vcf_benchmark_campaign.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcfBenchmarkCampaignReport {
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub selected_goals: Vec<String>,
    pub summary: VcfBenchmarkCampaignSummary,
    pub entries: Vec<VcfBenchmarkGoalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcfBenchmarkCampaignSummary {
    pub total_goals: usize,
    pub total_rows: usize,
    pub total_findings: usize,
    pub total_queue_entries: usize,
    pub status_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcfBenchmarkGoalEntry {
    pub goal_id: String,
    pub title: String,
    pub stage_ids: Vec<String>,
    pub scenarios: Vec<VcfScenario>,
    pub matched_rows: Vec<VcfMatrixRowRef>,
    pub appraisal_findings: Vec<AppraisalFinding>,
    pub hardening_entries: Vec<HardeningQueueEntry>,
    pub goal_checks: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcfScenario {
    pub scenario_id: String,
    pub focus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcfMatrixRowRef {
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

const G141_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "header-format-boundaries",
        focus: "malformed header and INFO/FORMAT boundaries",
    },
    ScenarioDefinition {
        scenario_id: "sample-and-sort-boundaries",
        focus: "duplicate samples and unsorted record handling",
    },
    ScenarioDefinition {
        scenario_id: "index-reference-mismatch",
        focus: "index validity and reference mismatch refusal",
    },
];

const G142_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "core-count-summaries",
        focus: "variant counts and pass-rate summaries",
    },
    ScenarioDefinition {
        scenario_id: "missingness-het-titv",
        focus: "missingness, heterozygosity, and Ti/Tv stability",
    },
    ScenarioDefinition {
        scenario_id: "sample-level-stats",
        focus: "per-sample summary stability",
    },
];

const G143_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "site-sample-filter-scope",
        focus: "site and sample scoped filter expressions",
    },
    ScenarioDefinition {
        scenario_id: "missingness-threshold-boundary",
        focus: "missingness threshold boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "subset-identity-guard",
        focus: "subset identity and sample list stability",
    },
];

const G144_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "left-normalization-boundary",
        focus: "left-normalization boundary behavior",
    },
    ScenarioDefinition {
        scenario_id: "multiallelic-decomposition",
        focus: "multiallelic decomposition and duplicate handling",
    },
    ScenarioDefinition {
        scenario_id: "raw-normalized-diff",
        focus: "raw-to-normalized genotype retention diff",
    },
];

const G145_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "build-and-contig-aliases",
        focus: "build identity and contig alias compatibility",
    },
    ScenarioDefinition {
        scenario_id: "fasta-and-index-context",
        focus: "FASTA checksum and index state validation",
    },
    ScenarioDefinition {
        scenario_id: "panel-map-compatibility",
        focus: "panel and genetic map compatibility boundaries",
    },
];

const G146_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "damage-remove-policy",
        focus: "damage remove policy boundary",
    },
    ScenarioDefinition {
        scenario_id: "damage-mask-policy",
        focus: "damage mask policy boundary",
    },
    ScenarioDefinition {
        scenario_id: "damage-annotate-policy",
        focus: "damage annotate policy and caveats",
    },
];

const G147_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "diploid-prerequisites",
        focus: "diploid calling prerequisites and refusal boundaries",
    },
    ScenarioDefinition {
        scenario_id: "diploid-ploidy-reference",
        focus: "ploidy and reference compatibility",
    },
    ScenarioDefinition {
        scenario_id: "diploid-confidence-boundary",
        focus: "diploid confidence and caveat boundary",
    },
];

const G148_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "pseudo-low-coverage-assumptions",
        focus: "low-coverage pseudohaploid assumptions",
    },
    ScenarioDefinition {
        scenario_id: "pseudo-adna-assumptions",
        focus: "aDNA pseudohaploid assumptions and caveats",
    },
    ScenarioDefinition {
        scenario_id: "pseudo-uncertainty-propagation",
        focus: "uncertainty propagation across outputs",
    },
];

const G149_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "gl-caller-outputs",
        focus: "genotype-likelihood caller output integrity",
    },
    ScenarioDefinition {
        scenario_id: "gl-propagation",
        focus: "GL/PL propagation across downstream stages",
    },
    ScenarioDefinition {
        scenario_id: "gl-compatibility",
        focus: "GL-bearing output downstream compatibility",
    },
];

const G150_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "phasing-panel-map-prereq",
        focus: "panel and map prerequisite enforcement",
    },
    ScenarioDefinition {
        scenario_id: "phasing-sample-prereq",
        focus: "sample-level phasing prerequisites",
    },
    ScenarioDefinition {
        scenario_id: "phasing-confidence-caveats",
        focus: "phasing confidence and caveat boundaries",
    },
];

const G151_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "panel-map-identity",
        focus: "panel and map identity compatibility",
    },
    ScenarioDefinition {
        scenario_id: "simulation-vs-production-labels",
        focus: "simulation and production label separation",
    },
    ScenarioDefinition {
        scenario_id: "imputation-confidence-surface",
        focus: "imputation confidence output surface",
    },
];

const G152_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sample-qc-and-missingness",
        focus: "sample QC and missingness boundaries",
    },
    ScenarioDefinition {
        scenario_id: "heterozygosity-relatedness-flags",
        focus: "heterozygosity and relatedness flag behavior",
    },
    ScenarioDefinition {
        scenario_id: "cohort-filter-impact",
        focus: "sample-count and filter impact on cohort QC",
    },
];

const G153_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "ld-pruning-boundary",
        focus: "LD pruning and marker count boundary",
    },
    ScenarioDefinition {
        scenario_id: "small-cohort-caveats",
        focus: "small cohort caveat and refusal behavior",
    },
    ScenarioDefinition {
        scenario_id: "sample-inclusion-guardrail",
        focus: "sample inclusion guardrail behavior",
    },
];

const G154_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "roh-marker-density",
        focus: "ROH marker density boundary",
    },
    ScenarioDefinition {
        scenario_id: "ibd-missingness-cohort-size",
        focus: "IBD missingness and cohort-size boundaries",
    },
    ScenarioDefinition {
        scenario_id: "method-assumption-caveats",
        focus: "ROH/IBD method assumption caveats",
    },
];

const G155_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "underpowered-cohort-refusal",
        focus: "underpowered cohort refusal behavior",
    },
    ScenarioDefinition {
        scenario_id: "missing-assumption-refusal",
        focus: "missing demography assumption refusal",
    },
    ScenarioDefinition {
        scenario_id: "incompatible-method-caveats",
        focus: "incompatible demography method caveats",
    },
];

const G156_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "annotation-source-version",
        focus: "annotation source and version provenance",
    },
    ScenarioDefinition {
        scenario_id: "field-coverage-drift",
        focus: "annotation field coverage and drift",
    },
    ScenarioDefinition {
        scenario_id: "gene-transcript-mapping",
        focus: "gene/transcript mapping stability",
    },
];

const G157_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "sv-like-record-detection",
        focus: "SV-like record detection and classification",
    },
    ScenarioDefinition {
        scenario_id: "small-variant-misuse-refusal",
        focus: "refusal when small-variant workflow is unsuitable",
    },
    ScenarioDefinition {
        scenario_id: "support-vs-refuse-boundary",
        focus: "support versus refusal boundary for SV-like inputs",
    },
];

const G158_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "population-handoff-trust-classes",
        focus: "population handoff trust-class propagation",
    },
    ScenarioDefinition {
        scenario_id: "filtered-normalized-handoff",
        focus: "filtered and normalized handoff boundaries",
    },
    ScenarioDefinition {
        scenario_id: "downstream-stage-caveat-flow",
        focus: "downstream caveat flow across population stages",
    },
];

const G159_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "stats-filter-scale",
        focus: "stats and filter scale behavior",
    },
    ScenarioDefinition {
        scenario_id: "normalize-annotation-scale",
        focus: "normalization and annotation scale behavior",
    },
    ScenarioDefinition {
        scenario_id: "runtime-memory-evidence-size",
        focus: "runtime/memory/evidence-size pressure",
    },
];

const G160_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "validation-normalization-template",
        focus: "validation and normalization template baseline",
    },
    ScenarioDefinition {
        scenario_id: "cohort-imputation-template",
        focus: "cohort QC and imputation template baseline",
    },
    ScenarioDefinition {
        scenario_id: "population-demography-template",
        focus: "population, ROH/IBD, and demography template baseline",
    },
];

const VCF_GOALS_CATALOG: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G141",
        title: "benchmark VCF validation",
        stage_ids: &["vcf.call", "vcf.filter"],
        scenarios: G141_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G142",
        title: "benchmark VCF stats",
        stage_ids: &["vcf.stats"],
        scenarios: G142_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G143",
        title: "benchmark VCF filtering",
        stage_ids: &["vcf.filter"],
        scenarios: G143_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G144",
        title: "benchmark VCF normalization",
        stage_ids: &["vcf.postprocess"],
        scenarios: G144_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G145",
        title: "benchmark VCF reference context",
        stage_ids: &["vcf.call", "vcf.prepare_reference_panel", "vcf.postprocess"],
        scenarios: G145_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G146",
        title: "benchmark damage-aware VCF filtering",
        stage_ids: &["vcf.damage_filter"],
        scenarios: G146_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G147",
        title: "benchmark diploid-calling boundary",
        stage_ids: &["vcf.call_diploid"],
        scenarios: G147_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G148",
        title: "benchmark pseudohaploid boundary",
        stage_ids: &["vcf.call_pseudohaploid"],
        scenarios: G148_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G149",
        title: "benchmark genotype-likelihood workflows",
        stage_ids: &["vcf.call_gl", "vcf.gl_propagation"],
        scenarios: G149_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G150",
        title: "benchmark phasing workflows",
        stage_ids: &["vcf.phasing", "vcf.prepare_reference_panel"],
        scenarios: G150_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G151",
        title: "benchmark imputation workflows",
        stage_ids: &[
            "vcf.impute",
            "vcf.imputation",
            "vcf.prepare_reference_panel",
            "vcf.postprocess",
        ],
        scenarios: G151_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G152",
        title: "benchmark cohort QC",
        stage_ids: &["vcf.qc", "vcf.stats", "vcf.filter"],
        scenarios: G152_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G153",
        title: "benchmark PCA/admixture guardrails",
        stage_ids: &["vcf.pca", "vcf.admixture", "vcf.population_structure", "vcf.filter"],
        scenarios: G153_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G154",
        title: "benchmark ROH/IBD boundaries",
        stage_ids: &["vcf.roh", "vcf.ibd", "vcf.filter"],
        scenarios: G154_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G155",
        title: "benchmark demography boundaries",
        stage_ids: &["vcf.demography", "vcf.ibd", "vcf.roh"],
        scenarios: G155_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G156",
        title: "benchmark annotation provenance",
        stage_ids: &["vcf.postprocess", "vcf.filter", "vcf.stats"],
        scenarios: G156_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G157",
        title: "benchmark structural-variant boundary",
        stage_ids: &["vcf.call", "vcf.filter", "vcf.qc"],
        scenarios: G157_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G158",
        title: "benchmark population-analysis handoff",
        stage_ids: &["vcf.postprocess", "vcf.pca", "vcf.roh", "vcf.ibd", "vcf.demography"],
        scenarios: G158_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G159",
        title: "benchmark VCF large-file behavior",
        stage_ids: &["vcf.stats", "vcf.filter", "vcf.postprocess", "vcf.impute"],
        scenarios: G159_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G160",
        title: "benchmark full VCF templates",
        stage_ids: &[
            "vcf.call",
            "vcf.filter",
            "vcf.stats",
            "vcf.qc",
            "vcf.impute",
            "vcf.pca",
            "vcf.roh",
            "vcf.ibd",
            "vcf.demography",
            "vcf.phasing",
        ],
        scenarios: G160_SCENARIOS,
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
    let available = VCF_GOALS_CATALOG
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
        return Err(anyhow!("vcf-benchmark-campaign requires at least one goal"));
    }
    let unknown = selected
        .iter()
        .filter(|goal| !available.contains(*goal))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(anyhow!(
            "unknown vcf goals requested: {}",
            unknown.join(",")
        ));
    }
    let mut ordered = VCF_GOALS_CATALOG
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

fn status_for_goal(rows: &[VcfMatrixRowRef], queue_entries: &[HardeningQueueEntry]) -> String {
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
    rows: &[VcfMatrixRowRef],
    findings: &[AppraisalFinding],
    queue_entries: &[HardeningQueueEntry],
) -> Vec<String> {
    match goal_id {
        "G141" => vec![
            format!("validation_rows_present={}", !rows.is_empty()),
            format!(
                "validation_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "validation_call_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "validation_filter_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "validation_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
            format!(
                "validation_failure_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "failure-class")
                    .count()
            ),
        ],
        "G142" => vec![
            format!("stats_rows_present={}", !rows.is_empty()),
            format!(
                "stats_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "stats_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "stats_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "stats_scientific_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G143" => vec![
            format!("filter_rows_present={}", !rows.is_empty()),
            format!(
                "filter_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
            format!(
                "filter_degraded_rows={}",
                rows.iter().filter(|row| row.readiness_class == "degraded").count()
            ),
            format!(
                "filter_repetition_floor={}",
                rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
            ),
            format!(
                "filter_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness"))
                    .count()
            ),
        ],
        "G144" => vec![
            format!("normalization_rows_present={}", !rows.is_empty()),
            format!(
                "normalization_postprocess_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "normalization_repetition_floor={}",
                rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
            ),
            format!(
                "normalization_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!(
                "normalization_findings={}",
                findings
                    .iter()
                    .filter(|finding| {
                        finding.appraiser_id == "artifact-validity"
                            || finding.appraiser_id == "scientific-output"
                    })
                    .count()
            ),
        ],
        "G145" => vec![
            format!("reference_context_rows_present={}", !rows.is_empty()),
            format!(
                "reference_context_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "reference_context_call_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call")
            ),
            format!(
                "reference_context_panel_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.prepare_reference_panel")
            ),
            format!(
                "reference_context_postprocess_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.postprocess")
            ),
            format!(
                "reference_context_failure_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "failure-class")
                    .count()
            ),
        ],
        "G146" => vec![
            format!("damage_filter_rows_present={}", !rows.is_empty()),
            format!(
                "damage_filter_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.damage_filter")
            ),
            format!(
                "damage_filter_repetition_floor={}",
                rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
            ),
            format!(
                "damage_filter_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "damage_filter_caveat_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G147" => vec![
            format!("diploid_rows_present={}", !rows.is_empty()),
            format!(
                "diploid_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call_diploid")
            ),
            format!(
                "diploid_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.failure_class.contains("readiness"))
                    .count()
            ),
            format!(
                "diploid_degraded_or_refuse_rows={}",
                rows.iter()
                    .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                    .count()
            ),
            format!(
                "diploid_refusal_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.severity == "critical")
                    .count()
            ),
        ],
        "G148" => vec![
            format!("pseudohaploid_rows_present={}", !rows.is_empty()),
            format!(
                "pseudohaploid_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call_pseudohaploid")
            ),
            format!(
                "pseudohaploid_repetition_floor={}",
                rows.iter().map(|row| row.repetitions).min().unwrap_or(0)
            ),
            format!(
                "pseudohaploid_ready_rows={}",
                rows.iter().filter(|row| row.readiness_class == "ready").count()
            ),
            format!(
                "pseudohaploid_caveat_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "failure-class")
                    .count()
            ),
        ],
        "G149" => vec![
            format!("gl_rows_present={}", !rows.is_empty()),
            format!(
                "gl_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "gl_call_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.call_gl")
            ),
            format!(
                "gl_propagation_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.gl_propagation")
            ),
            format!(
                "gl_findings={}",
                findings
                    .iter()
                    .filter(|finding| finding.appraiser_id == "scientific-output")
                    .count()
            ),
        ],
        "G150" => vec![
            format!("phasing_rows_present={}", !rows.is_empty()),
            format!(
                "phasing_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "phasing_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.phasing")
            ),
            format!(
                "phasing_panel_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.prepare_reference_panel")
            ),
            format!(
                "phasing_queue_entries={}",
                queue_entries
                    .iter()
                    .filter(|entry| entry.severity != "info")
                    .count()
            ),
        ],
        "G151" => vec![
            format!("imputation_rows_present={}", !rows.is_empty()),
            format!(
                "imputation_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "imputation_impute_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.impute")
            ),
            format!(
                "imputation_panel_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.prepare_reference_panel")
            ),
            format!(
                "imputation_refuse_rows={}",
                rows.iter().filter(|row| row.readiness_class == "refuse").count()
            ),
        ],
        "G152" => vec![
            format!("cohort_qc_rows_present={}", !rows.is_empty()),
            format!(
                "cohort_qc_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "cohort_qc_qc_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.qc")
            ),
            format!(
                "cohort_qc_stats_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.stats")
            ),
            format!(
                "cohort_qc_filter_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.filter")
            ),
        ],
        "G153" => vec![
            format!("population_guardrail_rows_present={}", !rows.is_empty()),
            format!(
                "population_guardrail_stage_count={}",
                rows.iter()
                    .map(|row| row.stage_id.clone())
                    .collect::<BTreeSet<_>>()
                    .len()
            ),
            format!(
                "population_guardrail_pca_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.pca")
            ),
            format!(
                "population_guardrail_admixture_stage_bound={}",
                rows.iter().any(|row| row.stage_id == "vcf.admixture")
            ),
            format!(
                "population_guardrail_refuse_rows={}",
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
) -> Vec<VcfBenchmarkGoalEntry> {
    let selected = selected_goal_ids.iter().cloned().collect::<BTreeSet<_>>();
    let mut entries = Vec::new();

    for goal in VCF_GOALS_CATALOG.iter().filter(|goal| selected.contains(goal.goal_id)) {
        let stage_set = goal.stage_ids.iter().copied().collect::<BTreeSet<_>>();
        let matched_rows = matrix
            .rows
            .iter()
            .filter(|row| stage_set.contains(row.stage_id.as_str()))
            .map(|row| VcfMatrixRowRef {
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

        entries.push(VcfBenchmarkGoalEntry {
            goal_id: goal.goal_id.to_string(),
            title: goal.title.to_string(),
            stage_ids: goal.stage_ids.iter().map(|stage| (*stage).to_string()).collect(),
            scenarios: goal
                .scenarios
                .iter()
                .map(|scenario| VcfScenario {
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

fn summarize(entries: &[VcfBenchmarkGoalEntry]) -> VcfBenchmarkCampaignSummary {
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
    VcfBenchmarkCampaignSummary {
        total_goals: entries.len(),
        total_rows,
        total_findings,
        total_queue_entries,
        status_counts,
    }
}

pub fn vcf_benchmark_campaign(args: &VcfBenchmarkCampaignArgs) -> Result<VcfBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_overrides: args.user_overrides.clone(),
        domain: "vcf".to_string(),
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
    let report = VcfBenchmarkCampaignReport {
        schema_version: VCF_CAMPAIGN_SCHEMA_VERSION.to_string(),
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
        build_goal_entries, selected_goal_ids, summarize, AppraisalFinding, HardeningQueueEntry,
        VcfMatrixRowRef, VCF_GOALS_CATALOG,
    };
    use crate::commands::hpc::{
        BenchmarkMatrixReport, BenchmarkMatrixRow, BenchmarkMatrixSummary, BenchmarkReadiness,
        BenchmarkSurfaceMatch,
    };

    fn row(row_id: &str, stage_id: &str, readiness: &str) -> BenchmarkMatrixRow {
        BenchmarkMatrixRow {
            row_id: row_id.to_string(),
            matrix_domain: "vcf".to_string(),
            stage_id: stage_id.to_string(),
            tool_id: "bcftools_v1".to_string(),
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
                matched_profile: "bcftools".to_string(),
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
            campaign_id: "campaign-vcf".to_string(),
            domain: "vcf".to_string(),
            domains: vec!["vcf".to_string()],
            generated_at: "0".to_string(),
            summary: BenchmarkMatrixSummary {
                total_rows: 20,
                readiness_counts: std::collections::BTreeMap::new(),
                domain_counts: std::collections::BTreeMap::new(),
            },
            rows: vec![
                row("v1", "vcf.call", "ready"),
                row("v2", "vcf.filter", "degraded"),
                row("v3", "vcf.stats", "ready"),
                row("v4", "vcf.postprocess", "ready"),
                row("v5", "vcf.prepare_reference_panel", "degraded"),
                row("v6", "vcf.damage_filter", "ready"),
                row("v7", "vcf.call_diploid", "refuse"),
                row("v8", "vcf.call_pseudohaploid", "ready"),
                row("v9", "vcf.call_gl", "ready"),
                row("v10", "vcf.gl_propagation", "ready"),
                row("v11", "vcf.phasing", "degraded"),
                row("v12", "vcf.impute", "degraded"),
                row("v13", "vcf.imputation", "ready"),
                row("v14", "vcf.qc", "ready"),
                row("v15", "vcf.pca", "degraded"),
                row("v16", "vcf.admixture", "ready"),
                row("v17", "vcf.population_structure", "ready"),
                row("v18", "vcf.roh", "degraded"),
                row("v19", "vcf.ibd", "ready"),
                row("v20", "vcf.demography", "refuse"),
            ],
        }
    }

    #[test]
    fn vcf_catalog_includes_iteration_12_and_13_goals() {
        assert_eq!(VCF_GOALS_CATALOG.len(), 20);
        assert_eq!(VCF_GOALS_CATALOG[0].goal_id, "G141");
        assert_eq!(VCF_GOALS_CATALOG[9].goal_id, "G150");
        assert_eq!(VCF_GOALS_CATALOG[10].goal_id, "G151");
        assert_eq!(VCF_GOALS_CATALOG[19].goal_id, "G160");
    }

    #[test]
    fn selected_goal_ids_reject_unknown_entries() {
        let err = selected_goal_ids(Some("G141,G999")).expect_err("expected invalid goal");
        assert!(err.to_string().contains("unknown vcf goals requested"));
    }

    #[test]
    fn goal_entries_map_rows_findings_and_queue() {
        let matrix = matrix_fixture();
        let selected = vec!["G141".to_string(), "G147".to_string(), "G149".to_string()];
        let findings = vec![
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "v2".to_string(),
                severity: "warning".to_string(),
                confidence: "high".to_string(),
                failure_class: "readiness-degraded".to_string(),
                result_scope: "encrypted-results".to_string(),
                summary: "x".to_string(),
                recommendation: "y".to_string(),
            },
            AppraisalFinding {
                appraiser_id: "failure-class".to_string(),
                row_id: "v7".to_string(),
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
            affected_rows: vec!["v7".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];

        let entries = build_goal_entries(&selected, &matrix, &findings, &queue);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].goal_id, "G141");
        assert_eq!(entries[1].goal_id, "G147");
        assert_eq!(entries[1].status, "requires-hardening");
        assert_eq!(entries[2].goal_id, "G149");
        assert_eq!(entries[2].matched_rows.len(), 2);

        let summary = summarize(&entries);
        assert_eq!(summary.total_goals, 3);
        assert_eq!(summary.total_rows, 5);
        assert_eq!(summary.status_counts.get("requires-hardening"), Some(&1));
    }

    #[test]
    fn summary_counts_mixed_status_entries() {
        let entries = vec![
            super::VcfBenchmarkGoalEntry {
                goal_id: "G141".to_string(),
                title: "benchmark VCF validation".to_string(),
                stage_ids: vec!["vcf.call".to_string(), "vcf.filter".to_string()],
                scenarios: Vec::new(),
                matched_rows: vec![VcfMatrixRowRef {
                    row_id: "v1".to_string(),
                    stage_id: "vcf.call".to_string(),
                    tool_id: "bcftools_v1".to_string(),
                    readiness_class: "ready".to_string(),
                    repetitions: 5,
                }],
                appraisal_findings: Vec::new(),
                hardening_entries: Vec::new(),
                goal_checks: Vec::new(),
                status: "ready-for-benchmark-run".to_string(),
            },
            super::VcfBenchmarkGoalEntry {
                goal_id: "G142".to_string(),
                title: "benchmark VCF stats".to_string(),
                stage_ids: vec!["vcf.stats".to_string()],
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
    fn goal_141_emits_validation_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G141".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "v2".to_string(),
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
            .any(|check| check.starts_with("validation_stage_count=2")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("validation_call_stage_bound=true")));
    }

    #[test]
    fn goal_142_emits_stats_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G142".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "v3".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "stats-drift".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("stats_degraded_rows=0")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("stats_stage_bound=true")));
    }

    #[test]
    fn goal_143_emits_filtering_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G143".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("filter_degraded_rows=1")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("filter_stage_bound=true")));
    }

    #[test]
    fn goal_144_emits_normalization_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G144".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "artifact-validity".to_string(),
            row_id: "v4".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "normalization-diff".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("normalization_repetition_floor=5")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("normalization_postprocess_stage_bound=true")));
    }

    #[test]
    fn goal_145_emits_reference_context_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G145".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "v5".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "reference-mismatch".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("reference_context_stage_count=3")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("reference_context_panel_bound=true")));
    }

    #[test]
    fn goal_146_emits_damage_aware_filtering_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G146".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "v6".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "damage-policy-drift".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("damage_filter_repetition_floor=5")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("damage_filter_stage_bound=true")));
    }

    #[test]
    fn goal_147_emits_diploid_boundary_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G147".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "v7".to_string(),
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
            .any(|check| check.starts_with("diploid_queue_entries=0")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("diploid_stage_bound=true")));
    }

    #[test]
    fn goal_148_emits_pseudohaploid_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G148".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "failure-class".to_string(),
            row_id: "v8".to_string(),
            severity: "warning".to_string(),
            confidence: "high".to_string(),
            failure_class: "uncertainty-caveat".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("pseudohaploid_repetition_floor=5")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("pseudohaploid_stage_bound=true")));
    }

    #[test]
    fn goal_149_emits_gl_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G149".to_string()];
        let findings = vec![AppraisalFinding {
            appraiser_id: "scientific-output".to_string(),
            row_id: "v10".to_string(),
            severity: "warning".to_string(),
            confidence: "medium".to_string(),
            failure_class: "gl-propagation-drift".to_string(),
            result_scope: "encrypted-results".to_string(),
            summary: "x".to_string(),
            recommendation: "y".to_string(),
        }];
        let entries = build_goal_entries(&selected, &matrix, &findings, &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("gl_stage_count=2")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("gl_propagation_stage_bound=true")));
    }

    #[test]
    fn goal_150_emits_phasing_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G150".to_string()];
        let queue = vec![HardeningQueueEntry {
            queue_id: "hardening-0002".to_string(),
            severity: "warning".to_string(),
            failure_class: "panel-map-gap".to_string(),
            recommendation: "pin map/panel revisions".to_string(),
            affected_rows: vec!["v11".to_string()],
            source_appraisers: vec!["failure-class".to_string()],
        }];
        let entries = build_goal_entries(&selected, &matrix, &[], &queue);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("phasing_stage_count=2")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("phasing_stage_bound=true")));
    }

    #[test]
    fn goal_151_emits_imputation_workflow_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G151".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("imputation_stage_count=4")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("imputation_impute_stage_bound=true")));
    }

    #[test]
    fn goal_152_emits_cohort_qc_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G152".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("cohort_qc_stage_count=3")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("cohort_qc_qc_stage_bound=true")));
    }

    #[test]
    fn goal_153_emits_population_guardrail_checks() {
        let matrix = matrix_fixture();
        let selected = vec!["G153".to_string()];
        let entries = build_goal_entries(&selected, &matrix, &[], &[]);
        assert_eq!(entries.len(), 1);
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("population_guardrail_stage_count=4")));
        assert!(entries[0]
            .goal_checks
            .iter()
            .any(|check| check.starts_with("population_guardrail_pca_stage_bound=true")));
    }
}
