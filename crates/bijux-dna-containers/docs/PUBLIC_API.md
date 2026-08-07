# bijux-dna-containers Public API

## Public Modules

- `build`
- `evidence`
- `execution_contract`
- `public_api`
- `resolve`
- `runtime_spec`
- `site_policy`

## Major Export Groups

- `build`: Docker metadata helpers plus Apptainer build requests, reports, append-only attempt paths,
  failure classification, bounded build execution, and canonical study image/evidence publication.
- `evidence`: typed container image-inventory, selected-run, and runtime-verification publications;
  strict embedded
  tool contracts; the complete canonical `ContainerCatalog` and `ContainerCatalogTool` model;
  `validate_container_catalog` roster and recipe closure; model-owned `ContainerCatalogLock`
  derivation and admission; catalog-derived Apptainer build
  preparation; `load_container_evidence_chain` cross-publication admission; offline contract smoke
  execution with bounded immutable streams; catalog-wide parallel runtime verification and
  portable report publication; exact-commit catalog and Apptainer recipe identity;
  `resolve_current_apptainer_image` selection with on-disk size and digest verification;
  no-clobber canonical image activation; reversible historical, obsolete, and interrupted image
  quarantine; no-copy external binary/SIF admission, access blockers, checksum/version replay, and
  stage command binding for licensing-restricted tools; focused catalog/image-inventory closure
  validation for execution planning; runtime inspection; deterministic image-inventory report
  rendering; validation, loading, JSON Schema rendering, and atomic schema writing.
- `execution_contract`: strict credential-reference loading, validation, and generated schema
  ownership for container executions without admitting secret values.
- `resolve`: environment errors, image and platform models, catalog loading, image resolution,
  cache helpers, local command probes, shell capture, smoke helpers, and reference registration.
- `runtime_spec`: pure compatibility checks between a platform and selected runtime.
- `site_policy`: typed site governance, hostname admission, and commit-addressed repository
  freezing under governed roots.
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
