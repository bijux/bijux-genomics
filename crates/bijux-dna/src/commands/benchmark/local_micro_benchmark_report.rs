use bijux_dna_api::v1::api::bench::{
    generate_micro_benchmark_report, MicroBenchmarkExpectedRow as ExpectedMicroCoverageRow,
    MicroBenchmarkReportRequest, MicroBenchmarkReportSources,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    MicroBenchmarkReport, DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH,
    DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH,
};
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::local_adna_micro_pipeline::{AdnaMicroPipelineReport, AdnaMicroPipelineRow};
use super::local_amplicon_micro_pipeline::{AmpliconMicroPipelineReport, AmpliconMicroPipelineRow};
use super::local_bam_micro_smoke_subset::{BamMicroSmokeFamilyRow, BamMicroSmokeSubsetReport};
use super::local_core_germline_micro_pipeline::{
    CoreGermlineMicroPipelineReport, CoreGermlineMicroPipelineRow,
};
use super::local_edna_micro_pipeline::{EdnaMicroPipelineReport, EdnaMicroPipelineRow};
use super::local_fastq_micro_smoke_subset::{
    FastqMicroSmokeFamilyRow, FastqMicroSmokeSubsetReport,
};
use super::local_micro_benchmark_run::{
    MicroBenchmarkLogRow, MicroBenchmarkOutputRow, MicroBenchmarkResultRow,
    MicroBenchmarkRunManifest,
};
use super::local_real_smoke_core_subset::{RealSmokeCoreSubsetReport, RealSmokeCoreSubsetRow};
use super::local_vcf_micro_smoke_subset::{VcfMicroSmokeFamilyRow, VcfMicroSmokeSubsetReport};

pub(crate) struct MicroBenchmarkSourceReports<'a> {
    pub(crate) real_smoke: &'a RealSmokeCoreSubsetReport,
    pub(crate) fastq: &'a FastqMicroSmokeSubsetReport,
    pub(crate) bam: &'a BamMicroSmokeSubsetReport,
    pub(crate) vcf: &'a VcfMicroSmokeSubsetReport,
    pub(crate) amplicon: &'a AmpliconMicroPipelineReport,
    pub(crate) adna: &'a AdnaMicroPipelineReport,
    pub(crate) edna: &'a EdnaMicroPipelineReport,
    pub(crate) core_germline: &'a CoreGermlineMicroPipelineReport,
}

pub(crate) fn render_micro_benchmark_report(
    repo_root: &Path,
    manifest: &MicroBenchmarkRunManifest,
    result_rows: &[MicroBenchmarkResultRow],
    output_rows: &[MicroBenchmarkOutputRow],
    log_rows: &[MicroBenchmarkLogRow],
    markdown_output_path: PathBuf,
    json_output_path: PathBuf,
    source_reports: MicroBenchmarkSourceReports<'_>,
) -> Result<MicroBenchmarkReport> {
    let expected = collect_expected_micro_rows(&source_reports);
    generate_micro_benchmark_report(MicroBenchmarkReportRequest {
        repo_root,
        sources: MicroBenchmarkReportSources {
            manifest,
            results: result_rows,
            outputs: output_rows,
            logs: log_rows,
            expected: &expected,
        },
        markdown_output_path: &markdown_output_path,
        json_output_path: &json_output_path,
    })
}

fn collect_expected_micro_rows(
    reports: &MicroBenchmarkSourceReports<'_>,
) -> Vec<ExpectedMicroCoverageRow> {
    let mut rows = Vec::new();
    rows.extend(
        reports
            .real_smoke
            .rows
            .iter()
            .map(|row| expected_real_smoke_row(reports.real_smoke.output_path.clone(), row)),
    );
    rows.extend(reports.fastq.rows.iter().map(|row| {
        expected_family_row(
            "fastq_micro_smoke_subset",
            "fastq",
            reports.fastq.output_path.clone(),
            row,
        )
    }));
    rows.extend(
        reports
            .bam
            .rows
            .iter()
            .map(|row| expected_bam_family_row(reports.bam.output_path.clone(), row)),
    );
    rows.extend(
        reports
            .vcf
            .rows
            .iter()
            .map(|row| expected_vcf_family_row(reports.vcf.output_path.clone(), row)),
    );
    rows.extend(reports.amplicon.rows.iter().map(|row| {
        expected_pipeline_row("amplicon_micro_pipeline", reports.amplicon.output_path.clone(), row)
    }));
    rows.extend(
        reports
            .adna
            .rows
            .iter()
            .map(|row| expected_adna_pipeline_row(reports.adna.output_path.clone(), row)),
    );
    rows.extend(
        reports
            .edna
            .rows
            .iter()
            .map(|row| expected_edna_pipeline_row(reports.edna.output_path.clone(), row)),
    );
    rows.extend(reports.core_germline.rows.iter().map(|row| {
        expected_core_germline_pipeline_row(reports.core_germline.output_path.clone(), row)
    }));
    rows.sort_by(|left, right| {
        left.component_id
            .cmp(&right.component_id)
            .then_with(|| left.execution_id.cmp(&right.execution_id))
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    rows
}

fn expected_real_smoke_row(
    source_report_path: String,
    row: &RealSmokeCoreSubsetRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "real_smoke_core_subset".to_string(),
        execution_id: row.execution_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_family_row(
    component_id: &str,
    domain: &str,
    source_report_path: String,
    row: &FastqMicroSmokeFamilyRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: component_id.to_string(),
        execution_id: row.family_id.clone(),
        domain: domain.to_string(),
        stage_id: row.representative_stage_id.clone(),
        tool_id: row.representative_tool_id.clone(),
        source_report_path,
    }
}

fn expected_bam_family_row(
    source_report_path: String,
    row: &BamMicroSmokeFamilyRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "bam_micro_smoke_subset".to_string(),
        execution_id: row.family_id.clone(),
        domain: "bam".to_string(),
        stage_id: row.representative_stage_id.clone(),
        tool_id: row.representative_tool_id.clone(),
        source_report_path,
    }
}

fn expected_vcf_family_row(
    source_report_path: String,
    row: &VcfMicroSmokeFamilyRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "vcf_micro_smoke_subset".to_string(),
        execution_id: row.family_id.clone(),
        domain: "vcf".to_string(),
        stage_id: row.representative_stage_id.clone(),
        tool_id: row.representative_tool_id.clone(),
        source_report_path,
    }
}

fn expected_pipeline_row(
    component_id: &str,
    source_report_path: String,
    row: &AmpliconMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: component_id.to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_adna_pipeline_row(
    source_report_path: String,
    row: &AdnaMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "adna_micro_pipeline".to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_edna_pipeline_row(
    source_report_path: String,
    row: &EdnaMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "edna_micro_pipeline".to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

fn expected_core_germline_pipeline_row(
    source_report_path: String,
    row: &CoreGermlineMicroPipelineRow,
) -> ExpectedMicroCoverageRow {
    ExpectedMicroCoverageRow {
        component_id: "core_germline_micro_pipeline".to_string(),
        execution_id: row.stage_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        source_report_path,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_api::v1::api::bench::{
        validate_micro_benchmark_report, MicroBenchmarkExecutionStatus,
    };
    use std::{collections::BTreeMap, fs};
    const SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH: &str =
        "configs/bench/local/scientific-acceptance-thresholds.toml";
    const STAGE_TOOL_RESOURCES_PATH: &str = "configs/bench/local/stage-tool-resources.toml";
    use crate::commands::benchmark::readiness::stage_tool_resources::FASTQ_STAGE_TOOL_RESOURCE_ORIGIN;
    use tempfile::tempdir;

    #[test]
    fn micro_benchmark_report_renders_governed_sections() {
        let temp = tempdir().expect("tempdir");
        let repo_root = temp.path();

        fs::create_dir_all(repo_root.join("configs/bench/local")).expect("create configs");
        fs::write(
            repo_root.join(SCIENTIFIC_ACCEPTANCE_THRESHOLDS_PATH),
            r#"
schema_version = "bijux.bench.local_scientific_acceptance_thresholds.v1"

[[rows]]
domain = "fastq"
stage_id = "fastq.validate_reads"
metric_id = "validated_reads"
metric_name = "validated_reads"
direction = "range"
tolerance_kind = "relative_fraction"
tolerance_value = 0.02
pass_rule = "remain_within_reference_range"
insufficiency_behavior = "refuse_stage_comparison"
required = true
declaration_origin = "test_contract"
"#,
        )
        .expect("write thresholds");
        fs::write(
            repo_root.join(STAGE_TOOL_RESOURCES_PATH),
            format!(
                r#"
schema_version = "bijux.bench.local_stage_tool_resources.v1"
classification_scope = "benchmark_ready_command_resources"

[[rows]]
domain = "fastq"
stage_id = "fastq.validate_reads"
tool_id = "fastqvalidator"
threads = 1
memory_gb = 1
walltime_minutes = 1
scratch_gb = 1
resource_origin = "{FASTQ_STAGE_TOOL_RESOURCE_ORIGIN}"
"#,
            ),
        )
        .expect("write resources");

        let fastq_report_path =
            repo_root.join("artifacts/benchmark-smoke/micro/fastq/MICRO_FASTQ_SUMMARY.json");
        let adna_report_path = repo_root
            .join("artifacts/benchmark-smoke/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json");
        let manifest_path =
            repo_root.join("artifacts/benchmark-smoke/micro/MICRO_BENCHMARK_RUN.json");
        let evidence_path =
            repo_root.join("artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json");
        let log_path = repo_root.join("artifacts/benchmark-smoke/micro/logs/MICRO_RUN.log");
        for path in
            [&fastq_report_path, &adna_report_path, &manifest_path, &evidence_path, &log_path]
        {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, "{}").expect("write file");
        }

        let fastq_report = FastqMicroSmokeSubsetReport {
            schema_version: "bijux.bench.local_fastq_micro_smoke_subset.v1",
            output_path: "artifacts/benchmark-smoke/micro/fastq/MICRO_FASTQ_SUMMARY.json".to_string(),
            family_count: 1,
            local_smoke_count: 1,
            container_needed_count: 0,
            unavailable_count: 0,
            passes_behavior_test: true,
            rows: vec![FastqMicroSmokeFamilyRow {
                family_id: "fastq.validate_reads".to_string(),
                surface_label: "Validate Reads".to_string(),
                stage_ids: vec!["fastq.validate_reads".to_string()],
                representative_stage_id: "fastq.validate_reads".to_string(),
                representative_tool_id: "fastqvalidator".to_string(),
                registered_binary: "fastqvalidator".to_string(),
                smoke_tool_id: "fastqvalidator".to_string(),
                smoke_path_kind: "host_stage_smoke".to_string(),
                smoke_runtime: "host".to_string(),
                smoke_command: "bijux-dna bench local materialize-stage --stage-id fastq.validate_reads".to_string(),
                smoke_support_path: None,
                execution_status: super::super::local_fastq_micro_smoke_subset::FastqMicroSmokeExecutionStatus::LocalSmoke,
                reason: "governed local smoke".to_string(),
                evidence_path: Some("artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json".to_string()),
                evidence_format: Some("json".to_string()),
                parsed_schema_version: Some("bijux.fastq.validate.local_smoke.report.v1".to_string()),
            }],
        };
        let adna_report = AdnaMicroPipelineReport {
            schema_version: "bijux.bench.local_adna_micro_pipeline.v1",
            command: "bijux-dna bench local run-adna-micro-pipeline",
            output_path: "artifacts/benchmark-smoke/micro/pipelines/adna/MICRO_ADNA_SUMMARY.json".to_string(),
            pipeline_id: "adna-pseudohaploid-fastq-bam-vcf",
            sample_id: "adna-micro".to_string(),
            reference_fasta_path: "reference.fasta".to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 1,
            handoff_count: 0,
            skipped_count: 1,
            passes_behavior_test: true,
            rows: vec![AdnaMicroPipelineRow {
                stage_id: "bam.contamination".to_string(),
                domain: "bam".to_string(),
                tool_id: "verifybamid2".to_string(),
                execution_mode: "skipped".to_string(),
                status: super::super::local_adna_micro_pipeline::AdnaMicroPipelineRowStatus::Skipped,
                reason: "synthetic aDNA micro execution does not claim panel-backed contamination evidence".to_string(),
                evidence_path: None,
                parsed_schema_version: None,
                consumed_inputs: BTreeMap::new(),
                outputs: BTreeMap::new(),
                metrics: BTreeMap::new(),
            }],
            handoffs: Vec::new(),
        };

        let empty_bam = BamMicroSmokeSubsetReport {
            schema_version: "bijux.bench.local_bam_micro_smoke_subset.v2",
            output_path: "artifacts/benchmark-smoke/micro/bam/MICRO_BAM_SUMMARY.json".to_string(),
            family_count: 0,
            local_smoke_count: 0,
            container_needed_count: 0,
            unavailable_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
        };
        let empty_vcf = VcfMicroSmokeSubsetReport {
            schema_version: "bijux.bench.local_vcf_micro_smoke_subset.v1",
            output_path: "artifacts/benchmark-smoke/micro/vcf/MICRO_VCF_SUMMARY.json".to_string(),
            family_count: 0,
            local_smoke_count: 0,
            container_needed_count: 0,
            unavailable_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
        };
        let empty_amplicon = AmpliconMicroPipelineReport {
            schema_version: "bijux.bench.local_amplicon_micro_pipeline.v1",
            command: "bijux-dna bench local run-amplicon-micro-pipeline",
            output_path:
                "artifacts/benchmark-smoke/micro/pipelines/amplicon/MICRO_AMPLICON_SUMMARY.json"
                    .to_string(),
            pipeline_id: "amplicon-asv-otu-no-vcf",
            corpus_manifest_path: "corpus.toml".to_string(),
            truth_manifest_path: "truth.toml".to_string(),
            sample_count: 0,
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 0,
            handoff_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
            handoffs: Vec::new(),
        };
        let empty_edna = EdnaMicroPipelineReport {
            schema_version: "bijux.bench.local_edna_micro_pipeline.v1",
            command: "bijux-dna bench local run-edna-micro-pipeline",
            output_path: "artifacts/benchmark-smoke/micro/pipelines/edna/MICRO_EDNA_SUMMARY.json"
                .to_string(),
            pipeline_id: "edna-taxonomy-fastq",
            corpus_manifest_path: "corpus.toml".to_string(),
            taxonomy_database_manifest_path: "taxonomy.toml".to_string(),
            sample_count: 0,
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 0,
            handoff_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
            handoffs: Vec::new(),
        };
        let empty_core = CoreGermlineMicroPipelineReport {
            schema_version: "bijux.bench.local_core_germline_micro_pipeline.v1",
            command: "bijux-dna bench local run-core-germline-micro-pipeline",
            output_path: "artifacts/benchmark-smoke/micro/pipelines/core-germline/MICRO_PIPELINE_SUMMARY.json"
                .to_string(),
            pipeline_id: "core-germline-fastq-bam-vcf",
            sample_id: "sample".to_string(),
            reference_fasta_path: "reference.fasta".to_string(),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            finished_at: "2026-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            stage_count: 0,
            handoff_count: 0,
            passes_behavior_test: true,
            rows: Vec::new(),
            handoffs: Vec::new(),
        };
        let empty_real = RealSmokeCoreSubsetReport {
            schema_version: "bijux.bench.local_real_smoke_core_subset.v1".to_string(),
            output_path: "artifacts/benchmark-smoke/micro/core/REAL_SMOKE_CORE_SUMMARY.json"
                .to_string(),
            execution_count: 0,
            stage_execution_count: 0,
            pipeline_bridge_count: 0,
            domain_counts: BTreeMap::new(),
            passes_behavior_test: true,
            rows: Vec::new(),
        };

        let manifest = MicroBenchmarkRunManifest {
            schema_version: "bijux.bench.local_micro_benchmark_run.v1".to_string(),
            manifest_path: "artifacts/benchmark-smoke/micro/MICRO_BENCHMARK_RUN.json".to_string(),
            run_root: "artifacts/benchmark-smoke/micro".to_string(),
            run_id: "micro-benchmark-test".to_string(),
            repo_revision: "0123456789abcdef0123456789abcdef01234567".to_string(),
            worktree_dirty: false,
            created_at_unix: 1,
            command: "bijux-dna bench run-micro".to_string(),
            component_reports: Vec::new(),
            result_rows_path: "artifacts/benchmark-smoke/micro/results/MICRO_RESULT_ROWS.json"
                .to_string(),
            output_rows_path: "artifacts/benchmark-smoke/micro/outputs/MICRO_OUTPUT_ROWS.json"
                .to_string(),
            log_rows_path: "artifacts/benchmark-smoke/micro/logs/MICRO_LOG_ROWS.json".to_string(),
            normalized_metrics_path:
                "artifacts/benchmark-smoke/micro/normalized-metrics/MICRO_NORMALIZED_METRICS.json"
                    .to_string(),
            result_row_count: 2,
            output_row_count: 1,
            log_row_count: 1,
            normalized_metric_row_count: 1,
            passes_behavior_test: true,
        };
        let result_rows = vec![
            MicroBenchmarkResultRow {
                execution_id: "fastq.validate_reads".to_string(),
                component_id: "fastq_micro_smoke_subset".to_string(),
                result_kind: super::super::local_micro_benchmark_run::MicroBenchmarkResultKind::FamilyRepresentative,
                domain: "fastq".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "fastq.validate_reads".to_string(),
                tool_id: "fastqvalidator".to_string(),
                status: MicroBenchmarkExecutionStatus::Succeeded,
                reason: "governed local smoke".to_string(),
                command: Some("bijux-dna bench local materialize-stage --stage-id fastq.validate_reads".to_string()),
                source_report_path: fastq_report.output_path.clone(),
                evidence_path: Some("artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json".to_string()),
                stage_result_manifest_path: None,
                normalized_metric_count: 1,
                output_count: 1,
                log_count: 0,
            },
            MicroBenchmarkResultRow {
                execution_id: "bam.contamination".to_string(),
                component_id: "adna_micro_pipeline".to_string(),
                result_kind: super::super::local_micro_benchmark_run::MicroBenchmarkResultKind::Stage,
                domain: "bam".to_string(),
                bridge_source_domain: None,
                bridge_target_domain: None,
                stage_id: "bam.contamination".to_string(),
                tool_id: "verifybamid2".to_string(),
                status: MicroBenchmarkExecutionStatus::Unavailable,
                reason: "synthetic aDNA micro execution does not claim panel-backed contamination evidence".to_string(),
                command: Some("bijux-dna bench local run-adna-micro-pipeline".to_string()),
                source_report_path: adna_report.output_path.clone(),
                evidence_path: None,
                stage_result_manifest_path: None,
                normalized_metric_count: 0,
                output_count: 0,
                log_count: 0,
            },
        ];
        let output_rows = vec![MicroBenchmarkOutputRow {
            execution_id: "fastq.validate_reads".to_string(),
            component_id: "fastq_micro_smoke_subset".to_string(),
            artifact_id: "evidence".to_string(),
            role: "family_evidence".to_string(),
            path: "artifacts/runs/bench/local-smoke/fastq.validate_reads/report.json".to_string(),
            exists: true,
            source: "fastq_micro_smoke_subset".to_string(),
        }];
        let log_rows = vec![MicroBenchmarkLogRow {
            execution_id: "micro.run".to_string(),
            component_id: "micro_benchmark_run".to_string(),
            role: "run_log".to_string(),
            path: "artifacts/benchmark-smoke/micro/logs/MICRO_RUN.log".to_string(),
            exists: true,
            source: "micro_benchmark_run".to_string(),
        }];

        let report = render_micro_benchmark_report(
            repo_root,
            &manifest,
            &result_rows,
            &output_rows,
            &log_rows,
            PathBuf::from(DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH),
            PathBuf::from(DEFAULT_MICRO_BENCHMARK_REPORT_JSON_PATH),
            MicroBenchmarkSourceReports {
                real_smoke: &empty_real,
                fastq: &fastq_report,
                bam: &empty_bam,
                vcf: &empty_vcf,
                amplicon: &empty_amplicon,
                adna: &adna_report,
                edna: &empty_edna,
                core_germline: &empty_core,
            },
        )
        .expect("render report");

        assert_eq!(report.complete_row_count, 1);
        assert_eq!(report.unavailable_row_count, 1);
        assert_eq!(report.failed_row_count, 0);
        assert_eq!(report.missing_row_count, 0);
        assert_eq!(report.runtime_row_count, 2);
        assert_eq!(report.memory_source_row_count, 2);
        assert_eq!(report.science_threshold_row_count, 1);
        assert!(report.passes_behavior_test);
        assert_report_admission(&manifest, &result_rows, &report).expect("report model admission");

        let markdown =
            fs::read_to_string(repo_root.join(DEFAULT_MICRO_BENCHMARK_REPORT_MARKDOWN_PATH))
                .expect("read markdown");
        assert!(markdown.contains("# Micro Benchmark Report"));
        assert!(markdown.contains("## Complete"));
        assert!(markdown.contains("## Failed"));
        assert!(markdown.contains("## Missing"));
        assert!(markdown.contains("## Unavailable"));
        assert!(markdown.contains("## Insufficient Data"));
        assert!(markdown.contains("## Runtime"));
        assert!(markdown.contains("## Memory Sources"));
        assert!(markdown.contains("## Science Thresholds"));
    }

    fn assert_report_admission(
        manifest: &MicroBenchmarkRunManifest,
        results: &[MicroBenchmarkResultRow],
        report: &MicroBenchmarkReport,
    ) -> Result<()> {
        let wire = serde_json::to_value(report)?;
        let replay: MicroBenchmarkReport = serde_json::from_value(wire.clone())?;
        assert_eq!(serde_json::to_value(replay)?, wire);
        let mut invalid = wire.clone();
        invalid["schema_version"] = serde_json::json!("unsupported.v1");
        assert!(serde_json::from_value::<MicroBenchmarkReport>(invalid).is_err());
        let mut invalid = wire;
        invalid["runtime_rows"][0]["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<MicroBenchmarkReport>(invalid).is_err());
        let mut changed = report.clone();
        changed.failed_row_count += 1;
        assert!(validate_micro_benchmark_report(manifest, results, &mut changed).is_err());
        assert!(!changed.passes_behavior_test);
        changed = report.clone();
        changed.execution_status_counts.insert("overflow".into(), usize::MAX);
        assert!(validate_micro_benchmark_report(manifest, results, &mut changed).is_err());
        assert!(!changed.passes_behavior_test);
        changed = report.clone();
        let mut mismatched = manifest.clone();
        mismatched.result_row_count += 1;
        assert!(validate_micro_benchmark_report(&mismatched, results, &mut changed).is_err());
        assert!(!changed.passes_behavior_test);
        validate_micro_benchmark_report(manifest, results, &mut changed)?;
        assert!(changed.passes_behavior_test);
        Ok(())
    }
}
