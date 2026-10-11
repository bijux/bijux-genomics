use std::path::PathBuf;
use std::process::Output;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::runtime::workspace::Workspace;

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

    /// Run independent processes concurrently and return outputs in declaration order.
    ///
    /// Completion is delivered through a channel as each child exits; no polling or sleeps are
    /// used. All launched children are collected before a launch failure is returned.
    ///
    /// # Errors
    /// Returns an error when a child cannot be launched or its completion cannot be collected.
    pub fn run_parallel(
        &self,
        specs: Vec<ParallelProcessSpec>,
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
                    let result = std::process::Command::new(&spec.program)
                        .args(&spec.args)
                        .envs(spec.envs.iter().map(|(key, value)| (key, value)))
                        .current_dir(current_dir)
                        .output()
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

#[cfg(test)]
mod tests {
    use super::{ParallelProcessSpec, ProcessRunner};
    use crate::runtime::workspace::Workspace;

    #[test]
    fn parallel_runner_collects_every_child_in_declaration_order() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let specs = ["first", "second"]
            .into_iter()
            .map(|label| ParallelProcessSpec {
                label: label.to_string(),
                program: "sh".to_string(),
                args: vec!["-c".to_string(), format!("printf '%s' {label}")],
                envs: Vec::new(),
                current_dir: None,
            })
            .collect();
        let outputs = ProcessRunner::new(&workspace).run_parallel(specs)?;
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
        let outputs = ProcessRunner::new(&workspace).run_parallel(vec![ParallelProcessSpec {
            label: "isolated".to_string(),
            program: "pwd".to_string(),
            args: Vec::new(),
            envs: Vec::new(),
            current_dir: Some(isolated.path().to_path_buf()),
        }])?;

        assert_eq!(
            String::from_utf8(outputs[0].output.stdout.clone())?.trim(),
            isolated.path().canonicalize()?.display().to_string()
        );
        Ok(())
    }
}
