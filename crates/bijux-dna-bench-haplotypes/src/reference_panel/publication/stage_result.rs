//! Construct the persisted stage result from declared and realized outputs.
use crate::reference_panel::model::GovernedVcfPrepareReferencePanelSmokeContract;
use crate::stage_result::path_relative_to_repo;
use crate::stage_result::BenchStageResultCommandV1;
use crate::stage_result::BenchStageResultManifestV1;
use crate::stage_result::BenchStageResultOutputV1;
use crate::stage_result::BenchStageResultResourceMetricSource;
use crate::stage_result::BenchStageResultResourceMetricsV1;
use crate::stage_result::BenchStageResultRuntimeV1;
use crate::stage_result::BenchStageResultStatus;
use crate::stage_result::BenchStageResultToolV1;
use crate::stage_result::BENCH_STAGE_RESULT_SCHEMA_VERSION;
use std::path::Path;

pub(crate) fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfPrepareReferencePanelSmokeContract,
    command: &str,
    output_entries: &[(&str, String, &Path, &str)],
    started_at: &str,
    finished_at: &str,
    elapsed_seconds: f64,
) -> BenchStageResultManifestV1 {
    BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: contract.stage.clone(),
        tool: BenchStageResultToolV1 { id: contract.tool.clone() },
        command: BenchStageResultCommandV1 { rendered: command.to_string() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: started_at.to_string(),
            finished_at: finished_at.to_string(),
            elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: output_entries
            .iter()
            .map(|(artifact_id, declared_path, realized_path, role)| BenchStageResultOutputV1 {
                artifact_id: (*artifact_id).to_string(),
                declared_path: declared_path.clone(),
                realized_path: path_relative_to_repo(repo_root, realized_path),
                role: (*role).to_string(),
                optional: false,
                exists: true,
            })
            .collect(),
    }
}
