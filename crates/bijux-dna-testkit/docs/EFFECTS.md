# Effects

This crate is test-only support code. Its effects must be deterministic,
minimal, and appropriate for helper code used by other crates' tests.

## Allowed Effects

- Read caller-provided fixture and policy text files.
- Parse caller-provided JSON fixture files.
- Create temporary directories through `tempfile`.
- Derive contained paths below `TEST_TMP_DIR` or the process temp directory.
- Read environment variables used only for deterministic snapshot redaction:
  `CARGO_MANIFEST_DIR`, `COMPUTERNAME`, `HOME`, `HOSTNAME`, `LC_ALL`, `LOGNAME`,
  `TEMP`, `TEST_TMP_DIR`, `TMP`, `TMPDIR`, `TZ`, and `USER`.
- Set `TZ=UTC` and `LC_ALL=C` through `install_snapshot_env` for tests.

## Forbidden Effects

- Process spawning or shell execution.
- Network access.
- Domain-specific production behavior.
- Source mutation, generated config writes, or workspace-wide artifact writes.
- Required dependencies on product crates.

## Enforcement

- `tests/boundaries.rs` locks layout, dependency, docs, and effect boundaries.
- `tests/guardrails.rs` runs the workspace guardrail policy.
- `tests/schemas.rs` locks public API and snapshot normalization behavior.
