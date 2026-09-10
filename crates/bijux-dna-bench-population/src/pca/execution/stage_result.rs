//! Owner: bijux-dna-bench-population
//! Project normalized observations into the persisted stage-result contract.
use crate::execution::stage_result::path_relative_to_repo;
use crate::execution::stage_result::validate_stage_result_manifest;
use crate::execution::stage_result::BenchStageResultCommandV1;
use crate::execution::stage_result::BenchStageResultManifestV1;
use crate::execution::stage_result::BenchStageResultOutputV1;
use crate::execution::stage_result::BenchStageResultResourceMetricSource;
use crate::execution::stage_result::BenchStageResultResourceMetricsV1;
use crate::execution::stage_result::BenchStageResultRuntimeV1;
use crate::execution::stage_result::BenchStageResultStatus;
use crate::execution::stage_result::BenchStageResultToolV1;
use crate::execution::stage_result::BENCH_STAGE_RESULT_SCHEMA_VERSION;
use crate::pca::model::LocalVcfPcaSmokeReport;
use crate::pca::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::pca::model::DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME;
use crate::pca::model::DEFAULT_OUTPUT_TSV_NAME;
use anyhow::bail;
use anyhow::Result;
use std::path::Path;

pub(crate) fn stage_result(
    repo_root: &Path,
    report: &LocalVcfPcaSmokeReport,
) -> Result<BenchStageResultManifestV1> {
    let mut outputs = vec![
        crate::execution::stage_result::ArtifactDeclaration {
            id: "pca_tsv",
            path: DEFAULT_OUTPUT_TSV_NAME,
            role: "table_output",
        }
        .observe(report.pca_tsv_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "pca_json",
            path: DEFAULT_OUTPUT_JSON_NAME,
            role: "report_output",
        }
        .observe(report.pca_json_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_eigenvec_tsv",
            path: DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME,
            role: "table_output",
        }
        .observe(report.source_eigenvec_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_eigenval_tsv",
            path: DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME,
            role: "table_output",
        }
        .observe(report.source_eigenval_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_pca_manifest_json",
            path: DEFAULT_OUTPUT_SOURCE_MANIFEST_NAME,
            role: "report_output",
        }
        .observe(report.source_pca_manifest_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_logs_txt",
            path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
            role: "log_output",
        }
        .observe(report.source_logs_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "pca_report",
            path: DEFAULT_OUTPUT_JSON_NAME,
            role: "report_output",
        }
        .observe(report.pca_json_path.clone(), true),
    ];
    outputs.extend(tool_outputs(repo_root, report)?);

    let stage_result_manifest = BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: report.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: report.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: report.command.clone() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: report.started_at.clone(),
            finished_at: report.finished_at.clone(),
            elapsed_seconds: report.elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs,
    };
    validate_stage_result_manifest(&stage_result_manifest)?;
    Ok(stage_result_manifest)
}

fn tool_outputs(
    repo_root: &Path,
    report: &LocalVcfPcaSmokeReport,
) -> Result<Vec<BenchStageResultOutputV1>> {
    let published_stage_root = repo_root.join(&report.output_root).join("artifacts/stage");
    let mut outputs = Vec::new();
    match report.tool_id.as_str() {
        "plink2" => outputs.extend([
            crate::execution::stage_result::ArtifactDeclaration {
                id: "pca_eigenvec",
                path: DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME,
                role: "table_output",
            }
            .observe(report.source_eigenvec_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "pca_eigenval",
                path: DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME,
                role: "table_output",
            }
            .observe(report.source_eigenval_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "plink2_log",
                path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
                role: "log_output",
            }
            .observe(report.source_logs_path.clone(), true),
        ]),
        "eigensoft" => outputs.extend([
            crate::execution::stage_result::ArtifactDeclaration {
                id: "convertf_par",
                path: "artifacts/stage/pca.convertf.par",
                role: "config_output",
            }
            .observe(
                path_relative_to_repo(repo_root, &published_stage_root.join("pca.convertf.par")),
                true,
            ),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "smartpca_par",
                path: "artifacts/stage/pca.smartpca.par",
                role: "config_output",
            }
            .observe(
                path_relative_to_repo(repo_root, &published_stage_root.join("pca.smartpca.par")),
                true,
            ),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "eigensoft_geno",
                path: "artifacts/stage/pca.geno",
                role: "matrix_output",
            }
            .observe(
                path_relative_to_repo(repo_root, &published_stage_root.join("pca.geno")),
                true,
            ),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "eigensoft_snp",
                path: "artifacts/stage/pca.snp",
                role: "index_output",
            }
            .observe(path_relative_to_repo(repo_root, &published_stage_root.join("pca.snp")), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "eigensoft_ind",
                path: "artifacts/stage/pca.ind",
                role: "sample_output",
            }
            .observe(path_relative_to_repo(repo_root, &published_stage_root.join("pca.ind")), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "smartpca_eigenvec",
                path: DEFAULT_OUTPUT_SOURCE_EIGENVEC_NAME,
                role: "table_output",
            }
            .observe(report.source_eigenvec_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "smartpca_eigenval",
                path: DEFAULT_OUTPUT_SOURCE_EIGENVAL_NAME,
                role: "table_output",
            }
            .observe(report.source_eigenval_path.clone(), true),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "smartpca_log",
                path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
                role: "log_output",
            }
            .observe(report.source_logs_path.clone(), true),
        ]),
        other => bail!("unsupported VCF PCA smoke tool `{other}`"),
    }
    Ok(outputs)
}
