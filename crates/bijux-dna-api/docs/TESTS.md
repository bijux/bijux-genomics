# Tests

This file is the single test documentation entrypoint for `bijux-dna-api`.

Repository-writing contract tests share one materialized sandbox per test
process. The support layer pins the process-wide repository override to that
sandbox and queues overlapping writers through an event-driven mutex while a
filesystem lock protects against cross-process collisions.
README files are intentionally not allowed under `tests/`.

## Suite Entrypoints

- `tests/boundaries.rs` loads architecture, docs-layout, and guardrail tests.
- `tests/schemas.rs` loads schema snapshots, public surface, and documentation
  alignment tests.
- `tests/contracts.rs` loads public behavior and integration contract tests.
- `tests/support/workspace_paths.rs` provides repository path helpers for integration
  tests and boundary aggregators.

## Boundary Tests

- `tests/boundaries/architecture.rs` protects the crate root shape, 10-docs
  allowance, absence of test READMEs, source tree layout, v1 namespace tree,
  and integration-test taxonomy.
- `tests/boundaries/command_inventory.rs` locks `docs/COMMANDS.md` as the
  complete managed API command and helper namespace inventory.
- `tests/boundaries/guardrails/args_module.rs` forbids vague `args.rs` modules.
- `tests/boundaries/guardrails/policies.rs` applies shared policy checks.
- `tests/boundaries/v1_cross_guardrails.rs` keeps cross-domain v1 modules from
  hard-coding stage id literals.

## Schema Tests

- `tests/schemas/v1_cross_api_stability.rs` locks response schema behavior.
- `tests/schemas/v1_cross_contract_handshake.rs` keeps contract fixtures aligned.
- `tests/schemas/v1_cross_docs_schema_snapshots.rs` ensures `docs/API.md`
  references the public schema snapshots.
- `tests/schemas/v1_cross_public_surface.rs` snapshots the public root export.
- `tests/schemas/v1_operator_failure_contract.rs` locks operator-facing failure
  envelopes.

## Contract Tests

- `tests/contracts/invocation_environment.rs` executes the public run entrypoint
  in fresh child processes, first sequentially and then with two real Local
  providers held in overlap. It checks private HOME, temporary, cache and work
  directories, resource values, unchanged parent environment and bounded child
  cleanup. The repository root stays fixed within each child. Its shell and
  FIFO providers and the current contracts executable are checked for byte and
  mode changes; small private fixtures and raw child results remain under
  `artifacts/testing/invocation-environment/` for diagnostics. Schedule this
  contract with capacity for its four workload processes plus supervision and
  transient cleanup; it performs no container or network operation.
- `tests/contracts/v1_cross_contract_spine.rs` checks the plan, manifest, and
  report contract spine.
- `tests/contracts/v1_cross_explain_roundtrip.rs` checks explainability
  round-tripping.
- `tests/contracts/v1_cross_public_contract.rs` checks curated public API usage.
- `tests/contracts/v1_dry_run_manifest.rs` checks dry-run manifest output.
- `tests/contracts/v1_fastq_small_integration.rs` checks a small FASTQ flow.
- `tests/contracts/fastq_amplicon_governance_contract.rs` checks governed FASTQ
  amplicon inputs and locks.

- Inline BAM contracts in `src/internal/handlers/cross/bam_exec_contracts.rs` check
  resumed alignment continuation, summary/manifest projection, and downstream
  binding refusals using accepted synthetic receipts in a bounded private child.
  These checks do not qualify cold aligner execution or scientific BAM outputs.

## Recommended Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --test contracts --no-default-features
```

Run `cargo test -p bijux-dna-api --all-features` before release-facing handoff
when feature-gated code changed.

Shared guardrail parser regressions are owned and executed by the policy crate. This boundary suite retains its crate-specific architecture checks; it does not compile another crate's test source. Constructor-only guardrail wrappers provide no source-check coverage and are retired.

The owned invocation environment contract also checks a failed Local provider: the crash bundle preserves the five recorded child environment values, and the parent environment remains unchanged.
