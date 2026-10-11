use super::qualify_micro_benchmark_run;
use anyhow::Result;
use bijux_dna_bench_model::{
    MicroBenchmarkComponentReport, MicroBenchmarkExecutionStatus, MicroBenchmarkLogRow,
    MicroBenchmarkNormalizedMetricRow, MicroBenchmarkOutputRow, MicroBenchmarkResultKind,
    MicroBenchmarkResultRow, MicroBenchmarkRunEvidence, MicroBenchmarkRunManifest,
    MICRO_BENCHMARK_RUN_COMMAND, MICRO_BENCHMARK_RUN_SCHEMA_VERSION,
};
use std::path::Path;

struct Fixture {
    manifest: MicroBenchmarkRunManifest,
    results: Vec<MicroBenchmarkResultRow>,
    outputs: Vec<MicroBenchmarkOutputRow>,
    logs: Vec<MicroBenchmarkLogRow>,
    metrics: Vec<MicroBenchmarkNormalizedMetricRow>,
}
impl Fixture {
    fn qualify(&mut self, root: &Path) -> Result<()> {
        qualify_micro_benchmark_run(
            root,
            &mut self.manifest,
            MicroBenchmarkRunEvidence {
                results: &self.results,
                outputs: &self.outputs,
                logs: &self.logs,
                metrics: &self.metrics,
            },
        )
    }
    fn materialize(&self, root: &Path) -> Result<()> {
        for path in self
            .manifest
            .component_reports
            .iter()
            .map(|row| row.report_path.as_str())
            .chain(self.results.iter().filter_map(|row| row.evidence_path.as_deref()))
            .chain(self.outputs.iter().map(|row| row.path.as_str()))
            .chain(self.logs.iter().map(|row| row.path.as_str()))
        {
            bijux_dna_infra::write_bytes(root.join(path), b"{}")?;
        }
        Ok(())
    }
}

fn result_rows() -> Vec<MicroBenchmarkResultRow> {
    use MicroBenchmarkExecutionStatus::{ContainerNeeded, Succeeded};
    use MicroBenchmarkResultKind::{FamilyRepresentative, PipelineBridge, Stage};
    [
        ("vcf.stats", "real_smoke_core_subset", "vcf", Succeeded, Stage),
        (
            "fastq.validate_reads",
            "fastq_micro_smoke_subset",
            "fastq",
            Succeeded,
            FamilyRepresentative,
        ),
        ("bam.align", "bam_micro_smoke_subset", "bam", ContainerNeeded, FamilyRepresentative),
        ("vcf.calling", "vcf_micro_smoke_subset", "vcf", Succeeded, FamilyRepresentative),
        ("vcf.stats", "adna_micro_pipeline", "vcf", Succeeded, PipelineBridge),
        (
            "benchmark.taxonomy_output_judgment",
            "edna_micro_pipeline",
            "benchmark",
            Succeeded,
            Stage,
        ),
        ("vcf.call", "core_germline_micro_pipeline", "vcf", Succeeded, PipelineBridge),
    ]
    .into_iter()
    .map(|(execution, component, domain, status, kind)| MicroBenchmarkResultRow {
        execution_id: execution.to_string(),
        component_id: component.to_string(),
        result_kind: kind,
        domain: domain.to_string(),
        bridge_source_domain: None,
        bridge_target_domain: None,
        stage_id: execution.to_string(),
        tool_id: "fixture-tool".to_string(),
        status,
        reason: "contract fixture".to_string(),
        command: None,
        source_report_path: format!("artifacts/benchmark-smoke/micro/{component}/summary.json"),
        evidence_path: (status == Succeeded)
            .then(|| format!("artifacts/benchmark-smoke/micro/{component}/{execution}.json")),
        stage_result_manifest_path: None,
        normalized_metric_count: if component == "real_smoke_core_subset" { 2 } else { 0 },
        output_count: usize::from(status == Succeeded),
        log_count: usize::from(component == "real_smoke_core_subset"),
    })
    .collect()
}

fn fixture() -> Fixture {
    let results = result_rows();
    let components = [
        "real_smoke_core_subset",
        "fastq_micro_smoke_subset",
        "bam_micro_smoke_subset",
        "vcf_micro_smoke_subset",
        "amplicon_micro_pipeline",
        "adna_micro_pipeline",
        "edna_micro_pipeline",
        "core_germline_micro_pipeline",
    ]
    .into_iter()
    .map(|component| MicroBenchmarkComponentReport {
        component_id: component.to_string(),
        report_path: format!("artifacts/benchmark-smoke/micro/{component}/summary.json"),
        row_count: results.iter().filter(|row| row.component_id == component).count(),
    })
    .collect();
    let outputs = results
        .iter()
        .filter_map(|row| {
            row.evidence_path.as_ref().map(|path| MicroBenchmarkOutputRow {
                execution_id: row.execution_id.clone(),
                component_id: row.component_id.clone(),
                artifact_id: "report".to_string(),
                role: "report".to_string(),
                path: path.clone(),
                exists: true,
                source: "fixture".to_string(),
            })
        })
        .collect::<Vec<_>>();
    let logs = vec![MicroBenchmarkLogRow {
        execution_id: "vcf.stats".to_string(),
        component_id: "real_smoke_core_subset".to_string(),
        role: "stdout".to_string(),
        path: "artifacts/benchmark-smoke/micro/run.log".to_string(),
        exists: true,
        source: "fixture".to_string(),
    }];
    let metrics = ["variant_count", "snp_count"]
        .into_iter()
        .map(|metric| MicroBenchmarkNormalizedMetricRow {
            execution_id: "vcf.stats".to_string(),
            component_id: "real_smoke_core_subset".to_string(),
            metric_id: metric.to_string(),
            value: serde_json::json!(42),
            source_path: "artifacts/benchmark-smoke/micro/metrics.json".to_string(),
        })
        .collect::<Vec<_>>();
    Fixture {
        manifest: manifest(components, results.len(), outputs.len(), logs.len(), metrics.len()),
        results,
        outputs,
        logs,
        metrics,
    }
}

fn manifest(
    components: Vec<MicroBenchmarkComponentReport>,
    result_count: usize,
    output_count: usize,
    log_count: usize,
    metric_count: usize,
) -> MicroBenchmarkRunManifest {
    MicroBenchmarkRunManifest {
        schema_version: MICRO_BENCHMARK_RUN_SCHEMA_VERSION.to_string(),
        command: MICRO_BENCHMARK_RUN_COMMAND.to_string(),
        manifest_path: "artifacts/benchmark-smoke/micro/MICRO_BENCHMARK_RUN.json".to_string(),
        run_root: "artifacts/benchmark-smoke/micro".to_string(),
        run_id: "micro-contract".to_string(),
        repo_revision: "0123456789abcdef0123456789abcdef01234567".to_string(),
        worktree_dirty: false,
        created_at_unix: 1,
        component_reports: components,
        result_rows_path: "artifacts/benchmark-smoke/micro/results.json".to_string(),
        output_rows_path: "artifacts/benchmark-smoke/micro/outputs.json".to_string(),
        log_rows_path: "artifacts/benchmark-smoke/micro/logs.json".to_string(),
        normalized_metrics_path: "artifacts/benchmark-smoke/micro/metrics.json".to_string(),
        result_row_count: result_count,
        output_row_count: output_count,
        log_row_count: log_count,
        normalized_metric_row_count: metric_count,
        passes_behavior_test: false,
    }
}

#[test]
fn micro_benchmark_contract_accepts_governed_row_sets() -> Result<()> {
    let root = bijux_dna_infra::temp_dir("micro-run-contract-")?;
    let mut fixture = fixture();
    fixture.materialize(root.path())?;
    fixture.qualify(root.path())?;
    assert!(fixture.manifest.passes_behavior_test);
    let encoded = serde_json::to_value(&fixture.manifest)?;
    let decoded: MicroBenchmarkRunManifest = serde_json::from_value(encoded.clone())?;
    assert_eq!(serde_json::to_value(decoded)?, encoded);
    let mut wrong_version = encoded.clone();
    wrong_version["schema_version"] = serde_json::json!("unsupported");
    assert!(serde_json::from_value::<MicroBenchmarkRunManifest>(wrong_version).is_err());
    let mut unknown = encoded;
    unknown["component_reports"][0]["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<MicroBenchmarkRunManifest>(unknown).is_err());
    let mut row = serde_json::to_value(&fixture.results[0])?;
    row["status"] = serde_json::json!("unknown");
    assert!(serde_json::from_value::<MicroBenchmarkResultRow>(row).is_err());
    fixture.manifest.result_row_count += 1;
    assert!(fixture.qualify(root.path()).is_err());
    assert!(!fixture.manifest.passes_behavior_test);
    fixture.manifest.result_row_count -= 1;
    fixture.qualify(root.path())?;
    fixture.results[2].normalized_metric_count = 1;
    assert!(fixture.qualify(root.path()).is_err());
    assert!(!fixture.manifest.passes_behavior_test);
    fixture.results[2].normalized_metric_count = 0;
    fixture.qualify(root.path())?;
    let missing = fixture.results[0]
        .evidence_path
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("fixture evidence"))?;
    bijux_dna_infra::remove_file(&root.path().join(missing))?;
    assert!(fixture.qualify(root.path()).is_err());
    assert!(!fixture.manifest.passes_behavior_test);
    Ok(())
}

#[test]
fn micro_run_requires_current_artifacts_and_scoped_component_paths() -> Result<()> {
    let root = bijux_dna_infra::temp_dir("micro-run-current-artifacts-")?;
    let mut fixture = fixture();
    let mut sidecar = fixture.outputs[0].clone();
    sidecar.artifact_id = "sidecar".into();
    sidecar.path = "artifacts/benchmark-smoke/micro/sidecar.json".into();
    fixture.outputs.push(sidecar);
    fixture.manifest.output_row_count += 1;
    fixture.results[0].output_count += 1;
    fixture.materialize(root.path())?;
    fixture.qualify(root.path())?;
    let sidecar_path = root.path().join("artifacts/benchmark-smoke/micro/sidecar.json");
    bijux_dna_infra::remove_file(&sidecar_path)?;
    assert!(fixture.qualify(root.path()).is_err());
    assert!(!fixture.manifest.passes_behavior_test);
    fixture.materialize(root.path())?;
    fixture.qualify(root.path())?;
    let log_path = root.path().join(&fixture.logs[0].path);
    bijux_dna_infra::remove_file(&log_path)?;
    assert!(fixture.qualify(root.path()).is_err());
    assert!(!fixture.manifest.passes_behavior_test);
    bijux_dna_infra::ensure_dir(&log_path)?;
    assert!(fixture.qualify(root.path()).is_err());
    bijux_dna_infra::remove_empty_dir(&log_path)?;
    fixture.materialize(root.path())?;
    fixture.qualify(root.path())?;
    let original = fixture.manifest.component_reports[0].report_path.clone();
    fixture.manifest.component_reports[0].report_path =
        original.replacen("micro/", "micro/../micro/", 1);
    assert!(root.path().join(&fixture.manifest.component_reports[0].report_path).is_file());
    assert!(fixture.qualify(root.path()).is_err());
    assert!(!fixture.manifest.passes_behavior_test);
    fixture.manifest.component_reports[0].report_path = original;
    // Directory output artifacts retain their existing presence semantics.
    bijux_dna_infra::remove_file(&sidecar_path)?;
    bijux_dna_infra::ensure_dir(&sidecar_path)?;
    fixture.qualify(root.path())?;
    assert!(fixture.manifest.passes_behavior_test);
    Ok(())
}
