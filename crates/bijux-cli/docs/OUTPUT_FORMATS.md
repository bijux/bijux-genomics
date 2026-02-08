# Output Formats

## Text (terminal)
- Help output is deterministic and snapshotted.
- Snapshots live under `crates/bijux-cli/tests/snapshots/*.txt`.

## JSON artifacts (dry-run / run)
- `run_manifest.json` follows the runtime schema snapshot in
  `crates/bijux-runtime/tests/fixtures/runtime_schema/default/run_manifest_v1.json`.
- Execution graphs are canonical JSON and validated by core contracts.

## Reports
- CLI renders reports via API helpers; schemas are tracked in the API and runtime snapshots.
