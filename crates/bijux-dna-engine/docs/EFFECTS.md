# Effects

`bijux-dna-engine` is an orchestrator. It may coordinate a caller-provided
`Runner` and verify declared filesystem artifacts, but it must not perform
backend execution effects itself.

## Allowed Effects

- Write `execution_record.json` under each step's `run_artifacts/` directory.
- Create the step `run_artifacts/` directory when recording execution truth.
- Read declared output paths to verify presence, non-empty payloads, and JSON
  parseability for JSON roles.
- Read required run artifacts under `run_artifacts/`.
- Emit tracing events and `EngineHooks` events.

## Forbidden Effects

- Spawn processes.
- Invoke Docker, Apptainer, Singularity, or any container runtime.
- Access the network.
- Select tools or containers.
- Plan workflows or interpret domain semantics.
- Read or write ad hoc files outside declared outputs and step
  `run_artifacts/`.

Backend execution effects belong in runner/runtime layers.

## Recording Truth Set

Every executed step must have these required per-step artifacts:

- `effective_config.json`
- `tool_invocation.json`
- `execution_record.json`
- `metrics.json`
- `stage_report.json`

Steps with declared `metrics_schema_ids` must also emit:

- `metrics_envelope.json`

Minimal layout:

```text
run_123/
  stage_0/
    run_artifacts/
      effective_config.json
      tool_invocation.json
      execution_record.json
      metrics.json
      stage_report.json
      metrics_envelope.json
```

`tool_invocation.json` records tool identity and invocation facts.
`effective_config.json` records resolved configuration.
`execution_record.json` records timing, attempt, and exit status.
`metrics_envelope.json` records typed metrics payloads whose schema must be
declared by the step.

## Error Context

Contract errors include the step id, artifact id, path, and message. Execution
errors wrap runner failures and orchestration failures; backend-specific context
belongs in runner-produced truth artifacts.

## Enforcement

- `tests/boundaries/effect_boundary.rs` rejects process/container references.
- `tests/contracts/recording/recording_completeness.rs` verifies required
  recording artifacts.
- `tests/contracts/recording/docs_recording_truth_set.rs` verifies this
  recording truth set remains documented.
