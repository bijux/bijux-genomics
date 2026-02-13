# ISOLATION

## Purpose
Define the runtime contract for `bin/isolate` and `bin/require-isolate`.

## `bin/isolate` Contract
- Computes `ISO_TAG` if unset: `<utc-timestamp>-<git-short-sha>-<pid>`.
- Computes `ISO_ROOT` if unset: `artifacts/isolates/<ISO_TAG>`.
- Creates isolate directories under `ISO_ROOT`:
  - `target/`
  - `cargo-home/`
  - `tmp/`
  - `logs/`
  - `out/`
- Exports:
  - `ISO_TAG`
  - `ISO_ROOT`
  - `CARGO_TARGET_DIR=$ISO_ROOT/target`
  - `CARGO_HOME=$ISO_ROOT/cargo-home`
  - `TMPDIR=$ISO_ROOT/tmp` (and `TMP`, `TEMP`)

## Flags
- `--print-root`: prints computed `ISO_ROOT` and exits.
- `--print-env`: prints key isolate env vars as stable `KEY=VALUE` lines and exits.
- `--print-tag`: prints computed `ISO_TAG` and exits.
- `--tag <name>`: sets explicit isolate tag used in `ISO_ROOT`.
- `--require-clean`: refuses to run when `ISO_ROOT` already exists unless `--reuse` is passed.
- `--require-empty-target-dir`: refuses to run when `ISO_ROOT` has any `target-*` entries unless `--reuse` is passed.
- `--reuse`: explicitly allows reuse of an existing isolate root.

## Tag Behavior
- If caller passes `--tag`, that value is authoritative.
- Otherwise if caller sets `ISO_TAG`, `bin/isolate` uses it.
- If caller sets `ISO_ROOT`, it must include `ISO_TAG` for predictable naming.
- Otherwise defaults are derived as above.

## Allowed Outputs
- Scripts and tooling must write only via `ISO_ROOT`-scoped env vars or under `artifacts/`.
- Scripts must not hardcode `artifacts/isolates/<...>` paths.

## `bin/require-isolate`
- Verifies required isolate env vars are present and path-scoped under `ISO_ROOT`.
- `--explain` prints a user-facing diagnostic and invocation guidance.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.
