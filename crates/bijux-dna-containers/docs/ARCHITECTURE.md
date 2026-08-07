# bijux-dna-containers Architecture

`bijux-dna-containers` owns environment facts: tool-image metadata, platform resolution,
runtime compatibility, cache path derivation, and reference registration. It must stay below API,
CLI, runner, planner, stage, and domain crates.

## Source Layout

- `src/lib.rs`: crate root and stable module exports.
- `src/public_api/`: facade re-export surface for consumers that need one stable import path.
- `src/execution_contract/`: strict typed admission for container execution declarations that
  describe credentials and dual-version pipeline inputs without owning runner effects. Pipeline
  wire models and semantic validation remain separate.
- `src/site_policy/`: site-governance declarations, host admission, and immutable repository
  freezing under policy-owned roots.
- `src/evidence/`: typed image-inventory, selected-run, and runtime-verification
  execution/publication, cross-publication evidence-chain validation, complete catalog and
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
8. Runtime verification resolves only canonical inventory selections, executes bounded offline
   probes in parallel, and binds immutable logs and manifests into the portable publication.
9. Credential-reference admission deserializes only declared metadata fields, rejects embedded
   values and inconsistent requirement flags, and supplies the generated checked schema.

## Boundaries

- The crate may inspect local host capabilities and declared cache paths.
- The crate may prepare reference index files only when explicitly requested by
  `ReferenceBuildRequest`.
- The crate must not own biological stage execution, planner semantics, user CLI routing, report
  rendering, or runner backends.
- Cross-runtime and scientific QA orchestration belongs in `bijux-dna-containers-qa`; canonical
  image identity and version/help admission remain environment evidence responsibilities.

## Change Rules

- Add a source directory only when it has a stable ownership concern.
- Update `docs/PUBLIC_API.md` when `src/lib.rs` exports change.
- Update `docs/COMMANDS.md` before adding or removing any `std::process::Command` use.
- Update boundary tests when the tree changes intentionally.
