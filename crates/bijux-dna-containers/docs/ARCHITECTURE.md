# bijux-dna-containers Architecture

`bijux-dna-containers` owns environment facts: tool-image metadata, platform resolution,
runtime compatibility, cache path derivation, and reference registration. It must stay below API,
CLI, runner, planner, stage, and domain crates.

## Source Layout

- `src/lib.rs`: crate root and stable module exports.
- `src/public_api/`: facade re-export surface for consumers that need one stable import path.
- `src/execution_contract/`: strict typed admission for container execution declarations that
  describe credentials and dual-version pipeline inputs without owning runner effects. Pipeline
  wire models, structural validation, execution-site admission, and inventory-bound currentization
  remain separate.
- `src/site_policy/`: site-governance declarations, host admission, and immutable repository
  freezing under policy-owned roots.
- `src/evidence/`: typed image-inventory, selected-run, runtime-verification, and raw security-probe
  publication/admission, cross-publication evidence-chain validation, complete catalog and
  recipe-closure validation, catalog-derived embedded tool identity contracts, and parallel
  immutable smoke evidence.
- `src/build/`: Docker metadata, bounded Apptainer attempts, and canonical no-clobber image and
  build-evidence publication under governed study roots.
- `src/resolve/types/`: serialized models, runtime enum, platform records, and environment errors.
- `src/resolve/catalog/`: TOML catalog loading, registry digest hydration, and image reference
  synthesis.
- `src/resolve/platform.rs`: platform-file loading and runner fallback selection.
- `src/resolve/cache/`: cache roots, Docker image inspection hooks, and Apptainer/Singularity SIF
  path derivation.
- `src/resolve/reference/`: reference-copy registration, digest records, and optional local index
  preparation.
- `src/resolve/commands.rs`, `shell.rs`, `smoke.rs`: bounded host-command helpers documented in
  `COMMANDS.md`.
- `src/runtime_spec/`: pure pairing of a selected runner with a resolved platform.

## Data Flow

1. Build services supply Docker metadata or publish an identity-bound Apptainer image from clean,
   committed inputs through staging, contract inspection, digest naming, and immutable evidence.
2. Platform TOML selects runner, architecture, image prefix, and container directory rules.
3. Image catalog TOML provides tool versions and optional digests.
4. Registry pins can hydrate missing digests from the local registry file.
5. Resolver functions return `ResolvedImage`, `PlatformSpec`, `RuntimeSpec`, or reference records.
6. The evidence-chain loader admits immutable catalog, image, and runtime publications and verifies
   their repository, inventory, selected-image, and retained-tool identities before qualification
   consumers compose higher-level policy decisions.
7. Tool-contract preparation hashes canonical catalog, recipe, source, interface, repository, and
   architecture inputs into a rendered definition; runtime inspection binds that non-circular
   identity to the externally computed immutable SIF digest.
8. QA-owned runtime and security collectors resolve only canonical inventory selections and bind
   immutable raw observations to container-owned publication types; container-owned admission
   verifies exact authorities, governed paths, and every referenced artifact digest.
9. Credential-reference admission deserializes only declared metadata fields, rejects embedded
   values and inconsistent requirement flags, and supplies the generated checked schema.

## Boundaries

- The crate may inspect local host capabilities and declared cache paths.
- The crate may prepare reference index files only when explicitly requested by
  `ReferenceBuildRequest`.
- The crate must not own biological stage execution, planner semantics, user CLI routing, report
  rendering, or runner backends.
- Container report-portfolio aggregation and paper-facing rendering belong to `bijux-dna-bench`;
  persisted portfolio records and freshness admission belong to `bijux-dna-bench-model`.
- Cross-runtime and scientific QA orchestration belongs in `bijux-dna-containers-qa`; canonical
  image identity and version/help admission remain environment evidence responsibilities.

## Change Rules

- Add a source directory only when it has a stable ownership concern.
- Update `docs/PUBLIC_API.md` when `src/lib.rs` exports change.
- Update `docs/COMMANDS.md` before adding or removing any `std::process::Command` use.
- Update boundary tests when the tree changes intentionally.

Qualification registry acquisition, legacy wire normalization, identifier rules, allocation, protocol/citation projection, admission, audit and serialization have separate modules under `evidence/qualification_registry`. Workflow composes those owners. Pure admission and allocation never load files. Audit paths are captured before projection. A run-index source digest and its decoded registry derive from the same acquired byte buffer; catalog and historical inputs remain separate observations rather than an atomic filesystem snapshot.

Image campaign ownership is explicit under `build/campaign`: wire models and schemas, pure scope/identity/progress admission, portable path policy, immutable storage, bounded worker dispatch, current image/receipt verification, and orchestration. Progress decoding delegates semantic validation before outcomes are reused. The five original private selection/attempt cases remain with admission, and public publication/recovery fixtures exercise the service facade. Worker and publication behavior is unchanged; independent source reads do not establish an atomic snapshot.

Result movement separates decoded observations, pure admission, repository effects, flattening and superseded-attempt lifecycle. Saved plans bind each movement to current lock/ledger identities, canonical source and destination, and checked inventory totals before mutation. Evidence digests do not authorize changing a destination or substituting a different attempt.

External artifact admission projects catalog version argv and interprets raw evidence. Shared process transport owns spawning, process-group cancellation, a30-second deadline and a64KiB combined stdout/stderr budget. Bounded collection rejects unsupported non-Unix platforms before launching. Invalid, excessive or timed-out probes cannot publish an admission record.

Image-storage plan creation acquires its decoded inventory and source digest from one UTF-8
buffer. Legacy normalization changes the value projection while the saved digest commits to the
original bytes. The private observation is consumed by storage creation; other image and catalog
files remain separate observations. Apply still rechecks the saved inventory identity before
mutating images. FIFO custody contracts exercise the real path facade under
`tests/contracts/image_storage/inventory_observation.rs`; recovery and report cases remain with
`tests/contracts/image_storage_reconciliation.rs`.
