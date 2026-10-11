use super::super::{
    all_domain_active_stage_tool_matrix::{
        render_all_domain_active_stage_tool_matrix,
        DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH,
    },
    bam_rendered_commands::{render_bam_commands, DEFAULT_BAM_RENDERED_COMMANDS_PATH},
    expected_benchmark_results::{
        render_expected_benchmark_results, DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH,
    },
};
use super::BamEvidenceAdapter;
use anyhow::Result;
use bijux_dna_bench_bam_evidence::{
    observations::{BindingKey, BindingSurface, CommandSurfaces},
    ports::ConsistencyEvidence,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
fn observe<'a>(
    output_path: String,
    rows: impl Iterator<Item = (&'a str, &'a str)>,
) -> BindingSurface {
    let bindings = rows
        .map(|(stage_id, tool_id)| BindingKey {
            stage_id: stage_id.to_string(),
            tool_id: tool_id.to_string(),
        })
        .collect::<Vec<_>>();
    let stage_count = bindings.iter().map(|key| &key.stage_id).collect::<BTreeSet<_>>().len();
    let tool_count = bindings.iter().map(|key| &key.tool_id).collect::<BTreeSet<_>>().len();
    BindingSurface { output_path, row_count: bindings.len(), stage_count, tool_count, bindings }
}
impl ConsistencyEvidence for BamEvidenceAdapter {
    fn publish_active_bindings(&self, repo_root: &Path) -> Result<BindingSurface> {
        let report = render_all_domain_active_stage_tool_matrix(
            repo_root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_PATH),
        )?;
        Ok(observe(
            report.output_path,
            report
                .rows
                .iter()
                .filter(|row| row.domain == "bam")
                .map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
        ))
    }
    fn publish_commands(&self, repo_root: &Path) -> Result<CommandSurfaces> {
        let report =
            render_bam_commands(repo_root, PathBuf::from(DEFAULT_BAM_RENDERED_COMMANDS_PATH))?;
        let mut commands = observe(
            report.output_path,
            report.rows.iter().map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
        );
        commands.row_count = report.row_count;
        commands.stage_count = report.stage_count;
        commands.tool_count = report.tool_count;
        Ok(CommandSurfaces { commands, argv_output_path: report.argv_output_path })
    }
    fn publish_expected_results(&self, repo_root: &Path) -> Result<BindingSurface> {
        let report = render_expected_benchmark_results(
            repo_root,
            PathBuf::from(DEFAULT_EXPECTED_BENCHMARK_RESULTS_PATH),
        )?;
        Ok(observe(
            report.output_path,
            report
                .rows
                .iter()
                .filter(|row| row.domain == "bam")
                .map(|row| (row.stage_id.as_str(), row.tool_id.as_str())),
        ))
    }
}
