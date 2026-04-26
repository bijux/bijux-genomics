# bijux-dna-environment Contracts

This crate owns the environment contracts consumed by API, runner, and environment QA layers.

## Owned Contracts

- `PlatformSpec`: selected runtime, architecture, image prefix, and container directory.
- `RuntimeKind`: supported runtime enum with stable string parsing and display.
- `RuntimeSpec`: compatibility check between a platform and selected runner.
- `ToolImageSpec`: catalog entry for tool name, version, optional digest, enabled state, and
  shipping policy.
- `ResolvedImage`: deterministic image reference plus runner and architecture.
- `ImageRef`: deterministic tag formatting helper.
- `ReferenceRecord`: digest-addressed reference cache record.
- `ReferenceBuildRequest`: explicit switches for optional reference indexes.

## Resolution Precedence

1. Explicit digest in `ToolImageSpec.digest`.
2. Versioned image tag from `ToolImageSpec.version`.
3. Registry-file hydration when the local pin file supplies a digest and the catalog entry does not.

Network lookups are not part of this contract.

## Schema Fixtures

Schema fixtures live under `tests/fixtures/env_schema/default/` and are checked by:

- `tests/schemas/schema/schema_snapshots.rs`
- `tests/contracts/matrix/reference_matrix.rs`

## Change Rules

- Adding an optional serialized field is compatible when defaulted and documented.
- Renaming, removing, or changing the meaning of a serialized field is breaking.
- Changing resolution precedence is breaking.
- Adding a managed host command requires `COMMANDS.md` and boundary tests.
- New catalog inputs must have fixture or contract coverage before becoming public behavior.

## Failure Patterns

- Unknown platform names return `EnvError::Platform`.
- Invalid runner names return `EnvError::Parse`.
- Empty tool names, image prefixes, architectures, versions, or digests return `EnvError::Image`.
- Missing Dockerfile version ARGs return `EnvError::Dockerfile`.
- Missing local config files surface as `EnvError::Io`.
