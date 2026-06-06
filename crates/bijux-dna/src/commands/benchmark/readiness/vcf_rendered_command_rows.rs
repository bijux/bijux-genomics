use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::benchmark_command_rows::render_shell_command;
use super::vcf_angsd_adapter::render_vcf_angsd_adapter;
use super::vcf_bcftools_adapter::render_vcf_bcftools_adapter;
use super::vcf_descent_family_adapter::render_vcf_descent_family_adapter;
use super::vcf_eigensoft_adapter::render_vcf_eigensoft_adapter;
use super::vcf_imputation_family_adapter::render_vcf_imputation_family_adapter;
use super::vcf_phasing_family_adapter::render_vcf_phasing_family_adapter;
use super::vcf_plink_family_adapter::render_vcf_plink_family_adapter;
use super::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct VcfRenderedCommandStep {
    pub(crate) step_id: String,
    pub(crate) step_kind: String,
    pub(crate) consumes_previous_stdout: bool,
    pub(crate) argv: Vec<String>,
    pub(crate) command: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct VcfRenderedCommandRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) readiness_kind: String,
    pub(crate) benchmark_status: String,
    pub(crate) command_source: String,
    pub(crate) command_steps: Vec<VcfRenderedCommandStep>,
    pub(crate) script_commands: Vec<String>,
    pub(crate) reason: String,
}

pub(crate) fn collect_vcf_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<VcfRenderedCommandRow>> {
    let canonical_pairs = collect_vcf_tool_serving_map_rows()?
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .map(|row| ((row.stage_id, row.tool_id), "benchmark_ready".to_string()))
        .collect::<Vec<_>>();
    let adapter_rows = collect_vcf_adapter_command_row_map(repo_root)?;

    let mut rows = Vec::with_capacity(canonical_pairs.len());
    for ((stage_id, tool_id), benchmark_status) in canonical_pairs {
        let row = adapter_rows
            .get(&(stage_id.clone(), tool_id.clone()))
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "VCF rendered command rows are missing adapter command coverage for `{stage_id}` / `{tool_id}`"
                )
            })?;
        if row.command_steps.is_empty() {
            return Err(anyhow!(
                "VCF rendered command row `{stage_id}` / `{tool_id}` has no command steps"
            ));
        }
        if row.benchmark_status != benchmark_status {
            return Err(anyhow!(
                "VCF rendered command row `{stage_id}` / `{tool_id}` drifted from canonical benchmark status (`{}` vs `{}`)",
                row.benchmark_status,
                benchmark_status
            ));
        }
        validate_vcf_rendered_command_row(&row)?;
        rows.push(row);
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_unique_vcf_rendered_command_rows(&rows)?;
    Ok(rows)
}

pub(crate) fn collect_vcf_adapter_command_row_map(
    repo_root: &Path,
) -> Result<BTreeMap<(String, String), VcfRenderedCommandRow>> {
    let temp_root = repo_root.join("artifacts/bench-readiness/vcf-rendered-command-rows");
    fs::create_dir_all(&temp_root).with_context(|| format!("create {}", temp_root.display()))?;

    let mut rows = Vec::new();

    let bcftools_report =
        render_vcf_bcftools_adapter(repo_root, temp_root.join("bcftools.adapter.json"))?;
    rows.extend(bcftools_report.rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_bcftools_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: step.consumes_previous_stdout,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));

    let angsd_report = render_vcf_angsd_adapter(repo_root, temp_root.join("angsd.adapter.json"))?;
    rows.extend(angsd_report.rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_angsd_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));

    let eigensoft_report =
        render_vcf_eigensoft_adapter(repo_root, temp_root.join("eigensoft.adapter.json"))?;
    rows.extend(eigensoft_report.rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_eigensoft_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));

    for tool_id in ["plink", "plink2"] {
        let report = render_vcf_plink_family_adapter(
            repo_root,
            tool_id,
            temp_root.join(format!("{tool_id}.adapter.json")),
        )?;
        rows.extend(report.rows.into_iter().map(|row| {
            VcfRenderedCommandRow {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                readiness_kind: "benchmark_ready".to_string(),
                benchmark_status: row.benchmark_status,
                command_source: "vcf_plink_family_adapter".to_string(),
                command_steps: row
                    .command_steps
                    .into_iter()
                    .map(|step| VcfRenderedCommandStep {
                        step_id: step.step_id,
                        step_kind: step.step_kind,
                        consumes_previous_stdout: false,
                        command: render_shell_command(&step.argv),
                        argv: step.argv,
                    })
                    .collect(),
                script_commands: Vec::new(),
                reason: row.reason,
            }
        }));
    }

    for tool_id in ["shapeit5", "eagle", "beagle"] {
        let report = render_vcf_phasing_family_adapter(
            repo_root,
            tool_id,
            temp_root.join(format!("{tool_id}.phasing.adapter.json")),
        )?;
        rows.extend(report.rows.into_iter().map(|row| {
            VcfRenderedCommandRow {
                stage_id: row.stage_id,
                tool_id: row.tool_id,
                readiness_kind: "benchmark_ready".to_string(),
                benchmark_status: row.benchmark_status,
                command_source: "vcf_phasing_family_adapter".to_string(),
                command_steps: row
                    .command_steps
                    .into_iter()
                    .map(|step| VcfRenderedCommandStep {
                        step_id: step.step_id,
                        step_kind: step.step_kind,
                        consumes_previous_stdout: false,
                        command: render_shell_command(&step.argv),
                        argv: step.argv,
                    })
                    .collect(),
                script_commands: Vec::new(),
                reason: row.reason,
            }
        }));
    }

    let imputation_report = render_vcf_imputation_family_adapter(
        repo_root,
        temp_root.join("imputation-family.adapter.json"),
    )?;
    rows.extend(imputation_report.rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_imputation_family_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));

    let descent_report = render_vcf_descent_family_adapter(
        repo_root,
        temp_root.join("descent-family.adapter.json"),
    )?;
    rows.extend(descent_report.rows.into_iter().map(|row| {
        VcfRenderedCommandRow {
            stage_id: row.stage_id,
            tool_id: row.tool_id,
            readiness_kind: "benchmark_ready".to_string(),
            benchmark_status: row.benchmark_status,
            command_source: "vcf_descent_family_adapter".to_string(),
            command_steps: row
                .command_steps
                .into_iter()
                .map(|step| VcfRenderedCommandStep {
                    step_id: step.step_id,
                    step_kind: step.step_kind,
                    consumes_previous_stdout: false,
                    command: render_shell_command(&step.argv),
                    argv: step.argv,
                })
                .collect(),
            script_commands: Vec::new(),
            reason: row.reason,
        }
    }));

    let mut row_map: BTreeMap<(String, String), VcfRenderedCommandRow> = BTreeMap::new();
    for mut row in rows {
        row.script_commands = render_script_commands(&row.command_steps)?;
        let key = (row.stage_id.clone(), row.tool_id.clone());
        if let Some(existing) = row_map.get(&(key.0.clone(), key.1.clone())) {
            if should_replace_existing_command_source(
                &key.0,
                &key.1,
                &existing.command_source,
                &row.command_source,
            ) {
                row_map.insert(key, row);
                continue;
            }
            return Err(anyhow!(
                "VCF rendered command row map encountered duplicate pair `{}` / `{}`",
                key.0,
                key.1
            ));
        }
        row_map.insert(key, row);
    }
    Ok(row_map)
}

fn render_script_commands(steps: &[VcfRenderedCommandStep]) -> Result<Vec<String>> {
    let mut commands = Vec::new();
    let mut current = String::new();
    let mut have_current = false;

    for step in steps {
        if step.argv.is_empty() {
            return Err(anyhow!(
                "VCF rendered command step `{}` has an empty argv vector",
                step.step_id
            ));
        }
        if step.consumes_previous_stdout {
            if !have_current {
                return Err(anyhow!(
                    "VCF rendered command step `{}` consumes previous stdout but no source command is open",
                    step.step_id
                ));
            }
            current.push_str(" | ");
            current.push_str(&step.command);
            continue;
        }
        if have_current {
            commands.push(current);
            current = String::new();
        }
        current.push_str(&step.command);
        have_current = true;
    }

    if have_current {
        commands.push(current);
    }
    if commands.is_empty() {
        return Err(anyhow!("VCF rendered command row produced no shell commands"));
    }
    Ok(commands)
}

fn validate_vcf_rendered_command_row(row: &VcfRenderedCommandRow) -> Result<()> {
    if row.command_steps.is_empty() {
        return Err(anyhow!(
            "VCF rendered command row `{}` / `{}` has no command steps",
            row.stage_id,
            row.tool_id
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
                    "VCF rendered command row `{}` / `{}` step `{}` has an empty executable",
                    row.stage_id,
                    row.tool_id,
                    step.step_id
                )
            })?;
        let command_lower = step.command.to_ascii_lowercase();
        if command_lower.contains("todo") {
            return Err(anyhow!(
                "VCF rendered command row `{}` / `{}` step `{}` still contains TODO text",
                row.stage_id,
                row.tool_id,
                step.step_id
            ));
        }
        if command_lower.contains("placeholder") {
            return Err(anyhow!(
                "VCF rendered command row `{}` / `{}` step `{}` still contains placeholder text",
                row.stage_id,
                row.tool_id,
                step.step_id
            ));
        }
        if executable.eq_ignore_ascii_case("echo") && command_lower.contains("echo execute") {
            return Err(anyhow!(
                "VCF rendered command row `{}` / `{}` step `{}` still contains `echo execute`",
                row.stage_id,
                row.tool_id,
                step.step_id
            ));
        }
    }
    Ok(())
}

fn ensure_unique_vcf_rendered_command_rows(rows: &[VcfRenderedCommandRow]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for row in rows {
        let key = (row.stage_id.as_str(), row.tool_id.as_str());
        if !seen.insert(key) {
            return Err(anyhow!(
                "VCF rendered command rows encountered duplicate pair `{}` / `{}`",
                row.stage_id,
                row.tool_id
            ));
        }
    }
    Ok(())
}

fn should_replace_existing_command_source(
    stage_id: &str,
    tool_id: &str,
    existing_source: &str,
    candidate_source: &str,
) -> bool {
    stage_id == "vcf.roh"
        && tool_id == "plink2"
        && existing_source == "vcf_plink_family_adapter"
        && candidate_source == "vcf_descent_family_adapter"
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::collect_vcf_rendered_command_rows;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_rendered_command_rows_follow_canonical_benchmark_ready_slice() {
        let rows = collect_vcf_rendered_command_rows(&repo_root())
            .expect("collect VCF rendered command rows");

        assert_eq!(rows.len(), 8);
        assert!(rows.iter().all(|row| row.benchmark_status == "benchmark_ready"));
        assert!(rows.iter().all(|row| !row.script_commands.is_empty()));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.call" && row.tool_id == "bcftools"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.stats" && row.tool_id == "bcftools"));
        assert!(rows.iter().all(|row| row.tool_id == "bcftools"));
    }
}
