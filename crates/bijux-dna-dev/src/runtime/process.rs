use std::process::Output;

use anyhow::{Context, Result};

use crate::runtime::workspace::Workspace;

#[derive(Debug)]
pub struct ProcessRunner<'a> {
    workspace: &'a Workspace,
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
}
