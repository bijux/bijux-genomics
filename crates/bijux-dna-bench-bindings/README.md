# Benchmark binding evidence

## Ownership

This crate owns complete FASTQ/BAM serving-map and expected-result evidence:
input admission, binding identity, artifact roles, report construction and publication.
The four serialized row/report records belong beside their constructing services.

## Boundaries

Explicit providers acquire canonical domain, planner and corpus observations.
Normal and test dependencies exclude CLI, API, benchmark aggregation, benchmark
models, native BAM/FASTQ/VCF, planners and runners. Core owns canonical identifiers;
infrastructure owns atomic filesystem primitives. No network calls, process execution,
root discovery or environment mutation belongs here. This crate does not execute
native tools or infer scientific compatibility from declaration coverage. Provider effects are local
canonical acquisition and any publication required by that source authority.

## Public API

`ServingContext` and `ExpectedContext` bind each request to its explicit root and
independently substitutable providers. See [public contracts](docs/PUBLIC_API.md).

## Architecture

Serving and expected results each group models, input admission, projection and
publication by responsibility. See [architecture](docs/ARCHITECTURE.md).

## Tests

Injected owner observations cover ambiguous bindings, missing declarations, artifact
roles and publication. Existing CLI cases cover canonical integration.
See [tests](docs/TESTS.md).
