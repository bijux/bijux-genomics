# Artifact Environment

## Purpose
Define the shared artifact environment contract for local development, CI, and operational automation.

## Scope
This document covers artifact, cargo, cache, and temporary output roots used by repository automation.

## Non-goals
- Defining scientific result schemas.
- Replacing per-command output contracts.

## Contract
- `ARTIFACT_ROOT` defaults to `artifacts/`.
- `ISO_ROOT` remains a compatibility alias for `ARTIFACT_ROOT`.
- Cargo build output lives under `artifacts/target/`.
- Cargo home lives under `artifacts/cargo/home/`.
- Temporary files live under `artifacts/tmp/`.
- Deterministic defaults remain `TZ=UTC` and `LC_ALL=C`.

## Usage Rules
- Make targets must prepare the environment through `makes/_macro.mk`.
- Make targets and helper entrypoints must export the shared artifact environment before invoking `bijux-dna-dev`.
- Automation must write only under `artifacts/`.
- Automation must not hardcode retired `artifacts/isolates/` paths.

## Contracts
- Root build outputs must stay out of the repository top level.
- Automation entrypoints must route generated outputs through the governed artifact environment.
