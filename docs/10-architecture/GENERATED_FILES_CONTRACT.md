# Generated Files Contract

Owner: Architecture
Scope: Generated and managed repository artifacts
Last reviewed: 2026-04-26
Contract version: v1

## Purpose
Separate authored source from generated or managed output.

## Allowed inputs
- Domain source files under `domain/`.
- Shared-standard managed inputs under `.bijux/shared/` and `.github/standards/`.
- Generator source code and checked-in config templates.

## Forbidden dependencies
- Generated files must not become a new hand-authored source of truth.
- Downstream generated views must not bypass the domain compiler or shared-standard sync source.

## Forbidden effects
- Policy tests must not rewrite generated files.
- Local command output must go under `artifacts/` unless the command intentionally updates a governed output.

## Validation command
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts generated_configs_policy --no-default-features`

## Failure modes
- Missing generated headers make manual edits indistinguishable from governed output.
- Generated config drift hides domain-source contract changes.
