# BAM benchmark evidence

## Ownership

This crate owns BAM command and parser declaration coverage, adapter output evidence,
report placement and consistency across active bindings. Their existing serialized
records remain paired with the services that construct and publish them.

## Boundaries

Canonical serving, planner, stage catalog, expected-result and command observations
enter through independent typed ports. Observation records are not serialized copies
of shared wire schemas. Normal and test dependencies exclude native BAM execution,
planners, CLI, API, benchmark aggregation and benchmark models. Core supplies canonical stage identifiers. Infrastructure supplies
local filesystem primitives. No process launch, network access, root discovery or
remote submission belongs here. Injected providers own their documented acquisition
and local publication effects. Declaration coverage does not certify scientific
normalization, fresh tool execution or downstream interchangeability.

## Public API

The five capability modules accept an explicit `EvidenceContext` and output path.
See [public contracts](docs/PUBLIC_API.md) for schema and failure behavior.

## Architecture

Models, input observations, ports, projections, semantic admission and publication
have distinct owners. See [architecture](docs/ARCHITECTURE.md).

## Tests

Owner regressions inject observed declarations and exercise rejection and publication;
CLI contract cases retain canonical acquisition coverage. See [tests](docs/TESTS.md).

## Readiness families

The `readiness` namespace owns contamination/sex/haplogroups,
recalibration/genotyping, damage/authenticity, insert size/GC bias,
overlap/endogenous content and kinship assessments. These consume canonical
serving and expected-result wire schemas from `bijux-dna-bench-bindings`.
`DeclarationEvidence` separates published observations from
`LocalProofWriter` native proof production. Both enter through an explicit
`ReadinessContext`. The `bam_downstream` feature admits retained downstream
bindings without importing their native execution crates.
