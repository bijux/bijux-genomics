//! Owner: hardening campaign corpora declarations and pure evidence projections.
use super::{CheckInputs, GoalDefinition, ScenarioDefinition};
use std::collections::BTreeSet;

const MODERN_WGS_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const ANCIENT_DNA_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const EDNA_METABARCODING_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const LOW_PASS_POPULATION_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const CONTAMINATION_HEAVY_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const NON_HUMAN_REFERENCE_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const MICROBIAL_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const STRUCTURAL_VARIANT_BOUNDARY_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const CNV_BOUNDARY_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const CRAM_BOUNDARY_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const LONG_READ_BOUNDARY_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const SAMPLE_SWAP_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const DUPLICATE_COHORT_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const PANEL_CHOICE_SENSITIVITY_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const ANNOTATION_DRIFT_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const QC_THRESHOLD_SENSITIVITY_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const ARTIFACT_MINIMIZATION_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const SHARED_STORAGE_IO_STRESS_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const DATABASE_BUILD_BENCHMARK_CORPUS_SCENARIOS: &[ScenarioDefinition] = &[
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

const TRUTH_SET_ANCHORED_CORPUS_MODE_SCENARIOS: &[ScenarioDefinition] = &[
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

pub(super) const GOALS: &[GoalDefinition] = &[
    GoalDefinition {
        goal_id: "G201",
        title: "build modern WGS benchmark corpus",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.call"],
        checks: modern_wgs_benchmark_corpus,
        scenarios: MODERN_WGS_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G202",
        title: "build ancient-DNA benchmark corpus",
        stage_ids: &["fastq.trim_reads", "bam.contamination", "vcf.filter"],
        checks: ancient_dna_benchmark_corpus,
        scenarios: ANCIENT_DNA_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G203",
        title: "build eDNA/metabarcoding benchmark corpus",
        stage_ids: &["fastq.profile_reads", "bam.contamination", "vcf.stats"],
        checks: edna_metabarcoding_benchmark_corpus,
        scenarios: EDNA_METABARCODING_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G204",
        title: "build low-pass population benchmark corpus",
        stage_ids: &["fastq.profile_reads", "vcf.impute", "vcf.stats"],
        checks: low_pass_population_benchmark_corpus,
        scenarios: LOW_PASS_POPULATION_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G205",
        title: "build contamination-heavy benchmark corpus",
        stage_ids: &["fastq.validate_reads", "bam.contamination", "vcf.filter"],
        checks: contamination_heavy_benchmark_corpus,
        scenarios: CONTAMINATION_HEAVY_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G206",
        title: "build non-human reference benchmark corpus",
        stage_ids: &["fastq.validate_reads", "bam.align", "vcf.postprocess"],
        checks: non_human_reference_benchmark_corpus,
        scenarios: NON_HUMAN_REFERENCE_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G207",
        title: "build microbial benchmark corpus",
        stage_ids: &["fastq.trim_reads", "bam.align", "vcf.call"],
        checks: microbial_benchmark_corpus,
        scenarios: MICROBIAL_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G208",
        title: "build structural-variant boundary corpus",
        stage_ids: &["bam.coverage", "vcf.call", "vcf.filter"],
        checks: structural_variant_boundary_corpus,
        scenarios: STRUCTURAL_VARIANT_BOUNDARY_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G209",
        title: "build CNV boundary corpus",
        stage_ids: &["bam.coverage", "vcf.filter", "vcf.stats"],
        checks: cnv_boundary_corpus,
        scenarios: CNV_BOUNDARY_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G210",
        title: "build CRAM boundary corpus",
        stage_ids: &["bam.validate", "bam.mapping_summary", "vcf.filter"],
        checks: cram_boundary_corpus,
        scenarios: CRAM_BOUNDARY_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G211",
        title: "build long-read boundary corpus",
        stage_ids: &["fastq.validate_reads", "bam.align", "bam.mapping_summary"],
        checks: long_read_boundary_corpus,
        scenarios: LONG_READ_BOUNDARY_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G212",
        title: "build sample-swap corpus",
        stage_ids: &["fastq.profile_reads", "bam.validate", "vcf.stats"],
        checks: sample_swap_corpus,
        scenarios: SAMPLE_SWAP_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G213",
        title: "build duplicate-cohort corpus",
        stage_ids: &["fastq.profile_reads", "bam.coverage", "vcf.stats"],
        checks: duplicate_cohort_corpus,
        scenarios: DUPLICATE_COHORT_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G214",
        title: "build panel-choice sensitivity corpus",
        stage_ids: &["vcf.impute", "vcf.call", "vcf.stats"],
        checks: panel_choice_sensitivity_corpus,
        scenarios: PANEL_CHOICE_SENSITIVITY_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G215",
        title: "build annotation-drift corpus",
        stage_ids: &["vcf.call", "vcf.filter", "vcf.postprocess"],
        checks: annotation_drift_corpus,
        scenarios: ANNOTATION_DRIFT_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G216",
        title: "build QC-threshold sensitivity corpus",
        stage_ids: &["fastq.trim_reads", "bam.coverage", "vcf.filter"],
        checks: qc_threshold_sensitivity_corpus,
        scenarios: QC_THRESHOLD_SENSITIVITY_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G217",
        title: "build artifact-minimization corpus",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.postprocess"],
        checks: artifact_minimization_corpus,
        scenarios: ARTIFACT_MINIMIZATION_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G218",
        title: "build shared-storage IO stress corpus",
        stage_ids: &["fastq.profile_reads", "bam.mapping_summary", "vcf.stats"],
        checks: shared_storage_io_stress_corpus,
        scenarios: SHARED_STORAGE_IO_STRESS_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G219",
        title: "build database-build benchmark corpus",
        stage_ids: &["bam.align", "vcf.call", "vcf.impute"],
        checks: database_build_benchmark_corpus,
        scenarios: DATABASE_BUILD_BENCHMARK_CORPUS_SCENARIOS,
    },
    GoalDefinition {
        goal_id: "G220",
        title: "add truth-set anchored corpus mode",
        stage_ids: &["fastq.validate_reads", "vcf.call", "vcf.filter"],
        checks: truth_set_anchored_corpus_mode,
        scenarios: TRUTH_SET_ANCHORED_CORPUS_MODE_SCENARIOS,
    },
];

fn modern_wgs_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
        format!("modern_wgs_findings_count={}", findings.len()),
    ]
}

fn ancient_dna_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    let queue_entries = input.queue_entries;
    vec![
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
                .filter(|finding| finding.failure_class.contains("contamination")
                    || finding.failure_class == "readiness-refuse")
                .count()
        ),
        format!("adna_corpus_queue_entries={}", queue_entries.len()),
    ]
}

fn edna_metabarcoding_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn low_pass_population_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
        format!("lowpass_corpus_findings_count={}", findings.len()),
    ]
}

fn contamination_heavy_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
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
        format!("contamination_heavy_queue_entries={}", queue_entries.len()),
    ]
}

fn non_human_reference_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
            findings.iter().filter(|finding| finding.result_scope == "encrypted-code").count()
        ),
        format!("nonhuman_corpus_findings_count={}", findings.len()),
    ]
}

fn microbial_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
        format!("microbial_corpus_findings_count={}", findings.len()),
    ]
}

fn structural_variant_boundary_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
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
            queue_entries.iter().filter(|entry| entry.severity != "info").count()
        ),
    ]
}

fn cnv_boundary_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn cram_boundary_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
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
        format!("cram_boundary_queue_entries={}", queue_entries.len()),
    ]
}

fn long_read_boundary_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn sample_swap_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn duplicate_cohort_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn panel_choice_sensitivity_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn annotation_drift_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn qc_threshold_sensitivity_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn artifact_minimization_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
            findings.iter().filter(|finding| finding.result_scope == "encrypted-code").count()
        ),
    ]
}

fn shared_storage_io_stress_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let queue_entries = input.queue_entries;
    vec![
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
    ]
}

fn database_build_benchmark_corpus(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}

fn truth_set_anchored_corpus_mode(input: &CheckInputs<'_>) -> Vec<String> {
    let rows = input.rows;
    let findings = input.findings;
    vec![
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
    ]
}
