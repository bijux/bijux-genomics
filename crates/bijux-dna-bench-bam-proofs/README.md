# BAM scientific benchmark proofs

## Ownership

Owns scientific output admission and completion assessment for authenticity, contamination,
damage, endogenous content, haplogroups, overlap correction, recalibration and sex.
Each capability retains its serialized records beside its assessment and publication.

## Boundaries

Canonical declaration reports enter through `ReadinessEvidence`; native operation paths
enter through `NativeProofWriter`. This crate must not discover a global repository root,
launch processes, select substitute tools or import CLI, API or planner crates. Canonical
BAM scientific summaries and parsers remain in the domain crate. A completion assessment
does not establish universal tool equivalence or fresh remote execution.

## Public API

Each stage accepts an explicit `ProofContext` and output path. See
[contracts](docs/PUBLIC_API.md) for error and publication semantics.

## Architecture

Each stage separates models, acquired observations, scientific checks, explanations and
publication. The declaration-only evidence crate can compile without this owner and its
BAM domain dependency. See [architecture](docs/ARCHITECTURE.md).

## Tests

Owner tests validate source rejection and scientific disagreement without native launches.
CLI contracts cover real canonical composition. See [tests](docs/TESTS.md).

Provider effects are explicit local artifact acquisition/publication; the owner publishes
reports through infrastructure primitives and performs no process or network effects.

Genotyping and kinship use typed downstream evidence adapters under `bam_downstream`.
The owner depends on canonical evidence and BAM semantics; parser/pipeline/VCF
orchestration remains injected, with no additional compile dependency on those services.
