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
use crate::population_structure::model::LocalVcfPopulationStructureSmokeReport;
use crate::population_structure::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME;
use anyhow::anyhow;
use anyhow::Result;
use std::path::Path;

pub(crate) fn stage_result(
    repo_root: &Path,
    report: &LocalVcfPopulationStructureSmokeReport,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::PopulationStructureStageOutputs,
) -> Result<BenchStageResultManifestV1> {
    let mut outputs = reported_artifacts(repo_root, report);
    outputs.extend(tool_outputs(repo_root, report, stage_outputs)?);
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
    report: &LocalVcfPopulationStructureSmokeReport,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::PopulationStructureStageOutputs,
) -> Result<Vec<BenchStageResultOutputV1>> {
    let published_stage_root = repo_root.join(&report.output_root).join("artifacts/stage");
    let mut outputs = Vec::new();
    if report.tool_id == "plink" {
        let population_mds = stage_outputs.population_mds.as_ref().ok_or_else(|| {
            anyhow!("governed PLINK population-structure smoke did not expose its MDS output")
        })?;
        let tool_log = stage_outputs.tool_log.as_ref().ok_or_else(|| {
            anyhow!("governed PLINK population-structure smoke did not expose its tool log")
        })?;
        outputs.extend([
            crate::execution::stage_result::ArtifactDeclaration {
                id: "population_mds",
                path: "artifacts/stage/population_structure_mds.mds",
                role: "table_output",
            }
            .observe(
                path_relative_to_repo(
                    repo_root,
                    &published_stage_root.join("population_structure_mds.mds"),
                ),
                population_mds.is_file(),
            ),
            crate::execution::stage_result::ArtifactDeclaration {
                id: "plink_log",
                path: "artifacts/stage/population_structure_mds.log",
                role: "log_output",
            }
            .observe(
                path_relative_to_repo(
                    repo_root,
                    &published_stage_root.join("population_structure_mds.log"),
                ),
                tool_log.is_file(),
            ),
        ]);
    }
    Ok(outputs)
}

fn reported_artifacts(
    repo_root: &Path,
    report: &LocalVcfPopulationStructureSmokeReport,
) -> Vec<crate::execution::stage_result::BenchStageResultOutputV1> {
    let published_stage_root = repo_root.join(&report.output_root).join("artifacts/stage");
    vec![
        crate::execution::stage_result::ArtifactDeclaration {
            id: "population_structure_json",
            path: DEFAULT_OUTPUT_REPORT_NAME,
            role: "report_output",
        }
        .observe(report.population_structure_json_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_population_structure_json",
            path: DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME,
            role: "report_output",
        }
        .observe(report.source_population_structure_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_pruned_variants_tsv",
            path: DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME,
            role: "table_output",
        }
        .observe(report.source_pruned_variants_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_logs_txt",
            path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
            role: "log_output",
        }
        .observe(report.source_logs_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_pca_report_json",
            path: DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME,
            role: "report_output",
        }
        .observe(report.source_pca_report_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "source_admixture_report_json",
            path: DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME,
            role: "report_output",
        }
        .observe(report.source_admixture_report_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "ld_prune_in",
            path: DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME,
            role: "table_output",
        }
        .observe(report.source_pruned_variants_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "ld_prune_out",
            path: "artifacts/stage/ld_prune_out.tsv",
            role: "table_output",
        }
        .observe(
            path_relative_to_repo(repo_root, &published_stage_root.join("ld_prune_out.tsv")),
            true,
        ),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "population_pca_eigenvec",
            path: "artifacts/stage/eigenvec.tsv",
            role: "table_output",
        }
        .observe(
            path_relative_to_repo(repo_root, &published_stage_root.join("eigenvec.tsv")),
            true,
        ),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "population_pca_eigenval",
            path: "artifacts/stage/eigenval.tsv",
            role: "table_output",
        }
        .observe(
            path_relative_to_repo(repo_root, &published_stage_root.join("eigenval.tsv")),
            true,
        ),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "prune_log",
            path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
            role: "log_output",
        }
        .observe(report.source_logs_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "pca_log",
            path: DEFAULT_OUTPUT_SOURCE_LOGS_NAME,
            role: "log_output",
        }
        .observe(report.source_logs_path.clone(), true),
        crate::execution::stage_result::ArtifactDeclaration {
            id: "population_structure_report",
            path: DEFAULT_OUTPUT_REPORT_NAME,
            role: "report_output",
        }
        .observe(report.population_structure_json_path.clone(), true),
    ]
}
