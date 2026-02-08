use std::cell::RefCell;
use std::collections::BTreeSet;
use std::time::Duration;

use anyhow::Result;
use bijux_dna_runtime::{Invocation, Runner, RunnerResult};

#[derive(Default)]
pub struct FakeRunner {
    calls: RefCell<Vec<String>>,
    fail_first: RefCell<BTreeSet<String>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fail_first(&self, step_id: &str) {
        self.fail_first.borrow_mut().insert(step_id.to_string());
    }

    pub fn calls(&self) -> Vec<String> {
        self.calls.borrow().clone()
    }
}

impl Runner for FakeRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        let plan = &invocation.step;
        let attempt = invocation.attempt;
        self.calls
            .borrow_mut()
            .push(format!("{}:{}", plan.step_id.0, attempt));
        let should_fail =
            self.fail_first.borrow_mut().remove(plan.step_id.as_str()) && attempt == 0;
        let run_artifacts = plan.out_dir.join("run_artifacts");
        bijux_dna_infra::ensure_dir(&run_artifacts)?;
        for name in [
            "metrics.json",
            "effective_config.json",
            "stage_report.json",
            "tool_invocation.json",
        ] {
            let path = run_artifacts.join(name);
            bijux_dna_infra::write_bytes(&path, "{}")?;
        }
        Ok(RunnerResult {
            exit_code: i32::from(should_fail),
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::from_millis(1),
            artifacts: Vec::new(),
        })
    }
}

pub struct DeterministicRunner;

impl Runner for DeterministicRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        let step = &invocation.step;
        let run_artifacts = step.out_dir.join("run_artifacts");
        bijux_dna_infra::ensure_dir(&run_artifacts)?;
        for name in [
            "metrics.json",
            "effective_config.json",
            "stage_report.json",
            "tool_invocation.json",
        ] {
            let path = run_artifacts.join(name);
            bijux_dna_infra::write_bytes(&path, "{}")?;
        }
        for output in &step.io.outputs {
            bijux_dna_infra::ensure_dir(
                output
                    .path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("output missing parent"))?,
            )?;
            bijux_dna_infra::write_bytes(&output.path, "deterministic")?;
        }
        Ok(RunnerResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::from_millis(1),
            artifacts: Vec::new(),
        })
    }
}

pub struct RecordingRunner;

impl Runner for RecordingRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        let step = &invocation.step;
        let run_artifacts = step.out_dir.join("run_artifacts");
        bijux_dna_infra::ensure_dir(&run_artifacts)?;
        for name in [
            "metrics.json",
            "effective_config.json",
            "stage_report.json",
            "tool_invocation.json",
            "execution_record.json",
        ] {
            let path = run_artifacts.join(name);
            bijux_dna_infra::write_bytes(&path, "{}")?;
        }
        for output in &step.io.outputs {
            bijux_dna_infra::ensure_dir(
                output
                    .path
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("output missing parent"))?,
            )?;
            bijux_dna_infra::write_bytes(&output.path, "data")?;
        }
        Ok(RunnerResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
            duration: Duration::from_millis(1),
            artifacts: Vec::new(),
        })
    }
}
