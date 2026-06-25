use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_expected_benchmark_results::{
    render_all_domain_expected_benchmark_results, AllDomainExpectedBenchmarkResultsReport,
    DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH,
};
use super::all_domain_parser_collector::{
    render_all_domain_parser_collector, AllDomainParserCollectorReport,
    AllDomainParserCollectorSourceKind, DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH,
};
use super::all_domain_rendered_commands::{
    render_all_domain_commands, AllDomainRenderedCommandsReport,
    DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use super::all_domain_stage_tool_table::{
    render_all_domain_stage_tool_table, AllDomainStageToolTableReport,
    DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH,
};
use super::full_benchmark_report::{
    render_full_benchmark_report, FullBenchmarkReport, DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH,
};
use crate::commands::benchmark::local_real_smoke_core_subset::{
    render_real_smoke_core_subset, RealSmokeCoreSubsetReport, RealSmokeCoreSubsetRow,
    DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH,
};
use crate::commands::benchmark::local_stage_inventory::{
    render_all_domain_stage_inventory, BenchLocalDomain, DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH: &str =
    "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.md";
pub(crate) const DEFAULT_FULL_BENCHMARK_DASHBOARD_JSON_PATH: &str =
    "benchmarks/readiness/all-domains/FASTQ_BAM_VCF_BENCHMARK_DASHBOARD.json";
const FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.full_benchmark_dashboard.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct FullBenchmarkDashboardMetric {
    pub(crate) metric_id: String,
    pub(crate) count: usize,
    pub(crate) source_path: String,
    pub(crate) source_field: String,
    pub(crate) detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FullBenchmarkDashboardReport {
    pub(crate) schema_version: &'static str,
    pub(crate) markdown_output_path: String,
    pub(crate) json_output_path: String,
    pub(crate) total_stages: usize,
    pub(crate) total_tools: usize,
    pub(crate) total_expected_jobs: usize,
    pub(crate) ready_jobs: usize,
    pub(crate) blocked_jobs: usize,
    pub(crate) missing_parsers: usize,
    pub(crate) missing_adapters: usize,
    pub(crate) missing_assets: usize,
    pub(crate) failed_real_smokes: usize,
    pub(crate) explicit_unsupported_pairs: usize,
    pub(crate) passes_behavior_test: bool,
    pub(crate) metrics: Vec<FullBenchmarkDashboardMetric>,
}

pub(crate) fn run_render_full_benchmark_dashboard(
    args: &parse::BenchReadinessRenderFullBenchmarkDashboardArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_full_benchmark_dashboard(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.markdown_output_path);
    }
    Ok(())
}

pub(crate) fn render_full_benchmark_dashboard(
    repo_root: &Path,
    markdown_output_path: PathBuf,
) -> Result<FullBenchmarkDashboardReport> {
    let markdown_output_path = repo_relative_path(repo_root, &markdown_output_path);
    let json_output_path = derive_json_output_path(&markdown_output_path);

    let stage_inventory = render_all_domain_stage_inventory(
        repo_root,
        &[BenchLocalDomain::Fastq, BenchLocalDomain::Bam, BenchLocalDomain::Vcf],
        PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH),
    )?;
    let stage_tool_table = render_all_domain_stage_tool_table(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_STAGE_TOOL_TABLE_PATH),
    )?;
    let expected_results = render_all_domain_expected_benchmark_results(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_EXPECTED_BENCHMARK_RESULTS_PATH),
    )?;
    let rendered_commands = render_all_domain_commands(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
    )?;
    let parser_collector = render_all_domain_parser_collector(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_PARSER_COLLECTOR_PATH),
    )?;
    let full_report = render_full_benchmark_report(
        repo_root,
        PathBuf::from(DEFAULT_FULL_BENCHMARK_REPORT_MARKDOWN_PATH),
    )?;
    let real_smoke = render_real_smoke_core_subset(
        repo_root,
        PathBuf::from(DEFAULT_REAL_SMOKE_CORE_SUBSET_PATH),
    )?;

    let total_stages = stage_inventory.total_stage_count;
    let total_tools = expected_results.tool_count;
    let total_expected_jobs = expected_results.row_count;
    let ready_jobs = full_report.present_row_count;
    let blocked_jobs = full_report.missing_result_row_count;
    let missing_parsers = count_missing_parsers(&expected_results, &parser_collector);
    let missing_adapters = count_missing_adapters(&expected_results, &rendered_commands);
    let missing_assets = count_missing_assets(&expected_results, &stage_tool_table);
    let failed_real_smokes = count_failed_real_smokes(&real_smoke);
    let explicit_unsupported_pairs = full_report.unsupported_pair_row_count;

    let metrics = vec![
        FullBenchmarkDashboardMetric {
            metric_id: "total_stages".to_string(),
            count: total_stages,
            source_path: stage_inventory.output_path.clone(),
            source_field: "total_stage_count".to_string(),
            detail: "governed all-domain local stage inventory".to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "total_tools".to_string(),
            count: total_tools,
            source_path: expected_results.output_path.clone(),
            source_field: "tool_count".to_string(),
            detail: "unique tools across canonical all-domain expected benchmark jobs".to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "total_expected_jobs".to_string(),
            count: total_expected_jobs,
            source_path: expected_results.output_path.clone(),
            source_field: "row_count".to_string(),
            detail: "canonical FASTQ, BAM, and VCF expected benchmark bindings".to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "ready_jobs".to_string(),
            count: ready_jobs,
            source_path: full_report.json_output_path.clone(),
            source_field: "present_row_count".to_string(),
            detail: "expected benchmark jobs with present governed result rows".to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "blocked_jobs".to_string(),
            count: blocked_jobs,
            source_path: full_report.json_output_path.clone(),
            source_field: "missing_result_row_count".to_string(),
            detail: "expected benchmark jobs still visible as missing_result rows".to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "missing_parsers".to_string(),
            count: missing_parsers,
            source_path: parser_collector.output_path.clone(),
            source_field: "expected_result_ids - fake_run_result_ids".to_string(),
            detail: "canonical result ids without governed fake-run parser evidence".to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "missing_adapters".to_string(),
            count: missing_adapters,
            source_path: rendered_commands.output_path.clone(),
            source_field: "expected_result_ids - rendered_command_result_ids".to_string(),
            detail: "canonical result ids without governed rendered command coverage".to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "missing_assets".to_string(),
            count: missing_assets,
            source_path: stage_tool_table.output_path.clone(),
            source_field: "expected_bindings - benchmark_ready_asset_bindings".to_string(),
            detail: "canonical benchmark bindings without assigned asset-profile coverage"
                .to_string(),
        },
        FullBenchmarkDashboardMetric {
            metric_id: "failed_real_smokes".to_string(),
            count: failed_real_smokes,
            source_path: real_smoke.output_path.clone(),
            source_field: "real_smoke_rows failing success contract".to_string(),
            detail: "governed real-smoke executions that do not satisfy their success contract"
                .to_string(),
        },
    ];

    let report = FullBenchmarkDashboardReport {
        schema_version: FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION,
        markdown_output_path: path_relative_to_repo(repo_root, &markdown_output_path),
        json_output_path: path_relative_to_repo(repo_root, &json_output_path),
        total_stages,
        total_tools,
        total_expected_jobs,
        ready_jobs,
        blocked_jobs,
        missing_parsers,
        missing_adapters,
        missing_assets,
        failed_real_smokes,
        explicit_unsupported_pairs,
        passes_behavior_test: false,
        metrics,
    };
    let report = ensure_full_benchmark_dashboard_contract(
        report,
        &stage_inventory.output_path,
        &expected_results,
        &stage_tool_table,
        &rendered_commands,
        &parser_collector,
        &full_report,
        &real_smoke,
    )?;

    if let Some(parent) = markdown_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&markdown_output_path, render_full_benchmark_dashboard_markdown(&report))
        .with_context(|| format!("write {}", markdown_output_path.display()))?;
    bijux_dna_infra::atomic_write_json(&json_output_path, &report)?;
    Ok(report)
}

fn count_missing_parsers(
    expected_results: &AllDomainExpectedBenchmarkResultsReport,
    parser_collector: &AllDomainParserCollectorReport,
) -> usize {
    let expected_result_ids =
        expected_results.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    let fake_run_result_ids = parser_collector
        .rows
        .iter()
        .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::FakeRun)
        .filter_map(|row| row.result_id.as_deref())
        .collect::<BTreeSet<_>>();
    expected_result_ids.difference(&fake_run_result_ids).count()
}

fn count_missing_adapters(
    expected_results: &AllDomainExpectedBenchmarkResultsReport,
    rendered_commands: &AllDomainRenderedCommandsReport,
) -> usize {
    let expected_result_ids =
        expected_results.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    let rendered_result_ids =
        rendered_commands.rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>();
    expected_result_ids.difference(&rendered_result_ids).count()
}

fn count_missing_assets(
    expected_results: &AllDomainExpectedBenchmarkResultsReport,
    stage_tool_table: &AllDomainStageToolTableReport,
) -> usize {
    let expected_bindings = expected_results
        .rows
        .iter()
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    let benchmark_ready_asset_bindings = stage_tool_table
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .filter(|row| !row.asset_profile_id.trim().is_empty())
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    expected_bindings.difference(&benchmark_ready_asset_bindings).count()
}

fn count_failed_real_smokes(report: &RealSmokeCoreSubsetReport) -> usize {
    report.rows.iter().filter(|row| !real_smoke_row_passes(row)).count()
}

fn real_smoke_row_passes(row: &RealSmokeCoreSubsetRow) -> bool {
    if row.normalized_metric_count == 0 {
        return false;
    }
    match row.stage_result_manifest_path.as_ref() {
        Some(_) => {
            row.manifest_status.as_deref() == Some("succeeded") && row.manifest_exit_code == Some(0)
        }
        None => true,
    }
}

fn ensure_full_benchmark_dashboard_contract(
    mut report: FullBenchmarkDashboardReport,
    stage_inventory_output_path: &str,
    expected_results: &AllDomainExpectedBenchmarkResultsReport,
    stage_tool_table: &AllDomainStageToolTableReport,
    rendered_commands: &AllDomainRenderedCommandsReport,
    parser_collector: &AllDomainParserCollectorReport,
    full_report: &FullBenchmarkReport,
    real_smoke: &RealSmokeCoreSubsetReport,
) -> Result<FullBenchmarkDashboardReport> {
    if report.metrics.len() != 9 {
        return Err(anyhow!(
            "full benchmark dashboard must keep exactly 9 governed summary metrics"
        ));
    }
    if report.markdown_output_path != DEFAULT_FULL_BENCHMARK_DASHBOARD_MARKDOWN_PATH
        || report.json_output_path != DEFAULT_FULL_BENCHMARK_DASHBOARD_JSON_PATH
    {
        return Err(anyhow!(
            "full benchmark dashboard output paths drifted from the governed locations"
        ));
    }
    if report.total_stages != 71
        || stage_inventory_output_path != DEFAULT_ALL_DOMAIN_STAGE_LIST_PATH
    {
        return Err(anyhow!(
            "full benchmark dashboard total stages drifted from the governed all-domain stage inventory"
        ));
    }
    if report.total_tools != expected_results.tool_count || report.total_tools != 71 {
        return Err(anyhow!(
            "full benchmark dashboard total tools drifted from the governed expected-result slice"
        ));
    }
    if report.total_tools != full_report.tool_centric_row_count {
        return Err(anyhow!(
            "full benchmark dashboard total tools must equal full benchmark report tool rows"
        ));
    }
    if report.total_expected_jobs != expected_results.row_count {
        return Err(anyhow!(
            "full benchmark dashboard total expected jobs drifted from the governed expected-result slice"
        ));
    }
    if report.ready_jobs != full_report.present_row_count {
        return Err(anyhow!(
            "full benchmark dashboard ready jobs drifted from the governed full benchmark report"
        ));
    }
    if report.blocked_jobs != full_report.missing_result_row_count {
        return Err(anyhow!(
            "full benchmark dashboard blocked jobs drifted from the governed missing-result count"
        ));
    }
    if report.total_expected_jobs != report.ready_jobs + report.blocked_jobs {
        return Err(anyhow!(
            "full benchmark dashboard expected jobs must equal ready plus blocked expected jobs"
        ));
    }

    let expected_bindings = expected_results
        .rows
        .iter()
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    let parser_bindings = parser_collector
        .rows
        .iter()
        .filter(|row| row.source_kind == AllDomainParserCollectorSourceKind::FakeRun)
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    if report.missing_parsers != expected_bindings.difference(&parser_bindings).count() {
        return Err(anyhow!(
            "full benchmark dashboard missing parser count drifted from parser collector coverage"
        ));
    }

    let rendered_bindings = rendered_commands
        .rows
        .iter()
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    if report.missing_adapters != expected_bindings.difference(&rendered_bindings).count() {
        return Err(anyhow!(
            "full benchmark dashboard missing adapter count drifted from rendered command coverage"
        ));
    }

    let asset_bindings = stage_tool_table
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .filter(|row| !row.asset_profile_id.trim().is_empty())
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    if report.missing_assets != expected_bindings.difference(&asset_bindings).count() {
        return Err(anyhow!(
            "full benchmark dashboard missing asset count drifted from stage-tool asset coverage"
        ));
    }

    let failed_real_smokes = count_failed_real_smokes(real_smoke);
    if report.failed_real_smokes != failed_real_smokes {
        return Err(anyhow!(
            "full benchmark dashboard failed real-smoke count drifted from the governed real-smoke subset"
        ));
    }
    if !full_report.passes_behavior_test || !real_smoke.passes_behavior_test {
        return Err(anyhow!(
            "full benchmark dashboard requires green full benchmark report and real-smoke subset behavior checks"
        ));
    }
    if report.explicit_unsupported_pairs != full_report.unsupported_pair_row_count
        || report.explicit_unsupported_pairs != 1
    {
        return Err(anyhow!(
            "full benchmark dashboard unsupported pair count drifted from the governed full benchmark report"
        ));
    }
    report.passes_behavior_test = true;
    Ok(report)
}

fn render_full_benchmark_dashboard_markdown(report: &FullBenchmarkDashboardReport) -> String {
    let mut markdown = String::new();
    markdown.push_str("# Full Benchmark Dashboard\n\n");
    markdown.push_str(
        "This dashboard is generated from governed machine-readable FASTQ, BAM, and VCF readiness outputs.\n\n",
    );
    markdown.push_str("| metric | count | source path | source field | detail |\n");
    markdown.push_str("| --- | ---: | --- | --- | --- |\n");
    for metric in &report.metrics {
        markdown.push_str(&format!(
            "| {} | {} | `{}` | `{}` | {} |\n",
            metric.metric_id, metric.count, metric.source_path, metric.source_field, metric.detail
        ));
    }
    markdown.push('\n');
    markdown.push_str(&format!(
        "Unsupported pairs tracked outside the expected-job slice: {}.\n",
        report.explicit_unsupported_pairs
    ));
    markdown
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
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

fn derive_json_output_path(markdown_output_path: &Path) -> PathBuf {
    markdown_output_path.with_extension("json")
}

#[cfg(test)]
mod tests {
    use super::{count_failed_real_smokes, real_smoke_row_passes};
    use crate::commands::benchmark::local_real_smoke_core_subset::{
        RealSmokeCoreSubsetExecutionKind, RealSmokeCoreSubsetReport, RealSmokeCoreSubsetRow,
    };
    use std::collections::BTreeMap;

    fn smoke_row(
        execution_id: &str,
        manifest_status: Option<&str>,
        manifest_exit_code: Option<i32>,
        normalized_metric_count: usize,
    ) -> RealSmokeCoreSubsetRow {
        RealSmokeCoreSubsetRow {
            execution_id: execution_id.to_string(),
            execution_kind: RealSmokeCoreSubsetExecutionKind::Stage,
            domain: "vcf".to_string(),
            bridge_source_domain: None,
            bridge_target_domain: None,
            stage_id: "vcf.stats".to_string(),
            tool_id: "bcftools".to_string(),
            corpus_id: "vcf_production_regression".to_string(),
            asset_profile_id: "vcf_cohort".to_string(),
            evidence_path: "artifacts/test.json".to_string(),
            parsed_schema_version: "schema".to_string(),
            stage_result_manifest_path: manifest_status
                .map(|_| "artifacts/stage-result.json".to_string()),
            manifest_status: manifest_status.map(str::to_string),
            manifest_exit_code,
            normalized_metric_count,
            normalized_metrics: BTreeMap::new(),
        }
    }

    #[test]
    fn real_smoke_row_requires_success_for_manifest_backed_rows() {
        assert!(real_smoke_row_passes(&smoke_row("ok", Some("succeeded"), Some(0), 1)));
        assert!(!real_smoke_row_passes(&smoke_row("failed", Some("failed"), Some(1), 1)));
        assert!(!real_smoke_row_passes(&smoke_row("empty", Some("succeeded"), Some(0), 0)));
    }

    #[test]
    fn failed_real_smoke_count_tracks_non_passing_rows() {
        let report = RealSmokeCoreSubsetReport {
            schema_version: "schema",
            output_path: "artifacts/test.json".to_string(),
            execution_count: 2,
            stage_execution_count: 2,
            pipeline_bridge_count: 0,
            domain_counts: BTreeMap::new(),
            passes_behavior_test: false,
            rows: vec![
                smoke_row("ok", Some("succeeded"), Some(0), 1),
                smoke_row("failed", Some("failed"), Some(1), 1),
            ],
        };
        assert_eq!(count_failed_real_smokes(&report), 1);
    }
}
