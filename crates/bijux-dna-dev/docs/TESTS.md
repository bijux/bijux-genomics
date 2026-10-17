# bijux-dna-dev Tests

## Coverage
- `tests/bootstrap.rs` checks developer executable freshness after direct and transitive source changes.
- `tests/contracts.rs` registers private workflow observation contracts under `tests/contracts/`: exact example golden-bundle transport and independently observed frontend cache comparisons. Scripted cache providers qualify control flow and report claims, not real SIF image or scientific reproducibility.
- The private cache observation owner also checks that purging its own directories preserves caller/foreign roots and refuses substituted symlinks before either removal.
- `src/runtime/workspace.rs` tests artifact root admission; CI destination contracts reuse that owner and cover nested symlink escapes.
- `tests/boundaries.rs` is the integration-test entrypoint for boundary-oriented coverage.
- `tests/boundaries/architecture.rs` enforces the documented crate tree and layout contract.
- `tests/boundaries/architecture.rs` also rejects Markdown files outside the root `README.md` and `docs/`.
- `tests/boundaries/command_inventory.rs` keeps `docs/COMMANDS.md` synchronized with `src/catalog`.
- `tests/boundaries/dependencies.rs` locks the direct dependency shape documented in [DEPENDENCIES.md](DEPENDENCIES.md).
- `tests/boundaries/guardrails.rs` checks the crate guardrail profile and boundary ownership.
- `tests/support/workspace_paths.rs` owns repository and crate path resolution for integration tests.
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

CI coverage contracts replay captured Nextest 0.9.100 library and integration event streams from `tests/fixtures/ci/nextest/`. The fixture capture manifest binds their source/binary hashes and format version. Adversarial cases reject omissions, duplicate identities, incomplete event sequences, incorrect suite counters, unexpected cases, failed outcomes and filtered inventories.

Private CI discovery tests live under `tests/boundaries/ci/` and compile into their owning developer binary. They do not add an integration executable or a second public test facade.

Fast-gate contracts live with their contract, compiler-cache and timing owners under `src/commands/ops/tooling/ci/fast/`. Full-shard contract, storage, archive and preflight tests live with those owners under `ci/sharded/`. They remain private tests in the existing developer binary; execution-budget, native-chunk and complete-lane contracts keep their existing owners.
