use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::scientific_acceptance_thresholds::{
    render_scientific_acceptance_thresholds, ScientificAcceptanceThresholdsReport,
    DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH,
};
use crate::commands::benchmark::local_corpus_fixture::bam::DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH;
use crate::commands::benchmark::local_corpus_fixture::fastq::DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH;
use crate::commands::benchmark::local_corpus_fixture::vcf::DEFAULT_VCF_MINI_MANIFEST_PATH;
use crate::commands::benchmark::local_cross_domain_sample_consistency::{
    render_cross_domain_sample_consistency, CrossDomainSampleConsistencyReport,
    DEFAULT_CROSS_DOMAIN_SAMPLE_CONSISTENCY_PATH,
};
use crate::commands::fixtures::paths::DEFAULT_BENCHMARK_FIXTURE_ROOT;
use crate::commands::fixtures::root_validation::{
    validate_benchmark_fixture_root, BenchmarkFixtureRootValidationReport,
    BenchmarkFixtureRootValidationRow, DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH,
};

pub(crate) const DEFAULT_SCIENCE_TRUTH_SETS_COMPLETE_PATH: &str =
    "benchmarks/readiness/science/SCIENCE_TRUTH_SETS_COMPLETE.json";
const SCIENCE_TRUTH_SETS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.science_truth_sets_complete.v1";

struct ScienceTruthRequirement {
    surface_id: &'static str,
    domain: &'static str,
    truth_fixture_id: &'static str,
    stage_ids: &'static [&'static str],
    acceptance_stage_ids: &'static [&'static str],
}

const SCIENCE_TRUTH_REQUIREMENTS: &[ScienceTruthRequirement] = &[
    ScienceTruthRequirement {
        surface_id: "fastq_trimming",
        domain: "fastq",
        truth_fixture_id: "fastq-trimming-truth",
        stage_ids: &["fastq.trim_reads", "fastq.trim_polyg_tails"],
        acceptance_stage_ids: &[],
    },
    ScienceTruthRequirement {
        surface_id: "fastq_duplicates",
        domain: "fastq",
        truth_fixture_id: "fastq-duplicates-truth",
        stage_ids: &["fastq.detect_duplicates_premerge", "fastq.remove_duplicates"],
        acceptance_stage_ids: &[],
    },
    ScienceTruthRequirement {
        surface_id: "fastq_taxonomy",
        domain: "fastq",
        truth_fixture_id: "fastq-taxonomy-truth",
        stage_ids: &["fastq.screen_taxonomy"],
        acceptance_stage_ids: &[],
    },
    ScienceTruthRequirement {
        surface_id: "amplicon_feature_inference",
        domain: "fastq",
        truth_fixture_id: "amplicon-truth",
        stage_ids: &[
            "fastq.remove_chimeras",
            "fastq.infer_asvs",
            "fastq.cluster_otus",
            "fastq.normalize_abundance",
        ],
        acceptance_stage_ids: &[],
    },
    ScienceTruthRequirement {
        surface_id: "bam_alignment",
        domain: "bam",
        truth_fixture_id: "bam-alignment-truth",
        stage_ids: &["bam.align"],
        acceptance_stage_ids: &["bam.align"],
    },
    ScienceTruthRequirement {
        surface_id: "bam_duplicate_insert",
        domain: "bam",
        truth_fixture_id: "bam-duplicate-insert-truth",
        stage_ids: &["bam.markdup", "bam.duplication_metrics", "bam.insert_size"],
        acceptance_stage_ids: &["bam.markdup", "bam.duplication_metrics"],
    },
    ScienceTruthRequirement {
        surface_id: "bam_gc_coverage",
        domain: "bam",
        truth_fixture_id: "bam-gc-coverage-truth",
        stage_ids: &["bam.coverage", "bam.gc_bias"],
        acceptance_stage_ids: &["bam.coverage"],
    },
    ScienceTruthRequirement {
        surface_id: "bam_damage_authenticity",
        domain: "bam",
        truth_fixture_id: "adna-damage-truth",
        stage_ids: &["bam.damage", "bam.authenticity"],
        acceptance_stage_ids: &["bam.damage", "bam.authenticity"],
    },
    ScienceTruthRequirement {
        surface_id: "bam_contamination",
        domain: "bam",
        truth_fixture_id: "adna-contamination-truth",
        stage_ids: &["bam.contamination"],
        acceptance_stage_ids: &["bam.contamination"],
    },
    ScienceTruthRequirement {
        surface_id: "bam_endogenous_content",
        domain: "bam",
        truth_fixture_id: "endogenous-truth",
        stage_ids: &["bam.endogenous_content"],
        acceptance_stage_ids: &[],
    },
    ScienceTruthRequirement {
        surface_id: "bam_sex_inference",
        domain: "bam",
        truth_fixture_id: "sex-inference-truth",
        stage_ids: &["bam.sex"],
        acceptance_stage_ids: &["bam.sex"],
    },
    ScienceTruthRequirement {
        surface_id: "bam_haplogroups",
        domain: "bam",
        truth_fixture_id: "haplogroup-truth",
        stage_ids: &["bam.haplogroups"],
        acceptance_stage_ids: &[],
    },
    ScienceTruthRequirement {
        surface_id: "vcf_genotyping",
        domain: "vcf",
        truth_fixture_id: "vcf-genotype-truth",
        stage_ids: &[
            "vcf.call_diploid",
            "vcf.call_pseudohaploid",
            "vcf.call_gl",
            "vcf.gl_propagation",
        ],
        acceptance_stage_ids: &["vcf.call_pseudohaploid", "vcf.call_gl", "vcf.gl_propagation"],
    },
    ScienceTruthRequirement {
        surface_id: "vcf_filtering",
        domain: "vcf",
        truth_fixture_id: "vcf-filter-truth",
        stage_ids: &["vcf.filter", "vcf.damage_filter"],
        acceptance_stage_ids: &["vcf.filter", "vcf.damage_filter"],
    },
    ScienceTruthRequirement {
        surface_id: "vcf_phasing_imputation",
        domain: "vcf",
        truth_fixture_id: "phasing-imputation-truth",
        stage_ids: &["vcf.phasing", "vcf.impute", "vcf.imputation_metrics"],
        acceptance_stage_ids: &["vcf.phasing", "vcf.impute", "vcf.imputation_metrics"],
    },
    ScienceTruthRequirement {
        surface_id: "vcf_population_structure",
        domain: "vcf",
        truth_fixture_id: "population-structure-truth",
        stage_ids: &["vcf.pca", "vcf.admixture", "vcf.population_structure"],
        acceptance_stage_ids: &["vcf.pca", "vcf.admixture", "vcf.population_structure"],
    },
    ScienceTruthRequirement {
        surface_id: "vcf_segments_demography",
        domain: "vcf",
        truth_fixture_id: "segments-demography-truth",
        stage_ids: &["vcf.roh", "vcf.ibd", "vcf.demography"],
        acceptance_stage_ids: &["vcf.ibd"],
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScienceTruthSurfaceStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ScienceTruthSurfaceRow {
    pub(crate) surface_id: String,
    pub(crate) domain: String,
    pub(crate) truth_fixture_id: String,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) acceptance_stage_ids: Vec<String>,
    pub(crate) truth_fixture_present: bool,
    pub(crate) truth_fixture_valid: bool,
    pub(crate) truth_manifest_path: Option<String>,
    pub(crate) truth_detail_path: Option<String>,
    pub(crate) accepted_stage_count: usize,
    pub(crate) missing_acceptance_stage_ids: Vec<String>,
    pub(crate) status: ScienceTruthSurfaceStatus,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ScienceTruthSetsCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) fixture_validation_path: String,
    pub(crate) fixture_validation_ok: bool,
    pub(crate) required_truth_fixture_count: usize,
    pub(crate) validated_required_truth_fixture_count: usize,
    pub(crate) missing_truth_fixture_ids: Vec<String>,
    pub(crate) invalid_truth_fixture_ids: Vec<String>,
    pub(crate) extra_validated_truth_fixture_ids: Vec<String>,
    pub(crate) cross_domain_consistency_path: String,
    pub(crate) cross_domain_consistency_status: String,
    pub(crate) cross_domain_source_link_failure_count: usize,
    pub(crate) scientific_acceptance_config_path: String,
    pub(crate) scientific_acceptance_row_count: usize,
    pub(crate) scientific_acceptance_stage_count: usize,
    pub(crate) science_surface_count: usize,
    pub(crate) science_stage_count: usize,
    pub(crate) acceptance_governed_stage_count: usize,
    pub(crate) accepted_stage_count: usize,
    pub(crate) missing_acceptance_stage_ids: Vec<String>,
    pub(crate) passes_gate: bool,
    pub(crate) rows: Vec<ScienceTruthSurfaceRow>,
}

pub(crate) fn render_science_truth_sets_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<ScienceTruthSetsCompleteReport> {
    let fixture_validation_report = validate_benchmark_fixture_root(
        repo_root,
        &repo_root.join(DEFAULT_BENCHMARK_FIXTURE_ROOT),
        &repo_root.join(DEFAULT_BENCHMARK_FIXTURE_ROOT_VALIDATION_REPORT_PATH),
    )?;
    let cross_domain_report = render_cross_domain_sample_consistency(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH),
        &repo_root.join(DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH),
        &repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH),
        &repo_root.join(DEFAULT_CROSS_DOMAIN_SAMPLE_CONSISTENCY_PATH),
    )?;
    let scientific_acceptance_report = render_scientific_acceptance_thresholds(
        repo_root,
        PathBuf::from(DEFAULT_SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
    )?;
    render_science_truth_sets_complete_from_prerequisites(
        repo_root,
        output_path,
        &fixture_validation_report,
        &cross_domain_report,
        &scientific_acceptance_report,
    )
}

pub(crate) fn render_science_truth_sets_complete_from_prerequisites(
    repo_root: &Path,
    output_path: PathBuf,
    fixture_validation_report: &BenchmarkFixtureRootValidationReport,
    cross_domain_report: &CrossDomainSampleConsistencyReport,
    scientific_acceptance_report: &ScientificAcceptanceThresholdsReport,
) -> Result<ScienceTruthSetsCompleteReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let expected_truth_rows_by_fixture =
        collect_expected_truth_rows_by_fixture(&fixture_validation_report.rows);
    let threshold_stage_ids = scientific_acceptance_report
        .rows
        .iter()
        .map(|row| row.stage_id.clone())
        .collect::<BTreeSet<_>>();
    let required_truth_fixture_ids = SCIENCE_TRUTH_REQUIREMENTS
        .iter()
        .map(|requirement| requirement.truth_fixture_id.to_string())
        .collect::<BTreeSet<_>>();

    let mut rows = Vec::with_capacity(SCIENCE_TRUTH_REQUIREMENTS.len());
    let mut missing_truth_fixture_ids = Vec::new();
    let mut invalid_truth_fixture_ids = Vec::new();
    let mut missing_acceptance_stage_ids = BTreeSet::<String>::new();
    let mut science_stage_ids = BTreeSet::<String>::new();
    let mut acceptance_stage_ids = BTreeSet::<String>::new();
    let mut accepted_stage_ids = BTreeSet::<String>::new();

    for requirement in SCIENCE_TRUTH_REQUIREMENTS {
        let truth_row = expected_truth_rows_by_fixture.get(requirement.truth_fixture_id).copied();
        let truth_fixture_present = truth_row.is_some();
        let truth_fixture_valid = truth_row.is_some_and(|row| row.valid);
        if !truth_fixture_present {
            missing_truth_fixture_ids.push(requirement.truth_fixture_id.to_string());
        } else if !truth_fixture_valid {
            invalid_truth_fixture_ids.push(requirement.truth_fixture_id.to_string());
        }

        let stage_ids = requirement
            .stage_ids
            .iter()
            .map(|stage_id| (*stage_id).to_string())
            .collect::<Vec<_>>();
        let acceptance_stage_ids_for_surface = requirement
            .acceptance_stage_ids
            .iter()
            .map(|stage_id| (*stage_id).to_string())
            .collect::<Vec<_>>();
        let accepted_stage_ids_for_surface = requirement
            .acceptance_stage_ids
            .iter()
            .filter(|stage_id| threshold_stage_ids.contains(**stage_id))
            .map(|stage_id| (*stage_id).to_string())
            .collect::<Vec<_>>();
        let missing_acceptance_stage_ids_for_surface = requirement
            .acceptance_stage_ids
            .iter()
            .filter(|stage_id| !threshold_stage_ids.contains(**stage_id))
            .map(|stage_id| (*stage_id).to_string())
            .collect::<Vec<_>>();

        science_stage_ids.extend(stage_ids.iter().cloned());
        acceptance_stage_ids.extend(acceptance_stage_ids_for_surface.iter().cloned());
        accepted_stage_ids.extend(accepted_stage_ids_for_surface.iter().cloned());
        missing_acceptance_stage_ids.extend(missing_acceptance_stage_ids_for_surface.iter().cloned());

        let status = if truth_fixture_valid && missing_acceptance_stage_ids_for_surface.is_empty() {
            ScienceTruthSurfaceStatus::Pass
        } else {
            ScienceTruthSurfaceStatus::Fail
        };
        rows.push(ScienceTruthSurfaceRow {
            surface_id: requirement.surface_id.to_string(),
            domain: requirement.domain.to_string(),
            truth_fixture_id: requirement.truth_fixture_id.to_string(),
            stage_ids,
            acceptance_stage_ids: acceptance_stage_ids_for_surface.clone(),
            truth_fixture_present,
            truth_fixture_valid,
            truth_manifest_path: truth_row.and_then(|row| row.manifest_path.clone()),
            truth_detail_path: truth_row.and_then(|row| row.detail_path.clone()),
            accepted_stage_count: accepted_stage_ids_for_surface.len(),
            missing_acceptance_stage_ids: missing_acceptance_stage_ids_for_surface.clone(),
            status,
            reason: science_truth_surface_reason(
                requirement,
                truth_fixture_present,
                truth_fixture_valid,
                &accepted_stage_ids_for_surface,
                &missing_acceptance_stage_ids_for_surface,
            ),
        });
    }

    rows.sort_by(|left, right| left.surface_id.cmp(&right.surface_id));

    let extra_validated_truth_fixture_ids = expected_truth_rows_by_fixture
        .iter()
        .filter(|(fixture_id, row)| row.valid && !required_truth_fixture_ids.contains(fixture_id.as_str()))
        .map(|(fixture_id, _)| fixture_id.clone())
        .collect::<Vec<_>>();
    let cross_domain_consistency_status = cross_domain_report.status.clone();
    let missing_acceptance_stage_ids = missing_acceptance_stage_ids.into_iter().collect::<Vec<_>>();
    let validated_required_truth_fixture_count =
        rows.iter().filter(|row| row.truth_fixture_valid).count();
    let passes_gate = fixture_validation_report.ok
        && cross_domain_consistency_status == "compatible"
        && missing_truth_fixture_ids.is_empty()
        && invalid_truth_fixture_ids.is_empty()
        && missing_acceptance_stage_ids.is_empty();

    let report = ScienceTruthSetsCompleteReport {
        schema_version: SCIENCE_TRUTH_SETS_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        fixture_validation_path: fixture_validation_report.output_path.clone(),
        fixture_validation_ok: fixture_validation_report.ok,
        required_truth_fixture_count: SCIENCE_TRUTH_REQUIREMENTS.len(),
        validated_required_truth_fixture_count,
        missing_truth_fixture_ids,
        invalid_truth_fixture_ids,
        extra_validated_truth_fixture_ids,
        cross_domain_consistency_path: cross_domain_report.output_path.clone(),
        cross_domain_consistency_status,
        cross_domain_source_link_failure_count: cross_domain_report.source_link_failures.len(),
        scientific_acceptance_config_path: scientific_acceptance_report.config_path.clone(),
        scientific_acceptance_row_count: scientific_acceptance_report.row_count,
        scientific_acceptance_stage_count: threshold_stage_ids.len(),
        science_surface_count: rows.len(),
        science_stage_count: science_stage_ids.len(),
        acceptance_governed_stage_count: acceptance_stage_ids.len(),
        accepted_stage_count: accepted_stage_ids.len(),
        missing_acceptance_stage_ids,
        passes_gate,
        rows,
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_json(&output_path, &report)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(report)
}

fn collect_expected_truth_rows_by_fixture<'a>(
    rows: &'a [BenchmarkFixtureRootValidationRow],
) -> BTreeMap<String, &'a BenchmarkFixtureRootValidationRow> {
    rows.iter()
        .filter(|row| row.fixture_kind == "expected_truth")
        .map(|row| (row.fixture_id.clone(), row))
        .collect()
}

fn science_truth_surface_reason(
    requirement: &ScienceTruthRequirement,
    truth_fixture_present: bool,
    truth_fixture_valid: bool,
    accepted_stage_ids: &[String],
    missing_acceptance_stage_ids: &[String],
) -> String {
    if !truth_fixture_present {
        return format!(
            "required science truth fixture `{}` is missing from the benchmark fixture validation report",
            requirement.truth_fixture_id
        );
    }
    if !truth_fixture_valid {
        return format!(
            "required science truth fixture `{}` failed benchmark fixture validation",
            requirement.truth_fixture_id
        );
    }
    if !missing_acceptance_stage_ids.is_empty() {
        return format!(
            "truth fixture `{}` covers science stages `{}` but scientific acceptance is still missing for `{}`",
            requirement.truth_fixture_id,
            requirement.stage_ids.join(", "),
            missing_acceptance_stage_ids.join(", ")
        );
    }
    if accepted_stage_ids.is_empty() {
        return format!(
            "truth fixture `{}` validates non-comparable science stages `{}`",
            requirement.truth_fixture_id,
            requirement.stage_ids.join(", ")
        );
    }
    format!(
        "truth fixture `{}` validates science stages `{}` and scientific acceptance governs `{}`",
        requirement.truth_fixture_id,
        requirement.stage_ids.join(", "),
        accepted_stage_ids.join(", ")
    )
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
