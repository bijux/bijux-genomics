# Effects

`bijux-dna-policies` is a read-only policy crate in production source.

## Allowed Production Effects
- Read repository files through deterministic traversal.
- Parse manifests, docs, source files, fixtures, and governed config files.
- Return diagnostics through `anyhow` and policy assertion messages.

## Forbidden Production Effects
- Process execution.
- Network access.
- Container or Docker APIs.
- File creation, deletion, or mutation.
- Snapshot blessing or generated-output rewrites.
- Runtime tool discovery.

## Test Effects
Tests may run Cargo metadata inspection and read fixtures, snapshots, and workspace files. Tests must still avoid mutating repository state except when a developer intentionally updates governed snapshots as part of a reviewed change.

## Enforcement
Effect boundaries are locked by source scans, dependency graph checks, and `docs/DEPENDENCIES.md`.
