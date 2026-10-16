//! Owner: pure FASTQ campaign preprocessing declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog::{
    FASTQ_CLUSTER_OTUS, FASTQ_DEDUPLICATE, FASTQ_DETECT_ADAPTERS, FASTQ_DETECT_DUPLICATES_PREMERGE,
    FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN, FASTQ_INFER_ASVS, FASTQ_NORMALIZE_ABUNDANCE,
    FASTQ_NORMALIZE_PRIMERS, FASTQ_REMOVE_CHIMERAS, FASTQ_TRIM, FASTQ_UMI,
};

const G108_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "adapter-rich",
        focus: "adapter-rich corpus detection sensitivity",
    },
    ScenarioDefinition { scenario_id: "adapter-free", focus: "adapter-free corpus specificity" },
    ScenarioDefinition {
        scenario_id: "backend-equivalence-window",
        focus: "cross-backend adapter detection equivalence",
    },
];

fn adapter_detection_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("adapter_rows_present={}", !rows.is_empty()),
        format!(
            "adapter_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_DETECT_ADAPTERS)
        ),
        format!(
            "adapter_backend_candidates={}",
            rows.iter().map(|row| row.tool_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "adapter_equivalence_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "backend-equivalence").count()
        ),
    ]
}

const G109_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "quality-minlen-policy",
        focus: "quality and min-length trimming policy",
    },
    ScenarioDefinition { scenario_id: "pair-retention", focus: "paired-end retention semantics" },
    ScenarioDefinition { scenario_id: "adna-damage-policy", focus: "aDNA-aware trimming policy" },
];

fn trimming_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("trim_rows_present={}", !rows.is_empty()),
        format!("trim_stage_bound={}", rows.iter().any(|row| row.stage_id == FASTQ_TRIM)),
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
    ]
}

const G110_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition { scenario_id: "umi-patterns", focus: "UMI pattern extraction coverage" },
    ScenarioDefinition {
        scenario_id: "failed-extraction",
        focus: "failed UMI extraction reporting",
    },
    ScenarioDefinition {
        scenario_id: "grouping-dedup-handoff",
        focus: "grouping and deduplication handoff",
    },
];

fn umi_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("umi_rows_present={}", !rows.is_empty()),
        format!("umi_extraction_stage_bound={}", rows.iter().any(|row| row.stage_id == FASTQ_UMI)),
        format!(
            "umi_grouping_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_DEDUPLICATE)
        ),
        format!(
            "umi_code_freeze_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "code-freeze").count()
        ),
    ]
}

const G111_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "prealign-duplicate-signal",
        focus: "prealignment duplicate signal consistency",
    },
    ScenarioDefinition {
        scenario_id: "complexity-model-regimes",
        focus: "library complexity model behavior across scales",
    },
    ScenarioDefinition {
        scenario_id: "complexity-caveat-propagation",
        focus: "duplicate/complexity caveat propagation",
    },
];

fn duplicate_complexity_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("duplicate_complexity_rows_present={}", !rows.is_empty()),
        format!(
            "duplicate_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_DETECT_DUPLICATES_PREMERGE)
        ),
        format!(
            "complexity_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN)
        ),
        format!(
            "duplicate_complexity_findings={}",
            findings
                .iter()
                .filter(|finding| {
                    finding.appraiser_id == "scientific-output"
                        || finding.appraiser_id == "runtime-performance"
                })
                .count()
        ),
    ]
}

const G116_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "primer-normalization-paths",
        focus: "primer normalization and trimming interplay",
    },
    ScenarioDefinition {
        scenario_id: "chimera-asv-otu-branches",
        focus: "chimera removal and ASV/OTU branch behavior",
    },
    ScenarioDefinition {
        scenario_id: "abundance-caveats",
        focus: "abundance normalization caveat handling",
    },
];

fn edna_preprocessing_checks(inputs: &CheckInputs<'_>) -> Vec<String> {
    let CheckInputs { rows, findings, .. } = inputs;
    vec![
        format!("edna_rows_present={}", !rows.is_empty()),
        format!(
            "edna_primer_stage_bound={}",
            rows.iter().any(|row| row.stage_id == FASTQ_NORMALIZE_PRIMERS)
        ),
        format!(
            "edna_branch_stages_covered={}",
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.stage_id.as_str(),
                        FASTQ_REMOVE_CHIMERAS
                            | FASTQ_INFER_ASVS
                            | FASTQ_CLUSTER_OTUS
                            | FASTQ_NORMALIZE_ABUNDANCE
                    )
                })
                .count()
        ),
        format!(
            "edna_scientific_findings={}",
            findings.iter().filter(|finding| finding.appraiser_id == "scientific-output").count()
        ),
    ]
}

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G108",
        title: "benchmark adapter detection",
        stage_ids: &[FASTQ_DETECT_ADAPTERS],
        scenarios: G108_SCENARIOS,
        checks: adapter_detection_checks,
    },
    GoalDefinition {
        goal_id: "G109",
        title: "benchmark FASTQ trimming",
        stage_ids: &[FASTQ_TRIM],
        scenarios: G109_SCENARIOS,
        checks: trimming_checks,
    },
    GoalDefinition {
        goal_id: "G110",
        title: "benchmark UMI extraction and grouping",
        stage_ids: &[FASTQ_UMI, FASTQ_DEDUPLICATE],
        scenarios: G110_SCENARIOS,
        checks: umi_checks,
    },
    GoalDefinition {
        goal_id: "G111",
        title: "benchmark FASTQ duplicate and complexity stages",
        stage_ids: &[FASTQ_DETECT_DUPLICATES_PREMERGE, FASTQ_ESTIMATE_LIBRARY_COMPLEXITY_PREALIGN],
        scenarios: G111_SCENARIOS,
        checks: duplicate_complexity_checks,
    },
    GoalDefinition {
        goal_id: "G116",
        title: "benchmark eDNA/metabarcoding preprocessing",
        stage_ids: &[
            FASTQ_NORMALIZE_PRIMERS,
            FASTQ_REMOVE_CHIMERAS,
            FASTQ_INFER_ASVS,
            FASTQ_CLUSTER_OTUS,
            FASTQ_NORMALIZE_ABUNDANCE,
        ],
        scenarios: G116_SCENARIOS,
        checks: edna_preprocessing_checks,
    },
];
