# bijux-dna-dev Architecture

`bijux-dna-dev` is the workspace development control plane. It is a binary crate with a small, durable tree:

```text
src/
├── main.rs
├── dev_entrypoint.rs
├── cli/
├── application/
├── catalog/
├── model/
├── runtime/
└── commands/
    ├── automation_boundary.rs
    ├── native_dispatch.rs
    ├── checks.rs
    ├── repo_checks.rs
    ├── repo_checks/
    │   ├── artifacts.rs
    │   ├── governance.rs
    │   └── workspace_contracts.rs
    ├── containers/
    │   ├── runtime/
    │   │   ├── mod.rs
    │   │   └── frontend_proofs.rs
    │   └── ...
    ├── domain/
    └── ops/
```

Layer responsibilities:

- `main.rs` and `dev_entrypoint.rs` own binary startup only.
- `cli/` owns parsing, routing, and execution reporting for the developer-facing surface.
- `application/` coordinates catalogs, commands, and runtime adapters into stable workflows.
- `catalog/` and `model/` define the durable command vocabulary and typed outcomes.
- `runtime/` owns workspace discovery and process boundaries.
- `commands/` owns repository-scoped effects and enforcement logic.

Command boundaries:

- `repo_checks.rs` is a curated facade over repository-check namespaces. Artifact guardrails, governance checks, and workspace contracts stay in separate files under `src/commands/repo_checks/`.
- `containers/runtime/mod.rs` owns general container runtime utilities. Frontend proof and report logic stays in `src/commands/containers/runtime/frontend_proofs.rs`.

This separation keeps production runtime crates free of development-only automation concerns while preserving a typed, testable control plane for workspace maintenance.
