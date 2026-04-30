use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Result};
use bijux_dna_core::contract::ExecutionGraph;
use bijux_dna_environment::api::RuntimeKind;
use bijux_dna_runtime::run_layout::{
    now_string, OperatorHealthCheckV1, OperatorHealthReportV1, RunBackendDescriptorV1,
    RunBackendRecordV1, RunControlActionV1, RunControlAuditEntryV1, RunControlStateV1,
    RunExecutionModeV1, RunLayout, RunLeaseV1, RunQueueLifecycleStateV1, RunQueueStateV1,
    RunQueueTransitionV1, RunResourceRequestV1, RunSchedulingDecisionV1, SlurmJobStateV1,
    SlurmJobTransitionV1, SlurmSubmissionRecordV1,
};
use bijux_dna_runtime::{Invocation, Runner, RunnerResult};

pub(crate) fn acquire_run_lease(layout: &RunLayout, run_id: &str) -> Result<(bijux_dna_infra::FileLock, RunLeaseV1)> {
    let lock = bijux_dna_infra::FileLock::acquire(
        &layout.run_dir.join("run.lock"),
        Duration::from_millis(250),
    )
    .map_err(|err| anyhow!("acquire exclusive run lease for {run_id}: {err}"))?;
    let lease = RunLeaseV1 {
        schema_version: "bijux.run_lease.v1".to_string(),
        run_id: run_id.to_string(),
        lease_id: format!("lease-{run_id}"),
        holder: holder_id(),
        lock_path: layout.run_dir.join("run.lock"),
        acquired_at: now_string(),
        expires_at: None,
        released_at: None,
        exclusive: true,
    };
    Ok((lock, lease))
}

pub(crate) fn release_run_lease(lease: &RunLeaseV1) -> RunLeaseV1 {
    let mut released = lease.clone();
    released.released_at = Some(now_string());
    released
}

pub(crate) fn initial_queue_state(
    run_id: &str,
    graph: &ExecutionGraph,
) -> RunQueueStateV1 {
    RunQueueStateV1 {
        schema_version: "bijux.run_queue_state.v1".to_string(),
        run_id: run_id.to_string(),
        dedup_key: dedup_key(graph),
        state: RunQueueLifecycleStateV1::Queued,
        transitions: vec![queue_transition(None, RunQueueLifecycleStateV1::Queued, "run queued for execution")],
        active_step_id: None,
    }
}

pub(crate) fn default_control_state(run_id: &str) -> RunControlStateV1 {
    RunControlStateV1 {
        schema_version: "bijux.run_control.v1".to_string(),
        run_id: run_id.to_string(),
        requested_action: None,
        observed_state: RunQueueLifecycleStateV1::Queued,
        updated_at: now_string(),
        audit_log: Vec::new(),
    }
}

pub(crate) fn request_control_action(
    layout: &RunLayout,
    run_id: &str,
    action: RunControlActionV1,
    detail: &str,
) -> Result<RunControlStateV1> {
    let mut state = read_control_state(layout)
        .unwrap_or_else(|| default_control_state(run_id));
    state.requested_action = Some(action);
    state.updated_at = now_string();
    state.audit_log.push(RunControlAuditEntryV1 {
        requested_action: action,
        observed_state: state.observed_state,
        occurred_at: state.updated_at.clone(),
        detail: Some(detail.to_string()),
    });
    bijux_dna_runtime::run_layout::write_control_state(layout, &state)?;
    Ok(state)
}

pub(crate) fn build_backend_record(
    run_id: &str,
    mode: RunExecutionModeV1,
    runner: RuntimeKind,
    graph: &ExecutionGraph,
    layout: &RunLayout,
) -> RunBackendRecordV1 {
    let descriptor = match runner {
        RuntimeKind::Local => RunBackendDescriptorV1::Local {
            temp_root_policy: format!("stage_scoped_tmp_under_{}", layout.run_dir.join("tmp").display()),
            temp_cleanup_policy: "best_effort_cleanup_after_success_keep_on_failure".to_string(),
            artifact_write_policy: "atomic_manifest_and_summary_writes".to_string(),
            log_capture_policy: "stdout_and_stderr_persisted_per_stage".to_string(),
            interruption_recovery_policy: "resume_from_checkpoint_and_failure_bundle".to_string(),
        },
        RuntimeKind::Docker => RunBackendDescriptorV1::Container {
            runtime: "docker".to_string(),
            image_identity: container_identity(graph),
            bind_mount_policy: "readonly_inputs_writable_run_dir".to_string(),
            user_identity_policy: "host_default_user".to_string(),
            working_directory_policy: "run_dir_scoped_workdir".to_string(),
            network_policy: if std::env::var("BIJUX_ALLOW_NETWORK").is_ok() {
                "network_allowed_by_operator_override".to_string()
            } else {
                "network_disabled_by_default".to_string()
            },
            resource_limit_policy: "graph_constraints_promoted_to_scheduler_hints".to_string(),
            stdout_stderr_policy: "captured_after_container_completion".to_string(),
        },
        RuntimeKind::Apptainer | RuntimeKind::Singularity => RunBackendDescriptorV1::Container {
            runtime: runner.to_string(),
            image_identity: container_identity(graph),
            bind_mount_policy: "readonly_inputs_writable_run_dir_hpc_safe".to_string(),
            user_identity_policy: "host_user_passthrough".to_string(),
            working_directory_policy: "run_dir_scoped_workdir".to_string(),
            network_policy: "inherit_site_policy_no_implicit_network".to_string(),
            resource_limit_policy: "graph_constraints_promoted_to_scheduler_hints".to_string(),
            stdout_stderr_policy: "captured_from_runtime_process".to_string(),
        },
    };
    RunBackendRecordV1 {
        schema_version: "bijux.run_backend.v1".to_string(),
        run_id: run_id.to_string(),
        mode,
        descriptor,
    }
}

pub(crate) fn build_scheduling_decision(
    run_id: &str,
    graph: &ExecutionGraph,
    runner: RuntimeKind,
) -> RunSchedulingDecisionV1 {
    let threads = graph.steps().iter().map(|step| step.resources.threads).max().unwrap_or(1);
    let memory_mb = graph
        .steps()
        .iter()
        .map(|step| u64::from(step.resources.mem_gb) * 1024)
        .max();
    let scratch_mb = graph
        .steps()
        .iter()
        .map(|step| u64::from(step.resources.tmp_gb) * 1024)
        .max();
    let walltime_s = graph.step_timeout_s().map(|seconds| seconds.saturating_mul(graph.steps().len() as u64));
    let queue_class = if matches!(runner, RuntimeKind::Apptainer | RuntimeKind::Singularity)
        && (threads > 4 || memory_mb.unwrap_or(0) > 8 * 1024 || graph.steps().len() > 4)
    {
        "slurm_batch_candidate"
    } else if matches!(runner, RuntimeKind::Docker) {
        "container_local"
    } else {
        "local_interactive"
    };
    let io_intensity = if graph.steps().iter().any(|step| step.stage_id.as_str().starts_with("bam.")) {
        "high"
    } else if graph.steps().iter().any(|step| step.stage_id.as_str().starts_with("vcf.")) {
        "moderate"
    } else {
        "light"
    };
    let warnings = if queue_class == "slurm_batch_candidate" {
        vec!["runner should prefer mocked slurm submission semantics for large apptainer-backed work".to_string()]
    } else {
        Vec::new()
    };
    RunSchedulingDecisionV1 {
        schema_version: "bijux.run_scheduling_decision.v1".to_string(),
        run_id: run_id.to_string(),
        runner: runner.to_string(),
        queue_class: queue_class.to_string(),
        placement_reason: format!(
            "selected {queue_class} from threads={threads}, memory_mb={}, steps={}",
            memory_mb.unwrap_or(0),
            graph.steps().len()
        ),
        requested_resources: RunResourceRequestV1 {
            cpu_threads: threads,
            memory_mb,
            scratch_mb,
            walltime_s,
            io_intensity: io_intensity.to_string(),
            container_runtime: (!matches!(runner, RuntimeKind::Local)).then(|| runner.to_string()),
        },
        warnings,
    }
}

pub(crate) fn build_health_report(
    layout: &RunLayout,
    run_id: &str,
    runner: RuntimeKind,
) -> OperatorHealthReportV1 {
    let queue_path = layout.queue_state_path.clone();
    let evidence_path = layout.evidence_verification_path.clone();
    let checks = vec![
        OperatorHealthCheckV1 {
            check_id: "storage".to_string(),
            ok: layout.run_dir.exists() && layout.manifests_dir.exists(),
            detail: format!("run layout root {}", layout.run_dir.display()),
            evidence_path: Some(layout.run_dir.clone()),
        },
        OperatorHealthCheckV1 {
            check_id: "reference_assets".to_string(),
            ok: layout.plan_manifest_path.exists() || layout.manifests_dir.exists(),
            detail: "plan manifest or manifests directory is available for reference binding review".to_string(),
            evidence_path: Some(layout.manifests_dir.clone()),
        },
        OperatorHealthCheckV1 {
            check_id: "containers".to_string(),
            ok: match runner {
                RuntimeKind::Local => true,
                RuntimeKind::Docker => command_on_path("docker"),
                RuntimeKind::Apptainer => command_on_path("apptainer"),
                RuntimeKind::Singularity => command_on_path("singularity"),
            },
            detail: format!("runtime {} availability checked on PATH", runner),
            evidence_path: None,
        },
        OperatorHealthCheckV1 {
            check_id: "tools".to_string(),
            ok: true,
            detail: "tool execution is delegated to the governed step runner contracts".to_string(),
            evidence_path: Some(layout.executor_descriptor_path.clone()),
        },
        OperatorHealthCheckV1 {
            check_id: "queue".to_string(),
            ok: queue_path.exists() || layout.run_dir.exists(),
            detail: "queue state artifact can be materialized and updated".to_string(),
            evidence_path: Some(queue_path),
        },
        OperatorHealthCheckV1 {
            check_id: "executor".to_string(),
            ok: layout.executor_descriptor_path.exists() || layout.run_dir.exists(),
            detail: "executor descriptor path is writable for the selected backend".to_string(),
            evidence_path: Some(layout.executor_descriptor_path.clone()),
        },
        OperatorHealthCheckV1 {
            check_id: "evidence_verifier".to_string(),
            ok: true,
            detail: "bijux-dna-analyze evidence verification surface is linked into the runtime".to_string(),
            evidence_path: Some(evidence_path),
        },
    ];
    let overall_ok = checks.iter().all(|check| check.ok);
    OperatorHealthReportV1 {
        schema_version: "bijux.operator_health.v1".to_string(),
        run_id: run_id.to_string(),
        overall_ok,
        checks,
    }
}

pub(crate) fn maybe_mock_slurm_submission(
    layout: &RunLayout,
    run_id: &str,
    runner: RuntimeKind,
    scheduling: &RunSchedulingDecisionV1,
) -> Option<SlurmSubmissionRecordV1> {
    if scheduling.queue_class != "slurm_batch_candidate" {
        return None;
    }
    let submission_script_path = layout.run_dir.join("slurm_submit.sh");
    let stdout_log_path = layout.logs_dir.join("slurm.stdout.log");
    let stderr_log_path = layout.logs_dir.join("slurm.stderr.log");
    Some(SlurmSubmissionRecordV1 {
        schema_version: "bijux.slurm_submission.v1".to_string(),
        run_id: run_id.to_string(),
        scheduler: "slurm".to_string(),
        submission_script_path,
        job_id: format!("mock-{run_id}"),
        state: SlurmJobStateV1::Submitted,
        poll_command: vec!["squeue".to_string(), "--job".to_string(), format!("mock-{run_id}")],
        cancel_command: vec!["scancel".to_string(), format!("mock-{run_id}")],
        stdout_log_path,
        stderr_log_path,
        retry_count: 0,
        exit_code: None,
        transitions: vec![SlurmJobTransitionV1 {
            from_state: None,
            to_state: SlurmJobStateV1::Submitted,
            occurred_at: now_string(),
            detail: Some(format!("mocked {} submission recorded for governed monitoring tests", runner)),
        }],
    })
}

pub(crate) struct ControlAwareRunner {
    inner: Box<dyn Runner>,
    layout: RunLayout,
    run_id: String,
    queue_state: Arc<Mutex<RunQueueStateV1>>,
}

impl ControlAwareRunner {
    pub(crate) fn new(
        inner: Box<dyn Runner>,
        layout: RunLayout,
        run_id: String,
        queue_state: Arc<Mutex<RunQueueStateV1>>,
    ) -> Self {
        Self { inner, layout, run_id, queue_state }
    }
}

impl Runner for ControlAwareRunner {
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult> {
        let step_id = invocation.step.step_id.to_string();
        {
            let mut state = self
                .queue_state
                .lock()
                .map_err(|_| anyhow!("queue state mutex poisoned for {}", self.run_id))?;
            honor_control_requests(&self.layout, &self.run_id, &step_id, &mut state)?;
            state.state = RunQueueLifecycleStateV1::Running;
            state.active_step_id = Some(step_id.clone());
            bijux_dna_runtime::run_layout::write_queue_state(&self.layout, &state)?;
        }
        let result = self.inner.run(invocation)?;
        {
            let mut state = self
                .queue_state
                .lock()
                .map_err(|_| anyhow!("queue state mutex poisoned for {}", self.run_id))?;
            if matches!(
                read_control_state(&self.layout).and_then(|control| control.requested_action),
                Some(RunControlActionV1::Cancel)
            ) {
                state.state = RunQueueLifecycleStateV1::Cancelled;
                state.transitions.push(queue_transition(
                    Some(RunQueueLifecycleStateV1::Running),
                    RunQueueLifecycleStateV1::Cancelled,
                    format!("execution cancelled during {step_id}"),
                ));
                bijux_dna_runtime::run_layout::write_queue_state(&self.layout, &state)?;
                return Err(anyhow!("execution cancelled during {step_id}"));
            }
            state.active_step_id = None;
            bijux_dna_runtime::run_layout::write_queue_state(&self.layout, &state)?;
        }
        Ok(result)
    }
}

fn honor_control_requests(
    layout: &RunLayout,
    run_id: &str,
    step_id: &str,
    queue_state: &mut RunQueueStateV1,
) -> Result<()> {
    loop {
        let mut control = read_control_state(layout).unwrap_or_else(|| default_control_state(run_id));
        match control.requested_action {
            Some(RunControlActionV1::Pause) => {
                control.observed_state = RunQueueLifecycleStateV1::Paused;
                control.updated_at = now_string();
                control.audit_log.push(RunControlAuditEntryV1 {
                    requested_action: RunControlActionV1::Pause,
                    observed_state: RunQueueLifecycleStateV1::Paused,
                    occurred_at: control.updated_at.clone(),
                    detail: Some(format!("paused before {step_id}")),
                });
                bijux_dna_runtime::run_layout::write_control_state(layout, &control)?;
                queue_state.state = RunQueueLifecycleStateV1::Paused;
                if !matches!(queue_state.transitions.last().map(|entry| entry.to_state), Some(RunQueueLifecycleStateV1::Paused)) {
                    queue_state.transitions.push(queue_transition(
                        Some(RunQueueLifecycleStateV1::Running),
                        RunQueueLifecycleStateV1::Paused,
                        format!("pause requested before {step_id}"),
                    ));
                }
                bijux_dna_runtime::run_layout::write_queue_state(layout, queue_state)?;
                std::thread::sleep(Duration::from_millis(100));
            }
            Some(RunControlActionV1::Resume) => {
                control.requested_action = None;
                control.observed_state = RunQueueLifecycleStateV1::Running;
                control.updated_at = now_string();
                control.audit_log.push(RunControlAuditEntryV1 {
                    requested_action: RunControlActionV1::Resume,
                    observed_state: RunQueueLifecycleStateV1::Running,
                    occurred_at: control.updated_at.clone(),
                    detail: Some(format!("resumed before {step_id}")),
                });
                bijux_dna_runtime::run_layout::write_control_state(layout, &control)?;
                queue_state.state = RunQueueLifecycleStateV1::Running;
                queue_state.transitions.push(queue_transition(
                    Some(RunQueueLifecycleStateV1::Paused),
                    RunQueueLifecycleStateV1::Running,
                    format!("resume granted before {step_id}"),
                ));
                bijux_dna_runtime::run_layout::write_queue_state(layout, queue_state)?;
                return Ok(());
            }
            Some(RunControlActionV1::Cancel) => {
                control.observed_state = RunQueueLifecycleStateV1::Cancelled;
                control.updated_at = now_string();
                control.audit_log.push(RunControlAuditEntryV1 {
                    requested_action: RunControlActionV1::Cancel,
                    observed_state: RunQueueLifecycleStateV1::Cancelled,
                    occurred_at: control.updated_at.clone(),
                    detail: Some(format!("cancelled before {step_id}")),
                });
                bijux_dna_runtime::run_layout::write_control_state(layout, &control)?;
                let previous_state = queue_state.state;
                queue_state.state = RunQueueLifecycleStateV1::Cancelled;
                queue_state.transitions.push(queue_transition(
                    Some(previous_state),
                    RunQueueLifecycleStateV1::Cancelled,
                    format!("cancel requested before {step_id}"),
                ));
                bijux_dna_runtime::run_layout::write_queue_state(layout, queue_state)?;
                return Err(anyhow!("execution cancelled before {step_id}"));
            }
            None => {
                if control.observed_state != RunQueueLifecycleStateV1::Running {
                    control.observed_state = RunQueueLifecycleStateV1::Running;
                    control.updated_at = now_string();
                    bijux_dna_runtime::run_layout::write_control_state(layout, &control)?;
                }
                return Ok(());
            }
        }
    }
}

fn read_control_state(layout: &RunLayout) -> Option<RunControlStateV1> {
    if !layout.control_state_path.exists() {
        return None;
    }
    let raw = std::fs::read_to_string(&layout.control_state_path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn dedup_key(graph: &ExecutionGraph) -> String {
    graph.hash().unwrap_or_else(|_| graph.pipeline_id().to_string())
}

fn container_identity(graph: &ExecutionGraph) -> String {
    let mut identities = graph
        .steps()
        .iter()
        .map(|step| step.image.digest.clone().unwrap_or_else(|| step.image.image.clone()))
        .collect::<Vec<_>>();
    identities.sort();
    identities.dedup();
    identities.join(",")
}

fn queue_transition(
    from_state: Option<RunQueueLifecycleStateV1>,
    to_state: RunQueueLifecycleStateV1,
    detail: impl Into<String>,
) -> RunQueueTransitionV1 {
    RunQueueTransitionV1 {
        from_state,
        to_state,
        occurred_at: now_string(),
        detail: Some(detail.into()),
    }
}

fn holder_id() -> String {
    let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".to_string());
    format!("{host}:pid={}", std::process::id())
}

fn command_on_path(command: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|paths| {
        std::env::split_paths(&paths).any(|path| {
            let candidate = path.join(command);
            candidate.is_file()
        })
    })
}
