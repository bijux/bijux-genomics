# Generated Files Contract

Owner: Architecture
Scope: Generated and managed repository artifacts
Last reviewed: 2026-04-26
Contract version: v1

## Purpose
Separate authored source from generated or managed output.

## Scope
- Generated repository configs, managed standards output, and governed inventories.
- The commands and policies that validate those generated surfaces.

## Non-goals
- Replacing the source documents that feed generators.
- Treating generated output as an authored authority.

## Contracts
- Authored source remains upstream of generated output.
- Generated files may be committed, but they must stay reproducible from their governed inputs.

## Allowed inputs
- Domain source files under [../../domain/](../../domain/).
- Shared-standard managed inputs under [../../.bijux/shared/](../../.bijux/shared/) and
  [../../.github/standards/](../../.github/standards/).
- Generator source code and checked-in config templates.

## Forbidden dependencies
- Generated files must not become a new hand-authored source of truth.
- Downstream generated views must not bypass the domain compiler or shared-standard sync source.

## Forbidden effects
- Policy tests must not rewrite generated files.
- Local command output must go under `artifacts/` unless the command intentionally updates a governed output.

## Validation command
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts generated_configs_policy --no-default-features`
- Generated config inventory and ownership guidance live in
  [../50-reference/CONFIGS_GUIDE.md](../50-reference/CONFIGS_GUIDE.md).
- The governed policy anchor lives in
  [../../crates/bijux-dna-policies/tests/contracts/tooling/governance_quality/generated_configs_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/governance_quality/generated_configs_policy.rs).

## Failure modes
- Missing generated headers make manual edits indistinguishable from governed output.
- Generated config drift hides domain-source contract changes.
