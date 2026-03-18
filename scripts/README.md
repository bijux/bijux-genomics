# scripts

Purpose: strict index of supported script areas and allowed usage.

Allowed subdirectories:
- `scripts/containers`: Docker/Apptainer build, lint, and smoke operations.
- `scripts/docs`: docs validation/generation entrypoints.
- `scripts/examples`: examples index, checks, and runner entrypoints.
- `scripts/hpc`: cluster-specific operational helpers.
- `scripts/lab`: manual lab workflow entrypoints.
- `scripts/smoke`: local smoke entrypoints via `scripts/smoke/run.sh`.
- `scripts/test`: test orchestration helpers.
- `scripts/tooling`: repo tooling wrappers; python logic lives in `scripts/tooling/python/`.

Internal-only:
- `scripts/_lib`: shared shell helpers only.
- `scripts/experimental`: quarantined non-supported scripts.
- `scripts/assets`: asset refresh helpers.

The checks control plane is `cargo run -p bijux-dev-dna -- checks ...`.
The compatibility entrypoint remains `./scripts/run.sh checks <check-id>`.
The containers control plane is `cargo run -p bijux-dev-dna -- containers ...`.
The compatibility entrypoints remain `./scripts/run.sh containers <command>` and `./scripts/containers/make.sh <command>`.
The domain control plane is `cargo run -p bijux-dev-dna -- domain ...`.
The compatibility entrypoint remains `./scripts/run.sh domain <command>`.

Use `./scripts/run.sh --list` to print the supported command surface from `scripts/SUPPORTED.toml`.

Requires: bash, rg, coreutils (plus script-specific tools documented inline).
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.
