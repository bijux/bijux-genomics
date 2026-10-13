use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::runtime::process::{DependencyProcessOutput, DependentProcessSpec};

use super::{
    cargo_lock_wait_events, ci_fast_group_env, duration_milliseconds, FastBuildAuthority,
    FastGroupContract, ParallelProcessSpec, ProcessRunner, Result, Workspace,
};

#[derive(Default)]
pub(super) struct FastGroupExecution {
    pub passed: BTreeSet<String>,
    pub failed: BTreeSet<String>,
    pub receipts: Vec<serde_json::Value>,
    pub stdout: String,
    pub stderr: String,
    pub group_durations: BTreeMap<String, u64>,
    pub compiler_lock_wait_events: usize,
    pub compiler_lock_wait_duration_upper_bound_milliseconds: u64,
    pub compiler_lock_wait_targets: BTreeSet<String>,
}

pub(super) fn run_groups(
    workspace: &Workspace,
    contract: &FastGroupContract,
    report_root: &Path,
    build_authority: &FastBuildAuthority,
    sccache_runtime_env: &[(String, String)],
    group_compiler_env: &[(String, String)],
) -> Result<FastGroupExecution> {
    let specs = contract
        .groups
        .iter()
        .map(|group| {
            Ok(DependentProcessSpec {
                process: ParallelProcessSpec {
                    label: group.id.clone(),
                    program: group.program.clone(),
                    args: group.args.clone(),
                    envs: ci_fast_group_env(
                        workspace,
                        build_authority,
                        &group.id,
                        sccache_runtime_env,
                        group_compiler_env,
                    )?,
                    current_dir: None,
                },
                dependencies: group.dependencies.clone(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let outputs =
        ProcessRunner::new(workspace).run_dependency_graph(specs, contract.max_parallelism)?;
    let mut execution = FastGroupExecution::default();
    for (group, completed) in contract.groups.iter().zip(outputs) {
        let command = std::iter::once(group.program.as_str())
            .chain(group.args.iter().map(String::as_str))
            .collect::<Vec<_>>();
        let completed = match completed {
            DependencyProcessOutput::BlockedDependency { label } => {
                execution.failed.insert(label);
                execution.group_durations.insert(group.id.clone(), 0);
                execution.stderr.push_str(&format!(
                    "ci-fast group {}: blocked by failed dependency\n",
                    group.id
                ));
                execution.receipts.push(serde_json::json!({
                    "id": group.id, "status": "blocked_dependency", "dependencies": group.dependencies,
                    "command": command, "duration_milliseconds": 0, "compiler_lock_wait_events": 0,
                    "build_target": build_authority.target.join(&group.id), "exit_code": 1, "log": serde_json::Value::Null,
                }));
                continue;
            }
            DependencyProcessOutput::Completed(completed) => completed,
        };
        let log_path = report_root.join(format!("{}.log", group.id));
        let mut log = completed.output.stdout;
        log.extend_from_slice(&completed.output.stderr);
        bijux_dna_infra::write_bytes(&log_path, &log)?;
        let exit_code = completed.output.status.code().unwrap_or(1);
        let duration_milliseconds = duration_milliseconds(completed.duration);
        let group_lock_wait_events = cargo_lock_wait_events(&log);
        execution.compiler_lock_wait_events += group_lock_wait_events;
        if group_lock_wait_events > 0 {
            execution.compiler_lock_wait_duration_upper_bound_milliseconds += duration_milliseconds;
            execution.compiler_lock_wait_targets.insert(group.id.clone());
        }
        execution.group_durations.insert(group.id.clone(), duration_milliseconds);
        if exit_code == 0 {
            execution.passed.insert(group.id.clone());
            execution.stdout.push_str(&format!(
                "ci-fast group {}: passed in {} ms ({})\n",
                group.id,
                duration_milliseconds,
                workspace.rel(&log_path).display()
            ));
        } else {
            execution.failed.insert(group.id.clone());
            execution.stderr.push_str(&format!(
                "ci-fast group {}: failed with exit {} ({})\n",
                group.id,
                exit_code,
                workspace.rel(&log_path).display()
            ));
        }
        execution.receipts.push(serde_json::json!({
            "id": group.id, "status": if exit_code == 0 { "passed" } else { "failed" },
            "dependencies": group.dependencies, "command": command,
            "duration_milliseconds": duration_milliseconds, "compiler_lock_wait_events": group_lock_wait_events,
            "build_target": build_authority.target.join(&group.id), "exit_code": exit_code,
            "log": workspace.rel(&log_path).to_string_lossy(),
        }));
    }
    Ok(execution)
}
