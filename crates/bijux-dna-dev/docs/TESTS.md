# bijux-dna-dev Tests

## Coverage
- `tests/guardrails.rs` checks the crate guardrail profile and boundary ownership.
- `src/commands/repo_checks.rs` carries repository-check contracts and policy-facing assertions.
- `src/commands/automation_boundary.rs` validates the allowed automation invocation surface.
- `src/commands/containers.rs` owns container-control contract coverage and generated-output checks.

## How to run
- `cargo test -p bijux-dna-dev`
- `cargo test -p bijux-dna-policies policy__boundaries__docs_spine__crate_docs_contract -- --nocapture`

## Notes
- Prefer targeted command-module tests when changing one automation surface.
- Re-run policy tests after updating docs, command names, or generated-control outputs.
