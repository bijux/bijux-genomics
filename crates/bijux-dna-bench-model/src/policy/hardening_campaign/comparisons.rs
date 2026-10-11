//! Owner: hardening campaign comparisons declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use bijux_dna_core::id_catalog as stage;
use std::collections::BTreeSet;

const FULL_FASTQ_STAGE_TOOL_CAMPAIGN_SCENARIOS: &[ScenarioDefinition] = &[
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

const FULL_BAM_STAGE_TOOL_CAMPAIGN_SCENARIOS: &[ScenarioDefinition] = &[
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

const FULL_VCF_STAGE_TOOL_CAMPAIGN_SCENARIOS: &[ScenarioDefinition] = &[
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

const FULL_CROSS_DOMAIN_CAMPAIGN_SCENARIOS: &[ScenarioDefinition] = &[
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

const ALL_DOMAIN_STAGED_CAMPAIGN_SCENARIOS: &[ScenarioDefinition] = &[
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

const COMPARE_LOCAL_AND_SLURM_EXECUTION_SCENARIOS: &[ScenarioDefinition] = &[
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

const COMPARE_APPTAINER_IMAGE_VERSIONS_SCENARIOS: &[ScenarioDefinition] = &[
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

const COMPARE_DATABASE_VERSIONS_SCENARIOS: &[ScenarioDefinition] = &[
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

const COMPARE_CORPUS_SCALES_SCENARIOS: &[ScenarioDefinition] = &[
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

const COMPARE_COLD_AND_WARM_STORAGE_MODES_SCENARIOS: &[ScenarioDefinition] = &[
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

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G241",
        title: "run full FASTQ stage/tool campaign",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::FASTQ_TRIM, stage::FASTQ_STATS_NEUTRAL],
        checks: full_fastq_stage_tool_campaign,
        scenarios: FULL_FASTQ_STAGE_TOOL_CAMPAIGN_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G242",
        title: "run full BAM stage/tool campaign",
        stage_ids: &[stage::BAM_VALIDATE, stage::BAM_ALIGN, stage::BAM_MAPPING_SUMMARY],
        checks: full_bam_stage_tool_campaign,
        scenarios: FULL_BAM_STAGE_TOOL_CAMPAIGN_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G243",
        title: "run full VCF stage/tool campaign",
        stage_ids: &[stage::VCF_CALL, stage::VCF_FILTER, stage::VCF_STATS],
        checks: full_vcf_stage_tool_campaign,
        scenarios: FULL_VCF_STAGE_TOOL_CAMPAIGN_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G244",
        title: "run full cross-domain campaign",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_ALIGN, stage::VCF_CALL],
        checks: full_cross_domain_campaign,
        scenarios: FULL_CROSS_DOMAIN_CAMPAIGN_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G245",
        title: "run all-domain staged campaign",
        stage_ids: &[stage::FASTQ_VALIDATE_READS, stage::BAM_VALIDATE, stage::VCF_POSTPROCESS],
        checks: all_domain_staged_campaign,
        scenarios: ALL_DOMAIN_STAGED_CAMPAIGN_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G246",
        title: "compare local and Slurm execution",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_COVERAGE, stage::VCF_FILTER],
        checks: compare_local_and_slurm_execution,
        scenarios: COMPARE_LOCAL_AND_SLURM_EXECUTION_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G247",
        title: "compare Apptainer image versions",
        stage_ids: &[stage::FASTQ_TRIM, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: compare_apptainer_image_versions,
        scenarios: COMPARE_APPTAINER_IMAGE_VERSIONS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G248",
        title: "compare database versions",
        stage_ids: &[stage::BAM_ALIGN, stage::VCF_CALL, stage::VCF_IMPUTE],
        checks: compare_database_versions,
        scenarios: COMPARE_DATABASE_VERSIONS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G249",
        title: "compare corpus scales",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_COVERAGE, stage::VCF_FILTER],
        checks: compare_corpus_scales,
        scenarios: COMPARE_CORPUS_SCALES_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G250",
        title: "compare cold and warm storage modes",
        stage_ids: &[stage::FASTQ_STATS_NEUTRAL, stage::BAM_MAPPING_SUMMARY, stage::VCF_STATS],
        checks: compare_cold_and_warm_storage_modes,
        scenarios: COMPARE_COLD_AND_WARM_STORAGE_MODES_SCENARIOS,
    },
];

fn full_fastq_stage_tool_campaign(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("full_fastq_rows_present={}", !rows.is_empty()),
        format!(
            "full_fastq_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "full_fastq_validate_trim_profile_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
        ),
        format!(
            "full_fastq_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("full_fastq_findings_count={}", findings.len()),
    ]
}

fn full_bam_stage_tool_campaign(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("full_bam_rows_present={}", !rows.is_empty()),
        format!(
            "full_bam_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "full_bam_validate_align_summary_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
        ),
        format!(
            "full_bam_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("full_bam_findings_count={}", findings.len()),
    ]
}

fn full_vcf_stage_tool_campaign(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("full_vcf_rows_present={}", !rows.is_empty()),
        format!(
            "full_vcf_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "full_vcf_call_filter_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "full_vcf_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("full_vcf_findings_count={}", findings.len()),
    ]
}

fn full_cross_domain_campaign(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("full_cross_domain_rows_present={}", !rows.is_empty()),
        format!(
            "full_cross_domain_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "full_cross_domain_profile_align_call_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
        ),
        format!(
            "full_cross_domain_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("full_cross_domain_findings_count={}", findings.len()),
    ]
}

fn all_domain_staged_campaign(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("all_domain_series_rows_present={}", !rows.is_empty()),
        format!(
            "all_domain_series_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "all_domain_series_validate_validate_postprocess_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_VALIDATE_READS)
                && rows.iter().any(|row| row.stage_id == stage::BAM_VALIDATE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_POSTPROCESS)
        ),
        format!(
            "all_domain_series_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("all_domain_series_findings_count={}", findings.len()),
    ]
}

fn compare_local_and_slurm_execution(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("local_slurm_parity_rows_present={}", !rows.is_empty()),
        format!(
            "local_slurm_parity_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "local_slurm_parity_trim_coverage_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "local_slurm_parity_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!("local_slurm_parity_findings_count={}", findings.len()),
    ]
}

fn compare_apptainer_image_versions(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("image_version_rows_present={}", !rows.is_empty()),
        format!(
            "image_version_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "image_version_trim_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_TRIM)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "image_version_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("image_version_findings_count={}", findings.len()),
    ]
}

fn compare_database_versions(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("database_version_rows_present={}", !rows.is_empty()),
        format!(
            "database_version_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "database_version_align_call_impute_bound={}",
            rows.iter().any(|row| row.stage_id == stage::BAM_ALIGN)
                && rows.iter().any(|row| row.stage_id == stage::VCF_CALL)
                && rows.iter().any(|row| row.stage_id == stage::VCF_IMPUTE)
        ),
        format!(
            "database_version_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("database_version_findings_count={}", findings.len()),
    ]
}

fn compare_corpus_scales(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("corpus_scale_rows_present={}", !rows.is_empty()),
        format!(
            "corpus_scale_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "corpus_scale_profile_coverage_filter_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_COVERAGE)
                && rows.iter().any(|row| row.stage_id == stage::VCF_FILTER)
        ),
        format!(
            "corpus_scale_degraded_or_refuse_rows={}",
            rows.iter()
                .filter(|row| row.readiness_class == "degraded" || row.readiness_class == "refuse")
                .count()
        ),
        format!("corpus_scale_findings_count={}", findings.len()),
    ]
}

fn compare_cold_and_warm_storage_modes(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
        format!("storage_mode_rows_present={}", !rows.is_empty()),
        format!(
            "storage_mode_stage_count={}",
            rows.iter().map(|row| row.stage_id.clone()).collect::<BTreeSet<_>>().len()
        ),
        format!(
            "storage_mode_profile_summary_stats_bound={}",
            rows.iter().any(|row| row.stage_id == stage::FASTQ_STATS_NEUTRAL)
                && rows.iter().any(|row| row.stage_id == stage::BAM_MAPPING_SUMMARY)
                && rows.iter().any(|row| row.stage_id == stage::VCF_STATS)
        ),
        format!(
            "storage_mode_non_ready_rows={}",
            rows.iter().filter(|row| row.readiness_class != "ready").count()
        ),
        format!("storage_mode_findings_count={}", findings.len()),
    ]
}
