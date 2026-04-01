# bijux-dna-dev Tests

## Coverage
- `tests/boundaries.rs` is the integration-test entrypoint for boundary-oriented coverage.
- `tests/boundaries/architecture.rs` enforces the documented crate tree and layout contract.
- `tests/boundaries/guardrails.rs` checks the crate guardrail profile and boundary ownership.
- `src/commands/repo_checks.rs` is the curated repository-check facade; detailed contracts live under `src/commands/repo_checks/`.
- `src/commands/containers/runtime/frontend_proofs.rs` owns Apptainer frontend proof and report checks.
- `src/commands/automation_boundary.rs` validates the allowed automation invocation surface.

## How to run
- `cargo test -p bijux-dna-dev`
- `cargo test -p bijux-dna-dev --test boundaries`
- `cargo test -p bijux-dna-policies policy__boundaries__docs_spine__crate_docs_contract -- --nocapture`

## Notes
- Prefer targeted boundary tests when changing tree layout or ownership.
- Prefer targeted command-module tests when changing one automation surface.
- Re-run policy tests after updating docs, command names, or generated-control outputs.
