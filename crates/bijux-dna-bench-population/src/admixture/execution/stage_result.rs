//! Owner: bijux-dna-bench-population
//! Project normalized observations into the persisted stage-result contract.
use crate::admixture::model::LocalVcfAdmixtureSmokeReport;
use crate::admixture::model::DEFAULT_OUTPUT_JSON_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME;
use crate::admixture::model::DEFAULT_OUTPUT_TSV_NAME;
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
use anyhow::Result;
use std::path::Path;

pub(crate) fn stage_result(
    repo_root: &Path,
    report: &LocalVcfAdmixtureSmokeReport,
    stage_root: &Path,
) -> Result<BenchStageResultManifestV1> {
    let published_stage_root = repo_root.join(&report.output_root).join("artifacts/stage");
    let mut outputs = vec![
        crate::execution::stage_result::ArtifactDeclaration {
            id: "admixture_tsv",
            path: DEFAULT_OUTPUT_TSV_NAME,
            role: "table_output",
        }
        .observe(report.admixture_tsv_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "admixture_json",
            path: DEFAULT_OUTPUT_JSON_NAME,
            role: "report_output",
        }
        .observe(report.admixture_json_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_q_matrix_tsv",
            path: DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME,
            role: "table_output",
        }
        .observe(report.source_q_matrix_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_k_selection_json",
            path: DEFAULT_OUTPUT_SOURCE_K_SELECTION_NAME,
            role: "report_output",
        }
        .observe(report.source_k_selection_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_logs_txt",
            path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
            role: "log_output",
        }
        .observe(report.source_logs_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "admixture_proxy_eigenvec",
            path: DEFAULT_OUTPUT_SOURCE_Q_MATRIX_NAME,
            role: "table_output",
        }
        .observe(report.source_q_matrix_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "admixture_proxy_eigenval",
            path: "artifacts/stage/admixture_proxy_eigenval.tsv",
            role: "table_output",
        }
        .observe(
            path_relative_to_repo(
                repo_root,
                &published_stage_root.join("admixture_proxy_eigenval.tsv"),
            ),
            true,
        ),
        crate::execution::stage_result::ArtifactDeclaration {
            id: &(format!("{}_log", report.tool_id)),
            path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
            role: "log_output",
        }
        .observe(report.source_logs_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "admixture_report",
            path: DEFAULT_OUTPUT_JSON_NAME,
            role: "report_output",
        }
        .observe(report.admixture_json_path.clone(), true),
    ];
    outputs.extend(tool_outputs(repo_root, report, stage_root));
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
    report: &LocalVcfAdmixtureSmokeReport,
    stage_root: &Path,
) -> Vec<BenchStageResultOutputV1> {
    let published_stage_root = repo_root.join(&report.output_root).join("artifacts/stage");
    let mut outputs = Vec::new();
    if report.tool_id == "plink" {
        for (artifact_id, suffix, role) in [
            ("bed_matrix", "bed", "binary_output"),
            ("bim_index", "bim", "table_output"),
            ("fam_samples", "fam", "table_output"),
            ("allele_frequency_frq", "frq", "table_output"),
            ("sample_missingness_imiss", "imiss", "table_output"),
            ("variant_missingness_lmiss", "lmiss", "table_output"),
        ] {
            let staged_path = stage_root.join(format!("admixture_plink.{suffix}"));
            outputs.push(
                crate::execution::stage_result::ArtifactDeclaration {
                    id: artifact_id,
                    path: &(format!("artifacts/stage/admixture_plink.{suffix}")),
                    role,
                }
                .observe(
                    path_relative_to_repo(
                        repo_root,
                        &published_stage_root.join(format!("admixture_plink.{suffix}")),
                    ),
                    staged_path.is_file(),
                ),
            );
        }
    }
    outputs
}
