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
└── commands/
    ├── automation_boundary.rs
    ├── checks.rs
    ├── command_support.rs
    ├── repo_checks.rs
    ├── containers/
    │   ├── runtime/
    │   ├── validation/
    │   └── versioning.rs
    ├── domain/
    ├── native_dispatch.rs
    ├── ops/
    └── repo_checks/
├── dev_entrypoint.rs
├── main.rs
├── model/
└── runtime/
```

Layer responsibilities:

- `main.rs` and `dev_entrypoint.rs` own binary startup only.
- `cli/` owns parsing, routing, and execution reporting for the developer-facing surface.
- `application/` coordinates catalogs, commands, and runtime adapters into stable workflows.
- `catalog/` and `model/` define the durable command vocabulary and typed outcomes.
- `runtime/` owns workspace discovery and process boundaries.
- `commands/` owns repository-scoped effects and enforcement logic. New side effects belong there behind typed catalog entries, not in `cli/` or `application/`.

Command boundaries:

- `repo_checks.rs` is a curated facade over repository-check namespaces. Artifact guardrails, governance checks, and workspace contracts stay in separate files under `src/commands/repo_checks/`.
- `containers/runtime/mod.rs` owns general container runtime utilities. Frontend proof and report logic stays in `src/commands/containers/runtime/frontend_proofs.rs`.
- `containers/validation/` owns policy-style validation operations that inspect container manifests, locks, generated reports, and registry parity.
- `domain/` owns developer automation around domain indexes, inventory, schema policy, registry locks, and tool governance. It may inspect domain contracts but must not become a production domain semantics crate.
- `ops/` owns non-domain operational workflows grouped by CLI surface: assets, docs, examples, HPC, lab, smoke, test, and tooling.

This separation keeps production runtime crates free of development-only automation concerns while preserving a typed, testable control plane for workspace maintenance.
