# bijux-dna-containers Effects

The crate is deterministic where it resolves declared data, and effectful where callers explicitly
ask for host-environment inspection or reference preparation.

## Allowed Reads

- `configs/runtime/platforms.toml`
- `configs/ci/tools/images.toml`
- `configs/ci/registry/tool_registry.toml`
- `containers/contracts/catalog.toml`
- Catalog-declared canonical Apptainer definitions
- Contract-bearing SIF files passed to `verify_apptainer_tool_contract`
- Dockerfiles passed to `extract_version_from_dockerfile`
- Reference FASTA files passed to `ReferenceRegistry::prepare_reference`
- Environment variables: `BIJUX_CACHE_ROOT`, `XDG_CACHE_HOME`, `HOME`, `BIJUX_HPC_ROOT`, and
  `BIJUX_APPTAINER_CONTAINER_DIR`

## Allowed Writes

- Reference cache directories under the resolved reference cache root.
- Copied FASTA files and requested reference index outputs.
- Test fixtures and temporary files under artifact-aware test roots.
- Generated embedded contracts and rendered definitions under a caller-provided repository
  `artifacts/` root.
- Apptainer SIF targets and append-only build attempt logs under caller-provided container roots.
- Hard-linked canonical SIF publications in an explicit active root.
- Same-filesystem image quarantine moves and their persisted apply or rollback plans.

## Allowed Processes

The complete process list is owned by `COMMANDS.md`. No process should be added without updating
that inventory and its boundary test.

## Forbidden Effects

- Pulling from or probing remote registries during resolution.
- Running biological pipeline stages.
- Mutating repository source, docs, configs, or fixtures from library code.
- Owning user-facing CLI routing.
- Deleting cache contents outside an explicit caller-owned maintenance command.
- Deleting SIF evidence during activation or storage reconciliation.

## Determinism Guarantees

- The same platform file, image catalog, and registry pin file produce the same resolved image.
- The same Dockerfile and tool selector produce the same parsed version.
- The same FASTA content produces the same reference digest.
- Host probes are intentionally not deterministic and must not be used as schema truth.
- Build retry classification and smoke admission are deterministic for the same diagnostics,
  contract regex, and probe outputs.
