# FASTQ Upstream Archive

`science/docs/upstream/fastq/` is the tracked contract surface for FASTQ-specific
upstream evidence packets.

[../README.md](../README.md) defines the broader upstream archive boundary, and
[container/README.md](container/README.md) explains the generated FASTQ
container evidence reports that roll these local packets into closure review.

## Scope

- tool source repositories used to validate FASTQ tool provenance
- project homepages and release pages used to confirm download surfaces
- papers or supplemental documentation tied to FASTQ tool claims

## Layout

- [tools/README.md](tools/README.md)
  operator-facing rules for per-tool evidence packets
- [tools/EVIDENCE_MAP.tsv](tools/EVIDENCE_MAP.tsv)
  tracked locator map for curated FASTQ tool evidence packets
- [STAGE_CLAIMS.tsv](STAGE_CLAIMS.tsv)
  machine-readable stage claim registry for empirical, policy, database,
  comparability, and order-justification claims
- [STAGE_LIBRARY_SUPPORT.tsv](STAGE_LIBRARY_SUPPORT.tsv)
  machine-readable library-type support, exclusion, and unsafe-use matrix for
  governed FASTQ stages
- [TOOL_RISK_REGISTRY.tsv](TOOL_RISK_REGISTRY.tsv)
  machine-readable known failure modes, assumption violations, and version,
  license, command-surface, and output-format authority locators for governed
  FASTQ tools
- [CONTAINER_DIGEST_BLOCKERS.tsv](CONTAINER_DIGEST_BLOCKERS.tsv)
  tracked blocker registry for FASTQ tools whose container references still need
  immutable release digests before world-class closure; header-only means no
  current FASTQ tool manifest contains a pending SHA placeholder
- [TAG_ONLY_CONTAINER_BLOCKERS.tsv](TAG_ONLY_CONTAINER_BLOCKERS.tsv)
  tracked blocker registry for admitted FASTQ production-registry entries whose
  `container_ref` still uses a tag rather than an immutable `@sha256:` digest
- [PLANNED_RUNTIME_BLOCKERS.tsv](PLANNED_RUNTIME_BLOCKERS.tsv)
  tracked blocker registry for planned FASTQ stage-tool bindings that still
  lack production registry admission, a container reference, or a runtime smoke
  surface
- [QA_COVERAGE_BLOCKERS.tsv](QA_COVERAGE_BLOCKERS.tsv)
  tracked blocker registry for admitted FASTQ execution-support stages that do
  not yet have behavioral environment-QA coverage
- [container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv](container/FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv)
  generated production-closure ledger for every FASTQ execution default; this is
  the single conservative rollup for release closure and must stay blocked while
  any evidence, asset, container, license, planner, SBOM, smoke, QA, or registry
  prerequisite is missing
- `tools/<tool-id>/**`
  untracked local payloads placed at the archive paths recorded in the science
  backlog and evidence map
- [../papers/README.md](../papers/README.md)
  contract surface for the paper or software-citation roots linked from the FASTQ evidence map

## Rules

- keep downloaded payloads and cloned repositories untracked
- keep locator maps and archive instructions tracked
- prefer one stable archive path per tool instead of mixing papers, repos, and
  notes into ad hoc folders
