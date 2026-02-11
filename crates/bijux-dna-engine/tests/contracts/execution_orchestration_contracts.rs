use std::time::Duration;

use anyhow::Result;
use bijux_dna_core::contract::{ArtifactRole, ArtifactSpec, RetryPolicy};
use bijux_dna_core::prelude::{ArtifactId, StepId};
use bijux_dna_engine::{CancellationToken, Engine, EngineConfig, EngineEvent};
use bijux_dna_runtime::{Invocation, Runner, RunnerResult};

use crate::support::{build_graph, execution_setup, plan_for};

#[derive(Clone, Copy)]
enum Mode {
    Success,
    FailOnceThenSuccess,
    Timeout,
}

struct ScenarioRunner {
    mode: Mode,
    write_required_artifacts: bool,
    write_outputs: bool,
    output_payload: &'static str,
    cancel_on_first_attempt: Option<CancellationToken>,
}

impl ScenarioRunner {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            write_required_artifacts: true,
            write_outputs: true,
            output_payload: "{}",
            cancel_on_first_attempt: None,
        }
    }
}

impl Runner for ScenarioRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        if invocation.attempt == 0 {
            if let Some(token) = &self.cancel_on_first_attempt {
                token.cancel();
            }
        }

        let run_artifacts = invocation.step.out_dir.join("run_artifacts");
        bijux_dna_infra::ensure_dir(&run_artifacts)?;
        if self.write_required_artifacts {
            for name in [
                "metrics.json",
                "effective_config.json",
                "stage_report.json",
                "tool_invocation.json",
            ] {
                bijux_dna_infra::write_bytes(&run_artifacts.join(name), "{}")?;
            }
        }

        if self.write_outputs {
            for output in &invocation.step.io.outputs {
                if let Some(parent) = output.path.parent() {
                    bijux_dna_infra::ensure_dir(parent)?;
                }
                bijux_dna_infra::write_bytes(&output.path, self.output_payload)?;
            }
        }

        let (exit_code, duration) = match self.mode {
            Mode::Success => (0, Duration::from_millis(1)),
            Mode::FailOnceThenSuccess => {
                let code = if invocation.attempt == 0 { 1 } else { 0 };
                (code, Duration::from_millis(1))
            }
            Mode::Timeout => (0, Duration::from_secs(2)),
        };

        Ok(RunnerResult {
            exit_code,
            stdout: String::new(),
            stderr: String::new(),
            duration,
            artifacts: Vec::new(),
        })
    }
}

#[test]
fn cancellation_token_and_engine_event_shapes_are_stable() {
    let token = CancellationToken::new();
    assert!(!token.is_cancelled());
    token.cancel();
    assert!(token.is_cancelled());
    assert!(!CancellationToken::default().is_cancelled());

    let start = EngineEvent::StepStart {
        step_id: StepId::new("A"),
        attempt: 0,
    };
    let end = EngineEvent::StepEnd {
        step_id: StepId::new("A"),
        attempt: 0,
        success: true,
    };
    let retry = EngineEvent::Retry {
        step_id: StepId::new("A"),
        attempt: 1,
        exit_code: 1,
    };
    let verified = EngineEvent::ArtifactVerified {
        step_id: StepId::new("A"),
        path: "out/file.json".to_string(),
    };
    let encoded = serde_json::to_string(&vec![start, end, retry, verified]);
    assert!(encoded.is_ok());
    let encoded = encoded.unwrap_or_else(|err| panic!("serialize engine events: {err}"));
    assert!(encoded.contains("step_start"));
    assert!(encoded.contains("step_end"));
    assert!(encoded.contains("retry"));
    assert!(encoded.contains("artifact_verified"));
}

#[test]
fn execute_plan_handles_cancel_before_and_during_retry() {
    let plan = build_graph(vec![plan_for("A")], Vec::new());
    let (_dir, layout) = execution_setup().unwrap_or_else(|err| panic!("layout: {err}"));
    let engine = Engine::default();

    let cancelled = CancellationToken::new();
    cancelled.cancel();
    let before_err = engine
        .execute(
            &plan,
            &ScenarioRunner::new(Mode::Success),
            &layout,
            None,
            Some(&cancelled),
        )
        .err()
        .unwrap_or_else(|| panic!("expected cancellation before execution"));
    assert!(before_err.to_string().contains("cancelled before"));

    let during = CancellationToken::new();
    let mut runner = ScenarioRunner::new(Mode::FailOnceThenSuccess);
    runner.cancel_on_first_attempt = Some(during.clone());
    let retry_plan = plan.with_retry_policy(RetryPolicy {
        max_attempts: 2,
        retry_on_exit_codes: vec![1],
    });
    let during_err = engine
        .execute(&retry_plan, &runner, &layout, None, Some(&during))
        .err()
        .unwrap_or_else(|| panic!("expected cancellation during execution"));
    assert!(during_err.to_string().contains("cancelled during"));
}

#[test]
fn execute_plan_reports_timeout_and_contract_errors() {
    let (_dir, layout) = execution_setup().unwrap_or_else(|err| panic!("layout: {err}"));
    let make_graph = |with_metrics_schema: bool| {
        let mut step = plan_for("A");
        let output_path = step.out_dir.join("output.json");
        step.io.outputs = vec![ArtifactSpec::required(
            ArtifactId::new("output"),
            output_path,
            ArtifactRole::MetricsJson,
        )];
        if with_metrics_schema {
            step.metrics_schema_ids = vec!["schema.v1".to_string()];
        }
        build_graph(vec![step], Vec::new())
    };

    let graph = make_graph(false);
    let timeout_err = Engine::new(EngineConfig {
        step_timeout_s: Some(1),
        deterministic_scheduler: true,
        retry_policy: None,
        max_parallelism: None,
    })
    .execute(
        &graph,
        &ScenarioRunner::new(Mode::Timeout),
        &layout,
        None,
        None,
    )
    .err()
    .unwrap_or_else(|| panic!("expected timeout error"));
    assert!(timeout_err.to_string().contains("exceeded timeout"));

    let graph = make_graph(false);
    let mut missing_output_runner = ScenarioRunner::new(Mode::Success);
    missing_output_runner.write_outputs = false;
    let missing_output_err = Engine::default()
        .execute(&graph, &missing_output_runner, &layout, None, None)
        .err()
        .unwrap_or_else(|| panic!("expected missing output contract error"));
    assert!(missing_output_err.to_string().contains("missing output"));

    let graph = make_graph(false);
    let mut invalid_metrics_runner = ScenarioRunner::new(Mode::Success);
    invalid_metrics_runner.output_payload = "not-json";
    let invalid_metrics_err = Engine::default()
        .execute(&graph, &invalid_metrics_runner, &layout, None, None)
        .err()
        .unwrap_or_else(|| panic!("expected parse error for metrics output"));
    assert!(invalid_metrics_err
        .to_string()
        .contains("metrics output not parseable"));

    let metrics_graph = make_graph(true);
    let missing_envelope_err = Engine::default()
        .execute(
            &metrics_graph,
            &ScenarioRunner::new(Mode::Success),
            &layout,
            None,
            None,
        )
        .err()
        .unwrap_or_else(|| panic!("expected missing metrics_envelope contract error"));
    assert!(missing_envelope_err
        .to_string()
        .contains("missing metrics_envelope.json"));
}
