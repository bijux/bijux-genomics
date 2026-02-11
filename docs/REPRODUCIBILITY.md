# REPRODUCIBILITY

Bijux reproducibility is enforced by manifests, canonical hashing, and immutable tool pins.

## Run Artifacts
Each run writes:
- `run_manifest.json`
- `profile_manifest.json`
- `run_manifest.lock.json`
- `tool_provenance.json`
- per-stage `metrics_envelope.json`

## Manifest Contracts
- `run_manifest.json` describes executed stages, outputs, provenance, and tool invocations.
- `profile_manifest.json` captures stage list, tool selections, param hashes, invariants preset, and schema versions.
- `run_manifest.lock.json` binds run + profile manifests to immutable digest-resolved tool entries.

## Hashes and Determinism
- Canonical JSON serialization is used before hashing.
- Reports embed `manifest_signature_sha256` from `run_manifest.json`.
- Metrics provenance carries `manifest_hash` so metrics are traceable to the producing manifest.

## Offline Reproduction
1. Use `run_manifest.lock.json` and local tool cache/images.
2. Verify `run_manifest.json` hash matches lock.
3. Re-run with same params and inputs; compare manifest/profile hashes.

## Change Contracts
- Default parameter changes must alter profile hashes and snapshots.
- Tool pin changes must alter `configs/tool_registry.lock.sha256`.
