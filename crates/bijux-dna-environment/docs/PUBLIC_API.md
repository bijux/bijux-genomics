# bijux-dna-environment Public API

## Public Modules

- `build`
- `evidence`
- `public_api`
- `resolve`
- `runtime_spec`

## Major Export Groups

- `build`: Docker metadata helpers plus Apptainer build requests, reports, append-only attempt paths,
  failure classification, and bounded build execution.
- `evidence`: typed container image-inventory and runtime-verification publications; strict embedded
  tool contracts; catalog-derived Apptainer build preparation; offline contract smoke execution;
  exact-commit catalog identity; runtime inspection; validation, loading, JSON Schema rendering,
  and atomic schema writing.
- `resolve`: environment errors, image and platform models, catalog loading, image resolution,
  cache helpers, local command probes, shell capture, smoke helpers, and reference registration.
- `runtime_spec`: pure compatibility checks between a platform and selected runtime.
- `public_api::api`: facade re-export for consumers that prefer one stable import path.

## Stability Rules

- `src/lib.rs` is the public module source of truth.
- New public modules require this file and `tests/public_api_docs.rs` to change together.
- New process-running APIs require `COMMANDS.md` and command boundary tests.
- Public model field changes must update `CONTRACTS.md` and schema tests.

## Stability Tiers

- Stable: the Public Modules and Major Export Groups documented in this file.
- Experimental: new runtime/build adapters are experimental until they are listed here and covered by the corresponding docs/tests.
- Internal: module-local helpers and any item not exported through the documented public modules or facade.
