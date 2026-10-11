//! Operational reporting decisions over explicit acquired observations.
use crate::{
    FullBenchmarkDashboardReport, FullBenchmarkReport, FullBenchmarkResultCollectorReport,
    OperationalBenchmarkReadyBlocker, OperationalBenchmarkResultCounts,
};
use std::collections::BTreeSet;

/// Borrowed observations from one operational readiness request.
/// Availability records successful acquisition, not independent scientific qualification.
#[derive(Clone, Copy, Default)]
pub struct OperationalReportingEvidence<'a> {
    pub expected_result_count: usize,
    pub collector: Option<&'a FullBenchmarkResultCollectorReport>,
    pub report: Option<&'a FullBenchmarkReport>,
    pub dashboard: Option<&'a FullBenchmarkDashboardReport>,
    pub vcf_catalog_available: bool,
    pub vcf_smoke_available: bool,
}

/// Caller-selected diagnostic paths for observations that could not be acquired.
#[derive(Clone, Copy)]
pub struct OperationalReportingPaths<'a> {
    pub collector: &'a str,
    pub report: &'a str,
    pub dashboard: &'a str,
    pub vcf_smoke: &'a str,
}

/// Preserve missing, insufficient, unsupported and failed-report observations as blockers.
/// These policies qualify local reporting behavior, not independent backend execution.
#[must_use]
pub fn evaluate_operational_reporting(
    evidence: OperationalReportingEvidence<'_>,
    paths: OperationalReportingPaths<'_>,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) -> OperationalBenchmarkResultCounts {
    check_collector(evidence.collector, paths.collector, blockers);
    check_report(evidence.report, paths.report, blockers);
    check_dashboard(evidence.dashboard, paths.dashboard, blockers);
    if !evidence.vcf_catalog_available || !evidence.vcf_smoke_available {
        blockers.insert(global_blocker(
            ReportingSubject::VcfValidation,
            "vcf_stage_untested",
            paths.vcf_smoke,
            "VCF catalog or local smoke readiness is not green",
        ));
    }
    OperationalBenchmarkResultCounts {
        benchmark_ready: evidence.expected_result_count,
        missing: evidence.collector.map_or(0, |report| report.missing_result_status_count),
        insufficient: evidence.collector.map_or(0, |report| report.insufficient_data_status_count),
        unsupported: evidence.collector.map_or(0, |report| report.unsupported_pair_status_count),
    }
}

fn check_collector(
    observation: Option<&FullBenchmarkResultCollectorReport>,
    missing_path: &str,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let Some(report) = observation {
        if report.missing_result_status_count == 0 {
            blockers.insert(global_blocker(
                ReportingSubject::Collector,
                "missing_result_omitted",
                &report.output_path,
                "full benchmark result collector omitted missing_result rows",
            ));
        }
        if report.insufficient_data_status_count == 0 {
            blockers.insert(global_blocker(
                ReportingSubject::InsufficientDemography,
                "insufficient_data_omitted",
                &report.output_path,
                "full benchmark result collector omitted insufficient_data rows",
            ));
        }
        if report.unsupported_pair_status_count == 0 {
            blockers.insert(global_blocker(
                ReportingSubject::UnsupportedFilter,
                "unsupported_pair_omitted",
                &report.output_path,
                "full benchmark result collector omitted unsupported_pair rows",
            ));
        }
    } else {
        blockers.insert(global_blocker(
            ReportingSubject::Collector,
            "surface_render_failed",
            missing_path,
            "Full benchmark result collector failed",
        ));
    }
}

fn check_report(
    observation: Option<&FullBenchmarkReport>,
    missing_path: &str,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let Some(report) = observation {
        if !report.passes_behavior_test {
            blockers.insert(global_blocker(
                ReportingSubject::Report,
                "report_contract_failed",
                &report.json_output_path,
                "full benchmark report failed its governed behavior contract",
            ));
        }
        if report.missing_result_row_count == 0 || report.unsupported_pair_row_count == 0 {
            blockers.insert(global_blocker(
                ReportingSubject::Report,
                "report_rows_omitted",
                &report.json_output_path,
                "full benchmark report omitted missing-result or unsupported rows",
            ));
        }
    } else {
        blockers.insert(global_blocker(
            ReportingSubject::Report,
            "surface_render_failed",
            missing_path,
            "Full benchmark report failed",
        ));
    }
}

fn check_dashboard(
    observation: Option<&FullBenchmarkDashboardReport>,
    missing_path: &str,
    blockers: &mut BTreeSet<OperationalBenchmarkReadyBlocker>,
) {
    if let Some(report) = observation {
        if !report.passes_behavior_test {
            blockers.insert(global_blocker(
                ReportingSubject::Dashboard,
                "dashboard_contract_failed",
                &report.json_output_path,
                "full benchmark dashboard failed its governed behavior contract",
            ));
        }
    } else {
        blockers.insert(global_blocker(
            ReportingSubject::Dashboard,
            "surface_render_failed",
            missing_path,
            "Full benchmark dashboard failed",
        ));
    }
}

#[derive(Clone, Copy)]
enum ReportingSubject {
    Collector,
    InsufficientDemography,
    UnsupportedFilter,
    Report,
    Dashboard,
    VcfValidation,
}

fn global_blocker(
    subject: ReportingSubject,
    blocker_type: &str,
    blocker_path: &str,
    detail: &str,
) -> OperationalBenchmarkReadyBlocker {
    let (domain, stage_id, tool_id, corpus_id, asset_profile_id) = match subject {
        ReportingSubject::Collector => (
            "cross",
            "benchmark.result_collector",
            "collector",
            "benchmark_ready",
            "benchmark_ready",
        ),
        ReportingSubject::InsufficientDemography => {
            ("vcf", "vcf.demography", "ibdne", "vcf_production_regression", "json_ibd_segments")
        }
        ReportingSubject::UnsupportedFilter => {
            ("vcf", "vcf.filter", "samtools", "vcf_production_regression", "vcf_cohort")
        }
        ReportingSubject::Report => {
            ("cross", "benchmark.report", "report", "benchmark_ready", "benchmark_ready")
        }
        ReportingSubject::Dashboard => {
            ("cross", "benchmark.dashboard", "dashboard", "benchmark_ready", "benchmark_ready")
        }
        ReportingSubject::VcfValidation => {
            ("vcf", "readiness.local_validation", "governed_validation", "vcf_mini", "vcf_cohort")
        }
    };
    OperationalBenchmarkReadyBlocker {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_id: corpus_id.to_string(),
        asset_profile_id: asset_profile_id.to_string(),
        blocker_type: blocker_type.to_string(),
        blocker_path: blocker_path.to_string(),
        detail: detail.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION, FULL_BENCHMARK_REPORT_SCHEMA_V1,
        FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1,
    };

    fn paths() -> OperationalReportingPaths<'static> {
        OperationalReportingPaths {
            collector: "missing-collector",
            report: "missing-report",
            dashboard: "missing-dashboard",
            vcf_smoke: "missing-vcf-smoke",
        }
    }

    #[test]
    fn unavailable_reporting_observations_remain_attributable_blockers() {
        let mut blockers = BTreeSet::new();
        let counts = evaluate_operational_reporting(
            OperationalReportingEvidence { expected_result_count: 7, ..Default::default() },
            paths(),
            &mut blockers,
        );
        assert_eq!(
            (counts.benchmark_ready, counts.missing, counts.insufficient, counts.unsupported),
            (7, 0, 0, 0)
        );
        assert_eq!(blockers.len(), 4);
        for (stage, kind, path) in [
            ("benchmark.result_collector", "surface_render_failed", "missing-collector"),
            ("benchmark.report", "surface_render_failed", "missing-report"),
            ("benchmark.dashboard", "surface_render_failed", "missing-dashboard"),
            ("readiness.local_validation", "vcf_stage_untested", "missing-vcf-smoke"),
        ] {
            assert!(blockers
                .iter()
                .any(|b| b.stage_id == stage && b.blocker_type == kind && b.blocker_path == path));
        }
    }

    #[test]
    fn reporting_policy_keeps_required_categories_and_failed_verdicts() -> anyhow::Result<()> {
        // Only summary policy is under test; these declarations are not admitted result evidence.
        let mut collector = collector()?;
        let mut report = report()?;
        let mut dashboard = dashboard()?;
        let mut blockers = BTreeSet::new();
        let counts = evaluate_operational_reporting(
            OperationalReportingEvidence {
                expected_result_count: 7,
                collector: Some(&collector),
                report: Some(&report),
                dashboard: Some(&dashboard),
                vcf_catalog_available: true,
                vcf_smoke_available: true,
            },
            paths(),
            &mut blockers,
        );
        assert!(blockers.is_empty());
        assert_eq!(
            (counts.benchmark_ready, counts.missing, counts.insufficient, counts.unsupported),
            (7, 2, 3, 1)
        );
        collector.missing_result_status_count = 0;
        collector.insufficient_data_status_count = 0;
        collector.unsupported_pair_status_count = 0;
        check_collector(Some(&collector), "unobserved", &mut blockers);
        assert_eq!(blockers.len(), 3);
        for (domain, stage, tool, corpus, asset, kind) in [
            (
                "cross",
                "benchmark.result_collector",
                "collector",
                "benchmark_ready",
                "benchmark_ready",
                "missing_result_omitted",
            ),
            (
                "vcf",
                "vcf.demography",
                "ibdne",
                "vcf_production_regression",
                "json_ibd_segments",
                "insufficient_data_omitted",
            ),
            (
                "vcf",
                "vcf.filter",
                "samtools",
                "vcf_production_regression",
                "vcf_cohort",
                "unsupported_pair_omitted",
            ),
        ] {
            assert!(blockers.iter().any(|b| (
                b.domain.as_str(),
                b.stage_id.as_str(),
                b.tool_id.as_str(),
                b.corpus_id.as_str(),
                b.asset_profile_id.as_str(),
                b.blocker_type.as_str(),
                b.blocker_path.as_str()
            ) == (
                domain,
                stage,
                tool,
                corpus,
                asset,
                kind,
                "observed-collector"
            )));
        }
        report.passes_behavior_test = false;
        report.missing_result_row_count = 0;
        check_report(Some(&report), "unobserved", &mut blockers);
        assert_eq!(blockers.len(), 5);
        for kind in ["report_contract_failed", "report_rows_omitted"] {
            assert!(blockers
                .iter()
                .any(|b| b.blocker_type == kind && b.blocker_path == "observed-report"));
        }
        dashboard.passes_behavior_test = false;
        check_dashboard(Some(&dashboard), "unobserved", &mut blockers);
        assert_eq!(blockers.len(), 6);
        assert!(blockers.iter().any(|b| b.blocker_type == "dashboard_contract_failed"
            && b.blocker_path == "observed-dashboard"));
        check_dashboard(Some(&dashboard), "unobserved", &mut blockers);
        assert_eq!(blockers.len(), 6);
        Ok(())
    }

    fn collector() -> anyhow::Result<FullBenchmarkResultCollectorReport> {
        Ok(serde_json::from_value(serde_json::json!({
            "schema_version": FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1, "output_path": "observed-collector",
            "row_count": 0, "benchmark_expected_row_count": 0, "pipeline_fake_run_row_count": 0, "fake_run_row_count": 0,
            "fake_failure_row_count": 0, "missing_result_audit_row_count": 0, "real_smoke_row_count": 0,
            "insufficient_data_row_count": 0, "unsupported_pair_row_count": 0, "missing_result_status_count": 2,
            "insufficient_data_status_count": 3, "unsupported_pair_status_count": 1, "passes_behavior_test": true,
            "surface_kind_counts": {}, "result_status_counts": {}, "domain_counts": {}, "rows": []
        }))?)
    }

    fn report() -> anyhow::Result<FullBenchmarkReport> {
        Ok(serde_json::from_value(serde_json::json!({
            "schema_version": FULL_BENCHMARK_REPORT_SCHEMA_V1, "markdown_output_path": "report.md", "json_output_path": "observed-report",
            "row_count": 0, "expected_result_row_count": 0, "explicit_unsupported_row_count": 0, "present_row_count": 0,
            "missing_result_row_count": 2, "unsupported_pair_row_count": 1, "stage_centric_row_count": 0, "tool_centric_row_count": 0,
            "corpus_centric_row_count": 0, "pipeline_centric_row_count": 0, "runtime_row_count": 0, "memory_row_count": 0,
            "comparable_metric_row_count": 0, "failure_row_count": 0, "failure_class_row_count": 0, "passes_behavior_test": true,
            "domain_counts": {}, "row_status_counts": {}, "rows": [], "stage_centric": [], "tool_centric": [], "corpus_centric": [],
            "pipeline_centric": {"row_count": 0, "unmapped_row_count": 0, "rows": []}, "runtime": [], "memory": [],
            "failures": {"simulated_failure_row_count": 0, "failure_class_row_count": 0, "simulated_failure_domain_counts": {}, "rows": [], "classification_rows": []},
            "missing_results": [], "comparable_metrics": [], "unsupported_pairs": []
        }))?)
    }

    fn dashboard() -> anyhow::Result<FullBenchmarkDashboardReport> {
        Ok(serde_json::from_value(serde_json::json!({
            "schema_version": FULL_BENCHMARK_DASHBOARD_SCHEMA_VERSION, "markdown_output_path": "dashboard.md", "json_output_path": "observed-dashboard",
            "total_stages": 0, "total_tools": 0, "total_expected_jobs": 0, "ready_jobs": 0, "blocked_jobs": 0,
            "missing_parsers": 0, "missing_adapters": 0, "missing_assets": 0, "failed_real_smokes": 0,
            "explicit_unsupported_pairs": 0, "passes_behavior_test": true, "metrics": []
        }))?)
    }
}
