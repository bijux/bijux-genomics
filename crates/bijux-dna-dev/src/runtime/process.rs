use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};

use crate::runtime::workspace::Workspace;

mod dependency_graph;
mod logged_output;

use logged_output::{run_with_logs, run_with_start_record};

#[derive(Debug, Clone)]
pub struct DependentProcessSpec {
    pub process: ParallelProcessSpec,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WeightedProcessSpec {
    pub process: DependentProcessSpec,
    pub resource_units: usize,
    pub terminal_dependencies: Vec<String>,
}

/// Logical provisioned envelopes, not operating-system usage limits.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProvisionedResourceUnits {
    pub memory_bytes: u64,
    pub scratch_bytes: u64,
    pub io_slots: usize,
}

#[derive(Debug, Clone)]
pub struct ProvisionedProcessSpec {
    pub process: WeightedProcessSpec,
    pub resources: ProvisionedResourceUnits,
}

#[derive(Debug)]
pub enum DependencyProcessOutput {
    Completed(ParallelProcessOutput),
    BlockedDependency { label: String },
}

#[derive(Debug)]
pub struct ProcessRunner<'a> {
    workspace: &'a Workspace,
}

#[derive(Debug, Clone)]
pub struct ParallelProcessSpec {
    pub label: String,
    pub program: String,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
    pub current_dir: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ParallelProcessOutput {
    pub label: String,
    pub output: Output,
    pub duration: Duration,
}

impl<'a> ProcessRunner<'a> {
    #[must_use]
    pub fn new(workspace: &'a Workspace) -> Self {
        Self { workspace }
    }

    /// # Errors
    /// Returns an error if the command cannot be launched.
    pub fn run(&self, argv: &[&str]) -> Result<Output> {
        let (program, program_args) =
            argv.split_first().context("process runner requires at least one argument")?;
        std::process::Command::new(program)
            .args(program_args)
            .current_dir(&self.workspace.root)
            .output()
            .with_context(|| format!("run {}", argv.join(" ")))
    }

    /// # Errors
    /// Returns an error if the command cannot be launched.
    pub fn run_owned(&self, program: &str, args: &[String]) -> Result<Output> {
        self.run_owned_with_env(program, args, &[])
    }

    /// # Errors
    /// Returns an error if the command cannot be launched.
    pub fn run_owned_with_env(
        &self,
        program: &str,
        args: &[String],
        envs: &[(String, String)],
    ) -> Result<Output> {
        std::process::Command::new(program)
            .args(args)
            .envs(envs.iter().map(|(key, value)| (key.as_str(), value.as_str())))
            .current_dir(&self.workspace.root)
            .output()
            .with_context(|| {
                if args.is_empty() {
                    format!("run {program}")
                } else {
                    format!("run {program} {}", args.join(" "))
                }
            })
    }

    /// Run independent processes with directly written stdout and stderr files.
    ///
    /// Files are visible while children run and survive coordinator interruption. Each label
    /// owns two files in the supplied directory; a new invocation replaces those prior logs.
    /// The returned output remains compatible with callers that aggregate completed results.
    ///
    /// # Errors
    /// Returns an error for ambiguous log labels, log IO failures, or child launch failures.
    pub fn run_parallel_logged(
        &self,
        specs: Vec<ParallelProcessSpec>,
        log_directory: &Path,
    ) -> Result<Vec<ParallelProcessOutput>> {
        prepare_log_directory(specs.iter().map(|spec| spec.label.as_str()), log_directory)?;
        self.collect_parallel(specs, log_directory)
    }

    /// Prepare logged dependency children after weighted admission, retaining terminal edges.
    ///
    /// The existing collector validates the graph and weights, refills on completion and drains
    /// launched peers after preparation/launch errors or panics. Logs and child verdicts persist.
    ///
    /// # Errors
    /// Returns an error for invalid labels/weights/dependencies, preparation, launch or log IO.
    pub fn run_prepared_weighted_parallel_logged<F>(
        &self,
        specs: Vec<WeightedProcessSpec>,
        max_parallelism: usize,
        resource_capacity: usize,
        log_directory: &Path,
        prepare: F,
    ) -> Result<Vec<ParallelProcessOutput>>
    where
        F: Fn(&ParallelProcessSpec) -> Result<ParallelProcessSpec> + Sync,
    {
        let execution_units = specs.iter().map(|spec| spec.resource_units).collect();
        self.run_prepared_transitioning_parallel_logged(
            specs,
            execution_units,
            max_parallelism,
            resource_capacity,
            log_directory,
            |spec| Ok((prepare(spec)?, (|_: &[u8]| Ok(false)) as fn(&[u8]) -> Result<bool>)),
        )
    }

    /// Lower a declared startup reservation only after an owner validates its first stdout record.
    /// Child output still goes directly to durable files; the observer never owns a pipe.
    ///
    /// # Errors
    /// Returns admission, preparation, observation, launch, wait or log IO errors after draining.
    pub(crate) fn run_prepared_transitioning_parallel_logged<F, O>(
        &self,
        specs: Vec<WeightedProcessSpec>,
        execution_units: Vec<usize>,
        max_parallelism: usize,
        resource_capacity: usize,
        log_directory: &Path,
        prepare: F,
    ) -> Result<Vec<ParallelProcessOutput>>
    where
        F: Fn(&ParallelProcessSpec) -> Result<(ParallelProcessSpec, O)> + Sync,
        O: FnMut(&[u8]) -> Result<bool>,
    {
        ensure!(max_parallelism > 0, "prepared process parallelism must be positive");
        ensure!(resource_capacity > 0, "weighted resource capacity must be positive");
        ensure!(execution_units.len() == specs.len(), "execution reservation roster differs");
        let reduced = specs
            .iter()
            .zip(&execution_units)
            .filter(|(spec, units)| **units < spec.resource_units)
            .map(|(spec, _)| spec.process.process.label.clone())
            .collect::<BTreeSet<_>>();
        prepare_log_directory(
            specs.iter().map(|spec| spec.process.process.label.as_str()),
            log_directory,
        )?;
        for spec in &specs {
            let receipt = serde_json::json!({"schema_version":"bijux.ci.logged_weighted_child.v1","label":spec.process.process.label,"state":"not_started","terminal":false,"all_passed":false});
            bijux_dna_infra::write_bytes(
                log_directory.join(format!("{}.receipt.json", spec.process.process.label)),
                &serde_json::to_vec_pretty(&receipt)?,
            )?;
        }
        dependency_graph::collect_transitioning(specs, max_parallelism, resource_capacity, execution_units, |admitted, execution_started| {
            let preparing=serde_json::json!({"schema_version":"bijux.ci.logged_weighted_child.v1","label":admitted.label,"state":"preparing","terminal":false,"all_passed":false});
            bijux_dna_infra::write_bytes(log_directory.join(format!("{}.receipt.json",admitted.label)),&serde_json::to_vec_pretty(&preparing)?)?;
            let mut start_observed = false;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<Output> {
                let (spec, mut observe) = prepare(admitted)?;
                ensure!(spec.label == admitted.label, "prepared weighted process changed admitted identity");
                let mut command = std::process::Command::new(&spec.program);
                command.args(&spec.args).envs(spec.envs.iter().map(|(key, value)| (key, value))).current_dir(spec.current_dir.as_ref().unwrap_or(&self.workspace.root));
                if reduced.contains(&spec.label) {
                    run_with_start_record(&mut command, log_directory, &spec.label, |line| {
                        if observe(line)? {
                            execution_started()?;
                            start_observed = true;
                        }
                        Ok(())
                    })
                } else {
                    run_with_logs(&mut command, log_directory, &spec.label)
                }
            })).unwrap_or_else(|_|Err(anyhow::anyhow!("prepared weighted process {} panicked",admitted.label)));
            let mut receipt = serde_json::json!({"schema_version":"bijux.ci.logged_weighted_child.v1","label":admitted.label,
                "terminal":true,"state":if result.is_ok() {"complete"} else {"error"},"exit_code":result.as_ref().ok().and_then(|output|output.status.code()),
                "execution_error":result.as_ref().err().map(|error|format!("{error:#}")),
                "all_passed":result.as_ref().is_ok_and(|output|output.status.success())});
            if reduced.contains(&admitted.label) {
                receipt["execution_start_observed"] = serde_json::json!(start_observed);
            }
            bijux_dna_infra::write_bytes(log_directory.join(format!("{}.receipt.json", admitted.label)), &serde_json::to_vec_pretty(&receipt)?)?;
            result
        })?.into_iter().map(|result| match result {
            DependencyProcessOutput::Completed(output) => Ok(output),
            DependencyProcessOutput::BlockedDependency { label } => anyhow::bail!("weighted logged process {label} was blocked"),
        }).collect()
    }

    /// Run a dependency graph with a bounded number of active children.
    ///
    /// Each completion immediately releases its slot and admits newly ready consumers. Failed
    /// children block their descendants while independent work continues. Results retain input
    /// order, and launched children are drained before a launch error is returned.
    ///
    /// # Errors
    /// Returns an error for an invalid graph or a child launch/collection failure.
    pub fn run_dependency_graph(
        &self,
        specs: Vec<DependentProcessSpec>,
        max_parallelism: usize,
    ) -> Result<Vec<DependencyProcessOutput>> {
        self.run_weighted_dependency_graph(
            specs
                .into_iter()
                .map(|process| WeightedProcessSpec {
                    process,
                    resource_units: 1,
                    terminal_dependencies: Vec::new(),
                })
                .collect(),
            max_parallelism,
            max_parallelism,
        )
    }

    /// Admit dependency-ready children against a shared resource budget and process limit.
    /// Terminal dependencies require predecessor completion regardless of its verdict; ordinary
    /// dependencies still require success. Every declared unit weight must fit before launch.
    ///
    /// # Errors
    /// Returns an error for invalid capacity/dependencies or child launch/collection failure.
    pub fn run_weighted_dependency_graph(
        &self,
        specs: Vec<WeightedProcessSpec>,
        max_parallelism: usize,
        resource_capacity: usize,
    ) -> Result<Vec<DependencyProcessOutput>> {
        let run = |spec: &ParallelProcessSpec| {
            let mut command = std::process::Command::new(&spec.program);
            command
                .args(&spec.args)
                .envs(spec.envs.iter().map(|(key, value)| (key, value)))
                .current_dir(spec.current_dir.as_ref().unwrap_or(&self.workspace.root));
            command.output().with_context(|| format!("run dependency process {}", spec.label))
        };
        if resource_capacity == max_parallelism
            && specs
                .iter()
                .all(|spec| spec.resource_units == 1 && spec.terminal_dependencies.is_empty())
        {
            dependency_graph::collect(
                specs.into_iter().map(|spec| spec.process).collect(),
                max_parallelism,
                run,
            )
        } else {
            dependency_graph::collect_weighted(specs, max_parallelism, resource_capacity, run)
        }
    }

    /// Reserve explicit RAM/disk/IO envelopes in the existing dependency collector.
    /// CPU and top-level process limits retain the weighted API semantics. Nested OS
    /// process counts and actual RAM/IO consumption are not hard-enforced by this API.
    ///
    /// # Errors
    /// Returns an error for malformed, overflowing or individually unfit allocations,
    /// invalid dependencies, or child launch/collection failure.
    pub fn run_provisioned_dependency_graph(
        &self,
        specs: Vec<ProvisionedProcessSpec>,
        max_parallelism: usize,
        worker_budget: usize,
        capacity: ProvisionedResourceUnits,
    ) -> Result<Vec<DependencyProcessOutput>> {
        let demands = specs.iter().map(|spec| spec.resources).collect();
        let specs = specs.into_iter().map(|spec| spec.process).collect();
        dependency_graph::collect_provisioned(
            specs,
            max_parallelism,
            worker_budget,
            Some(dependency_graph::AuxiliaryReservation { capacity, demands }),
            |spec| {
                let mut command = std::process::Command::new(&spec.program);
                command
                    .args(&spec.args)
                    .envs(spec.envs.iter().map(|(key, value)| (key, value)))
                    .current_dir(spec.current_dir.as_ref().unwrap_or(&self.workspace.root));
                command.output().with_context(|| format!("run provisioned process {}", spec.label))
            },
        )
    }

    fn collect_parallel(
        &self,
        specs: Vec<ParallelProcessSpec>,
        log_directory: &Path,
    ) -> Result<Vec<ParallelProcessOutput>> {
        let root = self.workspace.root.clone();
        let count = specs.len();
        let (sender, receiver) = mpsc::channel();
        std::thread::scope(|scope| {
            for (index, spec) in specs.into_iter().enumerate() {
                let sender = sender.clone();
                let root = root.clone();
                scope.spawn(move || {
                    let started = Instant::now();
                    let current_dir = spec.current_dir.as_ref().unwrap_or(&root);
                    let mut command = std::process::Command::new(&spec.program);
                    command
                        .args(&spec.args)
                        .envs(spec.envs.iter().map(|(key, value)| (key, value)))
                        .current_dir(current_dir);
                    let result = run_with_logs(&mut command, log_directory, &spec.label)
                        .with_context(|| format!("run parallel process {}", spec.label))
                        .map(|output| ParallelProcessOutput {
                            label: spec.label,
                            output,
                            duration: started.elapsed(),
                        });
                    let _ = sender.send((index, result));
                });
            }
            drop(sender);
            let mut outputs = (0..count).map(|_| None).collect::<Vec<_>>();
            let mut first_error = None;
            for (index, result) in receiver {
                match result {
                    Ok(output) => outputs[index] = Some(output),
                    Err(error) if first_error.is_none() => first_error = Some(error),
                    Err(_) => {}
                }
            }
            if let Some(error) = first_error {
                return Err(error);
            }
            outputs
                .into_iter()
                .enumerate()
                .map(|(index, output)| {
                    output.with_context(|| format!("parallel process {index} was not collected"))
                })
                .collect()
        })
    }
}

fn prepare_log_directory<'a>(
    labels: impl Iterator<Item = &'a str>,
    directory: &Path,
) -> Result<()> {
    let mut seen = BTreeSet::new();
    for label in labels {
        ensure!(!label.is_empty() && label.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"-_".contains(&byte)
        }), "parallel process log label must contain only lowercase letters, digits, '-' or '_': {label}");
        ensure!(seen.insert(label), "duplicate parallel process log label: {label}");
    }
    bijux_dna_infra::ensure_dir(directory)
        .with_context(|| format!("create process log directory {}", directory.display()))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        DependencyProcessOutput, DependentProcessSpec, ParallelProcessOutput, ParallelProcessSpec,
        ProcessRunner,
    };
    use crate::runtime::workspace::Workspace;

    #[test]
    fn prepared_weighted_logs_preserve_terminal_failure_and_drain_preparation_error(
    ) -> anyhow::Result<()> {
        use super::WeightedProcessSpec;
        let owner = tempfile::tempdir()?;
        let workspace = Workspace { root: owner.path().canonicalize()? };
        let directory = workspace.path("artifacts/native-children");
        let specs = [("failed", 2, Vec::new()), ("terminal", 2, vec!["failed".into()])]
            .into_iter()
            .map(|(label, resource_units, terminal_dependencies)| WeightedProcessSpec {
                process: DependentProcessSpec {
                    process: shell_spec(
                        label,
                        if label == "failed" { "exit 7" } else { "printf terminal" },
                    ),
                    dependencies: vec![],
                },
                resource_units,
                terminal_dependencies,
            })
            .collect();
        let outputs = ProcessRunner::new(&workspace).run_prepared_weighted_parallel_logged(
            specs,
            2,
            2,
            &directory,
            |spec| Ok(spec.clone()),
        )?;
        assert!(!outputs[0].output.status.success());
        assert_eq!(outputs[1].output.stdout, b"terminal");
        assert!(directory.join("terminal.receipt.json").is_file());
        let specs = ["preparation-error", "peer"]
            .into_iter()
            .map(|label| WeightedProcessSpec {
                process: DependentProcessSpec {
                    process: shell_spec(label, "printf drained"),
                    dependencies: vec![],
                },
                resource_units: 2,
                terminal_dependencies: vec![],
            })
            .collect();
        assert!(ProcessRunner::new(&workspace)
            .run_prepared_weighted_parallel_logged(specs, 2, 4, &directory, |spec| {
                if spec.label == "preparation-error" {
                    anyhow::bail!("native subset preparation error");
                }
                Ok(spec.clone())
            })
            .is_err());
        assert_eq!(std::fs::read(directory.join("peer.stdout.log"))?, b"drained");
        let terminal: serde_json::Value = serde_json::from_slice(&std::fs::read(
            directory.join("preparation-error.receipt.json"),
        )?)?;
        assert_eq!(terminal["all_passed"], false);
        assert!(terminal["execution_error"]
            .as_str()
            .is_some_and(|message| message.contains("preparation error")));
        Ok(())
    }

    #[test]
    fn parallel_runner_collects_every_child_in_declaration_order() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let specs = ["first", "second"]
            .into_iter()
            .map(|label| DependentProcessSpec {
                process: shell_spec(label, &format!("printf '%s' {label}")),
                dependencies: Vec::new(),
            })
            .collect();
        let outputs =
            completed_outputs(ProcessRunner::new(&workspace).run_dependency_graph(specs, 2)?)?;
        assert_eq!(
            outputs.iter().map(|output| output.label.as_str()).collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert_eq!(outputs[0].output.stdout, b"first");
        assert_eq!(outputs[1].output.stdout, b"second");
        Ok(())
    }

    #[test]
    fn parallel_runner_honors_isolated_working_directories() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let artifact_root = workspace.root.join("artifacts");
        bijux_dna_infra::ensure_dir(&artifact_root)?;
        let isolated = tempfile::tempdir_in(&artifact_root)?;
        let outputs = completed_outputs(ProcessRunner::new(&workspace).run_dependency_graph(
            vec![DependentProcessSpec {
                process: ParallelProcessSpec {
                    label: "isolated".to_string(),
                    program: "pwd".to_string(),
                    args: Vec::new(),
                    envs: Vec::new(),
                    current_dir: Some(isolated.path().to_path_buf()),
                },
                dependencies: Vec::new(),
            }],
            1,
        )?)?;

        assert_eq!(
            String::from_utf8(outputs[0].output.stdout.clone())?.trim(),
            isolated.path().canonicalize()?.display().to_string()
        );
        Ok(())
    }

    fn completed_outputs(
        outputs: Vec<DependencyProcessOutput>,
    ) -> anyhow::Result<Vec<ParallelProcessOutput>> {
        outputs
            .into_iter()
            .map(|output| match output {
                DependencyProcessOutput::Completed(output) => Ok(output),
                DependencyProcessOutput::BlockedDependency { label } => {
                    anyhow::bail!("independent child {label} was blocked")
                }
            })
            .collect()
    }

    fn shell_spec(label: &str, script: &str) -> ParallelProcessSpec {
        ParallelProcessSpec {
            label: label.to_string(),
            program: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            envs: Vec::new(),
            current_dir: None,
        }
    }

    fn unit_weighted_specs(labels: &[&str]) -> Vec<super::WeightedProcessSpec> {
        labels
            .iter()
            .map(|label| super::WeightedProcessSpec {
                process: DependentProcessSpec {
                    process: shell_spec(label, ""),
                    dependencies: Vec::new(),
                },
                resource_units: 1,
                terminal_dependencies: Vec::new(),
            })
            .collect()
    }

    #[test]
    fn prepared_logged_children_refill_slots_without_waiting_for_peer_preparation(
    ) -> anyhow::Result<()> {
        use std::sync::{Condvar, Mutex};

        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let logs = workspace.root.join("artifacts/logs");
        let signal = (Mutex::new(false), Condvar::new());
        let outputs = ProcessRunner::new(&workspace).run_prepared_weighted_parallel_logged(
            unit_weighted_specs(&["first", "peer", "consumer"]),
            2,
            2,
            &logs,
            |spec| {
                let label = spec.label.as_str();
                if label == "peer" {
                    let (ready, _) = signal
                        .1
                        .wait_timeout_while(
                            signal.0.lock().map_err(|_| {
                                anyhow::anyhow!("scheduler test mutex was poisoned")
                            })?,
                            Duration::from_secs(10),
                            |ready| !*ready,
                        )
                        .map_err(|_| {
                            anyhow::anyhow!("scheduler test condition mutex was poisoned")
                        })?;
                    anyhow::ensure!(*ready, "prepared consumer waited for an independent peer");
                } else if label == "consumer" {
                    *signal
                        .0
                        .lock()
                        .map_err(|_| anyhow::anyhow!("scheduler test mutex was poisoned"))? = true;
                    signal.1.notify_all();
                }
                Ok(shell_spec(label, &format!("printf '%s' {label}")))
            },
        )?;
        assert_eq!(
            outputs.iter().map(|output| output.label.as_str()).collect::<Vec<_>>(),
            ["first", "peer", "consumer"]
        );
        for output in outputs {
            assert!(output.output.status.success());
            assert_eq!(
                std::fs::read(logs.join(format!("{}.stdout.log", output.label)))?,
                output.label.as_bytes()
            );
        }
        Ok(())
    }

    #[test]
    fn invalid_prepared_admission_never_calls_preparation() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let logs = workspace.root.join("artifacts/logs");
        for (labels, capacity) in
            [(vec!["same", "same"], 2), (vec!["../escaped"], 1), (vec!["valid"], 0)]
        {
            assert!(ProcessRunner::new(&workspace)
                .run_prepared_weighted_parallel_logged(
                    unit_weighted_specs(&labels),
                    capacity,
                    capacity,
                    &logs,
                    |_| panic!("invalid prepared admission ran"),
                )
                .is_err());
            assert!(!logs.exists());
        }
        Ok(())
    }

    #[test]
    fn preparation_errors_and_panics_drain_peers_and_retain_their_logs() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        for panic in [false, true] {
            let logs = workspace.root.join(format!("artifacts/logs-{panic}"));
            let result = ProcessRunner::new(&workspace).run_prepared_weighted_parallel_logged(
                unit_weighted_specs(&["failed", "peer"]),
                2,
                2,
                &logs,
                |spec| {
                    let label = spec.label.as_str();
                    if label == "failed" {
                        assert!(!panic, "synthetic preparation panic");
                        anyhow::bail!("synthetic preparation error");
                    }
                    Ok(shell_spec(label, "printf retained"))
                },
            );
            assert!(result.is_err());
            assert_eq!(std::fs::read(logs.join("peer.stdout.log"))?, b"retained");
        }
        Ok(())
    }

    #[test]
    fn parallel_logs_are_visible_to_the_child_before_exit() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let logs = workspace.root.join("artifacts/logs");
        let mut spec = shell_spec(
            "live",
            "printf live; printf diagnostic >&2; test \"$(cat \"$STDOUT_LOG\")\" = live && test \"$(cat \"$STDERR_LOG\")\" = diagnostic",
        );
        spec.envs = vec![
            ("STDOUT_LOG".to_string(), logs.join("live.stdout.log").display().to_string()),
            ("STDERR_LOG".to_string(), logs.join("live.stderr.log").display().to_string()),
        ];
        let outputs = ProcessRunner::new(&workspace).run_parallel_logged(vec![spec], &logs)?;
        assert!(outputs[0].output.status.success());
        assert_eq!(outputs[0].output.stdout, b"live");
        assert_eq!(outputs[0].output.stderr, b"diagnostic");
        Ok(())
    }

    #[test]
    fn parallel_logs_retain_failures_and_collect_successful_peers() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let logs = workspace.root.join("artifacts/logs");
        let outputs = ProcessRunner::new(&workspace).run_parallel_logged(
            vec![
                shell_spec("failed", "printf failed >&2; exit 7"),
                shell_spec("passed", "printf passed"),
            ],
            &logs,
        )?;
        assert_eq!(outputs[0].output.status.code(), Some(7));
        assert_eq!(outputs[1].output.status.code(), Some(0));
        assert_eq!(std::fs::read(logs.join("failed.stderr.log"))?, b"failed");
        assert_eq!(std::fs::read(logs.join("passed.stdout.log"))?, b"passed");
        Ok(())
    }

    #[test]
    fn parallel_log_labels_reject_collisions_and_path_traversal_before_launch() -> anyhow::Result<()>
    {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let logs = workspace.root.join("artifacts/logs");
        for labels in [["same", "same"], ["safe", "../escaped"], ["safe", ""], ["safe", "Safe"]] {
            let specs =
                labels.into_iter().map(|label| shell_spec(label, "touch launched")).collect();
            assert!(ProcessRunner::new(&workspace).run_parallel_logged(specs, &logs).is_err());
            assert!(!workspace.root.join("launched").exists());
            assert!(!logs.exists());
        }
        Ok(())
    }

    #[test]
    fn parallel_logs_survive_a_peer_launch_failure() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let logs = workspace.root.join("artifacts/logs");
        let mut missing = shell_spec("missing", "");
        missing.program = workspace.root.join("absent-executable").display().to_string();
        let result = ProcessRunner::new(&workspace)
            .run_parallel_logged(vec![missing, shell_spec("survivor", "printf retained")], &logs);
        assert!(result.is_err());
        assert_eq!(std::fs::read(logs.join("survivor.stdout.log"))?, b"retained");
        Ok(())
    }
}
