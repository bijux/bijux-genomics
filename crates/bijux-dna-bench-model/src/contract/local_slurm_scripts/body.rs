//! Owner: bijux-dna-bench-model
//! Admission and construction of script-text inspection evidence.
use crate::{
    BenchmarkLocalSlurmScriptBodyEntry, BenchmarkLocalSlurmScriptBodyReport,
    BENCHMARK_LOCAL_SLURM_SCRIPT_BODY_SCHEMA_V1,
};
use anyhow::{ensure, Result};
use std::collections::BTreeSet;
use std::path::{Component, Path};

/// Build an inspection report without acquiring files or executing shell commands.
///
/// # Errors
/// Rejects malformed paths or inconsistent entry evidence.
pub fn build_benchmark_local_slurm_script_body_report(
    root_path: String,
    report_path: String,
    scripts: Vec<BenchmarkLocalSlurmScriptBodyEntry>,
) -> Result<BenchmarkLocalSlurmScriptBodyReport> {
    let report_findings = if scripts.is_empty() {
        vec![format!("no .sbatch files found under {root_path}")]
    } else {
        Vec::new()
    };
    let findings_count = scripts.iter().try_fold(report_findings.len(), |count, entry| {
        count
            .checked_add(entry.findings.len())
            .ok_or_else(|| anyhow::anyhow!("script finding count overflow"))
    })?;
    let report = BenchmarkLocalSlurmScriptBodyReport {
        schema_version: BENCHMARK_LOCAL_SLURM_SCRIPT_BODY_SCHEMA_V1.to_string(),
        root_path,
        report_path,
        script_count: scripts.len(),
        findings_count,
        ok: findings_count == 0,
        report_findings,
        scripts,
    };
    validate_benchmark_local_slurm_script_body_report(&report)?;
    Ok(report)
}

/// Admit wire identity, safe paths, unique entries and exact findings/verdict closure.
///
/// # Errors
/// Rejects unsupported versions, inconsistent counters or malformed inspection evidence.
pub fn validate_benchmark_local_slurm_script_body_report(
    report: &BenchmarkLocalSlurmScriptBodyReport,
) -> Result<()> {
    ensure!(
        report.schema_version == BENCHMARK_LOCAL_SLURM_SCRIPT_BODY_SCHEMA_V1,
        "unsupported Slurm script body schema"
    );
    require_path(&report.root_path)?;
    require_path(&report.report_path)?;
    ensure!(report.script_count == report.scripts.len(), "script count disagrees with entries");
    let expected_report_findings = if report.scripts.is_empty() {
        vec![format!("no .sbatch files found under {}", report.root_path)]
    } else {
        Vec::new()
    };
    ensure!(
        report.report_findings == expected_report_findings,
        "script report findings disagree with discovery"
    );
    let mut paths = BTreeSet::new();
    let mut findings_count = report.report_findings.len();
    for entry in &report.scripts {
        require_path(&entry.script_path)?;
        let path = Path::new(&entry.script_path);
        ensure!(
            path.starts_with(&report.root_path)
                && path.extension().is_some_and(|extension| extension == "sbatch"),
            "script lies outside the declared inventory"
        );
        ensure!(paths.insert(&entry.script_path), "script path is duplicated");
        ensure!(
            entry.findings.iter().all(|finding| !finding.trim().is_empty()),
            "script finding is blank"
        );
        ensure!(entry.ok == entry.findings.is_empty(), "script verdict disagrees with findings");
        ensure!(
            entry.has_bijux_dna_command
                != entry.findings.iter().any(|finding| finding == "missing `bijux-dna` command"),
            "script command observation disagrees with findings"
        );
        findings_count = findings_count
            .checked_add(entry.findings.len())
            .ok_or_else(|| anyhow::anyhow!("script finding count overflow"))?;
    }
    ensure!(
        report.findings_count == findings_count && report.ok == (findings_count == 0),
        "script report verdict or finding aggregate disagrees"
    );
    Ok(())
}

fn require_path(value: &str) -> Result<()> {
    ensure!(
        !value.trim().is_empty()
            && !value.contains('\\')
            && Path::new(value).components().all(|part| matches!(part, Component::Normal(_))),
        "script evidence requires a safe repository-relative path"
    );
    Ok(())
}

/// Inspect text using the governed placeholder and command-marker policy.
/// This is a lexical check, not shell parsing or proof that a tool executed.
#[must_use]
pub fn inspect_benchmark_local_slurm_script_body(
    script_path: String,
    body: &str,
) -> BenchmarkLocalSlurmScriptBodyEntry {
    let mut findings = Vec::new();
    let mut has_bijux_dna_command = false;
    let mut has_executable_command = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lowered = trimmed.to_ascii_lowercase();
        if lowered.contains("todo") {
            findings.push("contains a to-do marker".to_string());
        }
        if trimmed.starts_with('#') {
            continue;
        }
        if lowered.contains("bijux-dna") {
            has_bijux_dna_command = true;
        }
        if lowered.contains("placeholder") {
            findings.push("contains `placeholder`".to_string());
        }
        if lowered == "rc=0" {
            findings.push("contains unconditional `rc=0`".to_string());
        }
        if lowered.starts_with("echo execute")
            || lowered.starts_with("echo \"execute")
            || lowered.starts_with("echo 'execute")
        {
            findings.push("contains fake `echo execute` job body".to_string());
        }
        if !is_slurm_boilerplate_line(trimmed) {
            has_executable_command = true;
        }
    }

    if !has_bijux_dna_command {
        findings.push("missing `bijux-dna` command".to_string());
    }
    if !has_executable_command {
        findings.push("contains empty command body".to_string());
    }

    BenchmarkLocalSlurmScriptBodyEntry {
        script_path,
        has_bijux_dna_command,
        ok: findings.is_empty(),
        findings,
    }
}

fn is_slurm_boilerplate_line(line: &str) -> bool {
    if line == "set -euo pipefail" || line.starts_with("cd ") || line.starts_with("mkdir -p ") {
        return true;
    }
    if line.strip_prefix("export ").is_some_and(is_shell_assignment) || is_shell_assignment(line) {
        return true;
    }
    false
}

fn is_shell_assignment(line: &str) -> bool {
    let Some((name, _value)) = line.split_once('=') else {
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
