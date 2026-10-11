//! Owner: hardening campaign databases declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

const CORPUS_IMPROVEMENT_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "corpus-weakness-detection",
        focus: "detect weak corpus signals from appraisers",
    },
    ScenarioDefinition {
        scenario_id: "corpus-improvement-plan",
        focus: "generate corpus-improvement hardening plan",
    },
    ScenarioDefinition {
        scenario_id: "corpus-rerun-proof",
        focus: "capture rerun proof after corpus changes",
    },
];

const DATABASE_IMPROVEMENT_HARDENING_SCENARIOS: &[ScenarioDefinition] = &[
    ScenarioDefinition {
        scenario_id: "database-weakness-detection",
        focus: "detect DB incompatibility and weakness findings",
    },
    ScenarioDefinition {
        scenario_id: "database-remediation-plan",
        focus: "generate DB hardening remediation plan",
    },
    ScenarioDefinition {
        scenario_id: "database-rerun-proof",
        focus: "capture rerun proof after DB fixes",
    },
];

const KNOWN_SITES_DBSNP_LIKE_DATABASE_BUNDLES_SCENARIOS: &[ScenarioDefinition] = &[
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

const IMPUTATION_PANEL_BUNDLES_SCENARIOS: &[ScenarioDefinition] = &[
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

const TAXONOMY_DATABASE_VARIANTS_SCENARIOS: &[ScenarioDefinition] = &[
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

const CONTAMINANT_DATABASE_VARIANTS_SCENARIOS: &[ScenarioDefinition] = &[
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

const ADAPTER_AND_PRIMER_BANK_VARIANTS_SCENARIOS: &[ScenarioDefinition] = &[
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

const ANNOTATION_DATABASE_VARIANTS_SCENARIOS: &[ScenarioDefinition] = &[
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

const CORPUS_DATABASE_COMPATIBILITY_MATRIX_SCENARIOS: &[ScenarioDefinition] = &[
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

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G173",
        title: "add corpus-improvement hardening",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::FASTQ_STATS_NEUTRAL, stage::BAM_ALIGN],
        checks: corpus_improvement_hardening,
        scenarios: CORPUS_IMPROVEMENT_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G174",
        title: "add database-improvement hardening",
        stage_ids: &[stage::BAM_ALIGN, stage::VCF_CALL, stage::VCF_PREPARE_REFERENCE_PANEL],
        checks: database_improvement_hardening,
        scenarios: DATABASE_IMPROVEMENT_HARDENING_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G221",
        title: "add known-sites/dbSNP-like database bundles",
        stage_ids: &[stage::BAM_VALIDATE, stage::VCF_FILTER, stage::VCF_POSTPROCESS],
        checks: known_sites_dbsnp_like_database_bundles,
        scenarios: KNOWN_SITES_DBSNP_LIKE_DATABASE_BUNDLES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G222",
        title: "add imputation panel bundles",
        stage_ids: &[stage::VCF_CALL, stage::VCF_IMPUTE, stage::VCF_STATS],
        checks: imputation_panel_bundles,
        scenarios: IMPUTATION_PANEL_BUNDLES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G223",
        title: "add taxonomy database variants",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_CONTAMINATION, stage::VCF_STATS],
        checks: taxonomy_database_variants,
        scenarios: TAXONOMY_DATABASE_VARIANTS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G224",
        title: "add contaminant database variants",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_CONTAMINATION, stage::VCF_FILTER],
        checks: contaminant_database_variants,
        scenarios: CONTAMINANT_DATABASE_VARIANTS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G225",
        title: "add adapter and primer bank variants",
        stage_ids: &[stage::FASTQ_TRIM, stage::FASTQ_STATS_NEUTRAL, stage::BAM_CONTAMINATION],
        checks: adapter_and_primer_bank_variants,
        scenarios: ADAPTER_AND_PRIMER_BANK_VARIANTS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G226",
        title: "add annotation database variants",
        stage_ids: &[stage::VCF_CALL, stage::VCF_FILTER, stage::VCF_POSTPROCESS],
        checks: annotation_database_variants,
        scenarios: ANNOTATION_DATABASE_VARIANTS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G227",
        title: "add corpus/database compatibility matrix",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_ALIGN, stage::VCF_CALL],
        checks: corpus_database_compatibility_matrix,
        scenarios: CORPUS_DATABASE_COMPATIBILITY_MATRIX_SCENARIOS,
    },
];

fn corpus_improvement_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("corpus_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "corpus_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "corpus_hardening_profile_stage_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
        ),
        format!(
            "corpus_hardening_corpus_mismatch_findings={}",
            findings.iter().filter(|finding| finding.failure_class == "corpus-mismatch").count()
        ),
        format!(
            "corpus_hardening_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn database_improvement_hardening(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    vec![
        format!("database_hardening_rows_present={}", !rows.is_empty()),
        format!(
            "database_hardening_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "database_hardening_reference_panel_bound={}",
            rows.iter().any(|row| row.stage_id == stage::VCF_PREPARE_REFERENCE_PANEL)
        ),
        format!(
            "database_hardening_alignment_and_call_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
        ),
        format!(
            "database_hardening_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
    ]
}

fn known_sites_dbsnp_like_database_bundles(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("known_sites_rows_present={}", !rows.is_empty()),
        format!(
            "known_sites_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "known_sites_validate_filter_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "known_sites_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!("known_sites_findings_count={}", findings.len()),
    ]
}

fn imputation_panel_bundles(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("imputation_panel_rows_present={}", !rows.is_empty()),
        format!(
            "imputation_panel_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "imputation_panel_call_impute_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
                && rows.iter().any(|row| row.stage_id == stage::VCF_IMPUTE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "imputation_panel_degraded_rows={}",
            rows.iter().filter(|row| row.readiness_class == "degraded").count()
        ),
        format!("imputation_panel_findings_count={}", findings.len()),
    ]
}

fn taxonomy_database_variants(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("taxonomy_variant_rows_present={}", !rows.is_empty()),
        format!(
            "taxonomy_variant_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "taxonomy_variant_profile_contam_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "taxonomy_variant_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("taxonomy_variant_findings_count={}", findings.len()),
    ]
}

fn contaminant_database_variants(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
        format!("contaminant_variant_rows_present={}", !rows.is_empty()),
        format!(
            "contaminant_variant_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "contaminant_variant_validate_contam_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "contaminant_variant_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!("contaminant_variant_queue_entries={}", queue_entries.len()),
    ]
}

fn adapter_and_primer_bank_variants(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("adapter_primer_rows_present={}", !rows.is_empty()),
        format!(
            "adapter_primer_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "adapter_primer_trim_profile_contam_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_CONTAMINATION)
        ),
        format!(
            "adapter_primer_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("adapter_primer_findings_count={}", findings.len()),
    ]
}

fn annotation_database_variants(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("annotation_db_rows_present={}", !rows.is_empty()),
        format!(
            "annotation_db_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "annotation_db_call_filter_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "annotation_db_refuse_rows={}",
            rows.iter().filter(|row| row.readiness_class == "refuse").count()
        ),
        format!("annotation_db_findings_count={}", findings.len()),
    ]
}

fn corpus_database_compatibility_matrix(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("corpus_db_matrix_rows_present={}", !rows.is_empty()),
        format!(
            "corpus_db_matrix_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "corpus_db_matrix_validate_align_call_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
        ),
        format!(
            "corpus_db_matrix_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("corpus_db_matrix_findings_count={}", findings.len()),
    ]
}
