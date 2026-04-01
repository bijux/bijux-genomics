# EXAMPLE_RUN

## Fake run walkthrough
1. Create layout.
2. Prepare a tool-run directory under `tools/<tool>/run/<run_id>/`.
3. Write the execution manifest and tool-run manifest.
4. Analyze consumes the emitted manifests and run artifacts.

Artifacts:
- execution_manifest.json
- tools/<tool>/run/<run_id>/run_manifest.json
- tools/<tool>/run/<run_id>/run_artifacts/telemetry/events.jsonl
