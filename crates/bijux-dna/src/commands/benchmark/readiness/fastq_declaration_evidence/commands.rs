use super::super::{
    fastq_active_stage_tool_matrix::collect_fastq_active_stage_tool_matrix_rows,
    rendered_commands::collect_rendered_command_rows,
};
use anyhow::Result;
use bijux_dna_bench_fastq::readiness::{
    active::FastqActiveStageToolMatrixCollection,
    declarations::{observations::RenderedCommandObservation, ports::CommandEvidence},
};
use std::path::Path;
pub(super) struct CommandAdapter;
impl CommandEvidence for CommandAdapter {
    fn active(&self, root: &Path) -> Result<FastqActiveStageToolMatrixCollection> {
        collect_fastq_active_stage_tool_matrix_rows(root)
    }
    fn rendered(&self, root: &Path) -> Result<Vec<RenderedCommandObservation>> {
        Ok(collect_rendered_command_rows(root)?
            .into_iter()
            .map(|row| RenderedCommandObservation {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                readiness_kind: row.readiness_kind,
                argv: row.argv,
                command: row.command,
            })
            .collect())
    }
}
