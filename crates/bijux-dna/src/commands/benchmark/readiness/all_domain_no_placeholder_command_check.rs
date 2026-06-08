use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::all_domain_rendered_commands::{
    render_all_domain_commands, AllDomainRenderedCommandRow, AllDomainRenderedCommandStep,
    DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH: &str =
    "benchmarks/readiness/all-domains/no-placeholder-command-check.json";
const ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.all_domain_no_placeholder_command_check.v1";
const FINDING_TODO: &str = "contains_todo";
const FINDING_PLACEHOLDER: &str = "contains_placeholder";
const FINDING_ECHO_ONLY_EXECUTION: &str = "echo_only_execution";
const FINDING_UNCONDITIONAL_SUCCESS: &str = "unconditional_success";
const FINDING_EMPTY_EXECUTABLE: &str = "empty_executable";
const FINDING_MISSING_REAL_INVOCATION: &str = "missing_real_invocation";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainNoPlaceholderCommandStepAudit {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) executable: String,
    pub(crate) is_shell_wrapper: bool,
    pub(crate) command_heads: Vec<String>,
    pub(crate) has_real_invocation: bool,
    pub(crate) finding_types: Vec<String>,
    pub(crate) ok: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct AllDomainNoPlaceholderCommandRow {
    pub(crate) result_id: String,
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) command_source: String,
    pub(crate) command_step_count: usize,
    pub(crate) script_command_count: usize,
    pub(crate) has_real_invocation: bool,
    pub(crate) finding_count: usize,
    pub(crate) finding_types: Vec<String>,
    pub(crate) ok: bool,
    pub(crate) step_audits: Vec<AllDomainNoPlaceholderCommandStepAudit>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AllDomainNoPlaceholderCommandCheckReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) script_output_path: String,
    pub(crate) argv_output_path: String,
    pub(crate) row_count: usize,
    pub(crate) result_id_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) command_step_count: usize,
    pub(crate) shell_wrapped_step_count: usize,
    pub(crate) direct_step_count: usize,
    pub(crate) valid_row_count: usize,
    pub(crate) invalid_row_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) command_source_counts: BTreeMap<String, usize>,
    pub(crate) finding_type_counts: BTreeMap<String, usize>,
    pub(crate) violation_count: usize,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<AllDomainNoPlaceholderCommandRow>,
    pub(crate) violations: Vec<AllDomainNoPlaceholderCommandRow>,
}

pub(crate) fn run_render_all_domain_no_placeholder_command_check(
    args: &parse::BenchReadinessRenderAllDomainNoPlaceholderCommandCheckArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_all_domain_no_placeholder_command_check(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_all_domain_no_placeholder_command_check(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AllDomainNoPlaceholderCommandCheckReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let report = build_all_domain_no_placeholder_command_check_report(repo_root, &output_path)?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let payload = serde_json::to_vec_pretty(&report)
        .context("serialize all-domain no-placeholder command check report")?;
    fs::write(&output_path, payload).with_context(|| format!("write {}", output_path.display()))?;
    if !report.ok {
        return Err(anyhow!(
            "all-domain active commands still contain placeholder execution drift"
        ));
    }
    Ok(report)
}

fn build_all_domain_no_placeholder_command_check_report(
    repo_root: &Path,
    output_path: &Path,
) -> Result<AllDomainNoPlaceholderCommandCheckReport> {
    let rendered_report = render_all_domain_commands(
        repo_root,
        PathBuf::from(DEFAULT_ALL_DOMAIN_RENDERED_COMMANDS_PATH),
    )?;

    let mut rows = rendered_report.rows.iter().map(audit_row).collect::<Result<Vec<_>>>()?;
    rows.sort_by(|left, right| {
        left.domain
            .cmp(&right.domain)
            .then_with(|| left.stage_id.cmp(&right.stage_id))
            .then_with(|| left.tool_id.cmp(&right.tool_id))
            .then_with(|| left.result_id.cmp(&right.result_id))
    });

    let row_count = rows.len();
    let command_step_count = rows.iter().map(|row| row.command_step_count).sum::<usize>();
    let shell_wrapped_step_count = rows
        .iter()
        .flat_map(|row| row.step_audits.iter())
        .filter(|step| step.is_shell_wrapper)
        .count();
    let direct_step_count = command_step_count.saturating_sub(shell_wrapped_step_count);
    let valid_row_count = rows.iter().filter(|row| row.ok).count();
    let invalid_row_count = row_count.saturating_sub(valid_row_count);
    let result_id_count =
        rows.iter().map(|row| row.result_id.as_str()).collect::<BTreeSet<_>>().len();
    let stage_count = rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let tool_count = rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len();
    let mut domain_counts = BTreeMap::<String, usize>::new();
    let mut command_source_counts = BTreeMap::<String, usize>::new();
    let mut finding_type_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
        *command_source_counts.entry(row.command_source.clone()).or_default() += 1;
        for finding_type in &row.finding_types {
            *finding_type_counts.entry(finding_type.clone()).or_default() += 1;
        }
    }
    let violations = rows.iter().filter(|row| !row.ok).cloned().collect::<Vec<_>>();

    let report = AllDomainNoPlaceholderCommandCheckReport {
        schema_version: ALL_DOMAIN_NO_PLACEHOLDER_COMMAND_CHECK_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, output_path),
        script_output_path: rendered_report.output_path,
        argv_output_path: rendered_report.argv_output_path,
        row_count,
        result_id_count,
        stage_count,
        tool_count,
        command_step_count,
        shell_wrapped_step_count,
        direct_step_count,
        valid_row_count,
        invalid_row_count,
        domain_counts,
        command_source_counts,
        finding_type_counts,
        violation_count: violations.len(),
        ok: violations.is_empty(),
        rows,
        violations,
    };
    ensure_all_domain_no_placeholder_command_check_contract(&report)?;
    Ok(report)
}

fn audit_row(row: &AllDomainRenderedCommandRow) -> Result<AllDomainNoPlaceholderCommandRow> {
    let step_audits =
        row.command_steps.iter().map(|step| audit_step(row, step)).collect::<Result<Vec<_>>>()?;
    let mut finding_types = step_audits
        .iter()
        .flat_map(|step| step.finding_types.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    finding_types.sort();

    Ok(AllDomainNoPlaceholderCommandRow {
        result_id: row.result_id.clone(),
        domain: row.domain.clone(),
        stage_id: row.stage_id.clone(),
        tool_id: row.tool_id.clone(),
        corpus_id: row.corpus_id.clone(),
        asset_profile_id: row.asset_profile_id.clone(),
        command_source: row.command_source.clone(),
        command_step_count: row.command_steps.len(),
        script_command_count: row.script_commands.len(),
        has_real_invocation: step_audits.iter().all(|step| step.has_real_invocation),
        finding_count: step_audits.iter().map(|step| step.finding_types.len()).sum(),
        finding_types,
        ok: step_audits.iter().all(|step| step.ok),
        step_audits,
    })
}

fn audit_step(
    row: &AllDomainRenderedCommandRow,
    step: &AllDomainRenderedCommandStep,
) -> Result<AllDomainNoPlaceholderCommandStepAudit> {
    let executable = step.argv.first().map(|value| value.trim().to_string()).unwrap_or_default();
    let is_shell_wrapper = is_shell_wrapper(&executable);
    let command_heads = if is_shell_wrapper {
        extract_shell_command_heads(shell_payload(&step.argv))
    } else {
        Vec::new()
    };
    let has_real_invocation = if executable.is_empty() {
        false
    } else if is_shell_wrapper {
        shell_command_has_real_invocation(&command_heads)
    } else {
        direct_command_has_real_invocation(&executable)
    };

    let mut finding_types = Vec::new();
    let lowered_command = step.command.to_ascii_lowercase();
    let lowered_argv = step.argv.iter().map(|value| value.to_ascii_lowercase()).collect::<Vec<_>>();
    if executable.is_empty() {
        finding_types.push(FINDING_EMPTY_EXECUTABLE.to_string());
    }
    if lowered_command.contains("todo") || lowered_argv.iter().any(|value| value.contains("todo")) {
        finding_types.push(FINDING_TODO.to_string());
    }
    if lowered_command.contains("placeholder")
        || lowered_argv.iter().any(|value| value.contains("placeholder"))
    {
        finding_types.push(FINDING_PLACEHOLDER.to_string());
    }
    if is_echo_only_execution(&executable, shell_payload(&step.argv), &command_heads) {
        finding_types.push(FINDING_ECHO_ONLY_EXECUTION.to_string());
    }
    if is_unconditional_success(
        &executable,
        shell_payload(&step.argv),
        &command_heads,
        &lowered_command,
    ) {
        finding_types.push(FINDING_UNCONDITIONAL_SUCCESS.to_string());
    }
    if !has_real_invocation {
        finding_types.push(FINDING_MISSING_REAL_INVOCATION.to_string());
    }
    finding_types.sort();
    finding_types.dedup();

    if step.step_id.trim().is_empty() || step.step_kind.trim().is_empty() {
        return Err(anyhow!(
            "all-domain no-placeholder command check encountered a malformed step in `{}`",
            row.result_id
        ));
    }

    Ok(AllDomainNoPlaceholderCommandStepAudit {
        step_id: step.step_id.clone(),
        step_kind: step.step_kind.clone(),
        executable,
        is_shell_wrapper,
        command_heads,
        has_real_invocation,
        ok: finding_types.is_empty(),
        finding_types,
    })
}

fn ensure_all_domain_no_placeholder_command_check_contract(
    report: &AllDomainNoPlaceholderCommandCheckReport,
) -> Result<()> {
    if report.row_count != report.rows.len() {
        return Err(anyhow!(
            "all-domain no-placeholder command check report drifted from its row set"
        ));
    }
    if report.violation_count != report.violations.len() {
        return Err(anyhow!(
            "all-domain no-placeholder command check report drifted from its violation set"
        ));
    }
    if report.valid_row_count + report.invalid_row_count != report.row_count {
        return Err(anyhow!(
            "all-domain no-placeholder command check row counts no longer reconcile"
        ));
    }
    let actual_step_count = report.rows.iter().map(|row| row.command_step_count).sum::<usize>();
    if report.command_step_count != actual_step_count {
        return Err(anyhow!(
            "all-domain no-placeholder command check step count drifted from its rows"
        ));
    }
    if report.shell_wrapped_step_count + report.direct_step_count != report.command_step_count {
        return Err(anyhow!(
            "all-domain no-placeholder command check shell/direct step counts no longer reconcile"
        ));
    }
    if report.ok && report.violation_count != 0 {
        return Err(anyhow!(
            "all-domain no-placeholder command check cannot be ok while violations remain"
        ));
    }
    if !report.ok && report.violation_count == 0 {
        return Err(anyhow!(
            "all-domain no-placeholder command check must keep explicit violations when failing"
        ));
    }
    for row in &report.rows {
        if row.result_id.trim().is_empty()
            || row.domain.trim().is_empty()
            || row.stage_id.trim().is_empty()
            || row.tool_id.trim().is_empty()
            || row.command_source.trim().is_empty()
        {
            return Err(anyhow!(
                "all-domain no-placeholder command check row is missing required identity fields"
            ));
        }
        if row.command_step_count != row.step_audits.len() {
            return Err(anyhow!(
                "all-domain no-placeholder command row `{}` drifted from its step audit set",
                row.result_id
            ));
        }
    }
    Ok(())
}

fn is_shell_wrapper(executable: &str) -> bool {
    matches!(basename_lower(executable).as_deref(), Some("sh" | "bash"))
}

fn shell_payload(argv: &[String]) -> &str {
    argv.last().map(|value| value.as_str()).unwrap_or("")
}

fn direct_command_has_real_invocation(executable: &str) -> bool {
    let Some(base) = basename_lower(executable) else {
        return false;
    };
    !matches!(base.as_str(), "" | "echo" | "true" | ":" | "cd" | "mkdir" | "export" | "set")
}

fn shell_command_has_real_invocation(command_heads: &[String]) -> bool {
    command_heads.iter().any(|head| {
        !matches!(head.as_str(), "" | "echo" | "true" | ":" | "cd" | "mkdir" | "export" | "set")
    })
}

fn is_echo_only_execution(executable: &str, payload: &str, command_heads: &[String]) -> bool {
    if matches!(basename_lower(executable).as_deref(), Some("echo")) {
        return true;
    }
    if !is_shell_wrapper(executable) {
        return false;
    }
    if command_heads.is_empty() {
        let trimmed = payload.trim().to_ascii_lowercase();
        return trimmed.starts_with("echo ");
    }
    command_heads.iter().all(|head| head == "echo")
}

fn is_unconditional_success(
    executable: &str,
    payload: &str,
    command_heads: &[String],
    lowered_command: &str,
) -> bool {
    if matches!(basename_lower(executable).as_deref(), Some("true")) {
        return true;
    }
    if lowered_command.trim() == ":" {
        return true;
    }
    if !is_shell_wrapper(executable) {
        return false;
    }
    if command_heads.is_empty() {
        return payload_is_standalone_success(payload);
    }
    payload_is_standalone_success(payload)
}

fn payload_is_standalone_success(payload: &str) -> bool {
    let normalized_lines = payload
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_ascii_lowercase())
        .collect::<Vec<_>>();
    match normalized_lines.as_slice() {
        [] => false,
        [line] => matches!(line.as_str(), ":" | "true" | "exit 0" | "rc=0"),
        [assign, exit] => {
            assign == "rc=0" && matches!(exit.as_str(), "exit 0" | "exit $rc" | "exit \"$rc\"")
        }
        _ => false,
    }
}

fn extract_shell_command_heads(payload: &str) -> Vec<String> {
    let mut heads = Vec::new();
    let mut token = String::new();
    let mut current_head: Option<String> = None;
    let mut in_single = false;
    let mut in_double = false;
    let mut escape_next = false;

    let flush_token = |token: &mut String, current_head: &mut Option<String>| {
        if token.is_empty() {
            return;
        }
        let value = std::mem::take(token);
        if current_head.is_none() && !is_shell_control_token(&value) && !is_shell_assignment(&value)
        {
            *current_head = Some(value);
        }
    };
    let flush_segment = |heads: &mut Vec<String>, current_head: &mut Option<String>| {
        if let Some(head) = current_head.take() {
            heads.push(normalize_command_head(&head));
        }
    };

    for ch in payload.chars() {
        if escape_next {
            token.push(ch);
            escape_next = false;
            continue;
        }
        if ch == '\\' && !in_single {
            escape_next = true;
            continue;
        }
        if ch == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if !in_single && !in_double {
            if matches!(ch, ' ' | '\t' | '\r') {
                flush_token(&mut token, &mut current_head);
                continue;
            }
            if matches!(ch, '\n' | ';' | '|' | '&') {
                flush_token(&mut token, &mut current_head);
                flush_segment(&mut heads, &mut current_head);
                continue;
            }
        }
        token.push(ch);
    }
    flush_token(&mut token, &mut current_head);
    flush_segment(&mut heads, &mut current_head);
    heads
}

fn is_shell_control_token(token: &str) -> bool {
    let token = token.trim();
    token.is_empty()
        || matches!(
            token,
            "if" | "then"
                | "else"
                | "fi"
                | "do"
                | "done"
                | "for"
                | "while"
                | "case"
                | "esac"
                | "in"
                | "["
                | "]"
                | "test"
                | "{"
                | "}"
                | "function"
        )
        || token.ends_with("()")
        || token.ends_with("(){")
}

fn is_shell_assignment(token: &str) -> bool {
    let Some((name, _value)) = token.split_once('=') else {
        return false;
    };
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn normalize_command_head(head: &str) -> String {
    basename_lower(head).unwrap_or_else(|| head.trim().to_ascii_lowercase())
}

fn basename_lower(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let candidate = trimmed.rsplit('/').next().unwrap_or(trimmed);
    Some(candidate.to_ascii_lowercase())
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
