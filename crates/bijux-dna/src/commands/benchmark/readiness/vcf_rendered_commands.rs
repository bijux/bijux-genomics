use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
use bijux_dna_bench_vcf_evidence::commands::projection::{
    render_vcf_command_argv_jsonl, render_vcf_commands_shell_script,
};
pub(crate) use bijux_dna_bench_vcf_evidence::model::command_reports::VcfRenderedCommandsReport;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use super::vcf_rendered_command_rows::collect_vcf_rendered_command_rows_with_observations;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_RENDERED_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-rendered-commands.sh";
const VCF_RENDERED_COMMANDS_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_rendered_commands.v1";

pub(crate) fn run_render_vcf_commands(
    args: &parse::BenchReadinessRenderVcfCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_commands(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfRenderedCommandsReport> {
    let inputs = super::vcf_adapter_inputs::load(repo_root)?;
    let observations = VcfAdapterObservations::new(&inputs, repo_root);
    let expected=super::vcf_expected_benchmark_results::collect_vcf_expected_benchmark_result_rows_with_observations(&inputs,&observations)?;
    let retained=super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows_from_expected(repo_root,expected)?;
    render_vcf_commands_with_observations(repo_root, output_path, &observations, &retained)
}

pub(crate) fn render_vcf_commands_with_observations(
    repo_root: &Path,
    output_path: PathBuf,
    observations: &VcfAdapterObservations<'_>,
    retained: &[bijux_dna_bench_vcf_evidence::model::scope::VcfActiveStageToolMatrixRow],
) -> Result<VcfRenderedCommandsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let argv_output_path = companion_argv_output_path(&output_path);
    let rows = collect_vcf_rendered_command_rows_with_observations(observations, retained)?;
    let active_row_count = retained.iter().filter(|row| row.scope_state == "active").count();
    if rows.len() != active_row_count {
        bail!(
            "VCF rendered commands must cover every active VCF row (expected {active_row_count}, found {})",
            rows.len()
        );
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = argv_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let rendered_script = render_vcf_commands_shell_script(
        &rows,
        &repo_root_relative_to_output(repo_root, &output_path),
    );
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered_script.as_bytes())?;
    let argv_jsonl =
        render_vcf_command_argv_jsonl(&rows).context("render VCF command argv JSONL")?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv_jsonl.as_bytes())?;

    Ok(VcfRenderedCommandsReport {
        schema_version: VCF_RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        row_count: rows.len(),
        rows,
    })
}

fn companion_argv_output_path(output_path: &Path) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_VCF_RENDERED_COMMANDS_PATH);
    let argv_name = if let Some(stem) = file_name.strip_suffix(".sh") {
        format!("{stem}.argv.jsonl")
    } else {
        format!("{file_name}.argv.jsonl")
    };
    output_path.with_file_name(argv_name)
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn repo_root_relative_to_output(repo_root: &Path, output_path: &Path) -> String {
    let relative_output_path = output_path.strip_prefix(repo_root).unwrap_or(output_path);
    let depth = relative_output_path.parent().map_or(0, |parent| parent.components().count());
    if depth == 0 {
        ".".to_string()
    } else {
        std::iter::repeat_n("..", depth).collect::<Vec<_>>().join("/")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::commands::benchmark::readiness::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;

    use super::{render_vcf_commands, DEFAULT_VCF_RENDERED_COMMANDS_PATH};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn vcf_rendered_commands_report_tracks_canonical_rows() {
        let root = repo_root();
        let report = render_vcf_commands(&root, PathBuf::from(DEFAULT_VCF_RENDERED_COMMANDS_PATH))
            .expect("render VCF commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.vcf_rendered_commands.v1");
        assert_eq!(
            report.output_path,
            "artifacts/benchmark-smoke/readiness/vcf/vcf-rendered-commands.sh"
        );
        assert_eq!(
            report.argv_output_path,
            "artifacts/benchmark-smoke/readiness/vcf/vcf-rendered-commands.argv.jsonl"
        );
        let benchmark_ready_row_count = collect_vcf_tool_serving_map_rows()
            .expect("collect VCF tool-serving rows")
            .into_iter()
            .filter(|row| row.benchmark_status == "benchmark_ready")
            .count();
        assert_eq!(report.row_count, benchmark_ready_row_count);
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.prepare_reference_panel" && row.tool_id == "bcftools"
        }));
        assert!(report
            .rows
            .iter()
            .any(|row| { row.stage_id == "vcf.impute" && row.tool_id == "beagle" }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.admixture"
                && row.tool_id == "plink2"
                && row.command_steps.len() == 1
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.population_structure"
                && row.tool_id == "plink2"
                && !row.command_steps.is_empty()
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.pca" && row.tool_id == "plink2" && row.command_steps.len() == 1
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.pca" && row.tool_id == "eigensoft" && row.command_steps.len() == 4
        }));
        assert!(report.rows.iter().any(|row| row.stage_id == "vcf.qc" && row.tool_id == "plink"));
        assert!(report.rows.iter().any(|row| row.stage_id == "vcf.qc" && row.tool_id == "plink2"));
    }
}
