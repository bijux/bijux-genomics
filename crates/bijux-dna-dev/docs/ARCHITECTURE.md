# bijux-dna-dev Architecture

`bijux-dna-dev` is the workspace development control plane. It is a binary crate with a small root and a deliberately partitioned source tree:

```text
bijux-dna-dev/
├── Cargo.toml
├── README.md
├── docs/
├── src/
└── tests/

src/
├── application/
├── catalog/
├── cli/
├── commands/
│   ├── automation_boundary.rs
│   ├── checks.rs
│   ├── command_support.rs
│   ├── containers/
│   │   ├── runtime/
│   │   ├── validation/
│   │   └── versioning.rs
│   ├── domain/
│   ├── native_dispatch.rs
│   ├── ops/
│   ├── repo_checks/
│   └── repo_checks.rs
├── dev_entrypoint.rs
├── main.rs
├── model/
└── runtime/
    ├── compilation_cache.rs
    ├── compiled_products.rs
    ├── compiled_products/
    ├── crate_gate_inputs.rs
    ├── crate_gate_inputs/
    ├── gate_receipt.rs
    ├── local_tests/
    ├── mod.rs
    ├── owned_artifacts.rs
    ├── process.rs
    ├── process/
    ├── source_snapshot.rs
    ├── source_snapshot/
    │   └── isolation.rs
    ├── workspace.rs
    └── workspace_root.rs
```

Layer responsibilities:

- `main.rs` and `dev_entrypoint.rs` own binary startup only.
- `cli/` owns parsing, routing, and execution reporting for the developer-facing surface.
- `application/` coordinates catalogs, commands, and runtime adapters into stable workflows.
- `catalog/` and `model/` define the durable command vocabulary and typed outcomes.
- `runtime/` owns workspace discovery, artifact path admission, and process boundaries. Environment-selected output roots and declared CI destinations share the same workspace boundary checks.
  `compiled_products/` admits native executable inventories and seals finalized products.
  `crate_gate_inputs.rs` and its contracts capture immutable proof inputs, preserve producer
  provenance, and publish exact source-bound consumer bindings. `local_tests/` owns physical
  inputs, pinned tools, capacity observations, and typed resource requests. `process/` owns
  dependency-aware launches, weighted admission, terminal collection, and process evidence.
  `source_snapshot.rs` captures and materializes admitted tracked source snapshots.
  `source_snapshot/isolation.rs` admits and revalidates Git HEAD, tree, and tracked patch
  identity for isolated native execution. It rejects nonignored untracked inputs and records
  postconditions against the original identity instead of accepting changed worker source.
- `commands/` owns repository-scoped effects and enforcement logic. New side effects belong there behind typed catalog entries, not in `cli/` or `application/`.

Command boundaries:

- `repo_checks.rs` is a curated facade over repository-check namespaces. Artifact guardrails,
  CI build and dependency validation, governance checks, non-waiting enforcement, and workspace
  contracts stay in separate files under `src/commands/repo_checks/`.
- `containers/runtime/mod.rs` owns general container runtime utilities. Frontend proof and report logic stays in `src/commands/containers/runtime/frontend_proofs.rs`.
- `containers/validation/` owns policy-style validation operations that inspect container manifests, locks, generated reports, and registry parity.
- `domain/` owns developer automation around domain indexes, inventory, schema policy, registry locks, and tool governance. It may inspect domain contracts but must not become a production domain semantics crate.
- `ops/` owns non-domain operational workflows grouped by CLI surface: assets, docs, examples, HPC, lab, smoke, test, and tooling.

This separation keeps production runtime crates free of development-only automation concerns while preserving a typed, testable control plane for workspace maintenance.

Test ownership is detailed in [TESTS.md](TESTS.md):

- `tests/boundaries/architecture.rs` locks the root, source, command, runtime,
  and test tree shown here.
- `tests/boundaries/dependencies.rs` protects the dependency graph so
  development automation remains outside production crates.

`application/ci/` separates expected test inventory admission, pinned Nextest event admission, and exact cross-shard coverage reconciliation. These modules have no filesystem or process effects. The tooling CI command owns producer/shard execution, logs, hashes and receipt publication.

`runtime/source_snapshot.rs` captures the tracked worktree once as a pinned commit plus a binary Git patch. Archive and shard workspaces materialize that same input with one checkout and, only for changed sources, one patch application. Untracked evidence is excluded. CI commands own scratch lifetimes; snapshot materialization refuses existing destinations.

`application/ci/partition.rs` owns the qualified executor partition protocol and binary discovery filtersets. It consumes the producer inventory without IO. Coverage reconciliation checks each selected partition as well as the global case union. The existing locked `twox-hash` dependency supplies xxHash64 with only its 64-bit hash feature enabled.

The admitted full-test inventory also owns explicit CLI and developer executable paths for cross-package policy tests. `application/ci/executables.rs` rejects missing, ambiguous or escaping declarations, and the CI command maps those relative archive members into each shard extraction. Policy process support consumes the exact runtime binding; local unbound execution asks Cargo to establish freshness instead of trusting an existing debug file.

Archived test consumers receive their remapped workspace as repository authority and its `artifacts/` directory as product output authority. Executor-owned archive extraction, build targets, coverage and scratch paths remain separate under the shard storage contract. The root receipt records both authorities; workspace admission continues to reject foreign output roots.

The partition adapter follows the [pinned Nextest 0.9.100 implementation](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.100/nextest-runner/src/partition.rs); executor upgrades require fresh protocol qualification.

Local compiled test discovery groups the complete native binary metadata under the admitted worker budget. The pinned executor can launch two listing children for each binary; the planner accounts for that demand before dispatch. Discovery explicitly includes ordinary and ignored tests. `application/ci/discovery.rs` admits exact selected binary identities, unfiltered case counts, empty suites, ignored flags and unchanged build metadata. `runtime/compiled_products/native_inventory.rs` owns bounded processes, immutable metadata subsets and terminal source checks. Partition execution sequences native binary chunks within each private source root and interleaves ready partitions within the same budget.

`application/ci/events.rs` retains case start/result lifetimes across the pinned reporter's ignored-test frame restarts. Per-frame passed and failed counters remain exact, every started case must finally terminate, and final planned case membership governs coverage. A successful child exit or suite summary cannot replace missing case evidence. Its focused lifecycle contracts live in `events/pinned_frames.rs`.

Private CI discovery tests live under `tests/boundaries/ci/` and compile into their owning developer binary. They do not add an integration executable or a second public test facade.
