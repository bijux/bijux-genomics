# TOOL_ADMISSION

Canonical step-by-step process for admitting a tool into production workflows.

## Purpose
Define one authoritative workflow that ties domain -> registry -> containers -> docs -> examples.

## Scope
Applies to all tools entering planned/experimental/production states.

## Non-goals
- Replacing low-level container recipe style docs.
- Replacing domain scientific validation details.

## Contracts
- No tool is production until all steps below are complete and validated.
- Registry/config/container/docs/example surfaces must be mutually consistent.

## Required Path
1. Domain contract: add/update tool manifests under
   [domain/fastq/tools/](../../domain/fastq/tools),
   [domain/bam/tools/](../../domain/bam/tools), or
   [domain/vcf/tools/](../../domain/vcf/tools), plus stage bindings in
   [domain/fastq/index.yaml](../../domain/fastq/index.yaml),
   [domain/bam/index.yaml](../../domain/bam/index.yaml), or
   [domain/vcf/index.yaml](../../domain/vcf/index.yaml).
2. Registry/config contract: regenerate and validate
   [configs/ci/registry/tool_registry.toml](../../configs/ci/registry/tool_registry.toml),
   [configs/ci/tools/images.toml](../../configs/ci/tools/images.toml), and related lock files.
3. Container contract: add/update governed container surfaces under
   [containers/index.md](../../containers/index.md).
4. Docs contract: regenerate/update [docs/20-science/TOOL_INDEX.md](../20-science/TOOL_INDEX.md),
   linked operational notes under [docs/30-operations/index.md](../30-operations/index.md),
   and admission references.
5. Example contract: ensure at least one runnable example path is documented or validated through
   [examples/index.yaml](../../examples/index.yaml) where required by policy.
6. Gate contract: run lint and policy checks through [docs/30-operations/CI.md](../30-operations/CI.md)
   so parity across all above surfaces is enforced.

## Admission Gate
A tool is considered admitted only when registry, containers, QA, and docs are all consistent and CI passes.

## HPC Forward-compat
- If HPC is enabled, container pull/storage roots and output paths may differ from local defaults.
- Admission validity is based on contract parity and pinned artifacts, not local-path assumptions.

## Examples
- Planned -> experimental promotion with registry + container smoke + docs generation update.
- Production promotion only after lock parity and QA matrix pass.

## Failure modes
- Registry entry without container/smoke coverage causes policy failure.
- Docs updated without underlying registry/domain updates causes drift and regeneration failure.

## Imputation Tools Admission
This section applies to VCF downstream imputation/phasing tools such as `beagle`, `glimpse`, `impute5`, `shapeit5`, `eagle`, and `minimac4`.

Acceptance criteria:
- License clarity:
  - SPDX-compatible license metadata is recorded and reviewable.
- Reproducibility:
  - Tool version is pinned and represented in registry/version lock contracts.
- Offline build posture:
  - Build recipe is deterministic and does not rely on implicit runtime downloads.
- Deterministic versions:
  - No floating tags/branches (`latest`, `main`, `master`) in admitted configs.
- CLI stability:
  - `--help` and version command behavior are contract-checked in smoke policy.
- Domain contract:
  - Tool has manifests under [domain/vcf/tools/](../../domain/vcf/tools) and stage bindings in
    [domain/vcf/index.yaml](../../domain/vcf/index.yaml).
- Fixture contract:
  - At least one fixture exists for each admitted stage binding.
- Runtime contract:
  - Tool is containerized or explicitly marked external with rationale until containerized.

## Reference Panel Admission Addendum
Panel artifacts used by tool workflows must satisfy:
- Licensing:
  - Panel source license is documented and compatible with project distribution policy.
- Provenance:
  - Source origin and transformation lineage are recorded in panel metadata.
- Reproducibility:
  - Panel catalog and lock metadata include fixed version, URL, and checksum.
- No floating inputs:
  - Branch/tag-style moving references are not allowed for production panels.
