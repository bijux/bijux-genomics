# Local benchmark workflows

## Ownership

This crate composes amplicon, core germline and aDNA local benchmark workflows.
It owns workflow reports, stage handoffs, fixture observations and output lifecycle. Filesystem effects publish owned artifacts;
unknown files survive reruns, and injected operations own native fixture encoding.
The CLI supplies admitted tool bindings and concrete fixture operations.

## Boundaries

The default builds amplicon composition without native BAM/VCF execution.
The `genomics` feature enables the cross-domain workflows and their actual stage
dependencies. CLI, API and benchmark aggregation stay outside the normal graph.
Pure pipeline catalog/profile contracts remain in their existing owner.

## Public API

Each family exposes its report contracts and render service. Genomics composition
accepts explicit inputs from [the public interface](docs/PUBLIC_API.md).

## Architecture

Stage implementations, report models, observations, handoffs and publication are
separate responsibilities. See [architecture](docs/ARCHITECTURE.md).

## Tests

[Tests](docs/TESTS.md) distinguish output ownership and binding admission from
scientific native execution and public command compatibility.
