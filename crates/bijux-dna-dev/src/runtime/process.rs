use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{ensure, Context, Result};

use crate::runtime::workspace::Workspace;

mod dependency_graph;

#[derive(Debug, Clone)]
pub struct DependentProcessSpec {
    pub process: ParallelProcessSpec,
    pub dependencies: Vec<String>,
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

    /// Prepare and run independent logged children within the same bounded admission slots.
    ///
    /// Preparation starts only after admission. A completed child releases its slot immediately;
    /// its duration includes preparation. Labels and capacity are admitted before any preparation.
    /// Launch/preparation errors stop further admission and drain active peers, preserving logs.
    ///
    /// # Errors
    /// Returns an error for invalid labels/budget, preparation failure or child launch/collection.
    pub fn run_prepared_parallel_logged<F>(
        &self,
        labels: Vec<String>,
        max_parallelism: usize,
        log_directory: &Path,
        prepare: F,
    ) -> Result<Vec<ParallelProcessOutput>>
    where
        F: Fn(&str) -> Result<ParallelProcessSpec> + Sync,
    {
        ensure!(max_parallelism > 0, "prepared process parallelism must be positive");
        prepare_log_directory(labels.iter().map(String::as_str), log_directory)?;
        let specs = labels
            .into_iter()
            .map(|label| DependentProcessSpec {
                process: ParallelProcessSpec {
                    label,
                    program: String::new(),
                    args: Vec::new(),
                    envs: Vec::new(),
                    current_dir: None,
                },
                dependencies: Vec::new(),
            })
            .collect();
        dependency_graph::collect(specs, max_parallelism, |admitted| {
            let spec = prepare(&admitted.label)?;
            ensure!(
                spec.label == admitted.label,
                "prepared process changed its admitted log label"
            );
            let mut command = std::process::Command::new(&spec.program);
            command
                .args(&spec.args)
                .envs(spec.envs.iter().map(|(key, value)| (key, value)))
                .current_dir(spec.current_dir.as_ref().unwrap_or(&self.workspace.root));
            run_with_logs(&mut command, log_directory, &spec.label)
                .with_context(|| format!("run prepared process {}", spec.label))
        })?
        .into_iter()
        .map(|output| match output {
            DependencyProcessOutput::Completed(output) => Ok(output),
            DependencyProcessOutput::BlockedDependency { label } => {
                anyhow::bail!("independent prepared process {label} was blocked")
            }
        })
        .collect()
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
        dependency_graph::collect(specs, max_parallelism, |spec| {
            let mut command = std::process::Command::new(&spec.program);
            command
                .args(&spec.args)
                .envs(spec.envs.iter().map(|(key, value)| (key, value)))
                .current_dir(spec.current_dir.as_ref().unwrap_or(&self.workspace.root));
            command.output().with_context(|| format!("run dependency process {}", spec.label))
        })
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

fn run_with_logs(
    command: &mut std::process::Command,
    directory: &Path,
    label: &str,
) -> Result<Output> {
    let stdout_path = directory.join(format!("{label}.stdout.log"));
    let stderr_path = directory.join(format!("{label}.stderr.log"));
    let stdout = bijux_dna_infra::create_file(&stdout_path)
        .with_context(|| format!("create {}", stdout_path.display()))?;
    let stderr = bijux_dna_infra::create_file(&stderr_path)
        .with_context(|| format!("create {}", stderr_path.display()))?;
    let status =
        command.stdin(std::process::Stdio::null()).stdout(stdout).stderr(stderr).spawn()?.wait()?;
    Ok(Output {
        status,
        stdout: fs::read(&stdout_path)
            .with_context(|| format!("read {}", stdout_path.display()))?,
        stderr: fs::read(&stderr_path)
            .with_context(|| format!("read {}", stderr_path.display()))?,
    })
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

    #[test]
    fn prepared_logged_children_refill_slots_without_waiting_for_peer_preparation(
    ) -> anyhow::Result<()> {
        use std::sync::{Condvar, Mutex};

        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let logs = workspace.root.join("artifacts/logs");
        let signal = (Mutex::new(false), Condvar::new());
        let outputs = ProcessRunner::new(&workspace).run_prepared_parallel_logged(
            ["first", "peer", "consumer"].into_iter().map(String::from).collect(),
            2,
            &logs,
            |label| {
                if label == "peer" {
                    let (ready, _) = signal
                        .1
                        .wait_timeout_while(
                            signal.0.lock().unwrap(),
                            Duration::from_secs(10),
                            |ready| !*ready,
                        )
                        .unwrap();
                    anyhow::ensure!(*ready, "prepared consumer waited for an independent peer");
                } else if label == "consumer" {
                    *signal.0.lock().unwrap() = true;
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
                .run_prepared_parallel_logged(
                    labels.into_iter().map(String::from).collect(),
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
            let result = ProcessRunner::new(&workspace).run_prepared_parallel_logged(
                vec!["failed".into(), "peer".into()],
                2,
                &logs,
                |label| {
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
