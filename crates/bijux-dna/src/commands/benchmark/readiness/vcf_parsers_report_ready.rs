use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::{vcf_parser_fixture_inventory, VcfDomainStage};
use serde::Serialize;

use super::vcf_comparable_metrics::{
    render_vcf_comparable_metrics, DEFAULT_VCF_COMPARABLE_METRICS_PATH,
};
use super::vcf_expected_benchmark_results::{
    render_vcf_expected_benchmark_results, DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::vcf_missing_result_report::{
    render_vcf_missing_result_report, DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH,
};
use super::vcf_normalized_metrics_schema::render_vcf_normalized_metrics_schema;
use super::vcf_parser_coverage::{render_vcf_parser_coverage, DEFAULT_VCF_PARSER_COVERAGE_PATH};
use super::vcf_parser_failure_tests::{
    render_vcf_parser_failure_tests, DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH,
};
use super::vcf_report_map::{render_vcf_report_map, DEFAULT_VCF_REPORT_MAP_PATH};
use crate::commands::benchmark::schema_paths::{
    DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH, DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR,
};
use crate::commands::benchmark::schema_validation::{
    validate_vcf_schemas, DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_PARSERS_REPORT_READY_PATH: &str =
    "target/bench-readiness/VCF_PARSERS_REPORT_READY.json";
const VCF_PARSERS_REPORT_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_parsers_report_ready.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfParsersReportReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfParsersReportReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) parser_fixture_row_count: usize,
    pub(crate) benchmark_ready_parser_row_count: usize,
    pub(crate) expected_result_row_count: usize,
    pub(crate) report_map_row_count: usize,
    pub(crate) comparable_metric_row_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<VcfParsersReportReadyGoalCheck>,
}

pub(crate) fn run_render_vcf_parsers_report_ready(
    args: &parse::BenchReadinessRenderVcfParsersReportReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_parsers_report_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_PARSERS_REPORT_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_parsers_report_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfParsersReportReadyReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let parser_fixture_row_count = vcf_parser_fixture_inventory().len();
    let mut benchmark_ready_parser_row_count = 0usize;
    let mut expected_result_row_count = 0usize;
    let mut report_map_row_count = 0usize;
    let mut comparable_metric_row_count = 0usize;

    let mut parser_coverage_report = None;
    let mut expected_results_report = None;
    let mut report_map_report = None;

    record_goal_check(
        &mut checks,
        246,
        "vcf normalized metrics schemas",
        Some(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH.to_string()),
        || {
            let schema_report = render_vcf_normalized_metrics_schema(
                repo_root,
                PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH),
                PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR),
            )?;
            if schema_report.stage_count != 20 || schema_report.extension_count != 20 {
                bail!(
                    "VCF normalized metrics schema report drifted: stage_count={}, extension_count={}",
                    schema_report.stage_count,
                    schema_report.extension_count
                );
            }

            let validation_report = validate_vcf_schemas(
                repo_root,
                PathBuf::from(DEFAULT_VCF_SCHEMA_VALIDATION_REPORT_PATH),
                PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_SCHEMA_PATH),
                PathBuf::from(DEFAULT_VCF_NORMALIZED_METRICS_STAGE_DIR),
            )?;
            if !validation_report.passes_gate
                || !validation_report.shared_schema_matches
                || validation_report.stage_count != 20
                || validation_report.required_stage_count != 17
                || validation_report.exact_stage_schema_file_count != 20
            {
                bail!("VCF schema validation drifted from the governed pass state");
            }
            Ok("validated the governed shared and stage-specific VCF normalized metrics schemas"
                .to_string())
        },
    );

    record_goal_check(&mut checks, 247, "vcf bcftools parser fixtures", None, || {
        validate_parser_fixture_family(
            repo_root,
            "bcftools",
            &[
                VcfDomainStage::Call,
                VcfDomainStage::CallDiploid,
                VcfDomainStage::CallGl,
                VcfDomainStage::CallPseudohaploid,
                VcfDomainStage::DamageFilter,
                VcfDomainStage::Filter,
                VcfDomainStage::GlPropagation,
                VcfDomainStage::Postprocess,
                VcfDomainStage::PrepareReferencePanel,
                VcfDomainStage::Stats,
            ],
            "validated 10 governed bcftools VCF parser fixture rows",
        )
    });

    record_goal_check(&mut checks, 248, "vcf angsd parser fixtures", None, || {
        validate_parser_fixture_family(
            repo_root,
            "angsd",
            &[
                VcfDomainStage::CallGl,
                VcfDomainStage::CallPseudohaploid,
                VcfDomainStage::DamageFilter,
                VcfDomainStage::GlPropagation,
            ],
            "validated 4 governed ANGSD VCF parser fixture rows",
        )
    });

    record_goal_check(&mut checks, 249, "vcf plink family parser fixtures", None, || {
        validate_parser_fixture_family(
            repo_root,
            "plink",
            &[VcfDomainStage::Qc, VcfDomainStage::Admixture],
            "validated governed plink VCF parser fixture rows",
        )?;
        validate_parser_fixture_family(
            repo_root,
            "plink2",
            &[
                VcfDomainStage::Qc,
                VcfDomainStage::Pca,
                VcfDomainStage::Admixture,
                VcfDomainStage::PopulationStructure,
                VcfDomainStage::Roh,
            ],
            "validated governed plink2 VCF parser fixture rows",
        )?;
        Ok("validated 7 governed PLINK-family VCF parser fixture rows".to_string())
    });

    record_goal_check(&mut checks, 250, "vcf eigensoft parser fixtures", None, || {
        validate_parser_fixture_family(
            repo_root,
            "eigensoft",
            &[VcfDomainStage::Pca, VcfDomainStage::PopulationStructure],
            "validated 2 governed EIGENSOFT VCF parser fixture rows",
        )
    });

    record_goal_check(&mut checks, 251, "vcf phasing parser fixtures", None, || {
        validate_parser_fixture_family(
            repo_root,
            "shapeit5",
            &[VcfDomainStage::Phasing],
            "validated governed shapeit5 phasing parser fixture rows",
        )?;
        validate_parser_fixture_family(
            repo_root,
            "eagle",
            &[VcfDomainStage::Phasing],
            "validated governed eagle phasing parser fixture rows",
        )?;
        validate_parser_fixture_family(
            repo_root,
            "beagle",
            &[VcfDomainStage::Phasing],
            "validated governed beagle phasing parser fixture rows",
        )?;
        Ok("validated 3 governed VCF phasing parser fixture rows".to_string())
    });

    record_goal_check(&mut checks, 252, "vcf imputation parser fixtures", None, || {
        for tool_id in ["beagle", "glimpse", "impute5", "minimac4"] {
            validate_parser_fixture_family(
                repo_root,
                tool_id,
                &[VcfDomainStage::Impute, VcfDomainStage::Imputation],
                "validated governed VCF imputation parser fixture rows",
            )?;
        }
        Ok("validated 8 governed VCF imputation parser fixture rows".to_string())
    });

    record_goal_check(&mut checks, 253, "vcf segment parser fixtures", None, || {
        validate_parser_fixture_family(
            repo_root,
            "plink2",
            &[VcfDomainStage::Roh],
            "validated governed plink2 ROH parser fixture rows",
        )?;
        validate_parser_fixture_family(
            repo_root,
            "germline",
            &[VcfDomainStage::Ibd],
            "validated governed germline IBD parser fixture rows",
        )?;
        validate_parser_fixture_family(
            repo_root,
            "ibdseq",
            &[VcfDomainStage::Ibd],
            "validated governed ibdseq IBD parser fixture rows",
        )?;
        validate_parser_fixture_family(
            repo_root,
            "ibdhap",
            &[VcfDomainStage::Ibd],
            "validated governed ibdhap IBD parser fixture rows",
        )?;
        validate_parser_fixture_family(
            repo_root,
            "ibdne",
            &[VcfDomainStage::Demography],
            "validated governed ibdne demography parser fixture rows",
        )?;
        Ok("validated 5 governed VCF segment parser fixture rows".to_string())
    });

    record_goal_check(
        &mut checks,
        254,
        "vcf parser failure tests",
        Some(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH.to_string()),
        || {
            let report = render_vcf_parser_failure_tests(
                repo_root,
                PathBuf::from(DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH),
            )?;
            if report.row_count != 7 || report.passed_row_count != 7 || report.failed_row_count != 0
            {
                bail!("VCF parser failure tests drifted from the governed case set");
            }
            Ok("validated 7 governed malformed-output VCF parser failure cases".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        255,
        "vcf comparable metrics",
        Some(DEFAULT_VCF_COMPARABLE_METRICS_PATH.to_string()),
        || {
            let report = render_vcf_comparable_metrics(
                repo_root,
                PathBuf::from(DEFAULT_VCF_COMPARABLE_METRICS_PATH),
            )?;
            if report.stage_count != 12
                || report.multi_tool_stage_count != 12
                || report.retained_tool_row_count != 30
                || report.row_count != 33
            {
                bail!("VCF comparable metrics report drifted from the governed retained slice");
            }
            comparable_metric_row_count = report.row_count;
            Ok("validated governed comparable metrics across the retained multi-tool VCF stage slice".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        256,
        "vcf parser coverage",
        Some(DEFAULT_VCF_PARSER_COVERAGE_PATH.to_string()),
        || {
            let report = render_vcf_parser_coverage(
                repo_root,
                PathBuf::from(DEFAULT_VCF_PARSER_COVERAGE_PATH),
            )?;
            if report.stage_count != 8
                || report.tool_count != 1
                || report.row_count != 8
                || report.covered_row_count != 8
                || report.missing_row_count != 0
                || report.parser_coverage_percent != 100.0
            {
                bail!("VCF parser coverage drifted from the governed benchmark-ready slice");
            }
            benchmark_ready_parser_row_count = report.covered_row_count;
            parser_coverage_report = Some(report);
            Ok("validated full parser coverage across the 8 benchmark-ready VCF rows".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        257,
        "vcf expected benchmark results",
        Some(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH.to_string()),
        || {
            let report = render_vcf_expected_benchmark_results(
                repo_root,
                PathBuf::from(DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH),
            )?;
            if report.row_count != 8
                || report.stage_count != 8
                || report.tool_count != 1
                || report.corpus_count != 1
                || report.asset_profile_count != 3
            {
                bail!("VCF expected benchmark results drifted from the governed ready slice");
            }
            expected_result_row_count = report.row_count;
            expected_results_report = Some(report);
            Ok("validated 8 governed expected benchmark-ready VCF result rows".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        258,
        "vcf missing-result report",
        Some(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH.to_string()),
        || {
            let report = render_vcf_missing_result_report(
                repo_root,
                PathBuf::from(DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH),
            )?;
            if report.expected_row_count != 8
                || report.present_result_row_count != 7
                || report.missing_result_row_count != 1
                || !report.passes_behavior_test
            {
                bail!("VCF missing-result report drifted from the governed behavior contract");
            }
            Ok("validated governed missing-result behavior with one retained missing_result row"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        259,
        "vcf report map",
        Some(DEFAULT_VCF_REPORT_MAP_PATH.to_string()),
        || {
            let report =
                render_vcf_report_map(repo_root, PathBuf::from(DEFAULT_VCF_REPORT_MAP_PATH))?;
            if report.row_count != 8
                || report.stage_count != 8
                || report.tool_count != 1
                || report.section_count != 4
                || report.summary_table_count != 4
            {
                bail!("VCF report map drifted from the governed expected-result slice");
            }
            report_map_row_count = report.row_count;
            report_map_report = Some(report);
            Ok("validated governed report-map coverage across the expected benchmark-ready VCF rows".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        260,
        "vcf parser/report ready pair completeness",
        Some(DEFAULT_VCF_PARSERS_REPORT_READY_PATH.to_string()),
        || {
            let parser_pairs = parser_coverage_report
                .as_ref()
                .ok_or_else(|| anyhow!("goal 256 parser coverage report was not produced"))?
                .rows
                .iter()
                .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
                .collect::<BTreeSet<_>>();
            let expected_pairs = expected_results_report
                .as_ref()
                .ok_or_else(|| {
                    anyhow!("goal 257 expected benchmark results report was not produced")
                })?
                .rows
                .iter()
                .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
                .collect::<BTreeSet<_>>();
            let report_pairs = report_map_report
                .as_ref()
                .ok_or_else(|| anyhow!("goal 259 report map was not produced"))?
                .rows
                .iter()
                .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
                .collect::<BTreeSet<_>>();

            ensure_pair_sets_match("parser coverage", &expected_pairs, &parser_pairs)?;
            ensure_pair_sets_match("report map", &expected_pairs, &report_pairs)?;

            Ok(format!(
                "validated {} benchmark-ready VCF rows across parser coverage, expected results, and report-map coverage",
                expected_pairs.len()
            ))
        },
    );

    let report = build_vcf_parsers_report_ready_report(
        repo_root,
        &absolute_output_path,
        checks,
        parser_fixture_row_count,
        benchmark_ready_parser_row_count,
        expected_result_row_count,
        report_map_row_count,
        comparable_metric_row_count,
    );
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        bail!(
            "VCF parser/report readiness gate failed for goals {}; inspect {}",
            report.failing_goal_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(", "),
            report.output_path
        );
    }
    Ok(report)
}

fn validate_parser_fixture_family(
    repo_root: &Path,
    tool_id: &str,
    expected_stages: &[VcfDomainStage],
    detail: &str,
) -> Result<String> {
    let expected_stage_set =
        expected_stages.iter().map(|stage| stage.as_str().to_string()).collect::<BTreeSet<_>>();
    let rows = vcf_parser_fixture_inventory()
        .iter()
        .filter(|row| row.tool_id == tool_id && expected_stage_set.contains(row.stage.as_str()))
        .copied()
        .collect::<Vec<_>>();
    let actual_stage_set =
        rows.iter().map(|row| row.stage.as_str().to_string()).collect::<BTreeSet<_>>();
    let missing_stages =
        expected_stage_set.difference(&actual_stage_set).map(String::as_str).collect::<Vec<_>>();
    if !missing_stages.is_empty() {
        bail!(
            "VCF parser fixture family `{tool_id}` is missing stages [{}]",
            missing_stages.join(", ")
        );
    }

    for row in &rows {
        if row.parser_id.trim().is_empty() {
            bail!(
                "VCF parser fixture family `{tool_id}` is missing parser_id for stage `{}`",
                row.stage.as_str()
            );
        }
        let fixture_path = repo_root.join(row.fixture_path);
        if !fixture_path.exists() {
            bail!(
                "VCF parser fixture family `{tool_id}` is missing fixture path `{}`",
                row.fixture_path
            );
        }
    }
    if rows.len() != expected_stages.len() {
        bail!(
            "VCF parser fixture family `{tool_id}` expected {} governed rows for this goal slice but found {}",
            expected_stages.len(),
            rows.len()
        );
    }

    Ok(detail.to_string())
}

fn ensure_pair_sets_match(
    surface: &str,
    expected: &BTreeSet<(String, String)>,
    observed: &BTreeSet<(String, String)>,
) -> Result<()> {
    let missing = expected.difference(observed).cloned().collect::<Vec<_>>();
    let extra = observed.difference(expected).cloned().collect::<Vec<_>>();
    if missing.is_empty() && extra.is_empty() {
        return Ok(());
    }

    let format_pairs = |pairs: &[(String, String)]| {
        if pairs.is_empty() {
            "none".to_string()
        } else {
            pairs
                .iter()
                .map(|(stage_id, tool_id)| format!("{stage_id}:{tool_id}"))
                .collect::<Vec<_>>()
                .join(", ")
        }
    };

    bail!(
        "{surface} drifted from the canonical benchmark-ready VCF pair slice: missing [{}], extra [{}]",
        format_pairs(&missing),
        format_pairs(&extra)
    );
}

fn record_goal_check<F>(
    checks: &mut Vec<VcfParsersReportReadyGoalCheck>,
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    run: F,
) where
    F: FnOnce() -> Result<String>,
{
    match run() {
        Ok(detail) => checks.push(VcfParsersReportReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(VcfParsersReportReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: false,
            detail: format!("{error:#}"),
        }),
    }
}

fn build_vcf_parsers_report_ready_report(
    repo_root: &Path,
    output_path: &Path,
    checks: Vec<VcfParsersReportReadyGoalCheck>,
    parser_fixture_row_count: usize,
    benchmark_ready_parser_row_count: usize,
    expected_result_row_count: usize,
    report_map_row_count: usize,
    comparable_metric_row_count: usize,
) -> VcfParsersReportReadyReport {
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let checked_goal_count = checks.len();
    let passed_goal_count = checked_goal_count.saturating_sub(failed_goal_count);

    VcfParsersReportReadyReport {
        schema_version: VCF_PARSERS_REPORT_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        parser_fixture_row_count,
        benchmark_ready_parser_row_count,
        expected_result_row_count,
        report_map_row_count,
        comparable_metric_row_count,
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
        build_vcf_parsers_report_ready_report, VcfParsersReportReadyGoalCheck,
        DEFAULT_VCF_PARSERS_REPORT_READY_PATH, VCF_PARSERS_REPORT_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_parsers_report_ready_report_marks_failed_goal_ids_and_counts() {
        let root = repo_root();
        let checks = vec![
            VcfParsersReportReadyGoalCheck {
                goal_id: 246,
                surface: "vcf normalized metrics schemas".to_string(),
                output_path: Some("target/bench-readiness/vcf-schema-validation.json".to_string()),
                ok: true,
                detail: "ok".to_string(),
            },
            VcfParsersReportReadyGoalCheck {
                goal_id: 260,
                surface: "vcf parser/report ready pair completeness".to_string(),
                output_path: Some(DEFAULT_VCF_PARSERS_REPORT_READY_PATH.to_string()),
                ok: false,
                detail: "pair mismatch".to_string(),
            },
        ];

        let report = build_vcf_parsers_report_ready_report(
            &root,
            &root.join(DEFAULT_VCF_PARSERS_REPORT_READY_PATH),
            checks,
            38,
            8,
            8,
            8,
            33,
        );

        assert_eq!(report.schema_version, VCF_PARSERS_REPORT_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_PARSERS_REPORT_READY_PATH);
        assert_eq!(report.checked_goal_count, 2);
        assert_eq!(report.passed_goal_count, 1);
        assert_eq!(report.failed_goal_count, 1);
        assert_eq!(report.failing_goal_ids, vec![260]);
        assert_eq!(report.parser_fixture_row_count, 38);
        assert_eq!(report.benchmark_ready_parser_row_count, 8);
        assert_eq!(report.expected_result_row_count, 8);
        assert_eq!(report.report_map_row_count, 8);
        assert_eq!(report.comparable_metric_row_count, 33);
        assert!(!report.ok);
    }
}
