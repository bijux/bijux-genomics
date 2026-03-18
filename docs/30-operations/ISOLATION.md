# Artifact Environment

## Purpose
Define the shared artifact environment contract for local development, CI, and operational scripts.

## Contract
- `ARTIFACT_ROOT` defaults to `artifacts/`.
- `ISO_ROOT` remains a compatibility alias for `ARTIFACT_ROOT`.
- Cargo build output lives under `artifacts/target/`.
- Cargo home lives under `artifacts/cargo/home/`.
- Temporary files live under `artifacts/tmp/`.
- Deterministic defaults remain `TZ=UTC` and `LC_ALL=C`.

## Usage Rules
- Make targets must prepare the environment through `makes/_macro.mk`.
- Shell scripts must use `require_artifact_env` or `run_with_artifact_env` from `scripts/_lib/common.sh`.
- Scripts and tooling must write only under `artifacts/`.
- Scripts must not hardcode retired `artifacts/isolates/` paths.
