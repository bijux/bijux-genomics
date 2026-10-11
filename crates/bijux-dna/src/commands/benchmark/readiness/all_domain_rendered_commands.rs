use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::bench::{
    publish_rendered_command_artifacts, validate_all_domain_rendered_command_rows,
    AllDomainExpectedBenchmarkResultRow, ReadinessStageToolBinding as BindingKey,
};
pub(crate) use bijux_dna_api::v1::api::bench::{
    AllDomainRenderedCommandRow, AllDomainRenderedCommandStep, AllDomainRenderedCommandsReport,
};

use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::rendered_commands::collect_rendered_command_rows;
use super::tool_execution_modes::{
    validate_tool_execution_modes_path, DEFAULT_TOOL_EXECUTION_MODES_PATH,
};
use super::vcf_rendered_command_rows::{collect_vcf_rendered_command_rows, VcfRenderedCommandRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/rendered-commands-all-domains.sh";
pub(crate) const DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_ARGV_PATH: &str =
    "artifacts/benchmark-smoke/readiness/all-domains/rendered-commands-all-domains.argv.jsonl";

pub(crate) fn run_render_all_domain_commands(
    args: &parse::BenchReadinessRenderAllDomainCommandsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_commands(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_commands(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainRenderedCommandsReport> {
    let rows = collect_all_domain_rendered_command_rows(repo_root)?;
    publish_rendered_command_artifacts(repo_root, &output_path, rows)
}

pub(crate) fn collect_all_domain_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<AllDomainRenderedCommandRow>> {
    validate_tool_execution_modes_path(
        repo_root,
        &repo_root.join(DEFAULT_TOOL_EXECUTION_MODES_PATH),
    )
    .context("validate governed benchmark tool execution modes before all-domain rendering")?;

    let expected_rows = collect_all_domain_expected_benchmark_result_rows(repo_root)?;
    let expected_by_binding = expected_rows
        .into_iter()
        .map(|row| (binding_key(&row.domain, &row.stage_id, &row.tool_id), row))
        .collect::<BTreeMap<_, _>>();

    let mut rows = Vec::new();

    for row in collect_rendered_command_rows(repo_root)? {
        let domain = stage_domain(&row.stage_id)?;
        let binding = binding_key(domain, &row.stage_id, &row.tool_id);
        let expected = expected_by_binding.get(&binding).ok_or_else(|| {
            anyhow!(
                "all-domain rendered commands are missing expected-result coverage for `{}` / `{}` / `{}`",
                domain,
                row.stage_id,
                row.tool_id
            )
        })?;
        let step = AllDomainRenderedCommandStep {
            step_id: "invoke".to_string(),
            step_kind: "exec".to_string(),
            consumes_previous_stdout: false,
            argv: row.argv.clone(),
            command: row.command.clone(),
        };
        rows.push(AllDomainRenderedCommandRow {
            result_id: expected.result_id.clone(),
            domain: domain.to_string(),
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            corpus_id: expected.corpus_id.clone(),
            asset_profile_id: expected.asset_profile_id.clone(),
            readiness_kind: row.readiness_kind,
            benchmark_status: "benchmark_ready".to_string(),
            command_source: "fastq_bam_command_adapter".to_string(),
            command_steps: vec![step],
            script_commands: vec![row.command],
        });
    }

    for row in collect_vcf_rendered_command_rows(repo_root)? {
        rows.push(vcf_row_to_all_domain_row(&expected_by_binding, row)?);
    }

    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });
    validate_all_domain_rendered_command_rows(&expected_by_binding, &rows)?;
    Ok(rows)
}

fn vcf_row_to_all_domain_row(
    expected_by_binding: &BTreeMap<BindingKey, AllDomainExpectedBenchmarkResultRow>,
    row: VcfRenderedCommandRow,
) -> Result<AllDomainRenderedCommandRow> {
    let binding = binding_key("vcf", &row.stage_id, &row.tool_id);
    let expected = expected_by_binding.get(&binding).ok_or_else(|| {
        anyhow!(
            "all-domain rendered commands are missing expected-result coverage for `vcf` / `{}` / `{}`",
            row.stage_id,
            row.tool_id
        )
    })?;
    Ok(AllDomainRenderedCommandRow {
        result_id: expected.result_id.clone(),
        domain: "vcf".to_string(),
        stage_id: row.stage_id,
        tool_id: row.tool_id,
        corpus_id: expected.corpus_id.clone(),
        asset_profile_id: expected.asset_profile_id.clone(),
        readiness_kind: row.readiness_kind,
        benchmark_status: row.benchmark_status,
        command_source: row.command_source,
        command_steps: row
            .command_steps
            .into_iter()
            .map(|step| AllDomainRenderedCommandStep {
                step_id: step.step_id,
                step_kind: step.step_kind,
                consumes_previous_stdout: step.consumes_previous_stdout,
                argv: step.argv,
                command: step.command,
            })
            .collect(),
        script_commands: row.script_commands,
    })
}

fn stage_domain(stage_id: &str) -> Result<&'static str> {
    if stage_id.starts_with("fastq.") {
        Ok("fastq")
    } else if stage_id.starts_with("bam.") {
        Ok("bam")
    } else if stage_id.starts_with("vcf.") {
        Ok("vcf")
    } else {
        Err(anyhow!("all-domain rendered commands cannot derive a domain from stage `{stage_id}`"))
    }
}

fn binding_key(domain: &str, stage_id: &str, tool_id: &str) -> BindingKey {
    BindingKey {
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use super::{render_all_domain_commands, DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH};
    use crate::commands::benchmark::repo_locking::{acquire_cwd_lock, RepoProcessLock};

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn change_to(path: &PathBuf) -> Self {
            let previous = env::current_dir().expect("current dir");
            env::set_current_dir(path).expect("set current dir");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.previous).expect("restore current dir");
        }
    }

    #[test]
    fn all_domain_rendered_commands_report_tracks_governed_rows() {
        let root = repo_root();
        let _cwd_lock = acquire_cwd_lock();
        let _repo_lock =
            RepoProcessLock::acquire(&root, "benchmark-readiness-mutators").expect("repo lock");
        let _cwd_guard = CurrentDirGuard::change_to(&root);
        let report = render_all_domain_commands(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
        )
        .expect("render all-domain commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.all_domain_rendered_commands.v1");
        assert_eq!(
            report.output_path,
            "artifacts/benchmark-smoke/readiness/all-domains/rendered-commands-all-domains.sh"
        );
        assert_eq!(
            report.argv_output_path,
            "artifacts/benchmark-smoke/readiness/all-domains/rendered-commands-all-domains.argv.jsonl"
        );
        assert_eq!(report.result_id_count, report.row_count);
        assert!(report.domain_counts.contains_key("fastq"));
        assert!(report.domain_counts.contains_key("bam"));
        let vcf_row_count = report.rows.iter().filter(|row| row.domain == "vcf").count();
        assert_eq!(report.domain_counts.get("vcf"), Some(&vcf_row_count));
        assert_eq!(report.benchmark_status_counts.get("benchmark_ready"), Some(&report.row_count));
        let fastq_bam_count = report.domain_counts["fastq"] + report.domain_counts["bam"];
        assert_eq!(
            report.command_source_counts.get("fastq_bam_command_adapter"),
            Some(&fastq_bam_count)
        );
        assert_eq!(report.command_source_counts.get("vcf_bcftools_adapter"), Some(&11));
        assert_eq!(report.command_source_counts.get("vcf_eigensoft_adapter"), Some(&2));
        assert_eq!(report.command_source_counts.get("vcf_imputation_family_adapter"), Some(&8));
        assert_eq!(report.command_source_counts.get("vcf_angsd_adapter"), Some(&2));
        assert_eq!(report.command_source_counts.get("vcf_phasing_family_adapter"), Some(&4));
        assert_eq!(report.command_source_counts.get("vcf_plink_family_adapter"), Some(&7));
        assert_eq!(report.command_source_counts.get("vcf_descent_family_adapter"), Some(&5));
        assert!(report.rows.iter().all(|row| !row.command_steps.is_empty()));
    }
}
