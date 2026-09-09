//! Owner: bijux-dna-bench
//! Pure readiness evaluation over explicitly acquired benchmark reports.

use bijux_dna_bench_model::{
    AllDomainExpectedBenchmarkResultsReport, AllDomainParserCollectorReport,
    AllDomainParserCollectorSourceKind, AllDomainRenderedCommandsReport,
    AllDomainStageToolTableReport, BenchError, BenchmarkLocalAllDomainStageInventory,
    FullBenchmarkDashboardMetric, FullBenchmarkDashboardReport, FullBenchmarkReport,
    RealSmokeCoreSubsetReport, RealSmokeCoreSubsetRow, FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION,
};
use std::collections::BTreeSet;

/// Governed repository-relative locations supplied by the request adapter.
#[derive(Debug, Clone, Copy)]
pub struct FullBenchmarkDashboardPaths<'a> {
    pub markdown: &'a str,
    pub json: &'a str,
    pub stage_inventory: &'a str,
}

/// Reports acquired from one unchanged repository and artifact view.
/// This context borrows evidence; evaluation performs no IO or implicit acquisition.
#[derive(Debug, Clone, Copy)]
pub struct FullBenchmarkDashboardEvidence<'a> {
    pub stage_inventory: &'a BenchmarkLocalAllDomainStageInventory,
    pub stage_tool_table: &'a AllDomainStageToolTableReport,
    pub expected_results: &'a AllDomainExpectedBenchmarkResultsReport,
    pub rendered_commands: &'a AllDomainRenderedCommandsReport,
    pub parser_collector: &'a AllDomainParserCollectorReport,
    pub full_report: &'a FullBenchmarkReport,
    pub real_smoke: &'a RealSmokeCoreSubsetReport,
}

/// Evaluates coverage and consistency across the acquired readiness reports.
///
/// Paths must be normalized repository-relative strings from the caller's path adapter.
/// Rendered commands and parser coverage describe readiness, not backend execution provenance.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] when governed report relationships or paths drift.
pub fn evaluate_full_benchmark_dashboard(
    evidence: FullBenchmarkDashboardEvidence<'_>,
    paths: FullBenchmarkDashboardPaths<'_>,
    markdown_output_path: &str,
    json_output_path: &str,
) -> Result<FullBenchmarkDashboardReport, BenchError> {
    let FullBenchmarkDashboardEvidence {
        stage_inventory,
        stage_tool_table,
        expected_results,
        rendered_commands,
        parser_collector,
        full_report,
        real_smoke,
    } = evidence;
    let total_stages = stage_inventory.total_stage_count;
    let total_tools = expected_results.tool_count;
    let total_expected_jobs = expected_results.row_count;
    let ready_jobs = full_report.present_row_count;
    let blocked_jobs = full_report.missing_result_row_count;
    let missing_parsers = count_missing_parsers(expected_results, parser_collector);
    let missing_adapters = count_missing_adapters(expected_results, rendered_commands);
    let missing_assets = count_missing_assets(expected_results, stage_tool_table);
    let failed_real_smokes = count_failed_real_smokes(real_smoke);
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
        schema_version: FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION.to_string(),
        markdown_output_path: markdown_output_path.to_string(),
        json_output_path: json_output_path.to_string(),
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
    ensure_full_benchmark_dashboard_contract(report, evidence, paths)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
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
    evidence: FullBenchmarkDashboardEvidence<'_>,
    paths: FullBenchmarkDashboardPaths<'_>,
) -> Result<FullBenchmarkDashboardReport, BenchError> {
    let FullBenchmarkDashboardEvidence {
        stage_inventory,
        stage_tool_table,
        expected_results,
        rendered_commands,
        parser_collector,
        full_report,
        real_smoke,
    } = evidence;
    let stage_inventory_output_path = stage_inventory.output_path.as_str();
    if report.metrics.len() != 9 {
        return Err(BenchError::InvalidObservation {
            reason: "full benchmark dashboard must keep exactly 9 governed summary metrics"
                .to_string(),
        });
    }
    if report.markdown_output_path != paths.markdown || report.json_output_path != paths.json {
        return Err(BenchError::InvalidObservation {
            reason: "full benchmark dashboard output paths drifted from the governed locations"
                .to_string(),
        });
    }
    if report.total_stages != 71 || stage_inventory_output_path != paths.stage_inventory {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard total stages drifted from the governed all-domain stage inventory".to_string() });
    }
    if report.total_tools != expected_results.tool_count {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard total tools drifted from the governed expected-result slice".to_string() });
    }
    if report.total_tools != full_report.tool_centric_row_count {
        return Err(BenchError::InvalidObservation {
            reason:
                "full benchmark dashboard total tools must equal full benchmark report tool rows"
                    .to_string(),
        });
    }
    if report.total_expected_jobs != expected_results.row_count {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard total expected jobs drifted from the governed expected-result slice".to_string() });
    }
    if report.ready_jobs != full_report.present_row_count {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard ready jobs drifted from the governed full benchmark report".to_string() });
    }
    if report.blocked_jobs != full_report.missing_result_row_count {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard blocked jobs drifted from the governed missing-result count".to_string() });
    }
    if report.total_expected_jobs != report.ready_jobs + report.blocked_jobs {
        return Err(BenchError::InvalidObservation {
            reason:
                "full benchmark dashboard expected jobs must equal ready plus blocked expected jobs"
                    .to_string(),
        });
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
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard missing parser count drifted from parser collector coverage".to_string() });
    }

    let rendered_bindings = rendered_commands
        .rows
        .iter()
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    if report.missing_adapters != expected_bindings.difference(&rendered_bindings).count() {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard missing adapter count drifted from rendered command coverage".to_string() });
    }

    let asset_bindings = stage_tool_table
        .rows
        .iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .filter(|row| !row.asset_profile_id.trim().is_empty())
        .map(|row| binding_key(&row.domain, &row.stage_id, &row.tool_id))
        .collect::<BTreeSet<_>>();
    if report.missing_assets != expected_bindings.difference(&asset_bindings).count() {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard missing asset count drifted from stage-tool asset coverage".to_string() });
    }

    let failed_real_smokes = count_failed_real_smokes(real_smoke);
    if report.failed_real_smokes != failed_real_smokes {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard failed real-smoke count drifted from the governed real-smoke subset".to_string() });
    }
    if !full_report.passes_behavior_test || !real_smoke.passes_behavior_test {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard requires green full benchmark report and real-smoke subset behavior checks".to_string() });
    }
    if report.explicit_unsupported_pairs != full_report.unsupported_pair_row_count {
        return Err(BenchError::InvalidObservation { reason: "full benchmark dashboard unsupported pair count drifted from the governed full benchmark report".to_string() });
    }
    report.passes_behavior_test = true;
    Ok(report)
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{count_failed_real_smokes, real_smoke_row_passes};
    use bijux_dna_bench_model::{
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
            schema_version: "schema".to_string(),
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
