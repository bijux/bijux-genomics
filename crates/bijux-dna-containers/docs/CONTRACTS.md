# bijux-dna-containers Contracts

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
- `ContainerCredentialReference`: strict no-secret credential metadata with an explicit
  no-publication rule, consistent requirement flag, unique reference IDs, and generated schema.
- `ContainerPipelineContract`: strict version-dispatched admission for immutable v1 history and
  current v2 declarations. Unknown and cross-version fields fail before execution-site effects.
- `ContainerPipelineSiteAdmission`: canonical repository, image, input, result-root, and Slurm
  resource identities reverified against one governed host before submission.
- `ContainerPipelineCurrentizationPlan`: inventory-bound repository and image identity updates for
  every tracked pipeline declaration. Source and target hashes make interrupted atomic-per-file
  application resumable and reject unplanned content drift.
- `ContainerImageInventoryPublication`: immutable image selection and build identity evidence.
- `CurrentApptainerImage`: absolute current retained SIF identity reverified against the selected
  inventory candidate at consumption time.
- `ImageInventoryDerivationRequest`: governed repository, study, image, build-evidence, and run-lock
  roots used to derive a portable publication without delegating selection to contract scripts.
- `ImageInventoryScan`: complete typed identities for scanned SIFs, build-evidence sources, and
  repository or study run references; the publication records the exact serialized scan digest.
- `RuntimeVerificationPublication`: runtime evidence tied to catalog, inventory, repository, and
  image identities.
- `RunEvidenceLock`, `RunAttemptLedger`, and `RunEvidenceIndex`: immutable selected execution,
  supersession lineage, and portable final-run publication with independently counted execution
  and scientific states.
- `ContainerToolContract`: the strict identity installed exactly once at
  `/usr/share/bijux/tool-contract.json` in each retained tool image.
- `ToolImageIdentityEvidence`: persisted image digest, complete embedded tool contract, and exact
  inspected labels required by current runtime verification.
- `ApptainerRecipeIdentity`: exact committed definition digest plus ordered `%files` Git objects;
  build evidence matches only when status, commit, definition, and complete context agree.
- `ContainerCatalog` and `ContainerCatalogTool`: the complete fail-closed model loaded from
  `containers/contracts/catalog.toml`; consumers must not define reduced or competing projections.
  Catalog capability distinguishes prohibited redistribution from governed operator-provided
  execution through `external_artifact_supported` and its allowed artifact kinds.
- `ContainerCatalogLock`: exact catalog-byte identity and per-tool contract digests derived by this
  crate; unknown, stale, future-version, or unresolved-source locks fail admission.
- `ContainerCatalogValidation`: exact 89-tool/85-retained/four-unsupported roster and bidirectional
  catalog-to-Apptainer/Docker recipe closure.
- `ContainerEvidenceChain`: admitted catalog, image-inventory, and runtime publications with exact
  current repository commit, catalog, raw-inventory, selected-image, retained-tool, definition, and
  `%files` build-context closure.
- `ApptainerBuildRequest` and `ApptainerBuildReport`: bounded build execution with immutable,
  monotonically numbered attempt logs.
- `CanonicalApptainerImagePublicationRequest` and `CanonicalApptainerImagePublication`: one clean
  commit-to-study transaction with a staged image, verified embedded contract, digest-addressed
  final SIF, pinned base identity, builder environment, timed append-only build log, persisted
  inspection identity, and no-clobber `bijux.container.build_evidence.v1` record.
- `CanonicalApptainerImageDestination`: explicit active-root or commit-addressed campaign ownership;
  campaign publication cannot silently write the active image surface.
- `ImageCampaignPlan` and `ImageCampaignExecution`: exact retained roster, commit, catalog, roots,
  worker policy, per-tool publications or failures, resumable progress, append-only invocation
  history, and an all-passed canonical terminal record.
- `ImageActivationPlan`: a no-clobber, resumable, reversible publication of every selected
  canonical campaign image into the active image root by verified same-filesystem hard link.
- `ImageStoragePlan` and `ImageStorageReport`: a resumable, reversible migration of historical,
  unreferenced obsolete, and interrupted images out of the active top level while preserving
  exact file identity and historical run-reference authority through the source inventory.
- `ApptainerSmokeManifest`: offline probe evidence whose version verdict is derived from the exact
  `ContainerToolContract.expected_version_regex` contract.
- `SecurityProbePublication`: exact raw SBOM, vulnerability, secret, signature, structural, and
  runtime observations bound to repository, catalog, inventory, scanner, database, image, and tool
  contract identities. Admission rejects authority drift, unsafe governed URIs, incomplete states,
  and every missing or digest-mismatched referenced artifact without deriving a policy verdict.
- `ExternalArtifactAdmission`: immutable no-copy admission for an operator-provided binary or SIF,
  including lawful-access basis, checksum, size, live cataloged version probe, permitted stage IDs,
  and an explicit non-redistribution decision. When access is unavailable, the same contract owns a
  `blocked_external_access` record rather than silently disabling integration.
- `ExternalArtifactCommandBinding`: a reverified stage invocation that replaces the cataloged
  executable with the admitted binary or wraps it in `apptainer exec`; artifact and catalog drift
  fail before execution.

## Evidence Publications

The image-inventory derivation hashes every scanned SIF and source evidence file, admits only
immutable names backed by exact build evidence, retains non-canonical images only when immutable
run locks reference them, and rejects ambiguous current or historical selections. Its raw digest
commits to the complete scan even when obsolete images are omitted from the public candidate set.
Run-reference closure is scoped to image digests present under the explicit image root; locks for
images outside that inventory surface remain owned by their own image inventory and are not copied
into an unrelated commit-addressed publication.
The image-inventory and runtime-verification loaders reject malformed identities, duplicate tools
or candidates, and selected images that are not the unique canonical admitted candidate. The
evidence-chain loader also rejects cross-publication identity, version, disposition, selected-image,
and roster drift. Runtime publication v2 binds each tool to immutable image-identity evidence and an
execution-site smoke manifest; the manifest retains bounded probe outcomes and exact stdout/stderr
identities. Legacy v1 publications remain readable but cannot qualify the current evidence chain.
Unknown descriptive fields remain forward-compatible; every identity and decision field consumed
by qualification is typed. The complete chain does not trust the inventory's `current_recipe`
label: it recomputes the exact committed definition and `%files` identity and requires matching
built/passed evidence for every retained selected image.

Embedded contracts reject unknown fields and identify the exact catalog, canonical recipe, source,
repository commit, architecture, build evidence, expected binary, and version interface. The SIF
digest is deliberately computed outside the SIF because embedding its own digest would be circular;
`ToolImageIdentityEvidence` binds that digest to the validated internal contract and matching labels.
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

The credential-reference schema remains at `containers/contracts/config-credential.schema.json`
for compatibility with archived run contracts and is derived from `ContainerCredentialReference`.

## Change Rules

- Adding an optional serialized field is compatible when defaulted and documented.
- Renaming, removing, or changing the meaning of a serialized field is breaking.
- Changing resolution precedence is breaking.
- Adding a managed host command requires `COMMANDS.md` and boundary tests.
- New catalog inputs must have fixture or contract coverage before becoming public behavior.
- External artifacts must remain outside the repository and distributable active image root; only
  identity and probe evidence may be published.

## Failure Patterns

- Unknown platform names return `EnvError::Platform`.
- Invalid runner names return `EnvError::Parse`.
- Empty tool names, image prefixes, architectures, versions, or digests return `EnvError::Image`.
- Missing Dockerfile version ARGs return `EnvError::Dockerfile`.
- Missing local config files surface as `EnvError::Io`.
