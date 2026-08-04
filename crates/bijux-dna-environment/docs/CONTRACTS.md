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
- `RuntimeVerificationPublication`: runtime evidence tied to catalog, inventory, repository, and
  image identities.
- `ContainerToolContract`: the strict identity installed exactly once at
  `/usr/share/bijux/tool-contract.json` in each retained tool image.
- `ContainerCatalog` and `ContainerCatalogTool`: the complete fail-closed model loaded from
  `containers/contracts/catalog.toml`; consumers must not define reduced or competing projections.
- `ContainerCatalogValidation`: exact 89-tool/85-retained/four-unsupported roster and bidirectional
  catalog-to-Apptainer/Docker recipe closure.
- `ContainerEvidenceChain`: admitted catalog, image-inventory, and runtime publications with exact
  catalog, repository, raw-inventory, selected-image, and retained-tool closure.
- `ApptainerBuildRequest` and `ApptainerBuildReport`: bounded build execution with immutable,
  monotonically numbered attempt logs.
- `ApptainerSmokeManifest`: offline probe evidence whose version verdict is derived from the exact
  `ContainerToolContract.expected_version_regex` contract.

## Evidence Publications

The image-inventory and runtime-verification loaders reject malformed identities, duplicate tools
or candidates, and selected images that are not admitted candidates. The evidence-chain loader
also rejects cross-publication identity, version, disposition, selected-image, and roster drift.
Unknown descriptive fields remain forward-compatible; every identity and decision field consumed
by qualification is typed.

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
