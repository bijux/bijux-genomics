# bijux-dna Architecture

`bijux-dna` is the CLI adapter crate. It should stay thin: parse arguments, prepare the process
context, route commands to the correct adapter, render output, and delegate durable genomics logic
to API/domain/runtime crates.

## Crate Root

```text
crates/bijux-dna/
├── Cargo.toml
├── README.md
├── docs/
├── src/
└── tests/
```

Only `README.md` belongs at the crate root. All other crate documentation belongs in `docs/`.

## Source Tree

```text
src/
├── bin/
│   └── bijux-dna.rs
├── cli_entrypoint.rs
├── commands/
├── lib.rs
├── process_exit.rs
└── public_api/
```

- `bin/bijux-dna.rs` is a thin process wrapper.
- `cli_entrypoint.rs` captures argv/cwd and calls command routing without owning command behavior.
- `lib.rs` exports only the curated public surface.
- `process_exit.rs` maps categorized operator failures to stable exit behavior.
- `public_api/` exposes only testable CLI entrypoints and explicit helper namespaces.

## Command Tree

`commands/` is partitioned by durable responsibility:

- `commands/router/`: argv normalization, root command dispatch, cwd/environment setup, and
  observability debug/collect routing.
- `commands/cli/`: parser types, rendering helpers, plan command glue, validation helpers, and
  environment/registry command support.
- `commands/planning/`: run-plan and dry-run planning entrypoints.
- `commands/status/`: runtime and repository status inspection.
- `commands/corpus/`: curated corpus materialization, normalization, validation, listing, and diff.
- `commands/benchmark/`: benchmark config, workspace, publication, corpus, suite, and FASTQ/BAM
  benchmark flows.
- `commands/fastq/`: FASTQ meta-command dispatch and API mediation.
- `commands/bam/`, `commands/vcf/`: domain-facing CLI adapters only.
- `commands/ena/`, `commands/hpc/`, `commands/example.rs`: focused operator helpers that do not
  belong in router or support.
- `commands/support/`: shared command helpers with no command ownership.

## Test Tree

```text
tests/
├── boundaries.rs
├── boundaries/
├── contracts.rs
├── contracts/
├── guardrails.rs
├── snapshots/
└── workspace_paths.rs
```

- `boundaries.rs` aggregates layout, dependency, public-surface, and process-spawn checks.
- `contracts.rs` aggregates command behavior, dry-run, bank, and HPC layout contracts.
- `snapshots/` stores help and serialized public-surface locks.
- `workspace_paths.rs` centralizes repo/crate root resolution for integration tests.

Empty placeholder test buckets are not part of the ideal tree. Add a bucket only when it contains
executable tests or governed snapshots.

## Dependency Direction

The CLI may depend on `bijux-dna-api` and minimal support crates needed for parsing, rendering, and
declared filesystem effects. Domain semantics, stage execution, runner behavior, and planner policy
must live behind API or domain-owned boundaries.

## Enforcement

- `tests/boundaries/architecture_tree.rs` enforces the crate tree.
- `tests/boundaries/guardrails/deps.rs` enforces CLI dependency exclusions.
- Workspace policy tests enforce docs placement, public surface, and dependency graph contracts.
