use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::local_vcf_admixture_smoke::run_local_vcf_admixture_smoke;
use super::local_vcf_call_diploid_smoke::run_local_vcf_call_diploid_smoke;
use super::local_vcf_call_gl_smoke::run_local_vcf_call_gl_smoke;
use super::local_vcf_call_pseudohaploid_smoke::run_local_vcf_call_pseudohaploid_smoke;
use super::local_vcf_call_smoke::run_local_vcf_call_smoke;
use super::local_vcf_damage_filter_smoke::run_local_vcf_damage_filter_smoke;
use super::local_vcf_demography_smoke::run_local_vcf_demography_smoke;
use super::local_vcf_filter_smoke::run_local_vcf_filter_smoke;
use super::local_vcf_gl_propagation_smoke::run_local_vcf_gl_propagation_smoke;
use super::local_vcf_ibd_smoke::run_local_vcf_ibd_smoke;
use super::local_vcf_imputation_metrics_smoke::run_local_vcf_imputation_metrics_smoke;
use super::local_vcf_impute_smoke::run_local_vcf_impute_smoke;
use super::local_vcf_pca_smoke::run_local_vcf_pca_smoke;
use super::local_vcf_phasing_smoke::run_local_vcf_phasing_smoke;
use super::local_vcf_population_structure_smoke::run_local_vcf_population_structure_smoke;
use super::local_vcf_prepare_reference_panel_smoke::run_local_vcf_prepare_reference_panel_smoke;
use super::local_vcf_qc_smoke::run_local_vcf_qc_smoke;
use super::local_vcf_roh_smoke::run_local_vcf_roh_smoke;
use super::local_vcf_stats_smoke::run_local_vcf_stats_smoke;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_SMOKE_SUITE_READY_PATH: &str =
    "target/local-smoke/VCF_SMOKE_SUITE_READY.json";
const LOCAL_VCF_SMOKE_SUITE_READY_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_smoke_suite_ready.v1";

const GOVERNED_VCF_CALL_TOOL_ID: &str = "bcftools";
const GOVERNED_VCF_QC_TOOL_ID: &str = "plink2";
const GOVERNED_VCF_PANEL_WORKFLOW_TOOL_ID: &str = "shapeit5";
const GOVERNED_VCF_IMPUTE_TOOL_ID: &str = "beagle";
const GOVERNED_VCF_COHORT_TOOL_ID: &str = "plink2";
const GOVERNED_VCF_IBD_TOOL_ID: &str = "germline";
const GOVERNED_VCF_DEMOGRAPHY_TOOL_ID: &str = "ibdne";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfSmokeSuiteGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfSmokeSuiteReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<LocalVcfSmokeSuiteGoalCheck>,
}

pub(crate) fn run_validate_vcf_smoke_suite_ready(
    args: &parse::BenchLocalValidateVcfSmokeSuiteReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_vcf_smoke_suite_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_SMOKE_SUITE_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn validate_vcf_smoke_suite_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfSmokeSuiteReadyReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();

    record_goal_check(
        &mut checks,
        211,
        "vcf.call smoke",
        Some("target/local-smoke/vcf.call/bcftools/calls.vcf.gz".to_string()),
        || {
            let _report = run_local_vcf_call_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed single-sample call smoke output".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        212,
        "vcf.call_diploid smoke",
        Some("target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz".to_string()),
        || {
            let _report = run_local_vcf_call_diploid_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed diploid genotype smoke output".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        213,
        "vcf.call_pseudohaploid smoke",
        Some("target/local-smoke/vcf.call_pseudohaploid/bcftools/pseudohaploid.vcf.gz".to_string()),
        || {
            let _report =
                run_local_vcf_call_pseudohaploid_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed pseudohaploid smoke output and replay stability".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        214,
        "vcf.call_gl smoke",
        Some("target/local-smoke/vcf.call_gl/bcftools/gl.vcf.gz".to_string()),
        || {
            let _report = run_local_vcf_call_gl_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed likelihood-bearing GL smoke output".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        215,
        "vcf.damage_filter smoke",
        Some(
            "target/local-smoke/vcf.damage_filter/bcftools/damage_filter_summary.json".to_string(),
        ),
        || {
            let _report = run_local_vcf_damage_filter_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed damage-filter smoke evidence and removal counts".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        216,
        "vcf.gl_propagation smoke",
        Some(
            "target/local-smoke/vcf.gl_propagation/bcftools/gl_propagation_report.json".to_string(),
        ),
        || {
            let _report = run_local_vcf_gl_propagation_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed GL propagation smoke survival evidence".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        217,
        "vcf.filter smoke",
        Some("target/local-smoke/vcf.filter/bcftools/filter_explain.json".to_string()),
        || {
            let _report = run_local_vcf_filter_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed filter smoke thresholds and reviewer-facing breakdown"
                .to_string())
        },
    );
    record_goal_check(
        &mut checks,
        218,
        "vcf.stats smoke",
        Some("target/local-smoke/vcf.stats/bcftools/stats.json".to_string()),
        || {
            let _report = run_local_vcf_stats_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed stats smoke metrics and normalized ti/tv output".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        219,
        "vcf.qc smoke",
        Some("target/local-smoke/vcf.qc/plink2/qc.json".to_string()),
        || {
            let _report = run_local_vcf_qc_smoke(repo_root, GOVERNED_VCF_QC_TOOL_ID)?;
            Ok("validated governed QC smoke exclusion evidence".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        220,
        "vcf.prepare_reference_panel smoke",
        Some("target/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz".to_string()),
        || {
            let _report =
                run_local_vcf_prepare_reference_panel_smoke(repo_root, GOVERNED_VCF_CALL_TOOL_ID)?;
            Ok("validated governed prepared-panel normalization and deduplication evidence"
                .to_string())
        },
    );
    record_goal_check(
        &mut checks,
        221,
        "vcf.phasing smoke",
        Some("target/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz".to_string()),
        || {
            let _report =
                run_local_vcf_phasing_smoke(repo_root, GOVERNED_VCF_PANEL_WORKFLOW_TOOL_ID)?;
            Ok("validated governed phasing smoke output and phase-block evidence".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        222,
        "vcf.impute smoke",
        Some("target/local-smoke/vcf.impute/beagle/imputed.vcf.gz".to_string()),
        || {
            let _report = run_local_vcf_impute_smoke(repo_root, GOVERNED_VCF_IMPUTE_TOOL_ID)?;
            Ok("validated governed impute smoke output and masked-truth evidence".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        223,
        "vcf.imputation_metrics smoke",
        Some(
            "target/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json".to_string(),
        ),
        || {
            let _report =
                run_local_vcf_imputation_metrics_smoke(repo_root, GOVERNED_VCF_IMPUTE_TOOL_ID)?;
            Ok("validated governed imputation metrics smoke summary".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        224,
        "vcf.pca smoke",
        Some("target/local-smoke/vcf.pca/plink2/pca.json".to_string()),
        || {
            let _report = run_local_vcf_pca_smoke(repo_root, GOVERNED_VCF_COHORT_TOOL_ID)?;
            Ok("validated governed PCA smoke report with sample and eigenvalue evidence"
                .to_string())
        },
    );
    record_goal_check(
        &mut checks,
        225,
        "vcf.admixture smoke",
        Some("target/local-smoke/vcf.admixture/plink2/admixture.json".to_string()),
        || {
            let _report = run_local_vcf_admixture_smoke(repo_root, GOVERNED_VCF_COHORT_TOOL_ID)?;
            Ok("validated governed admixture smoke report and structured insufficiency evidence"
                .to_string())
        },
    );
    record_goal_check(
        &mut checks,
        226,
        "vcf.population_structure smoke",
        Some(
            "target/local-smoke/vcf.population_structure/plink2/population_structure.json"
                .to_string(),
        ),
        || {
            let _report =
                run_local_vcf_population_structure_smoke(repo_root, GOVERNED_VCF_COHORT_TOOL_ID)?;
            Ok("validated governed population-structure smoke report over PCA and admixture outputs".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        227,
        "vcf.roh smoke",
        Some("target/local-smoke/vcf.roh/plink2/roh.json".to_string()),
        || {
            let _report = run_local_vcf_roh_smoke(repo_root, GOVERNED_VCF_COHORT_TOOL_ID)?;
            Ok("validated governed ROH smoke report and per-sample segment evidence".to_string())
        },
    );
    record_goal_check(
        &mut checks,
        228,
        "vcf.ibd smoke",
        Some("target/local-smoke/vcf.ibd/germline/ibd.json".to_string()),
        || {
            let _report = run_local_vcf_ibd_smoke(repo_root, GOVERNED_VCF_IBD_TOOL_ID)?;
            Ok("validated governed IBD smoke report and localized insufficient-overlap probe"
                .to_string())
        },
    );
    record_goal_check(
        &mut checks,
        229,
        "vcf.demography smoke",
        Some("target/local-smoke/vcf.demography/ibdne/demography.json".to_string()),
        || {
            let _report =
                run_local_vcf_demography_smoke(repo_root, GOVERNED_VCF_DEMOGRAPHY_TOOL_ID)?;
            Ok("validated governed demography smoke report and deterministic insufficient-data probe".to_string())
        },
    );

    let report = build_vcf_smoke_suite_ready_report(repo_root, &absolute_output_path, checks);
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        bail!(
            "VCF smoke suite gate failed for goals {}; inspect {}",
            report.failing_goal_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(", "),
            report.output_path
        );
    }
    Ok(report)
}

fn record_goal_check<F>(
    checks: &mut Vec<LocalVcfSmokeSuiteGoalCheck>,
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    run: F,
) where
    F: FnOnce() -> Result<String>,
{
    match run() {
        Ok(detail) => checks.push(LocalVcfSmokeSuiteGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(LocalVcfSmokeSuiteGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: false,
            detail: format!("{error:#}"),
        }),
    }
}

fn build_vcf_smoke_suite_ready_report(
    repo_root: &Path,
    output_path: &Path,
    checks: Vec<LocalVcfSmokeSuiteGoalCheck>,
) -> LocalVcfSmokeSuiteReadyReport {
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let checked_goal_count = checks.len();
    let passed_goal_count = checked_goal_count.saturating_sub(failed_goal_count);

    LocalVcfSmokeSuiteReadyReport {
        schema_version: LOCAL_VCF_SMOKE_SUITE_READY_SCHEMA_VERSION,
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
        build_vcf_smoke_suite_ready_report, LocalVcfSmokeSuiteGoalCheck,
        DEFAULT_VCF_SMOKE_SUITE_READY_PATH, LOCAL_VCF_SMOKE_SUITE_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_smoke_suite_ready_report_marks_failed_goal_ids() {
        let repo_root = repo_root();
        let checks = vec![
            LocalVcfSmokeSuiteGoalCheck {
                goal_id: 211,
                surface: "vcf.call smoke".to_string(),
                output_path: Some("target/local-smoke/vcf.call/bcftools/calls.vcf.gz".to_string()),
                ok: true,
                detail: "ok".to_string(),
            },
            LocalVcfSmokeSuiteGoalCheck {
                goal_id: 229,
                surface: "vcf.demography smoke".to_string(),
                output_path: Some(
                    "target/local-smoke/vcf.demography/ibdne/demography.json".to_string(),
                ),
                ok: false,
                detail: "insufficient-data probe missing".to_string(),
            },
        ];

        let report = build_vcf_smoke_suite_ready_report(
            &repo_root,
            &repo_root.join(DEFAULT_VCF_SMOKE_SUITE_READY_PATH),
            checks,
        );

        assert_eq!(report.schema_version, LOCAL_VCF_SMOKE_SUITE_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_SMOKE_SUITE_READY_PATH);
        assert_eq!(report.checked_goal_count, 2);
        assert_eq!(report.passed_goal_count, 1);
        assert_eq!(report.failed_goal_count, 1);
        assert_eq!(report.failing_goal_ids, vec![229]);
        assert!(!report.ok);
    }

    #[test]
    fn vcf_smoke_suite_ready_report_marks_clean_goal_slice() {
        let repo_root = repo_root();
        let checks = vec![
            LocalVcfSmokeSuiteGoalCheck {
                goal_id: 211,
                surface: "vcf.call smoke".to_string(),
                output_path: Some("target/local-smoke/vcf.call/bcftools/calls.vcf.gz".to_string()),
                ok: true,
                detail: "ok".to_string(),
            },
            LocalVcfSmokeSuiteGoalCheck {
                goal_id: 212,
                surface: "vcf.call_diploid smoke".to_string(),
                output_path: Some(
                    "target/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz".to_string(),
                ),
                ok: true,
                detail: "ok".to_string(),
            },
        ];

        let report = build_vcf_smoke_suite_ready_report(
            &repo_root,
            &repo_root.join(DEFAULT_VCF_SMOKE_SUITE_READY_PATH),
            checks,
        );

        assert_eq!(report.checked_goal_count, 2);
        assert_eq!(report.passed_goal_count, 2);
        assert_eq!(report.failed_goal_count, 0);
        assert!(report.failing_goal_ids.is_empty());
        assert!(report.ok);
    }
}
