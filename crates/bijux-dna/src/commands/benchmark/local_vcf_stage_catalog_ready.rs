use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::local_corpus_fixture::vcf::{
    validate_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};
use super::local_vcf_no_empty_output::{
    validate_vcf_no_empty_output, DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH,
};
use super::local_vcf_reference_compatibility::{
    render_vcf_reference_compatibility, DEFAULT_VCF_REFERENCE_COMPATIBILITY_PATH,
};
use super::local_vcf_sample_compatibility::{
    render_vcf_sample_compatibility, DEFAULT_VCF_SAMPLE_COMPATIBILITY_PATH,
};
use super::local_vcf_smoke_root::{render_vcf_smoke_root, DEFAULT_VCF_SMOKE_ROOT_PATH};
use super::local_vcf_stage_catalog::{render_vcf_stage_catalog, DEFAULT_VCF_STAGE_CATALOG_PATH};
use super::local_vcf_stage_matrix::{
    render_vcf_stage_matrix, validate_vcf_stage_matrix, DEFAULT_VCF_STAGE_MATRIX_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;
use crate::commands::fixtures::build::vcf::{
    build_vcf_mini_fixture, DEFAULT_VCF_MINI_REGENERATION_ROOT,
};
use crate::commands::fixtures::expected::vcf::validate_vcf_expected_truth_manifest_path;
use crate::commands::fixtures::paths::{
    benchmark_corpus_manifest_path, benchmark_fixture_root_path,
};

pub(crate) const DEFAULT_VCF_STAGE_CATALOG_READY_PATH: &str =
    "target/local-ready/VCF_STAGE_CATALOG_READY.json";
const LOCAL_VCF_STAGE_CATALOG_READY_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_stage_catalog_ready.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfStageCatalogReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfStageCatalogReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<LocalVcfStageCatalogReadyGoalCheck>,
}

pub(crate) fn run_validate_vcf_stage_catalog_ready(
    args: &parse::BenchLocalValidateVcfStageCatalogReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_vcf_stage_catalog_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_STAGE_CATALOG_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_vcf_stage_catalog_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfStageCatalogReadyReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let manifest_path = repo_root.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
    let reference_output_path = repo_root.join(DEFAULT_VCF_REFERENCE_COMPATIBILITY_PATH);
    let sample_output_path = repo_root.join(DEFAULT_VCF_SAMPLE_COMPATIBILITY_PATH);
    let regeneration_root = repo_root.join(DEFAULT_VCF_MINI_REGENERATION_ROOT);

    let mut checks = Vec::new();

    record_goal_check(
        &mut checks,
        201,
        "vcf stage catalog",
        Some(DEFAULT_VCF_STAGE_CATALOG_PATH.to_string()),
        || {
            let report =
                render_vcf_stage_catalog(repo_root, PathBuf::from(DEFAULT_VCF_STAGE_CATALOG_PATH))?;
            Ok(format!(
                "wrote {} rows with {} supported and {} planned stages",
                report.stage_count, report.supported_stage_count, report.planned_stage_count
            ))
        },
    );

    record_goal_check(
        &mut checks,
        202,
        "vcf stage matrix",
        Some(DEFAULT_VCF_STAGE_MATRIX_PATH.to_string()),
        || {
            let matrix_report =
                render_vcf_stage_matrix(repo_root, PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH))?;
            let validation_report = validate_vcf_stage_matrix(
                repo_root,
                PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH),
                true,
            )?;
            Ok(format!(
                "validated {} rows across {} stages in strict mode using {} required tools",
                matrix_report.row_count,
                validation_report.stage_count,
                validation_report.required_tool_count
            ))
        },
    );

    record_goal_check(
        &mut checks,
        203,
        "vcf mini corpus fixture",
        Some(DEFAULT_VCF_MINI_MANIFEST_PATH.to_string()),
        || {
            let report = validate_vcf_corpus_fixture_manifest_path(repo_root, &manifest_path)?;
            Ok(format!(
                "validated corpus `{}` with {} samples, {} populations, and {} variant sets",
                report.corpus_id,
                report.sample_count,
                report.population_count,
                report.variant_sets.len()
            ))
        },
    );

    record_goal_check(
        &mut checks,
        204,
        "vcf expected truth",
        Some("benchmarks/tests/fixtures/corpora/vcf-mini/expected".to_string()),
        || {
            let report = validate_vcf_expected_truth_manifest_path(repo_root, &manifest_path)?;
            Ok(format!(
                "validated {} truth files for {} cohort samples and {} pairs",
                report.truth_file_count, report.cohort_sample_count, report.pair_count
            ))
        },
    );

    record_goal_check(
        &mut checks,
        205,
        "vcf reference compatibility",
        Some(DEFAULT_VCF_REFERENCE_COMPATIBILITY_PATH.to_string()),
        || {
            let report = render_vcf_reference_compatibility(
                repo_root,
                &manifest_path,
                &reference_output_path,
            )?;
            if report.status != "compatible" {
                bail!(
                    "reference compatibility reported `{}` with missing {:?} and extra {:?}",
                    report.status,
                    report.missing_contigs,
                    report.extra_contigs
                );
            }
            Ok(format!(
                "validated {} reference contigs across {} variant sets",
                report.contig_count,
                report.variant_sets.len()
            ))
        },
    );

    record_goal_check(
        &mut checks,
        206,
        "vcf sample compatibility",
        Some(DEFAULT_VCF_SAMPLE_COMPATIBILITY_PATH.to_string()),
        || {
            let report =
                render_vcf_sample_compatibility(repo_root, &manifest_path, &sample_output_path)?;
            if report.status != "compatible" {
                bail!(
                    "sample compatibility reported `{}` with missing metadata {:?}, population labels {:?}, and sex labels {:?}",
                    report.status,
                    report.missing_metadata,
                    report.missing_population_labels,
                    report.missing_sex_labels
                );
            }
            Ok(format!(
                "validated {} downstream samples across {} stage consumers",
                report.vcf_samples.len(),
                report.downstream_stage_ids.len()
            ))
        },
    );

    record_goal_check(
        &mut checks,
        207,
        "vcf mini regeneration",
        Some(format!("{DEFAULT_VCF_MINI_REGENERATION_ROOT}/manifest.json")),
        || {
            let source_manifest_path = benchmark_corpus_manifest_path(
                &benchmark_fixture_root_path(repo_root, None),
                "vcf-mini",
            );
            let report =
                build_vcf_mini_fixture(repo_root, &source_manifest_path, &regeneration_root)?;
            if !report.governed_counts_match {
                bail!("regenerated fixture counts drifted from the governed fixture contract");
            }
            Ok(format!(
                "regenerated {} files with {} truth files and matching governed counts",
                report.generated_fixture_file_count, report.generated_truth_counts.truth_file_count
            ))
        },
    );

    record_goal_check(
        &mut checks,
        208,
        "vcf smoke root",
        Some(DEFAULT_VCF_SMOKE_ROOT_PATH.to_string()),
        || {
            let first =
                render_vcf_smoke_root(repo_root, PathBuf::from(DEFAULT_VCF_SMOKE_ROOT_PATH))?;
            let second =
                render_vcf_smoke_root(repo_root, PathBuf::from(DEFAULT_VCF_SMOKE_ROOT_PATH))?;
            if first.run_id != second.run_id
                || first.root_path != second.root_path
                || first.rows != second.rows
            {
                bail!("repeated VCF smoke-root renders drifted in stage/tool identity or paths");
            }
            Ok(format!(
                "rendered deterministic smoke root `{}` with {} stages and {} tool pairs",
                first.run_id, first.stage_count, first.tool_pair_count
            ))
        },
    );

    record_goal_check(
        &mut checks,
        209,
        "vcf no-empty-output gate",
        Some(DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH.to_string()),
        || {
            let report = validate_vcf_no_empty_output(
                repo_root,
                PathBuf::from(DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH),
                true,
            )?;
            Ok(format!(
                "validated {} declared outputs with {} non-empty rows",
                report.checked_output_count, report.non_empty_output_count
            ))
        },
    );

    let report = build_vcf_stage_catalog_ready_report(repo_root, &absolute_output_path, checks);
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        bail!(
            "VCF stage catalog readiness gate failed for goals {}; inspect {}",
            report.failing_goal_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(", "),
            report.output_path
        );
    }
    Ok(report)
}

fn record_goal_check<F>(
    checks: &mut Vec<LocalVcfStageCatalogReadyGoalCheck>,
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    run: F,
) where
    F: FnOnce() -> Result<String>,
{
    match run() {
        Ok(detail) => checks.push(LocalVcfStageCatalogReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(LocalVcfStageCatalogReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: false,
            detail: format!("{error:#}"),
        }),
    }
}

fn build_vcf_stage_catalog_ready_report(
    repo_root: &Path,
    output_path: &Path,
    checks: Vec<LocalVcfStageCatalogReadyGoalCheck>,
) -> LocalVcfStageCatalogReadyReport {
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let checked_goal_count = checks.len();
    let passed_goal_count = checked_goal_count.saturating_sub(failed_goal_count);

    LocalVcfStageCatalogReadyReport {
        schema_version: LOCAL_VCF_STAGE_CATALOG_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        ok: failed_goal_count == 0,
        checks,
    }
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        build_vcf_stage_catalog_ready_report, LocalVcfStageCatalogReadyGoalCheck,
        DEFAULT_VCF_STAGE_CATALOG_READY_PATH, LOCAL_VCF_STAGE_CATALOG_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_stage_catalog_ready_report_marks_failed_goal_ids() {
        let repo_root = repo_root();
        let checks = vec![
            LocalVcfStageCatalogReadyGoalCheck {
                goal_id: 201,
                surface: "vcf stage catalog".to_string(),
                output_path: Some("benchmarks/configs/local/vcf-stage-catalog.toml".to_string()),
                ok: true,
                detail: "ok".to_string(),
            },
            LocalVcfStageCatalogReadyGoalCheck {
                goal_id: 209,
                surface: "vcf no-empty-output gate".to_string(),
                output_path: Some("target/local-ready/vcf/no-empty-output-check.json".to_string()),
                ok: false,
                detail: "zero-byte artifact detected".to_string(),
            },
        ];

        let report = build_vcf_stage_catalog_ready_report(
            &repo_root,
            &repo_root.join(DEFAULT_VCF_STAGE_CATALOG_READY_PATH),
            checks,
        );

        assert_eq!(report.schema_version, LOCAL_VCF_STAGE_CATALOG_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_STAGE_CATALOG_READY_PATH);
        assert_eq!(report.checked_goal_count, 2);
        assert_eq!(report.passed_goal_count, 1);
        assert_eq!(report.failed_goal_count, 1);
        assert_eq!(report.failing_goal_ids, vec![209]);
        assert!(!report.ok);
    }

    #[test]
    fn vcf_stage_catalog_ready_report_marks_clean_goal_slice() {
        let repo_root = repo_root();
        let checks = vec![
            LocalVcfStageCatalogReadyGoalCheck {
                goal_id: 201,
                surface: "vcf stage catalog".to_string(),
                output_path: Some("benchmarks/configs/local/vcf-stage-catalog.toml".to_string()),
                ok: true,
                detail: "ok".to_string(),
            },
            LocalVcfStageCatalogReadyGoalCheck {
                goal_id: 202,
                surface: "vcf stage matrix".to_string(),
                output_path: Some("benchmarks/configs/local/vcf-stage-matrix.toml".to_string()),
                ok: true,
                detail: "ok".to_string(),
            },
        ];

        let report = build_vcf_stage_catalog_ready_report(
            &repo_root,
            &repo_root.join(DEFAULT_VCF_STAGE_CATALOG_READY_PATH),
            checks,
        );

        assert_eq!(report.checked_goal_count, 2);
        assert_eq!(report.passed_goal_count, 2);
        assert_eq!(report.failed_goal_count, 0);
        assert!(report.failing_goal_ids.is_empty());
        assert!(report.ok);
    }
}
