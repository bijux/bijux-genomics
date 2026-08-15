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
- `evidence`: typed container image-inventory, selected-run, runtime-verification, and raw
  security-probe publications with generated schemas, exact current-authority admission, and
  stable-authority baseline admission with complete artifact verification;
  owner-separated immutable attempt layouts with shared manifests and required evidence directories;
  strict embedded tool contracts; the complete canonical `ContainerCatalog` and
  `ContainerCatalogTool` model; permanent qualification run definitions and namespace,
  lifecycle, supersession, acceptance, and citation records;
  `validate_container_catalog` roster and recipe closure; model-owned `ContainerCatalogLock`
  derivation and admission; catalog-derived Apptainer build
  preparation; `load_container_evidence_chain` cross-publication admission; offline contract smoke
  execution with bounded immutable streams; deterministic runtime-verification report rendering;
  exact-commit catalog and Apptainer recipe identity;
  delivery-aware Apptainer selection with on-disk size and digest verification through either the
  current distributable inventory or an exact external-artifact admission;
  no-clobber canonical image activation and activation-derived current inventory requests;
  reversible historical, obsolete, and interrupted image
  quarantine; no-copy external binary/SIF admission, access blockers, checksum/version replay, and
  stage command binding for licensing-restricted tools; focused catalog/image-inventory closure
  validation for execution planning; runtime inspection; deterministic image-inventory report
  rendering; validation, loading, JSON Schema rendering, and atomic schema writing.
- `execution_contract`: strict credential-reference loading, validation, and generated schema
  ownership; dual-version pipeline declaration and execution-site admission; and resumable
  inventory-bound repository/image currentization without execution or scheduler submission.
- `resolve`: environment errors, image and platform models, catalog loading, image resolution,
  cache helpers, local command probes, shell capture, smoke helpers, and reference registration.
- `runtime_spec`: pure compatibility checks between a platform and selected runtime.
- `site_policy`: typed site governance, hostname admission, commit-addressed repository freezing
  under governed roots, checksum-bound Git-bundle archival, self-contained bundle proof without a
  duplicate working tree, explicit retirement and rollback, and effect-free admission of existing
  clean, self-contained, read-only snapshots.
- `public_api::api`: facade re-export for consumers that prefer one stable import path.

Catalog-wide runtime and security probe orchestration is intentionally owned by
`bijux-dna-containers-qa` and is not exported by this production crate.
Container report-portfolio models, validation, aggregation, and paper-facing rendering are owned
by `bijux-dna-bench-model` and `bijux-dna-bench`, not this environment-evidence crate.

## Stability Rules

- `src/lib.rs` is the public module source of truth.
- New public modules require this file and `tests/public_api_docs.rs` to change together.
- New process-running APIs require `COMMANDS.md` and command boundary tests.
- Public model field changes must update `CONTRACTS.md` and schema tests.

## Stability Tiers

- Stable: the Public Modules and Major Export Groups documented in this file.
- Experimental: new runtime/build adapters are experimental until they are listed here and covered by the corresponding docs/tests.
- Internal: module-local helpers and any item not exported through the documented public modules or facade.
