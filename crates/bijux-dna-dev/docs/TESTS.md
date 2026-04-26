# bijux-dna-dev Tests

## Coverage
- `tests/boundaries.rs` is the integration-test entrypoint for boundary-oriented coverage.
- `tests/boundaries/architecture.rs` enforces the documented crate tree and layout contract.
- `tests/boundaries/guardrails.rs` checks the crate guardrail profile and boundary ownership.
- `src/commands/repo_checks.rs` is the curated repository-check facade; detailed contracts live under `src/commands/repo_checks/`.
- `src/commands/containers/runtime/frontend_proofs.rs` owns Apptainer frontend proof and report checks.
- `src/commands/automation_boundary.rs` validates the allowed automation invocation surface.

## How to run
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-dev --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-dev --no-default-features --test boundaries`
- `cargo test -p bijux-dna-policies policy__boundaries__docs_spine__crate_docs_contract -- --nocapture`

## Lint Gates
- `cargo fmt -p bijux-dna-dev -- --check`
- `cargo clippy -p bijux-dna-dev --all-targets -- -D warnings`
- Keep the crate-level lint profile strict for unsafe code, debug output, direct printing, and panic-prone unwrap/expect usage.
- Use narrow lint allowances only for style-heavy Clippy groups where the current command implementation intentionally favors explicit control flow.

## Notes
- Prefer targeted boundary tests when changing tree layout or ownership.
- Prefer targeted command-module tests when changing one automation surface.
- Re-run policy tests after updating docs, command names, or generated-control outputs.
- Do not add README files under `tests/`; crate narrative docs belong under `docs/`.
