# RUNTIME_CONTRACT

## Canonical narrative
`RunLayout` defines where artifacts live. `RunManifest` summarizes the run. `RunRecord` captures per-step execution. Provenance ties tool identity to inputs. Telemetry events provide execution traces.

## Owned JSON artifacts
- `run_layout.json`
- `run_manifest.json`
- `run_record.json`
- `run_provenance.json`

See `ARTIFACTS.md` for the full file inventory under a run layout.

## Reference
See `tests/reference/reference_example.rs` for a minimal end-to-end example.

## Example
```json
{ "schema_version": "bijux.run_manifest.v1", "graph_hash": "sha256:..." }
```
