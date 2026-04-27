# Production Guarantees

## Purpose
State what the project guarantees for production runs, and what remains best-effort.

## Scope
Applies to CI gates, runtime isolation, config/registry contracts, and report artifacts.

## Non-goals
- Guaranteeing upstream tool behavior bugs are impossible.
- Guaranteeing cluster-specific performance outcomes.

## Contracts
- Contract configs are schema-validated and owner-mapped in [CI.md](CI.md).
- Production tool versions are pinned; floating versions are rejected through
  [TOOL_ADMISSION.md](../50-reference/TOOL_ADMISSION.md).
- Isolated runs must keep writable build/temp paths under `ISO_ROOT` as defined in
  [ISOLATION.md](ISOLATION.md).
- Generated contract docs must be regenerated from source-of-truth inputs through
  [DOCS_BUILD_REPRODUCIBLE.md](DOCS_BUILD_REPRODUCIBLE.md).

## Deterministic / Reproducible Guarantees
- `make ci` gate sequence and isolate contract are fixed.
- Registry lock and config snapshots are reproducible from canonical inputs.
- Report and manifest contracts are validated by policy/contract tests under
  [REPRODUCIBILITY.md](REPRODUCIBILITY.md).

## Best-effort Areas
- Third-party upstream availability and package mirrors.
- Hardware-specific runtime variance (especially HPC filesystem throughput).
- Optional or planned tools not yet promoted to production.

## HPC Forward-compat
- When HPC profile is enabled, paths, cache, and outputs move to HPC-configured roots.
- Container pulling behavior may shift from local cache to site-managed storage.
- Determinism contracts remain; only physical storage/runtime location changes.

## Examples
- A production registry edit without lock update is rejected.
- A non-isolated execution attempt fails with actionable isolate guidance.

## Failure modes
- Using non-pinned tools breaks reproducibility guarantees.
- Writing outputs outside isolate/artifacts breaks contract enforcement.
