use std::collections::BTreeSet;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::vcf_active_stage_tool_matrix;
use super::vcf_admixture_ready;
use super::vcf_call_diploid_ready;
use super::vcf_call_gl_ready;
use super::vcf_call_pseudohaploid_ready;
use super::vcf_call_ready;
use super::vcf_damage_filter_ready;
use super::vcf_descent_family_adapter;
use super::vcf_expected_benchmark_results;
use super::vcf_filter_ready;
use super::vcf_gl_propagation_ready;
use super::vcf_imputation_family_adapter;
use super::vcf_imputation_metrics_ready;
use super::vcf_local_container_smoke;
use super::vcf_parser_fixture_coverage;
use super::vcf_pca_ready;
use super::vcf_phasing_family_adapter;
use super::vcf_population_structure_ready;
use super::vcf_prepare_reference_panel_ready;
use super::vcf_qc_ready;
use super::vcf_rendered_command_rows;
use super::vcf_rendered_commands;
use super::vcf_report_map;
use super::vcf_stats_ready;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_ALL_RETAINED_TOOLS_COMPLETE_PATH: &str =
    "benchmarks/readiness/vcf/VCF_ALL_RETAINED_TOOLS_COMPLETE.json";
const VCF_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_all_retained_tools_complete.v1";
const EXPECTED_CHECKED_GOAL_COUNT: usize = 24;
const EXPECTED_RETAINED_ROW_COUNT: usize = 44;
const EXPECTED_RETAINED_STAGE_COUNT: usize = 20;
const EXPECTED_RETAINED_TOOL_COUNT: usize = 17;
const EXPECTED_ACTIVE_ROW_COUNT: usize = 20;
const EXPECTED_REMOVED_ROW_COUNT: usize = 24;
const EXPECTED_ACTIVE_STAGE_COUNT: usize = 17;
const EXPECTED_ACTIVE_TOOL_COUNT: usize = 6;
const EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT: usize = 19;
const EXPECTED_CONTAINER_SMOKE_ROW_COUNT: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct VcfBindingKey {
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAllRetainedToolsCompleteGoalCheck {
    pub(crate) goal_id: u32,
    pub(crate) surface: String,
    pub(crate) output_path: Option<String>,
    pub(crate) ok: bool,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAllRetainedToolsCompleteReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) checked_goal_count: usize,
    pub(crate) passed_goal_count: usize,
    pub(crate) failed_goal_count: usize,
    pub(crate) failing_goal_ids: Vec<u32>,
    pub(crate) retained_row_count: usize,
    pub(crate) retained_stage_count: usize,
    pub(crate) retained_tool_count: usize,
    pub(crate) active_row_count: usize,
    pub(crate) removed_row_count: usize,
    pub(crate) active_stage_count: usize,
    pub(crate) active_tool_count: usize,
    pub(crate) expected_result_row_count: usize,
    pub(crate) rendered_command_row_count: usize,
    pub(crate) parser_fixture_row_count: usize,
    pub(crate) local_smoke_row_count: usize,
    pub(crate) local_smoke_host_stage_row_count: usize,
    pub(crate) local_smoke_container_row_count: usize,
    pub(crate) report_map_row_count: usize,
    pub(crate) ok: bool,
    pub(crate) checks: Vec<VcfAllRetainedToolsCompleteGoalCheck>,
}

pub(crate) fn run_render_vcf_all_retained_tools_complete(
    args: &parse::BenchReadinessRenderVcfAllRetainedToolsCompleteArgs,
) -> Result<()> {
    let repo_root = env::current_dir().context("resolve current directory")?;
    let report = render_vcf_all_retained_tools_complete(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ALL_RETAINED_TOOLS_COMPLETE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_all_retained_tools_complete(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAllRetainedToolsCompleteReport> {
    let _cwd_guard = CurrentDirGuard::change_to(repo_root);
    let absolute_output_path = repo_relative_path(repo_root, &output_path);
    if let Some(parent) = absolute_output_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut checks = Vec::new();

    let mut retained_row_count = 0usize;
    let mut retained_stage_count = 0usize;
    let mut retained_tool_count = 0usize;
    let mut active_row_count = 0usize;
    let mut removed_row_count = 0usize;
    let mut active_stage_count = 0usize;
    let mut active_tool_count = 0usize;
    let mut expected_result_row_count = 0usize;
    let mut rendered_command_row_count = 0usize;
    let mut parser_fixture_row_count = 0usize;
    let mut local_smoke_row_count = 0usize;
    let mut local_smoke_host_stage_row_count = 0usize;
    let mut local_smoke_container_row_count = 0usize;
    let mut report_map_row_count = 0usize;
    let mut retained_binding_keys = BTreeSet::<VcfBindingKey>::new();
    let mut active_binding_keys = BTreeSet::<VcfBindingKey>::new();

    record_goal_check(
        &mut checks,
        336,
        "vcf active stage tool matrix",
        Some(vcf_active_stage_tool_matrix::DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH.to_string()),
        || {
            let report = vcf_active_stage_tool_matrix::render_vcf_active_stage_tool_matrix(
                repo_root,
                PathBuf::from(
                    vcf_active_stage_tool_matrix::DEFAULT_VCF_ACTIVE_STAGE_TOOL_MATRIX_PATH,
                ),
            )?;
            if report.row_count != EXPECTED_RETAINED_ROW_COUNT
                || report.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || report.tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || report.active_row_count != EXPECTED_ACTIVE_ROW_COUNT
                || report.complete_row_count != 0
                || report.removed_row_count != EXPECTED_REMOVED_ROW_COUNT
            {
                bail!(
                    "VCF retained matrix drifted: rows={}, stages={}, tools={}, active={}, complete={}, removed={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.active_row_count,
                    report.complete_row_count,
                    report.removed_row_count
                );
            }

            retained_binding_keys =
                report.rows.iter().map(binding_key_from_matrix_row).collect::<BTreeSet<_>>();
            active_binding_keys = report
                .rows
                .iter()
                .filter(|row| row.scope_state == "active")
                .map(binding_key_from_matrix_row)
                .collect::<BTreeSet<_>>();
            retained_row_count = report.row_count;
            retained_stage_count = report.stage_count;
            retained_tool_count = report.tool_count;
            active_row_count = report.active_row_count;
            removed_row_count = report.removed_row_count;
            active_stage_count = report
                .rows
                .iter()
                .filter(|row| row.scope_state == "active")
                .map(|row| row.stage_id.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            active_tool_count = report
                .rows
                .iter()
                .filter(|row| row.scope_state == "active")
                .map(|row| row.tool_id.as_str())
                .collect::<BTreeSet<_>>()
                .len();
            if active_stage_count != EXPECTED_ACTIVE_STAGE_COUNT
                || active_tool_count != EXPECTED_ACTIVE_TOOL_COUNT
            {
                bail!(
                    "VCF active retained matrix drifted: active_stages={active_stage_count}, active_tools={active_tool_count}"
                );
            }
            Ok(
                "validated the governed 44-row retained VCF matrix with 20 active bindings across 20 stages"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        337,
        "vcf.call readiness",
        Some(vcf_call_ready::DEFAULT_VCF_CALL_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_call_ready::DEFAULT_VCF_CALL_READY_PATH,
                "vcf.call readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.call` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        338,
        "vcf.call_diploid readiness",
        Some(vcf_call_diploid_ready::DEFAULT_VCF_CALL_DIPLOID_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_call_diploid_ready::DEFAULT_VCF_CALL_DIPLOID_READY_PATH,
                "vcf.call_diploid readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.call_diploid` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        339,
        "vcf.call_pseudohaploid readiness",
        Some(vcf_call_pseudohaploid_ready::DEFAULT_VCF_CALL_PSEUDOHAPLOID_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_call_pseudohaploid_ready::DEFAULT_VCF_CALL_PSEUDOHAPLOID_READY_PATH,
                "vcf.call_pseudohaploid readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.call_pseudohaploid` row end-to-end"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        340,
        "vcf.call_gl readiness",
        Some(vcf_call_gl_ready::DEFAULT_VCF_CALL_GL_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_call_gl_ready::DEFAULT_VCF_CALL_GL_READY_PATH,
                "vcf.call_gl readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.call_gl` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        341,
        "vcf.damage_filter readiness",
        Some(vcf_damage_filter_ready::DEFAULT_VCF_DAMAGE_FILTER_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_damage_filter_ready::DEFAULT_VCF_DAMAGE_FILTER_READY_PATH,
                "vcf.damage_filter readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.damage_filter` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        342,
        "vcf.gl_propagation readiness",
        Some(vcf_gl_propagation_ready::DEFAULT_VCF_GL_PROPAGATION_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_gl_propagation_ready::DEFAULT_VCF_GL_PROPAGATION_READY_PATH,
                "vcf.gl_propagation readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.gl_propagation` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        343,
        "vcf.filter readiness",
        Some(vcf_filter_ready::DEFAULT_VCF_FILTER_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_filter_ready::DEFAULT_VCF_FILTER_READY_PATH,
                "vcf.filter readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.filter` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        344,
        "vcf.stats readiness",
        Some(vcf_stats_ready::DEFAULT_VCF_STATS_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_stats_ready::DEFAULT_VCF_STATS_READY_PATH,
                "vcf.stats readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.stats` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        345,
        "vcf.qc readiness",
        Some(vcf_qc_ready::DEFAULT_VCF_QC_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_qc_ready::DEFAULT_VCF_QC_READY_PATH,
                "vcf.qc readiness",
                3,
            )?;
            Ok("validated the governed `vcf.qc` rows for bcftools, plink, and plink2".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        346,
        "vcf.prepare_reference_panel readiness",
        Some(
            vcf_prepare_reference_panel_ready::DEFAULT_VCF_PREPARE_REFERENCE_PANEL_READY_PATH
                .to_string(),
        ),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_prepare_reference_panel_ready::DEFAULT_VCF_PREPARE_REFERENCE_PANEL_READY_PATH,
                "vcf.prepare_reference_panel readiness",
                1,
            )?;
            Ok("validated the governed bcftools `vcf.prepare_reference_panel` row end-to-end"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        347,
        "vcf.phasing retained family coverage",
        Some(vcf_phasing_family_adapter::DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH.to_string()),
        || {
            ensure_single_phasing_output(
                &read_governed_json(
                    repo_root,
                    vcf_phasing_family_adapter::DEFAULT_VCF_SHAPEIT5_ADAPTER_PATH,
                )?,
                "shapeit5",
                "benchmark_ready",
                true,
                "shapeit5 phasing adapter",
            )?;
            ensure_single_phasing_output(
                &read_governed_json(
                    repo_root,
                    vcf_phasing_family_adapter::DEFAULT_VCF_BEAGLE_ADAPTER_PATH,
                )?,
                "beagle",
                "not_benchmark_ready",
                false,
                "beagle phasing adapter",
            )?;
            ensure_single_phasing_output(
                &read_governed_json(
                    repo_root,
                    vcf_phasing_family_adapter::DEFAULT_VCF_EAGLE_ADAPTER_PATH,
                )?,
                "eagle",
                "not_benchmark_ready",
                false,
                "eagle phasing adapter",
            )?;
            Ok(
                "validated the retained phasing family with shapeit5 active and beagle/eagle governed as non-benchmark-ready alternatives"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        348,
        "vcf.impute retained family coverage",
        Some(vcf_imputation_family_adapter::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH.to_string()),
        || {
            let payload = read_governed_json(
                repo_root,
                vcf_imputation_family_adapter::DEFAULT_VCF_IMPUTATION_FAMILY_ADAPTER_PATH,
            )?;
            if json_u64(&payload, "row_count") != Some(8)
                || json_u64(&payload, "tool_count") != Some(4)
                || json_u64(&payload, "benchmark_ready_row_count") != Some(2)
                || json_u64(&payload, "parser_output_row_count") != Some(8)
                || json_u64(&payload, "missing_input_test_passed_row_count") != Some(8)
            {
                bail!(
                    "VCF imputation-family coverage drifted: rows={}, tools={}, benchmark_ready={}, parser_outputs={}, missing_input_passed={}",
                    json_u64(&payload, "row_count").unwrap_or_default(),
                    json_u64(&payload, "tool_count").unwrap_or_default(),
                    json_u64(&payload, "benchmark_ready_row_count").unwrap_or_default(),
                    json_u64(&payload, "parser_output_row_count").unwrap_or_default(),
                    json_u64(&payload, "missing_input_test_passed_row_count").unwrap_or_default()
                );
            }
            ensure_imputation_family_output(&payload, "beagle", "vcf.impute", "benchmark_ready")?;
            ensure_imputation_family_output(
                &payload,
                "glimpse",
                "vcf.impute",
                "not_benchmark_ready",
            )?;
            ensure_imputation_family_output(
                &payload,
                "impute5",
                "vcf.impute",
                "not_benchmark_ready",
            )?;
            ensure_imputation_family_output(
                &payload,
                "minimac4",
                "vcf.impute",
                "not_benchmark_ready",
            )?;
            Ok(
                "validated the retained imputation family with the governed beagle `vcf.impute` row and explicit non-benchmark-ready alternatives"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        349,
        "vcf.imputation_metrics readiness",
        Some(vcf_imputation_metrics_ready::DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_imputation_metrics_ready::DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH,
                "vcf.imputation_metrics readiness",
                1,
            )?;
            Ok("validated the governed beagle `vcf.imputation_metrics` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        350,
        "vcf.pca readiness",
        Some(vcf_pca_ready::DEFAULT_VCF_PCA_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_pca_ready::DEFAULT_VCF_PCA_READY_PATH,
                "vcf.pca readiness",
                2,
            )?;
            Ok("validated the governed eigensoft and plink2 `vcf.pca` rows end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        351,
        "vcf.admixture readiness",
        Some(vcf_admixture_ready::DEFAULT_VCF_ADMIXTURE_READY_PATH.to_string()),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_admixture_ready::DEFAULT_VCF_ADMIXTURE_READY_PATH,
                "vcf.admixture readiness",
                1,
            )?;
            Ok("validated the governed plink2 `vcf.admixture` row end-to-end".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        352,
        "vcf.population_structure readiness",
        Some(
            vcf_population_structure_ready::DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH.to_string(),
        ),
        || {
            validate_direct_vcf_ready_output(
                repo_root,
                vcf_population_structure_ready::DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH,
                "vcf.population_structure readiness",
                1,
            )?;
            Ok("validated the governed plink2 `vcf.population_structure` row end-to-end"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        353,
        "vcf.roh retained family coverage",
        Some(vcf_descent_family_adapter::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || {
            let payload = read_governed_json(
                repo_root,
                vcf_descent_family_adapter::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
            )?;
            ensure_descent_family_output_contract(&payload)?;
            ensure_descent_family_output(&payload, "plink2", "vcf.roh", "benchmark_ready")?;
            Ok(
                "validated the governed plink2 `vcf.roh` retained-family row with benchmark-ready command coverage"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        354,
        "vcf.ibd retained family coverage",
        Some(vcf_descent_family_adapter::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || {
            let payload = read_governed_json(
                repo_root,
                vcf_descent_family_adapter::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
            )?;
            ensure_descent_family_output_contract(&payload)?;
            ensure_descent_family_output(&payload, "germline", "vcf.ibd", "benchmark_ready")?;
            ensure_descent_family_output(&payload, "ibdhap", "vcf.ibd", "not_benchmark_ready")?;
            ensure_descent_family_output(&payload, "ibdseq", "vcf.ibd", "not_benchmark_ready")?;
            Ok(
                "validated the retained IBD family with germline benchmark-ready and ibdhap/ibdseq explicitly governed as non-benchmark-ready"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        355,
        "vcf.demography retained family coverage",
        Some(vcf_descent_family_adapter::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH.to_string()),
        || {
            let payload = read_governed_json(
                repo_root,
                vcf_descent_family_adapter::DEFAULT_VCF_DESCENT_FAMILY_ADAPTER_PATH,
            )?;
            ensure_descent_family_output_contract(&payload)?;
            ensure_descent_family_output(&payload, "ibdne", "vcf.demography", "benchmark_ready")?;
            Ok(
                "validated the governed ibdne `vcf.demography` retained-family row with benchmark-ready command coverage"
                    .to_string(),
            )
        },
    );

    record_goal_check(
        &mut checks,
        356,
        "vcf local and container smoke coverage",
        Some(vcf_local_container_smoke::DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH.to_string()),
        || {
            let report = vcf_local_container_smoke::render_vcf_local_container_smoke(
                repo_root,
                PathBuf::from(vcf_local_container_smoke::DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH),
            )?;
            if report.row_count != EXPECTED_RETAINED_ROW_COUNT
                || report.stage_count != EXPECTED_RETAINED_STAGE_COUNT
                || report.tool_count != EXPECTED_RETAINED_TOOL_COUNT
                || report.host_stage_smoke_row_count != EXPECTED_HOST_STAGE_SMOKE_ROW_COUNT
                || report.container_smoke_row_count != EXPECTED_CONTAINER_SMOKE_ROW_COUNT
            {
                bail!(
                    "VCF local/container smoke drifted: rows={}, stages={}, tools={}, host_stage_smokes={}, container_smokes={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.host_stage_smoke_row_count,
                    report.container_smoke_row_count
                );
            }
            let observed_bindings =
                report.rows.iter().map(binding_key_from_local_smoke_row).collect::<BTreeSet<_>>();
            if observed_bindings != retained_binding_keys {
                bail!(
                    "VCF local/container smoke drifted from the retained binding slice: missing={:?} extra={:?}",
                    retained_binding_keys
                        .difference(&observed_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    observed_bindings
                        .difference(&retained_binding_keys)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            local_smoke_row_count = report.row_count;
            local_smoke_host_stage_row_count = report.host_stage_smoke_row_count;
            local_smoke_container_row_count = report.container_smoke_row_count;
            Ok("validated governed host-vs-container smoke coverage for every retained VCF row"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        357,
        "vcf parser fixture coverage",
        Some(vcf_parser_fixture_coverage::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH.to_string()),
        || {
            let report = vcf_parser_fixture_coverage::render_vcf_parser_fixture_coverage(
                repo_root,
                PathBuf::from(
                    vcf_parser_fixture_coverage::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH,
                ),
            )?;
            if report.row_count != EXPECTED_ACTIVE_ROW_COUNT
                || report.stage_count != EXPECTED_ACTIVE_STAGE_COUNT
                || report.tool_count != EXPECTED_ACTIVE_TOOL_COUNT
                || report.covered_row_count != EXPECTED_ACTIVE_ROW_COUNT
                || report.missing_row_count != 0
                || (report.parser_fixture_coverage_percent - 100.0).abs() > f64::EPSILON
            {
                bail!(
                    "VCF parser fixture coverage drifted: rows={}, stages={}, tools={}, covered={}, missing={}, percent={}",
                    report.row_count,
                    report.stage_count,
                    report.tool_count,
                    report.covered_row_count,
                    report.missing_row_count,
                    report.parser_fixture_coverage_percent
                );
            }
            let observed_bindings = report
                .rows
                .iter()
                .map(binding_key_from_parser_fixture_row)
                .collect::<BTreeSet<_>>();
            if observed_bindings != active_binding_keys {
                bail!(
                    "VCF parser fixture coverage drifted from the active binding slice: missing={:?} extra={:?}",
                    active_binding_keys
                        .difference(&observed_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    observed_bindings
                        .difference(&active_binding_keys)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            parser_fixture_row_count = report.row_count;
            Ok("validated 100% parser fixture coverage for the active VCF binding slice"
                .to_string())
        },
    );

    record_goal_check(
        &mut checks,
        358,
        "vcf rendered commands",
        Some(vcf_rendered_commands::DEFAULT_VCF_RENDERED_COMMANDS_PATH.to_string()),
        || {
            let report = vcf_rendered_commands::render_vcf_commands(
                repo_root,
                PathBuf::from(vcf_rendered_commands::DEFAULT_VCF_RENDERED_COMMANDS_PATH),
            )?;
            if report.row_count != EXPECTED_ACTIVE_ROW_COUNT {
                bail!(
                    "VCF rendered commands drifted from the active VCF binding slice: rows={}",
                    report.row_count
                );
            }
            let observed_bindings = report
                .rows
                .iter()
                .map(binding_key_from_rendered_command_row)
                .collect::<BTreeSet<_>>();
            if observed_bindings != active_binding_keys {
                bail!(
                    "VCF rendered command bindings drifted from the active slice: missing={:?} extra={:?}",
                    active_binding_keys
                        .difference(&observed_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    observed_bindings
                        .difference(&active_binding_keys)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            rendered_command_row_count = report.row_count;
            Ok("validated one rendered command bundle for every active VCF binding".to_string())
        },
    );

    record_goal_check(
        &mut checks,
        359,
        "vcf report map",
        Some(vcf_report_map::DEFAULT_VCF_REPORT_MAP_PATH.to_string()),
        || {
            let expected_results_report =
                vcf_expected_benchmark_results::render_vcf_expected_benchmark_results(
                    repo_root,
                    PathBuf::from(
                        vcf_expected_benchmark_results::DEFAULT_VCF_EXPECTED_BENCHMARK_RESULTS_PATH,
                    ),
                )?;
            let report = vcf_report_map::render_vcf_report_map(
                repo_root,
                PathBuf::from(vcf_report_map::DEFAULT_VCF_REPORT_MAP_PATH),
            )?;
            if expected_results_report.row_count != EXPECTED_ACTIVE_ROW_COUNT
                || report.row_count != EXPECTED_ACTIVE_ROW_COUNT
            {
                bail!(
                    "VCF expected-result/report-map counts drifted: expected_results={}, report_map={}",
                    expected_results_report.row_count,
                    report.row_count
                );
            }
            let expected_bindings = expected_results_report
                .rows
                .iter()
                .map(binding_key_from_expected_result_row)
                .collect::<BTreeSet<_>>();
            let report_bindings =
                report.rows.iter().map(binding_key_from_report_map_row).collect::<BTreeSet<_>>();
            if expected_bindings != active_binding_keys || report_bindings != active_binding_keys {
                bail!(
                    "VCF expected-result/report-map bindings drifted from the active slice: expected_missing={:?} expected_extra={:?} report_missing={:?} report_extra={:?}",
                    active_binding_keys
                        .difference(&expected_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    expected_bindings
                        .difference(&active_binding_keys)
                        .cloned()
                        .collect::<Vec<_>>(),
                    active_binding_keys
                        .difference(&report_bindings)
                        .cloned()
                        .collect::<Vec<_>>(),
                    report_bindings
                        .difference(&active_binding_keys)
                        .cloned()
                        .collect::<Vec<_>>()
                );
            }
            expected_result_row_count = expected_results_report.row_count;
            report_map_row_count = report.row_count;
            Ok("validated report-map placement for every active VCF expected-result row"
                .to_string())
        },
    );

    let passed_goal_count = checks.iter().filter(|check| check.ok).count();
    let failing_goal_ids =
        checks.iter().filter(|check| !check.ok).map(|check| check.goal_id).collect::<Vec<_>>();
    let failed_goal_count = failing_goal_ids.len();
    let report = VcfAllRetainedToolsCompleteReport {
        schema_version: VCF_ALL_RETAINED_TOOLS_COMPLETE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        checked_goal_count: checks.len(),
        passed_goal_count,
        failed_goal_count,
        failing_goal_ids,
        retained_row_count,
        retained_stage_count,
        retained_tool_count,
        active_row_count,
        removed_row_count,
        active_stage_count,
        active_tool_count,
        expected_result_row_count,
        rendered_command_row_count,
        parser_fixture_row_count,
        local_smoke_row_count,
        local_smoke_host_stage_row_count,
        local_smoke_container_row_count,
        report_map_row_count,
        ok: failed_goal_count == 0
            && checks.len() == EXPECTED_CHECKED_GOAL_COUNT
            && retained_row_count == EXPECTED_RETAINED_ROW_COUNT
            && active_row_count == EXPECTED_ACTIVE_ROW_COUNT
            && expected_result_row_count == EXPECTED_ACTIVE_ROW_COUNT
            && rendered_command_row_count == EXPECTED_ACTIVE_ROW_COUNT
            && parser_fixture_row_count == EXPECTED_ACTIVE_ROW_COUNT
            && report_map_row_count == EXPECTED_ACTIVE_ROW_COUNT
            && local_smoke_row_count == EXPECTED_RETAINED_ROW_COUNT,
        checks,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    if !report.ok {
        return Err(anyhow!(
            "every governed VCF retained-tool gate from goals 336 through 359 must remain complete"
        ));
    }
    Ok(report)
}

fn validate_direct_vcf_ready_output(
    repo_root: &Path,
    output_path: &str,
    label: &str,
    expected_rows: u64,
) -> Result<()> {
    let payload = read_governed_json(repo_root, output_path)?;
    if json_u64(&payload, "retained_row_count") != Some(expected_rows)
        || json_u64(&payload, "active_row_count") != Some(expected_rows)
        || json_u64(&payload, "complete_row_count") != Some(expected_rows)
        || json_u64(&payload, "incomplete_row_count") != Some(0)
        || json_u64(&payload, "checked_surface_count") != Some(8)
        || json_u64(&payload, "violation_count") != Some(0)
        || json_bool(&payload, "ok") != Some(true)
    {
        bail!(
            "{label} drifted from the governed complete pass state: retained={}, active={}, complete={}, incomplete={}, checked_surfaces={}, violations={}, ok={}",
            json_u64(&payload, "retained_row_count").unwrap_or_default(),
            json_u64(&payload, "active_row_count").unwrap_or_default(),
            json_u64(&payload, "complete_row_count").unwrap_or_default(),
            json_u64(&payload, "incomplete_row_count").unwrap_or_default(),
            json_u64(&payload, "checked_surface_count").unwrap_or_default(),
            json_u64(&payload, "violation_count").unwrap_or_default(),
            json_bool(&payload, "ok").unwrap_or(false)
        );
    }
    Ok(())
}

fn ensure_single_phasing_output(
    payload: &Value,
    tool_id: &str,
    expected_benchmark_status: &str,
    expect_benchmark_ready_row: bool,
    label: &str,
) -> Result<()> {
    let expected_benchmark_ready_row_count = u64::from(expect_benchmark_ready_row);
    if json_u64(payload, "row_count") != Some(1)
        || json_u64(payload, "benchmark_ready_row_count")
            != Some(expected_benchmark_ready_row_count)
        || json_u64(payload, "parser_output_row_count") != Some(1)
        || json_u64(payload, "indexed_row_count") != Some(1)
        || json_u64(payload, "missing_input_test_passed_row_count") != Some(1)
    {
        bail!(
            "{label} drifted: rows={}, benchmark_ready={}, parser_outputs={}, indexed={}, missing_input_passed={}",
            json_u64(payload, "row_count").unwrap_or_default(),
            json_u64(payload, "benchmark_ready_row_count").unwrap_or_default(),
            json_u64(payload, "parser_output_row_count").unwrap_or_default(),
            json_u64(payload, "indexed_row_count").unwrap_or_default(),
            json_u64(payload, "missing_input_test_passed_row_count").unwrap_or_default()
        );
    }
    let row = find_json_row(payload, tool_id, "vcf.phasing")?;
    if row_field_str(row, "benchmark_status") != Some(expected_benchmark_status)
        || row_field_bool(row, "argv_validation_passed") != Some(true)
        || row_field_bool(row, "missing_input_test_passed") != Some(true)
    {
        bail!(
            "{label} row drifted: tool=`{}`, stage=`{}`, benchmark_status=`{}`, argv_ok={}, missing_input_ok={}",
            row_field_str(row, "tool_id").unwrap_or(""),
            row_field_str(row, "stage_id").unwrap_or(""),
            row_field_str(row, "benchmark_status").unwrap_or(""),
            row_field_bool(row, "argv_validation_passed").unwrap_or(false),
            row_field_bool(row, "missing_input_test_passed").unwrap_or(false)
        );
    }
    Ok(())
}

fn ensure_imputation_family_output(
    payload: &Value,
    tool_id: &str,
    stage_id: &str,
    expected_benchmark_status: &str,
) -> Result<()> {
    let row = find_json_row(payload, tool_id, stage_id)?;
    if row_field_str(row, "benchmark_status") != Some(expected_benchmark_status)
        || row_field_bool(row, "argv_validation_passed") != Some(true)
        || row_field_bool(row, "missing_input_test_passed") != Some(true)
    {
        bail!(
            "VCF imputation-family row `{stage_id}` / `{tool_id}` drifted: benchmark_status=`{}`, argv_ok={}, missing_input_ok={}",
            row_field_str(row, "benchmark_status").unwrap_or(""),
            row_field_bool(row, "argv_validation_passed").unwrap_or(false),
            row_field_bool(row, "missing_input_test_passed").unwrap_or(false)
        );
    }
    Ok(())
}

fn ensure_descent_family_output_contract(payload: &Value) -> Result<()> {
    if json_u64(payload, "row_count") != Some(5)
        || json_u64(payload, "tool_count") != Some(5)
        || json_u64(payload, "benchmark_ready_row_count") != Some(3)
        || json_u64(payload, "parser_output_row_count") != Some(5)
        || json_u64(payload, "normalized_output_row_count") != Some(5)
        || json_u64(payload, "missing_input_test_passed_row_count") != Some(5)
    {
        bail!(
            "VCF descent-family coverage drifted: rows={}, tools={}, benchmark_ready={}, parser_outputs={}, normalized_outputs={}, missing_input_passed={}",
            json_u64(payload, "row_count").unwrap_or_default(),
            json_u64(payload, "tool_count").unwrap_or_default(),
            json_u64(payload, "benchmark_ready_row_count").unwrap_or_default(),
            json_u64(payload, "parser_output_row_count").unwrap_or_default(),
            json_u64(payload, "normalized_output_row_count").unwrap_or_default(),
            json_u64(payload, "missing_input_test_passed_row_count").unwrap_or_default()
        );
    }
    Ok(())
}

fn ensure_descent_family_output(
    payload: &Value,
    tool_id: &str,
    stage_id: &str,
    expected_benchmark_status: &str,
) -> Result<()> {
    let row = find_json_row(payload, tool_id, stage_id)?;
    if row_field_str(row, "benchmark_status") != Some(expected_benchmark_status)
        || row_field_bool(row, "argv_validation_passed") != Some(true)
        || row_field_bool(row, "missing_input_test_passed") != Some(true)
    {
        bail!(
            "VCF descent-family row `{stage_id}` / `{tool_id}` drifted: benchmark_status=`{}`, argv_ok={}, missing_input_ok={}",
            row_field_str(row, "benchmark_status").unwrap_or(""),
            row_field_bool(row, "argv_validation_passed").unwrap_or(false),
            row_field_bool(row, "missing_input_test_passed").unwrap_or(false)
        );
    }
    Ok(())
}

fn read_governed_json(repo_root: &Path, relative_path: &str) -> Result<Value> {
    let absolute_path = repo_root.join(relative_path);
    let payload = std::fs::read_to_string(&absolute_path)
        .with_context(|| format!("read {}", absolute_path.display()))?;
    serde_json::from_str(&payload)
        .with_context(|| format!("parse governed JSON {}", absolute_path.display()))
}

fn json_u64(payload: &Value, key: &str) -> Option<u64> {
    payload.get(key).and_then(Value::as_u64)
}

fn json_bool(payload: &Value, key: &str) -> Option<bool> {
    payload.get(key).and_then(Value::as_bool)
}

fn find_json_row<'a>(payload: &'a Value, tool_id: &str, stage_id: &str) -> Result<&'a Value> {
    payload
        .get("rows")
        .and_then(Value::as_array)
        .and_then(|rows| {
            rows.iter().find(|row| {
                row_field_str(row, "tool_id") == Some(tool_id)
                    && row_field_str(row, "stage_id") == Some(stage_id)
            })
        })
        .ok_or_else(|| anyhow!("governed report is missing `{stage_id}` / `{tool_id}`"))
}

fn row_field_str<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

fn row_field_bool(row: &Value, key: &str) -> Option<bool> {
    row.get(key).and_then(Value::as_bool)
}

fn binding_key_from_matrix_row(
    row: &vcf_active_stage_tool_matrix::VcfActiveStageToolMatrixRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_local_smoke_row(
    row: &vcf_local_container_smoke::VcfLocalContainerSmokeRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_parser_fixture_row(
    row: &vcf_parser_fixture_coverage::VcfParserFixtureCoverageRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_rendered_command_row(
    row: &vcf_rendered_command_rows::VcfRenderedCommandRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_expected_result_row(
    row: &vcf_expected_benchmark_results::VcfExpectedBenchmarkResultRow,
) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn binding_key_from_report_map_row(row: &vcf_report_map::VcfReportMapRow) -> VcfBindingKey {
    VcfBindingKey { stage_id: row.stage_id.clone(), tool_id: row.tool_id.clone() }
}

fn record_goal_check<F>(
    checks: &mut Vec<VcfAllRetainedToolsCompleteGoalCheck>,
    goal_id: u32,
    surface: impl Into<String>,
    output_path: Option<String>,
    check: F,
) where
    F: FnOnce() -> Result<String>,
{
    let surface = surface.into();
    match check() {
        Ok(detail) => checks.push(VcfAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: true,
            detail,
        }),
        Err(error) => checks.push(VcfAllRetainedToolsCompleteGoalCheck {
            goal_id,
            surface,
            output_path,
            ok: false,
            detail: error.to_string(),
        }),
    }
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> Self {
        let previous = env::current_dir().expect("current dir");
        env::set_current_dir(path).expect("set current dir");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        env::set_current_dir(&self.previous).expect("restore current dir");
    }
}
