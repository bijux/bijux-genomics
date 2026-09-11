use bijux_dna_bench_vcf_evidence::commands::admission::{
    ensure_unique_vcf_rendered_command_rows, render_script_commands,
    validate_vcf_rendered_command_row,
};
pub(crate) use bijux_dna_bench_vcf_evidence::model::commands::{
    VcfRenderedCommandRow, VcfRenderedCommandStep,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::vcf_active_stage_tool_matrix::collect_vcf_active_stage_tool_matrix_rows;
use super::vcf_angsd_adapter::render_vcf_angsd_adapter;
use super::vcf_bcftools_adapter::render_vcf_bcftools_adapter;
use super::vcf_descent_family_adapter::render_vcf_descent_family_adapter;
use super::vcf_eigensoft_adapter::render_vcf_eigensoft_adapter;
use super::vcf_imputation_family_adapter::render_vcf_imputation_family_adapter;
use super::vcf_phasing_family_adapter::render_vcf_phasing_family_adapter;
use super::vcf_plink_family_adapter::render_vcf_plink_family_adapter;
use bijux_dna_bench_vcf_evidence::commands::projection::render_shell_command;

pub(crate) fn collect_vcf_rendered_command_rows(
    repo_root: &Path,
) -> Result<Vec<VcfRenderedCommandRow>> {
    let canonical_pairs = collect_vcf_active_stage_tool_matrix_rows(repo_root)?
        .into_iter()
        .filter(|row| row.scope_state == "active")
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

    for tool_id in ["shapeit", "shapeit5", "eagle", "beagle"] {
        let report = render_vcf_phasing_family_adapter(
            repo_root,
            tool_id,
            temp_root.join(format!("{tool_id}.phasing.adapter.json")),
        )?;
        rows.extend(report.rows.into_iter().filter(|row| row.stage_id == "vcf.phasing").map(
            |row| {
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
            },
        ));
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

    use crate::commands::benchmark::readiness::vcf_tool_serving_map::collect_vcf_tool_serving_map_rows;

    use super::collect_vcf_rendered_command_rows;

    fn repo_root() -> PathBuf {
        crate::commands::support::workspace_root::resolve_repo_root().expect("repo root")
    }

    #[test]
    fn vcf_rendered_command_rows_follow_canonical_benchmark_ready_slice() {
        let rows = collect_vcf_rendered_command_rows(&repo_root())
            .expect("collect VCF rendered command rows");

        let benchmark_ready_row_count = collect_vcf_tool_serving_map_rows()
            .expect("collect VCF tool-serving rows")
            .into_iter()
            .filter(|row| row.benchmark_status == "benchmark_ready")
            .count();
        assert_eq!(rows.len(), benchmark_ready_row_count);
        assert!(rows.iter().all(|row| row.benchmark_status == "benchmark_ready"));
        assert!(rows.iter().all(|row| !row.script_commands.is_empty()));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.call" && row.tool_id == "bcftools"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.stats" && row.tool_id == "bcftools"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.impute" && row.tool_id == "beagle"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.qc" && row.tool_id == "plink2"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit5"));
        assert!(rows.iter().any(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit"));
    }
}
