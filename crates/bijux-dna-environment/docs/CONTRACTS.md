# bijux-dna-environment Contracts

This crate owns the environment contracts consumed by API, runner, and environment QA layers.

## Owned Contracts

- `PlatformSpec`: selected runtime, architecture, image prefix, and container directory.
- `RuntimeKind`: supported runtime enum with stable string parsing and display.
- `RuntimeSpec`: compatibility check between a platform and selected runner.
- `ToolImageSpec`: catalog entry for tool name, version, optional digest, enabled state, and
  shipping policy.
- `ResolvedImage`: deterministic image reference plus runner and architecture.
- `ImageRef`: deterministic tag formatting helper.
- `ReferenceRecord`: digest-addressed reference cache record.
- `ReferenceBuildRequest`: explicit switches for optional reference indexes.
- `ContainerImageInventoryPublication`: immutable image selection and build identity evidence.
- `CurrentApptainerImage`: absolute current retained SIF identity reverified against the selected
  inventory candidate at consumption time.
- `ImageInventoryDerivationRequest`: governed repository, study, image, build-evidence, and run-lock
  roots used to derive a portable publication without delegating selection to contract scripts.
- `ImageInventoryScan`: complete typed identities for scanned SIFs, build-evidence sources, and
  repository or study run references; the publication records the exact serialized scan digest.
- `RuntimeVerificationPublication`: runtime evidence tied to catalog, inventory, repository, and
  image identities.
- `ContainerToolContract`: the strict identity installed exactly once at
  `/usr/share/bijux/tool-contract.json` in each retained tool image.
- `ApptainerRecipeIdentity`: exact committed definition digest plus ordered `%files` Git objects;
  build evidence matches only when status, commit, definition, and complete context agree.
- `ContainerCatalog` and `ContainerCatalogTool`: the complete fail-closed model loaded from
  `containers/contracts/catalog.toml`; consumers must not define reduced or competing projections.
- `ContainerCatalogValidation`: exact 89-tool/85-retained/four-unsupported roster and bidirectional
  catalog-to-Apptainer/Docker recipe closure.
- `ContainerEvidenceChain`: admitted catalog, image-inventory, and runtime publications with exact
  current repository commit, catalog, raw-inventory, selected-image, retained-tool, definition, and
  `%files` build-context closure.
- `ApptainerBuildRequest` and `ApptainerBuildReport`: bounded build execution with immutable,
  monotonically numbered attempt logs.
- `CanonicalApptainerImagePublicationRequest` and `CanonicalApptainerImagePublication`: one clean
  commit-to-study transaction with a staged image, verified embedded contract, digest-addressed
  final SIF, append-only build log, and no-clobber `bijux.container.build_evidence.v1` record.
- `ApptainerSmokeManifest`: offline probe evidence whose version verdict is derived from the exact
  `ContainerToolContract.expected_version_regex` contract.

## Evidence Publications

The image-inventory derivation hashes every scanned SIF and source evidence file, admits only
immutable names backed by exact build evidence, retains non-canonical images only when immutable
run locks reference them, and rejects ambiguous current or historical selections. Its raw digest
commits to the complete scan even when obsolete images are omitted from the public candidate set.
The image-inventory and runtime-verification loaders reject malformed identities, duplicate tools
or candidates, and selected images that are not the unique canonical admitted candidate. The
evidence-chain loader also rejects cross-publication identity, version, disposition, selected-image,
and roster drift. New runtime evidence binds each tool to an immutable execution-site manifest;
the manifest retains bounded probe outcomes and exact stdout/stderr identities.
Unknown descriptive fields remain forward-compatible; every identity and decision field consumed
by qualification is typed. The complete chain does not trust the inventory's `current_recipe`
label: it recomputes the exact committed definition and `%files` identity and requires matching
built/passed evidence for every retained selected image.

Embedded contracts reject unknown fields and identify the exact catalog, canonical recipe, source,
repository commit, architecture, build evidence, expected binary, and version interface. The SIF
digest is deliberately computed outside the SIF because embedding its own digest would be circular;
`VerifiedToolImageIdentity` binds that digest to the validated internal contract and matching labels.
Build retries are permitted only for the environment-owned transient transport taxonomy. Permanent
recipe, source, and command failures stop after the first invocation. Smoke admission does not
assume semantic-version punctuation; valid contract formats such as `r1206` and `2026_06` are
accepted when the exact catalog regex matches.

## Resolution Precedence

1. Explicit digest in `ToolImageSpec.digest`.
2. Versioned image tag from `ToolImageSpec.version`.
3. Registry-file hydration when the local pin file supplies a digest and the catalog entry does not.

Network lookups are not part of this contract.

## Schema Fixtures

Schema fixtures live under `tests/fixtures/env_schema/default/` and are checked by:

- `tests/schemas/schema/schema_snapshots.rs`
- `tests/contracts/matrix/reference_matrix.rs`

Container evidence schemas live under `containers/contracts/schemas/`. Generate and check the
complete inventory scan schema with these developer-control-plane command IDs:

- `generate-image-inventory-scan-schema`
- `check-image-inventory-scan-schema`

## Change Rules

- Adding an optional serialized field is compatible when defaulted and documented.
- Renaming, removing, or changing the meaning of a serialized field is breaking.
- Changing resolution precedence is breaking.
- Adding a managed host command requires `COMMANDS.md` and boundary tests.
- New catalog inputs must have fixture or contract coverage before becoming public behavior.

## Failure Patterns

- Unknown platform names return `EnvError::Platform`.
- Invalid runner names return `EnvError::Parse`.
- Empty tool names, image prefixes, architectures, versions, or digests return `EnvError::Image`.
- Missing Dockerfile version ARGs return `EnvError::Dockerfile`.
- Missing local config files surface as `EnvError::Io`.
