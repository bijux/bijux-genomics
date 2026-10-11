use crate::commands::cli::{parse, render};
use anyhow::{Context, Result};
pub(crate) use bijux_dna_bench_fastq::readiness::commands::{
    FastqRenderedCommandRow, FastqRenderedCommandsReport, DEFAULT_FASTQ_RENDERED_COMMANDS_PATH,
};
use std::path::{Path, PathBuf};
pub(crate) fn run_render_fastq_commands(
    args: &parse::BenchReadinessRenderFastqCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_fastq_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_FASTQ_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}
pub(crate) fn render_fastq_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<FastqRenderedCommandsReport> {
    bijux_dna_bench_fastq::readiness::commands::render_fastq_commands(
        &super::fastq_declaration_evidence::commands(repo_root),
        &output_path,
    )
}
pub(crate) fn collect_fastq_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<FastqRenderedCommandRow>> {
    bijux_dna_bench_fastq::readiness::commands::collect_fastq_rendered_command_rows(
        &super::fastq_declaration_evidence::commands(repo_root),
    )
}
#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{render_fastq_commands, DEFAULT_FASTQ_RENDERED_COMMANDS_PATH};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn fastq_rendered_commands_report_tracks_active_fastq_rows() {
        let root = repo_root();
        let report =
            render_fastq_commands(&root, PathBuf::from(DEFAULT_FASTQ_RENDERED_COMMANDS_PATH))
                .expect("render FASTQ commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.fastq_rendered_commands.v1");
        assert_eq!(
            report.output_path,
            "artifacts/benchmark-smoke/readiness/fastq/fastq-rendered-commands.sh"
        );
        assert_eq!(
            report.argv_output_path,
            "artifacts/benchmark-smoke/readiness/fastq/fastq-rendered-commands.argv.jsonl"
        );
        assert_eq!(report.row_count, 71);
        assert_eq!(report.stage_count, 27);
        assert_eq!(report.tool_count, 43);
        assert_eq!(report.command_source_counts.get("fastq_bam_command_adapter"), Some(&71));
        assert!(report.rows.iter().all(|row| {
            row.benchmark_status == "benchmark_ready"
                && row.command_source == "fastq_bam_command_adapter"
                && row.command_steps.len() == 1
                && row.command_steps[0].step_id == "invoke"
                && !row.command_steps[0].argv.is_empty()
                && row.script_commands.len() == 1
                && !row.script_commands[0].trim().is_empty()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.cluster_otus"
                && row.tool_id == "vsearch"
                && row.command_steps[0].argv.first().map(String::as_str) == Some("vsearch")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.screen_taxonomy"
                && row.tool_id == "kraken2"
                && row.asset_profile_id == "database_artifact_id+taxonomy_database_root"
                && row.command_steps[0].argv.first().map(String::as_str) == Some("sh")
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "fastq.infer_asvs"
                && row.tool_id == "dada2"
                && row.command_steps[0].argv.first().map(String::as_str) == Some("dada2")
        }));
        let seqprep_trim = report
            .rows
            .iter()
            .find(|row| row.stage_id == "fastq.trim_reads" && row.tool_id == "seqprep")
            .expect("SeqPrep trim command row");
        let seqprep_script = seqprep_trim.command_steps[0].argv.join(" ");
        assert!(seqprep_script.contains("'seqprep' '-f'"));
        assert!(seqprep_script.contains("'-r'"));
        assert!(seqprep_script.contains("reads_with_trim_signals_R2.fastq"));
        assert!(seqprep_script.contains("trim_report.seqprep.log"));
    }
}
