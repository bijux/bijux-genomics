# bin

Purpose: runtime boundary helpers for strict isolated execution.

## Isolation Contract
- `bin/isolate` creates and exports an isolated runtime rooted at `artifacts/isolates/<tag>`.
- `bin/require-isolate` validates that the current shell already satisfies the isolate contract.

Required environment variables:
- `ISO_TAG`
- `ISO_RUN_ID`
- `ISO_ROOT`
- `CARGO_TARGET_DIR`
- `CARGO_HOME`
- `TMPDIR`
- `TMP`
- `TEMP`

Required path invariants:
- `ISO_ROOT` must be under `artifacts/isolates/`.
- `CARGO_TARGET_DIR`, `CARGO_HOME`, `TMPDIR`, `TMP`, `TEMP` must all be inside `ISO_ROOT`.

## Enforced Behavior
- Deterministic env defaults are exported by `bin/isolate` (`TZ=UTC`, `LC_ALL=C`).
- Isolate can enforce clean roots (`--require-clean`) and target hygiene (`--require-empty-target-dir`).
- `require-isolate` fails fast when contract vars/paths are missing or invalid.

## Forbidden Behavior
- No implicit fallback from non-isolated execution to auto-created isolate dirs.
- No writes outside `ISO_ROOT` for isolate-managed temp/build paths.
- No scripts should bypass `require-isolate` for commands that mutate build/test outputs.

## Usage
- Print computed root: `./bin/isolate --print-root`
- Print isolate env contract: `./bin/isolate --print-env`
- Run command in clean isolate:
  - `./bin/isolate --tag demo --require-clean sh -ceu 'echo "$ISO_ROOT"'`
- Enforce target hygiene:
  - `./bin/isolate --tag demo --require-empty-target-dir sh -ceu 'true'`
- Validate current shell:
  - `./bin/require-isolate`
- Explain failure and remediation:
  - `./bin/require-isolate --explain`
