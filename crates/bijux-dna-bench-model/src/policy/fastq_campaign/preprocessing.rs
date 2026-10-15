//! Owner: pure FASTQ campaign preprocessing declarations and checks.
use super::{BTreeSet, CheckInputs, GoalDefinition, ScenarioDefinition};

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
            rows.iter().any(|row| row.stage_id == "fastq.detect_adapters")
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
        format!("trim_stage_bound={}", rows.iter().any(|row| row.stage_id == "fastq.trim_reads")),
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
            rows.iter().any(|row| row.stage_id == "fastq.detect_duplicates_premerge")
        ),
        format!(
            "complexity_stage_bound={}",
            rows.iter().any(|row| row.stage_id == "fastq.estimate_library_complexity_prealign")
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
            rows.iter().any(|row| row.stage_id == "fastq.normalize_primers")
        ),
        format!(
            "edna_branch_stages_covered={}",
            rows.iter()
                .filter(|row| {
                    matches!(
                        row.stage_id.as_str(),
                        "fastq.remove_chimeras"
                            | "fastq.infer_asvs"
                            | "fastq.cluster_otus"
                            | "fastq.normalize_abundance"
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
        stage_ids: &["fastq.detect_adapters"],
        scenarios: G108_SCENARIOS,
        checks: adapter_detection_checks,
    },
    GoalDefinition {
        goal_id: "G109",
        title: "benchmark FASTQ trimming",
        stage_ids: &["fastq.trim_reads"],
        scenarios: G109_SCENARIOS,
        checks: trimming_checks,
    },
    GoalDefinition {
        goal_id: "G110",
        title: "benchmark UMI extraction and grouping",
        stage_ids: &["fastq.extract_umis", "fastq.remove_duplicates"],
        scenarios: G110_SCENARIOS,
        checks: umi_checks,
    },
    GoalDefinition {
        goal_id: "G111",
        title: "benchmark FASTQ duplicate and complexity stages",
        stage_ids: &[
            "fastq.detect_duplicates_premerge",
            "fastq.estimate_library_complexity_prealign",
        ],
        scenarios: G111_SCENARIOS,
        checks: duplicate_complexity_checks,
    },
    GoalDefinition {
        goal_id: "G116",
        title: "benchmark eDNA/metabarcoding preprocessing",
        stage_ids: &[
            "fastq.normalize_primers",
            "fastq.remove_chimeras",
            "fastq.infer_asvs",
            "fastq.cluster_otus",
            "fastq.normalize_abundance",
        ],
        scenarios: G116_SCENARIOS,
        checks: edna_preprocessing_checks,
    },
];
