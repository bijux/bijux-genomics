use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_expected_benchmark_results::collect_all_domain_expected_benchmark_result_rows;
use super::rendered_commands::collect_rendered_command_rows;
use super::tool_execution_modes::{
    validate_tool_execution_modes_path, DEFAULT_TOOL_EXECUTION_MODES_PATH,
};
use super::vcf_rendered_command_rows::{collect_vcf_rendered_command_rows, VcfRenderedCommandRow};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH: &str =
    "benchmarks/readiness/rendered-commands-all-domains.sh";
const ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_rendered_commands.v1";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct BindingKey {
    domain: String,
    stage_id: String,
    tool_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainRenderedCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) argv: Vec<String>,
    pub(crate) command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainRenderedCommandRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<AllDomainRenderedCommandStep>,
    pub(crate) script_commands: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainRenderedCommandArgvStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainRenderedCommandArgvRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<AllDomainRenderedCommandArgvStep>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainRenderedCommandsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) argv_output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) benchmark_status_counts: BTreeMap<String, usize>,
    pub(crate) command_source_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<AllDomainRenderedCommandRow>,
}

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
    let output_path = repo_relative_path(repo_root, &output_path);
    let argv_output_path = companion_argv_output_path(&output_path);
    let rows = collect_all_domain_rendered_command_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    if let Some(parent) = argv_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let rendered_script = render_all_domain_commands_shell_script(&rows);
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered_script.as_bytes())?;
    let argv_jsonl = render_all_domain_command_argv_jsonl(&rows)
        .context("render all-domain command argv JSONL")?;
    bijux_dna_infra::atomic_write_bytes(&argv_output_path, argv_jsonl.as_bytes())?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut benchmark_status_counts = BTreeMap::<String, usize>::new();
    let mut command_source_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *benchmark_status_counts.entry(row.benchmark_status.clone()).or_default() += 1;
        *command_source_counts.entry(row.command_source.clone()).or_default() += 1;
    }

    Ok(AllDomainRenderedCommandsReport {
        schema_version: ALL_DOMAIN_RENDERED_COMMANDS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        argv_output_path: path_relative_to_repo(repo_root, &argv_output_path),
        row_count: rows.len(),
        result_id_count: rows
            .iter()
            .map(|row| row.result_id.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        domain_counts,
        benchmark_status_counts,
        command_source_counts,
        rows,
    })
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
        .map(|row| {
            (
                binding_key(&row.domain, &row.stage_id, &row.tool_id),
                ExpectedBindingRow {
                    result_id: row.result_id,
                    corpus_id: row.corpus_id,
                    asset_profile_id: row.asset_profile_id,
                },
            )
        })
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
    ensure_all_domain_rendered_command_contract(&expected_by_binding, &rows)?;
    Ok(rows)
}

#[derive(Debug, Clone)]
struct ExpectedBindingRow {
    result_id: String,
    corpus_id: String,
    asset_profile_id: String,
}

fn vcf_row_to_all_domain_row(
    expected_by_binding: &BTreeMap<BindingKey, ExpectedBindingRow>,
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

fn ensure_all_domain_rendered_command_contract(
    expected_by_binding: &BTreeMap<BindingKey, ExpectedBindingRow>,
    rows: &[AllDomainRenderedCommandRow],
) -> Result<()> {
    if rows.len() != expected_by_binding.len() {
        return Err(anyhow!(
            "all-domain rendered commands must keep exactly one row per benchmark-ready result binding (expected {}, found {})",
            expected_by_binding.len(),
            rows.len()
        ));
    }

    let mut seen_result_ids = BTreeSet::<&str>::new();
    let mut seen_bindings = BTreeSet::<BindingKey>::new();
    for row in rows {
        if row.result_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.corpus_id.trim().is_empty()
            || row.asset_profile_id.trim().is_empty()
            || row.readiness_kind.trim().is_empty()
            || row.benchmark_status.trim().is_empty()
            || row.command_source.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain rendered command row `{}` is missing required columns",
                row.result_id
            ));
        }
        if !seen_result_ids.insert(row.result_id.as_str()) {
            return Err(anyhow!(
                "all-domain rendered commands contain duplicate result_id `{}`",
                row.result_id
            ));
        }
        if !seen_bindings.insert(binding_key(&row.domain, &row.stage_id, &row.tool_id)) {
            return Err(anyhow!(
                "all-domain rendered commands contain duplicate `{}` / `{}` / `{}` bindings",
                row.domain,
                row.stage_id,
                row.tool_id
            ));
        }
        if row.benchmark_status != "benchmark_ready" {
            return Err(anyhow!(
                "all-domain rendered command row `{}` drifted from the benchmark-ready slice",
                row.result_id
            ));
        }
        if row.command_steps.is_empty() || row.script_commands.is_empty() {
            return Err(anyhow!(
                "all-domain rendered command row `{}` must keep command steps and shell commands",
                row.result_id
            ));
        }
        for step in &row.command_steps {
            let executable = step
                .argv
                .first()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    anyhow!(
                        "all-domain rendered command row `{}` step `{}` has an empty executable",
                        row.result_id,
                        step.step_id
                    )
                })?;
            let command_lower = step.command.to_ascii_lowercase();
            if command_lower.contains("todo")
                || command_lower.contains("placeholder")
                || (executable.eq_ignore_ascii_case("echo")
                    && command_lower.contains("echo execute"))
            {
                return Err(anyhow!(
                    "all-domain rendered command row `{}` step `{}` still contains placeholder execution text",
                    row.result_id,
                    step.step_id
                ));
            }
        }
    }

    let missing = expected_by_binding
        .iter()
        .filter(|(binding, expected)| {
            !rows.iter().any(|row| {
                row.result_id == expected.result_id
                    && row.domain == binding.domain
                    && row.stage_id == binding.stage_id
                    && row.tool_id == binding.tool_id
            })
        })
        .map(|(_, expected)| expected.result_id.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(anyhow!(
            "all-domain rendered commands are missing governed result ids: {}",
            missing.join(", ")
        ));
    }

    if rows.len() != expected_by_binding.len() {
        return Err(anyhow!(
            "all-domain rendered commands must retain exactly one benchmark-ready row per expected binding (expected {}, found {})",
            expected_by_binding.len(),
            rows.len(),
        ));
    }

    require_rendered_row(
        rows,
        "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
        "fastq",
        "fastq.screen_taxonomy",
        "kraken2",
        "fastq_bam_command_adapter",
    )?;
    require_rendered_row(
        rows,
        "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king",
        "bam",
        "bam.kinship",
        "king",
        "fastq_bam_command_adapter",
    )?;
    require_rendered_row(
        rows,
        "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
        "vcf",
        "vcf.call",
        "bcftools",
        "vcf_bcftools_adapter",
    )?;

    Ok(())
}

fn require_rendered_row(
    rows: &[AllDomainRenderedCommandRow],
    result_id: &str,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    command_source: &str,
) -> Result<()> {
    let row = rows
        .iter()
        .find(|row| row.result_id == result_id)
        .ok_or_else(|| anyhow!("all-domain rendered commands are missing `{result_id}`"))?;
    if row.domain != domain
        || row.stage_id != stage_id
        || row.tool_id != tool_id
        || row.command_source != command_source
        || row.command_steps.is_empty()
        || row.script_commands.is_empty()
    {
        return Err(anyhow!(
            "all-domain rendered command row `{result_id}` drifted from the governed contract"
        ));
    }
    Ok(())
}

fn render_all_domain_commands_shell_script(rows: &[AllDomainRenderedCommandRow]) -> String {
    let mut rendered = String::from("#!/usr/bin/env bash\nset -euo pipefail\n");
    rendered.push_str("repo_root=\"$(cd \"$(dirname \"${BASH_SOURCE[0]}\")/../..\" && pwd)\"\n");
    rendered.push_str("cd \"$repo_root\"\n\n");
    for (index, row) in rows.iter().enumerate() {
        rendered.push_str(&format!(
            "# {} / {} / {} / {}\n",
            row.result_id, row.domain, row.stage_id, row.tool_id
        ));
        for command in &row.script_commands {
            rendered.push_str(command);
            rendered.push('\n');
        }
        if index + 1 < rows.len() {
            rendered.push('\n');
        }
    }
    rendered
}

fn render_all_domain_command_argv_jsonl(rows: &[AllDomainRenderedCommandRow]) -> Result<String> {
    let mut rendered = String::new();
    for row in rows {
        let line = serde_json::to_string(&AllDomainRenderedCommandArgvRow {
            result_id: row.result_id.clone(),
            domain: row.domain.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            corpus_id: row.corpus_id.clone(),
            asset_profile_id: row.asset_profile_id.clone(),
            readiness_kind: row.readiness_kind.clone(),
            benchmark_status: row.benchmark_status.clone(),
            command_source: row.command_source.clone(),
            command_steps: row
                .command_steps
                .iter()
                .map(|step| AllDomainRenderedCommandArgvStep {
                    step_id: step.step_id.clone(),
                    step_kind: step.step_kind.clone(),
                    consumes_previous_stdout: step.consumes_previous_stdout,
                    argv: step.argv.clone(),
                })
                .collect(),
        })
        .context("serialize all-domain rendered command argv row")?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
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

fn companion_argv_output_path(output_path: &Path) -> PathBuf {
    let file_name = output_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH);
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::PathBuf;

    use super::{render_all_domain_commands, DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
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
        let _cwd_guard = CurrentDirGuard::change_to(&root);
        let report = render_all_domain_commands(
            &root,
            PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
        )
        .expect("render all-domain commands");

        assert_eq!(report.schema_version, "bijux.bench.readiness.all_domain_rendered_commands.v1");
        assert_eq!(report.output_path, "benchmarks/readiness/rendered-commands-all-domains.sh");
        assert_eq!(
            report.argv_output_path,
            "benchmarks/readiness/rendered-commands-all-domains.argv.jsonl"
        );
        assert_eq!(report.result_id_count, report.row_count);
        assert_eq!(report.domain_counts.get("fastq"), Some(&63));
        assert_eq!(report.domain_counts.get("bam"), Some(&49));
        assert_eq!(report.domain_counts.get("vcf"), Some(&19));
        assert_eq!(report.benchmark_status_counts.get("benchmark_ready"), Some(&report.row_count));
        assert_eq!(report.command_source_counts.get("fastq_bam_command_adapter"), Some(&112));
        assert_eq!(report.command_source_counts.get("vcf_bcftools_adapter"), Some(&11));
        assert_eq!(report.command_source_counts.get("vcf_eigensoft_adapter"), Some(&1));
        assert_eq!(report.command_source_counts.get("vcf_imputation_family_adapter"), Some(&2));
        assert_eq!(report.command_source_counts.get("vcf_phasing_family_adapter"), Some(&1));
        assert_eq!(report.command_source_counts.get("vcf_plink_family_adapter"), Some(&4));
        assert!(report.rows.iter().all(|row| !row.command_steps.is_empty()));
    }
}
