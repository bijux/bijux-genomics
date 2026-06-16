use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows;
use super::vcf_adapter_missing_input_tests::{
    render_vcf_adapter_missing_input_tests, DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH,
};
use super::vcf_adapter_output_coverage::{
    render_vcf_adapter_output_coverage, VcfAdapterOutputCoverageStatus,
    DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH,
};
use super::vcf_angsd_adapter::{render_vcf_angsd_adapter, DEFAULT_VCF_ANGSD_ADAPTER_PATH};
use super::vcf_bcftools_adapter::{render_vcf_bcftools_adapter, DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH};
use super::vcf_descent_family_adapter::{
    render_vcf_descent_family_adapter, DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
};
use super::vcf_eigensoft_adapter::{
    render_vcf_eigensoft_adapter, DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH,
};
use super::vcf_imputation_family_adapter::{
    render_vcf_imputation_family_adapter, DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH,
};
use super::vcf_matrix_registry_consistency::{
    render_vcf_matrix_registry_consistency, DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH,
};
use super::vcf_orphan_tools::{render_vcf_orphan_tools, DEFAULT_VCF_ORPHAN_TOOLS_PATH};
use super::vcf_phasing_family_adapter::{
    render_vcf_phasing_family_adapter, DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
    DEFAULT_VCF_EAGLE_ADAPTER_PATH, DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
};
use super::vcf_plink_family_adapter::{
    render_vcf_plink_family_adapter, DEFAULT_VCF_PLINK2_ADAPTER_PATH,
    DEFAULT_VCF_PLINK_ADAPTER_PATH,
};
use super::vcf_rendered_commands::{render_vcf_commands, DEFAULT_VCF_RENDERED_COMMANDS_PATH};
use super::vcf_tool_serving_map::{render_vcf_tool_serving_map, DEFAULT_VCF_TOOL_SERVING_MAP_PATH};
use super::vcf_undercovered_stages::{
    render_vcf_undercovered_stages, DEFAULT_VCF_UNDERCOVERED_STAGES_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_ADAPTERS_READY_PATH: &str =
    "benchmarks/readiness/VCF_ADAPTERS_READY.json";
const VCF_ADAPTERS_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_adapters_ready.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAdaptersReadyGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAdaptersReadyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) benchmark_ready_pair_count: usize,
    pub(crate) adapter_complete_pair_count: usize,
    pub(crate) output_complete_pair_count: usize,
    pub(crate) rendered_command_pair_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<VcfAdaptersReadyGoalCheck>,
}

pub(crate) fn run_render_vcf_adapters_ready(
    args: &parse::BenchReadinessRenderVcfAdaptersReadyArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_adapters_ready(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ADAPTERS_READY_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_adapters_ready(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAdaptersReadyReport> {
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();
    let mut benchmark_ready_pair_count = 0usize;
    let mut adapter_complete_pair_count = 0usize;
    let mut output_complete_pair_count = 0usize;
    let mut rendered_command_pair_count = 0usize;

    let mut tool_serving_map_report = None;
    let mut bcftools_adapter_report = None;
    let mut shapeit5_adapter_report = None;
    let mut imputation_family_adapter_report = None;
    let mut adapter_output_coverage_report = None;
    let mut plink_adapter_report = None;
    let mut plink2_family_adapter_report = None;
    let mut rendered_commands_report = None;

    record_goal_check(
        &mut checks,
        231,
        "vcf tool-serving map",
        Some(DEFAULT_VCF_TOOL_SERVING_MAP_PATH.to_string()),
        || {
            let report = render_vcf_tool_serving_map(
                repo_root,
                PathBuf::from(DEFAULT_VCF_TOOL_SERVING_MAP_PATH),
            )?;
            if report.row_count != 23
                || report.stage_count != 20
                || report.tool_count != 8
                || report.benchmark_ready_row_count != 20
                || report.not_benchmark_ready_row_count != 3
            {
                bail!(
                    "VCF tool-serving map drifted: rows={}, stages={}, tools={}, benchmark_ready={}, not_benchmark_ready={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.benchmark_ready_row_count,
                    report.not_benchmark_ready_row_count
                );
            }
            benchmark_ready_pair_count = report.benchmark_ready_row_count;
            tool_serving_map_report = Some(report);
            Ok("validated 23 governed VCF stage-tool rows with 20 canonical benchmark-ready pairs"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        232,
        "vcf orphan tools",
        Some(DEFAULT_VCF_ORPHAN_TOOLS_PATH.to_string()),
        || {
            let report =
                render_vcf_orphan_tools(repo_root, PathBuf::from(DEFAULT_VCF_ORPHAN_TOOLS_PATH))?;
            if report.orphan_count != 8
                || report.required_tool_count != 16
                || report.registered_tool_count != 16
                || report.served_tool_count != 8
                || report.rows.iter().any(|row| {
                    row.served_stage_count != 0 || row.decision != "future_not_benchmark_ready"
                })
            {
                bail!("VCF orphan-tool report drifted from the governed orphan tool slice");
            }
            Ok(
                "validated 8 governed orphan VCF tools with explicit future_not_benchmark_ready decisions"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        233,
        "vcf undercovered stages",
        Some(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH.to_string()),
        || {
            let report = render_vcf_undercovered_stages(
                repo_root,
                PathBuf::from(DEFAULT_VCF_UNDERCOVERED_STAGES_PATH),
            )?;
            if report.stage_count != 20 || report.undercovered_stage_count != 10 {
                bail!(
                    "VCF undercovered-stage report drifted: stage_count={}, undercovered_stage_count={}",
                    report.stage_count,
                    report.undercovered_stage_count
                );
            }
            let future_not_benchmark_ready =
                report.decision_counts.get("future_not_benchmark_ready").copied().unwrap_or(0);
            let limit_to_specialized_tool =
                report.decision_counts.get("limit_to_specialized_tool").copied().unwrap_or(0);
            if future_not_benchmark_ready != 9 || limit_to_specialized_tool != 1 {
                bail!(
                    "VCF undercovered-stage decisions drifted: future_not_benchmark_ready={future_not_benchmark_ready}, limit_to_specialized_tool={limit_to_specialized_tool}"
                );
            }
            Ok(
                "validated 10 governed undercovered VCF stages with explicit future vs specialized decisions"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        234,
        "vcf matrix-registry consistency",
        Some(DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH.to_string()),
        || {
            let output_path = PathBuf::from(DEFAULT_VCF_MATRIX_REGISTRY_CONSISTENCY_PATH);
            let _ = render_vcf_matrix_registry_consistency(repo_root, output_path.clone());
            let payload = std::fs::read_to_string(repo_root.join(&output_path))
                .with_context(|| format!("read {}", repo_root.join(&output_path).display()))?;
            let report: serde_json::Value =
                serde_json::from_str(&payload).context("parse VCF matrix-registry JSON")?;
            if report.get("passes_gate").and_then(serde_json::Value::as_bool) != Some(true)
                || report.get("stage_count").and_then(serde_json::Value::as_u64) != Some(20)
                || report.get("matrix_row_count").and_then(serde_json::Value::as_u64) != Some(23)
                || report.get("registry_pair_count").and_then(serde_json::Value::as_u64) != Some(42)
                || report
                    .get("benchmark_ready_registry_pair_count")
                    .and_then(serde_json::Value::as_u64)
                    != Some(16)
                || report.get("unregistered_matrix_pair_count").and_then(serde_json::Value::as_u64)
                    != Some(0)
                || report
                    .get("missing_benchmark_ready_registry_pair_count")
                    .and_then(serde_json::Value::as_u64)
                    != Some(0)
                || report
                    .get("rows")
                    .and_then(serde_json::Value::as_array)
                    .is_none_or(|rows| !rows.is_empty())
            {
                bail!("VCF matrix-registry consistency gate drifted from the governed clean pass state");
            }
            Ok("validated clean agreement between the governed VCF stage matrix and registry"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        235,
        "vcf bcftools adapter",
        Some(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH.to_string()),
        || {
            let report = render_vcf_bcftools_adapter(
                repo_root,
                PathBuf::from(DEFAULT_VCF_BCFTOOLS_ADAPTER_PATH),
            )?;
            if report.row_count != 11
                || report.supported_row_count != 11
                || report.planned_row_count != 0
                || report.argv_valid_row_count != report.row_count
                || report.missing_input_test_passed_row_count != report.row_count
                || report.indexed_row_count != 9
            {
                bail!(
                    "VCF bcftools adapter report drifted from the governed retained row contract"
                );
            }
            adapter_complete_pair_count += report
                .rows
                .iter()
                .filter(|row| {
                    row.benchmark_status == "benchmark_ready"
                        && row.argv_validation_passed
                        && row.missing_input_test_passed
                })
                .count();
            bcftools_adapter_report = Some(report);
            Ok("validated executable governed bcftools adapter rows with passing missing-input probes".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        236,
        "vcf angsd adapter",
        Some(DEFAULT_VCF_ANGSD_ADAPTER_PATH.to_string()),
        || {
            let report =
                render_vcf_angsd_adapter(repo_root, PathBuf::from(DEFAULT_VCF_ANGSD_ADAPTER_PATH))?;
            if report.row_count != 4
                || report.supported_stage_row_count != 4
                || report.benchmark_ready_row_count != 0
                || report.argv_valid_row_count != report.row_count
                || report.missing_input_test_passed_row_count != report.row_count
                || report.bam_list_row_count != 3
                || report.parser_output_row_count != report.row_count
            {
                bail!(
                    "VCF angsd adapter report drifted from the governed low-coverage row contract"
                );
            }
            Ok("validated retained ANGSD VCF adapter rows and their missing-input behavior"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        237,
        "vcf plink family adapters",
        Some("benchmarks/readiness/adapters/plink.vcf.json".to_string()),
        || {
            let plink_report = render_vcf_plink_family_adapter(
                repo_root,
                "plink",
                PathBuf::from(DEFAULT_VCF_PLINK_ADAPTER_PATH),
            )?;
            let plink2_family_report = render_vcf_plink_family_adapter(
                repo_root,
                "plink2",
                PathBuf::from(DEFAULT_VCF_PLINK2_ADAPTER_PATH),
            )?;
            if plink_report.row_count != 2
                || plink_report.benchmark_ready_row_count != 1
                || plink_report.parser_output_row_count != plink_report.row_count
                || plink_report.normalized_metrics_row_count != plink_report.row_count
                || plink_report.raw_output_declared_row_count != plink_report.row_count
                || plink_report.missing_input_test_passed_row_count != plink_report.row_count
            {
                bail!("VCF plink adapter report drifted from the governed retained row contract");
            }
            if plink2_family_report.row_count != 5
                || plink2_family_report.benchmark_ready_row_count != 5
                || plink2_family_report.parser_output_row_count != plink2_family_report.row_count
                || plink2_family_report.normalized_metrics_row_count
                    != plink2_family_report.row_count
                || plink2_family_report.raw_output_declared_row_count
                    != plink2_family_report.row_count
                || plink2_family_report.missing_input_test_passed_row_count
                    != plink2_family_report.row_count
            {
                bail!(
                    "VCF plink2 adapter report drifted from the governed benchmarked row contract"
                );
            }
            adapter_complete_pair_count += plink_report
                .rows
                .iter()
                .filter(|row| {
                    row.benchmark_status == "benchmark_ready"
                        && row.argv_validation_passed
                        && row.missing_input_test_passed
                })
                .count();
            adapter_complete_pair_count += plink2_family_report
                .rows
                .iter()
                .filter(|row| {
                    row.benchmark_status == "benchmark_ready"
                        && row.argv_validation_passed
                        && row.missing_input_test_passed
                })
                .count();
            plink_adapter_report = Some(plink_report);
            plink2_family_adapter_report = Some(plink2_family_report);
            Ok(
                "validated governed plink and plink2 adapter rows with explicit raw and normalized outputs"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        238,
        "vcf eigensoft adapter",
        Some(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH.to_string()),
        || {
            let report = render_vcf_eigensoft_adapter(
                repo_root,
                PathBuf::from(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH),
            )?;
            if report.row_count != 2
                || report.benchmark_ready_row_count != 1
                || report.conversion_output_row_count != report.row_count
                || report.pca_output_row_count != report.row_count
                || report.missing_input_test_passed_row_count != report.row_count
            {
                bail!(
                    "VCF eigensoft adapter report drifted from the governed retained row contract"
                );
            }
            Ok("validated retained EIGENSOFT VCF adapter rows and explicit convertf/smartpca outputs".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        239,
        "vcf phasing family adapters",
        Some(DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH.to_string()),
        || {
            let shapeit5_report = render_vcf_phasing_family_adapter(
                repo_root,
                "shapeit5",
                PathBuf::from(DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH),
            )?;
            let eagle_report = render_vcf_phasing_family_adapter(
                repo_root,
                "eagle",
                PathBuf::from(DEFAULT_VCF_EAGLE_ADAPTER_PATH),
            )?;
            let beagle_report = render_vcf_phasing_family_adapter(
                repo_root,
                "beagle",
                PathBuf::from(DEFAULT_VCF_BEAGLE_ADAPTER_PATH),
            )?;
            for report in [&shapeit5_report, &eagle_report, &beagle_report] {
                if report.row_count != 1
                    || report.parser_output_row_count != 1
                    || report.indexed_row_count != 1
                    || report.missing_input_test_passed_row_count != 1
                {
                    bail!("VCF phasing adapter report for `{}` drifted from the governed retained row contract", report.tool_id);
                }
            }
            if shapeit5_report.benchmark_ready_row_count != 1
                || eagle_report.benchmark_ready_row_count != 0
                || beagle_report.benchmark_ready_row_count != 0
            {
                bail!("VCF phasing benchmark-ready ownership drifted across retained backends");
            }
            shapeit5_adapter_report = Some(shapeit5_report);
            Ok("validated retained phasing adapter rows for shapeit5, eagle, and beagle"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        240,
        "vcf imputation family adapter",
        Some(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH.to_string()),
        || {
            let report = render_vcf_imputation_family_adapter(
                repo_root,
                PathBuf::from(DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH),
            )?;
            if report.row_count != 8
                || report.tool_count != 4
                || report.benchmark_ready_row_count != 2
                || report.parser_output_row_count != report.row_count
                || report.missing_input_test_passed_row_count != report.row_count
            {
                bail!("VCF imputation-family adapter report drifted from the governed retained row contract");
            }
            imputation_family_adapter_report = Some(report);
            Ok("validated retained imputation-family adapter rows with explicit parser outputs and missing-input probes".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        241,
        "vcf descent family adapter",
        Some(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || {
            let report = render_vcf_descent_family_adapter(
                repo_root,
                PathBuf::from(DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH),
            )?;
            if report.row_count != 5
                || report.tool_count != 5
                || report.benchmark_ready_row_count != 3
                || report.parser_output_row_count != report.row_count
                || report.missing_input_test_passed_row_count != report.row_count
            {
                bail!("VCF descent-family adapter report drifted from the governed retained row contract");
            }
            Ok("validated retained IBD, ROH, and demography adapter rows".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        242,
        "vcf adapter output coverage",
        Some(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH.to_string()),
        || {
            let report = render_vcf_adapter_output_coverage(
                repo_root,
                PathBuf::from(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH),
            )?;
            if report.row_count != 39
                || report.benchmark_ready_row_count != 20
                || report.benchmark_ready_complete_row_count != 20
                || report.benchmark_ready_incomplete_row_count != 0
                || report.complete_row_count != 36
                || report.incomplete_row_count != 3
            {
                bail!("VCF adapter output coverage report drifted from the governed completeness contract");
            }
            output_complete_pair_count = report
                .rows
                .iter()
                .filter(|row| {
                    row.benchmark_status == "benchmark_ready"
                        && row.status == VcfAdapterOutputCoverageStatus::Complete
                })
                .count();
            adapter_output_coverage_report = Some(report);
            Ok("validated complete raw-output and normalized-output declarations for the benchmark-ready VCF slice".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        243,
        "vcf adapter missing-input tests",
        Some(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH.to_string()),
        || {
            let report = render_vcf_adapter_missing_input_tests(
                repo_root,
                PathBuf::from(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH),
            )?;
            if report.row_count != 10
                || report.passed_row_count != 10
                || report.failed_row_count != 0
                || report.adapter_row_count != 9
                || report.support_row_count != 1
            {
                bail!("VCF adapter missing-input report drifted from the governed required-role contract");
            }
            Ok("validated all 10 governed VCF missing-input roles before tool execution"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        244,
        "vcf rendered commands",
        Some(DEFAULT_VCF_RENDERED_COMMANDS_PATH.to_string()),
        || {
            let report =
                render_vcf_commands(repo_root, PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH))?;
            let active_pair_count = collect_vcf_active_stage_tool_matrix_rows(repo_root)?
                .into_iter()
                .filter(|row| row.scope_state == "active")
                .count();
            if report.row_count != active_pair_count {
                bail!("VCF rendered commands drifted from the governed active VCF command slice");
            }
            let script_path = repo_root.join(&report.output_path);
            let argv_path = repo_root.join(&report.argv_output_path);
            let syntax = Command::new("bash")
                .arg("-n")
                .arg(&script_path)
                .current_dir(repo_root)
                .output()
                .with_context(|| format!("run bash -n on {}", script_path.display()))?;
            if !syntax.status.success() {
                bail!(
                    "VCF rendered commands shell script is not parseable by bash -n:\n{}",
                    String::from_utf8_lossy(&syntax.stderr)
                );
            }
            let argv_lines = std::fs::read_to_string(&argv_path)
                .with_context(|| format!("read {}", argv_path.display()))?
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count();
            if argv_lines != report.row_count {
                bail!(
                    "VCF rendered command argv JSONL drifted: expected {} rows but found {argv_lines}",
                    report.row_count
                );
            }
            rendered_command_pair_count = report.row_count;
            rendered_commands_report = Some(report);
            Ok("validated active VCF shell and argv command rendering with bash syntax coverage"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        245,
        "vcf adapter benchmark-ready pair completeness",
        Some(DEFAULT_VCF_ADAPTERS_READY_PATH.to_string()),
        || {
            let tool_serving_map_report = tool_serving_map_report
                .as_ref()
                .ok_or_else(|| anyhow!("VCF tool-serving map check did not produce a report"))?;
            let bcftools_adapter_report = bcftools_adapter_report
                .as_ref()
                .ok_or_else(|| anyhow!("VCF bcftools adapter check did not produce a report"))?;
            let plink_adapter_report = plink_adapter_report
                .as_ref()
                .ok_or_else(|| anyhow!("VCF plink adapter check did not produce a report"))?;
            let plink2_family_adapter_report = plink2_family_adapter_report
                .as_ref()
                .ok_or_else(|| anyhow!("VCF plink2 adapter check did not produce a report"))?;
            let adapter_output_coverage_report =
                adapter_output_coverage_report.as_ref().ok_or_else(|| {
                    anyhow!("VCF adapter output coverage check did not produce a report")
                })?;
            let shapeit5_adapter_report = shapeit5_adapter_report
                .as_ref()
                .ok_or_else(|| anyhow!("VCF shapeit5 adapter check did not produce a report"))?;
            let eigensoft_adapter_report = render_vcf_eigensoft_adapter(
                repo_root,
                PathBuf::from(DEFAULT_VCF_EIGENSOFT_ADAPTER_PATH),
            )?;
            let imputation_family_adapter_report =
                imputation_family_adapter_report.as_ref().ok_or_else(|| {
                    anyhow!("VCF imputation-family adapter check did not produce a report")
                })?;
            let rendered_commands_report = rendered_commands_report
                .as_ref()
                .ok_or_else(|| anyhow!("VCF rendered commands check did not produce a report"))?;

            let benchmark_ready_pairs = tool_serving_map_report
                .rows
                .iter()
                .filter(|row| row.benchmark_status == "benchmark_ready")
                .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
                .collect::<BTreeSet<_>>();
            let mut adapter_pairs = BTreeSet::new();
            adapter_pairs.extend(
                bcftools_adapter_report
                    .rows
                    .iter()
                    .filter(|row| {
                        row.benchmark_status == "benchmark_ready"
                            && row.argv_validation_passed
                            && row.missing_input_test_passed
                            && benchmark_ready_pairs
                                .contains(&(row.stage_id.clone(), row.tool_id.clone()))
                    })
                    .map(|row| (row.stage_id.clone(), row.tool_id.clone())),
            );
            adapter_pairs.extend(
                plink_adapter_report
                    .rows
                    .iter()
                    .filter(|row| {
                        row.benchmark_status == "benchmark_ready"
                            && row.argv_validation_passed
                            && row.missing_input_test_passed
                            && benchmark_ready_pairs
                                .contains(&(row.stage_id.clone(), row.tool_id.clone()))
                    })
                    .map(|row| (row.stage_id.clone(), row.tool_id.clone())),
            );
            adapter_pairs.extend(
                plink2_family_adapter_report
                    .rows
                    .iter()
                    .filter(|row| {
                        row.benchmark_status == "benchmark_ready"
                            && row.argv_validation_passed
                            && row.missing_input_test_passed
                            && benchmark_ready_pairs
                                .contains(&(row.stage_id.clone(), row.tool_id.clone()))
                    })
                    .map(|row| (row.stage_id.clone(), row.tool_id.clone())),
            );
            adapter_pairs.extend(
                shapeit5_adapter_report
                    .rows
                    .iter()
                    .filter(|row| {
                        row.benchmark_status == "benchmark_ready"
                            && row.argv_validation_passed
                            && row.missing_input_test_passed
                            && benchmark_ready_pairs
                                .contains(&(row.stage_id.clone(), row.tool_id.clone()))
                    })
                    .map(|row| (row.stage_id.clone(), row.tool_id.clone())),
            );
            adapter_pairs.extend(
                eigensoft_adapter_report
                    .rows
                    .iter()
                    .filter(|row| {
                        row.benchmark_status == "benchmark_ready"
                            && row.missing_input_test_passed
                            && benchmark_ready_pairs
                                .contains(&(row.stage_id.clone(), row.tool_id.clone()))
                    })
                    .map(|row| (row.stage_id.clone(), row.tool_id.clone())),
            );
            adapter_pairs.extend(
                imputation_family_adapter_report
                    .rows
                    .iter()
                    .filter(|row| {
                        row.benchmark_status == "benchmark_ready"
                            && row.argv_validation_passed
                            && row.missing_input_test_passed
                            && benchmark_ready_pairs
                                .contains(&(row.stage_id.clone(), row.tool_id.clone()))
                    })
                    .map(|row| (row.stage_id.clone(), row.tool_id.clone())),
            );
            let output_pairs = adapter_output_coverage_report
                .rows
                .iter()
                .filter(|row| {
                    row.benchmark_status == "benchmark_ready"
                        && row.status == VcfAdapterOutputCoverageStatus::Complete
                })
                .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
                .collect::<BTreeSet<_>>();
            let rendered_pairs = rendered_commands_report
                .rows
                .iter()
                .map(|row| (row.stage_id.clone(), row.tool_id.clone()))
                .collect::<BTreeSet<_>>();

            ensure_pair_sets_match(
                "bcftools adapter completeness",
                &benchmark_ready_pairs,
                &adapter_pairs,
            )?;
            ensure_pair_sets_match(
                "adapter output completeness",
                &benchmark_ready_pairs,
                &output_pairs,
            )?;
            ensure_pair_sets_match(
                "rendered command coverage",
                &benchmark_ready_pairs,
                &rendered_pairs,
            )?;
            adapter_complete_pair_count = adapter_pairs.len();

            Ok(format!(
                "validated {} benchmark-ready VCF pairs across tool-serving map, executable adapters, output declarations, and rendered commands",
                benchmark_ready_pairs.len()
            ))
        },
    );

    let report = build_vcf_adapters_ready_report(
        repo_root,
        &absolute_output_path,
        checks,
        benchmark_ready_pair_count,
        adapter_complete_pair_count,
        output_complete_pair_count,
        rendered_command_pair_count,
    );
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        bail!(
            "VCF adapter readiness gate failed for goals {}; inspect {}",
            report.failing_goal_ids.iter().map(u32::to_string).collect::<Vec<_>>().join(", "),
            report.output_path
        );
    }
    Ok(report)
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
    checks: &mut Vec<VcfAdaptersReadyGoalCheck>,
    goal_id: u32,
    surface: &str,
    output_path: Option<String>,
    run: F,
) where
    F: FnOnce() -> Result<String>,
{
    match run() {
        Ok(detail) => checks.push(VcfAdaptersReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(VcfAdaptersReadyGoalCheck {
            goal_id,
            surface: surface.to_string(),
            output_path,
            ok: false,
            detail: format!("{error:#}"),
        }),
    }
}

fn build_vcf_adapters_ready_report(
    repo_root: &Path,
    output_path: &Path,
    checks: Vec<VcfAdaptersReadyGoalCheck>,
    benchmark_ready_pair_count: usize,
    adapter_complete_pair_count: usize,
    output_complete_pair_count: usize,
    rendered_command_pair_count: usize,
) -> VcfAdaptersReadyReport {
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let checked_goal_count = checks.len();
    let passed_goal_count = checked_goal_count.saturating_sub(failed_goal_count);

    VcfAdaptersReadyReport {
        schema_version: VCF_ADAPTERS_READY_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        checked_goal_count,
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        benchmark_ready_pair_count,
        adapter_complete_pair_count,
        output_complete_pair_count,
        rendered_command_pair_count,
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
        build_vcf_adapters_ready_report, VcfAdaptersReadyGoalCheck,
        DEFAULT_VCF_ADAPTERS_READY_PATH, VCF_ADAPTERS_READY_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_adapters_ready_report_marks_failed_goal_ids_and_pair_counts() {
        let root = repo_root();
        let checks = vec![
            VcfAdaptersReadyGoalCheck {
                goal_id: 231,
                surface: "vcf tool-serving map".to_string(),
                output_path: Some("benchmarks/readiness/vcf-tool-serving-map.tsv".to_string()),
                ok: true,
                detail: "ok".to_string(),
            },
            VcfAdaptersReadyGoalCheck {
                goal_id: 245,
                surface: "vcf adapter benchmark-ready pair completeness".to_string(),
                output_path: Some(DEFAULT_VCF_ADAPTERS_READY_PATH.to_string()),
                ok: false,
                detail: "pair mismatch".to_string(),
            },
        ];

        let report = build_vcf_adapters_ready_report(
            &root,
            &root.join(DEFAULT_VCF_ADAPTERS_READY_PATH),
            checks,
            15,
            15,
            15,
            15,
        );

        assert_eq!(report.schema_version, VCF_ADAPTERS_READY_SCHEMA_VERSION);
        assert_eq!(report.output_path, DEFAULT_VCF_ADAPTERS_READY_PATH);
        assert_eq!(report.checked_goal_count, 2);
        assert_eq!(report.passed_goal_count, 1);
        assert_eq!(report.failed_goal_count, 1);
        assert_eq!(report.failing_goal_ids, vec![245]);
        assert_eq!(report.benchmark_ready_pair_count, 15);
        assert_eq!(report.adapter_complete_pair_count, 15);
        assert_eq!(report.output_complete_pair_count, 15);
        assert_eq!(report.rendered_command_pair_count, 15);
        assert!(!report.ok);
    }
}
