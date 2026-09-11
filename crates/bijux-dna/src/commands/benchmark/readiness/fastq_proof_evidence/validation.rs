use super::super::{
    fastq_comparable_metrics::{
        render_fastq_comparable_metrics, FastqComparableMetricContractStatus,
        DEFAULT_FASTQ_COMPARABLE_METRICS_PATH,
    },
    rendered_command_argv::{render_command_argv, DEFAULT_RENDERED_COMMAND_ARGV_PATH},
};
use crate::commands::benchmark::local_stage_commands::materialize_local_stage;
use anyhow::Result;
use bijux_dna_bench_fastq::proofs::inputs::{
    observations::{CommandArgvObservation, ComparableContract, ComparableMetricObservation},
    ports::ValidationEvidence,
};
use std::path::Path;
pub(super) struct ValidationAdapter;
impl ValidationEvidence for ValidationAdapter {
    fn metrics(&self, root: &Path) -> Result<Vec<ComparableMetricObservation>> {
        Ok(render_fastq_comparable_metrics(root, DEFAULT_FASTQ_COMPARABLE_METRICS_PATH.into())?
            .rows
            .into_iter()
            .map(|row| ComparableMetricObservation {
                stage_id: row.stage_id,
                shared_metric_fields: row.shared_metric_fields,
                comparison_contract_status: match row.comparison_contract_status {
                    FastqComparableMetricContractStatus::Declared => ComparableContract::Declared,
                    FastqComparableMetricContractStatus::MissingSharedMetrics => {
                        ComparableContract::MissingSharedMetrics
                    }
                },
            })
            .collect())
    }
    fn argv(&self, root: &Path) -> Result<Vec<CommandArgvObservation>> {
        Ok(render_command_argv(root, DEFAULT_RENDERED_COMMAND_ARGV_PATH.into())?
            .rows
            .into_iter()
            .map(|row| CommandArgvObservation {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                readiness_kind: row.readiness_kind,
                argv: row.argv,
            })
            .collect())
    }
    fn materialize(&self, root: &Path, stage_id: &str) -> Result<()> {
        materialize_local_stage(root, stage_id).map(|_| ())
    }
}
