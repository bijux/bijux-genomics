# scripts

Purpose: strict index of supported script areas and allowed usage.

Allowed subdirectories:
- `scripts/checks`: CI/make policy and safety checks.
- `scripts/containers`: Docker/Apptainer build, lint, and smoke operations.
- `scripts/docs`: docs validation/generation entrypoints.
- `scripts/domain`: domain validation and drift checks.
- `scripts/hpc`: cluster-specific operational helpers.
- `scripts/lab`: manual lab workflow entrypoints.
- `scripts/smoke`: local smoke entrypoints via `scripts/smoke/run.sh`.
- `scripts/test`: test orchestration helpers.
- `scripts/tooling`: repo tooling wrappers; python logic lives in `scripts/tooling/python/`.

Internal-only:
- `scripts/_lib`: shared shell helpers only.
- `scripts/experimental`: quarantined non-supported scripts.
- `scripts/assets`: asset refresh helpers.

The source of truth for supported scripts is `scripts/checks/supported_scripts.txt`.

Requires: bash, rg, coreutils (plus script-specific tools documented inline).
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.
