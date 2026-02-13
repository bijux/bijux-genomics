# Scripts Layout

Top-level script categories:

- `scripts/check-*.sh`: CI guardrails and policy checks.
- `scripts/containers/`: container build/smoke entrypoints.
- `scripts/docs/`: docs-only entrypoints and pinned requirements.
- `scripts/hpc/lunarc/`: Lunarc sync utilities.
- `scripts/lab/`: manual lab workflows (never invoked from CI workflows).
- `scripts/smoke/`: local quick smoke commands.
- `scripts/_lib/`: shared shell helpers (not direct entrypoints).
- `scripts/experimental/`: unsupported scripts not wired to Make targets.

Supported entrypoints policy:

- A script is **supported** only if referenced by a Make target.
- Non-Make scripts must live under `scripts/experimental/`.
- CI must execute scripts via Make targets only.

Utilities:

- `scripts/inventory.sh` generates `artifacts/scripts/inventory.md`.
- `scripts/check-ci-shell-scripts.sh` lints shell scripts used by Make/CI.
