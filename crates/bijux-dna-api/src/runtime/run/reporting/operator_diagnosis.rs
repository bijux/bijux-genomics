use super::Result;
use crate::request_args::{
    OperatorDiagnosisCommandV1, OperatorDiagnosisRequestV1, OperatorDiagnosisResponseV1,
};

/// Build operator diagnosis commands and state summary for a run directory.
///
/// # Errors
/// Returns an error if run-control, queue, or health contracts are unreadable.
pub fn operator_diagnosis(request: &OperatorDiagnosisRequestV1) -> Result<OperatorDiagnosisResponseV1> {
    let layout =
        bijux_dna_runtime::run_layout::RunLayout::from_run_dir(request.run_dir.to_path_buf());
    let run_state = read_json::<bijux_dna_runtime::run_layout::RunStateV1>(&layout.run_state_path)?;
    let queue_state = read_json::<bijux_dna_runtime::run_layout::RunQueueStateV1>(&layout.queue_state_path)?;
    let control_state = read_json::<bijux_dna_runtime::run_layout::RunControlStateV1>(&layout.control_state_path)?;
    let health_report = read_json::<bijux_dna_runtime::run_layout::OperatorHealthReportV1>(&layout.health_report_path)?;

    let run_id = run_state
        .as_ref()
        .map(|value| value.run_id.clone())
        .or_else(|| queue_state.as_ref().map(|value| value.run_id.clone()))
        .or_else(|| control_state.as_ref().map(|value| value.run_id.clone()))
        .unwrap_or_else(|| {
            request
                .run_dir
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("run")
                .to_string()
        });

    let mut commands = vec![
        command(
            "inspect_run_state",
            vec![
                "cat".to_string(),
                layout.run_state_path.display().to_string(),
            ],
            "inspect lifecycle mode/state transitions",
            Some(layout.run_state_path.clone()),
        ),
        command(
            "inspect_queue_state",
            vec![
                "cat".to_string(),
                layout.queue_state_path.display().to_string(),
            ],
            "inspect scheduler queue state and active step",
            Some(layout.queue_state_path.clone()),
        ),
        command(
            "inspect_control_audit",
            vec![
                "cat".to_string(),
                layout.control_state_path.display().to_string(),
            ],
            "inspect pause/resume/cancel audit trail",
            Some(layout.control_state_path.clone()),
        ),
        command(
            "inspect_operator_health",
            vec![
                "cat".to_string(),
                layout.health_report_path.display().to_string(),
            ],
            "inspect runner/storage/tools health checks",
            Some(layout.health_report_path.clone()),
        ),
    ];

    if layout.failure_path.exists() {
        commands.push(command(
            "inspect_failure_record",
            vec!["cat".to_string(), layout.failure_path.display().to_string()],
            "inspect failure code and retryability",
            Some(layout.failure_path.clone()),
        ));
    }
    if layout.slurm_submission_path.exists() {
        commands.push(command(
            "inspect_slurm_submission",
            vec![
                "cat".to_string(),
                layout.slurm_submission_path.display().to_string(),
            ],
            "inspect mocked/recorded slurm lifecycle",
            Some(layout.slurm_submission_path.clone()),
        ));
    }

    Ok(OperatorDiagnosisResponseV1 {
        schema_version: "bijux.operator_diagnosis.v1".to_string(),
        run_dir: request.run_dir.clone(),
        run_id,
        queue_state: queue_state.as_ref().map(|value| value.state),
        requested_action: control_state.as_ref().and_then(|value| value.requested_action),
        health_ok: health_report.as_ref().is_some_and(|value| value.overall_ok),
        has_failure_record: layout.failure_path.exists(),
        commands,
    })
}

fn command(
    command_id: &str,
    argv: Vec<String>,
    purpose: &str,
    evidence_path: Option<std::path::PathBuf>,
) -> OperatorDiagnosisCommandV1 {
    OperatorDiagnosisCommandV1 {
        command_id: command_id.to_string(),
        argv,
        purpose: purpose.to_string(),
        evidence_path,
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let raw = std::fs::read(path)?;
    let value = serde_json::from_slice::<T>(&raw)?;
    Ok(Some(value))
}
